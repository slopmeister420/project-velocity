# Graphics

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
  (or a new load) despawns it. Tag the game's 3D camera with `TrackCamera`: it is set to order `WORLD_CAMERA_ORDER` (-40) / no colour
  clear / no tonemapping, the four sky cameras (orders -44..-41, render layers 1-4) follow its rotation and take its
  `RenderTarget`. Use a vertical fov of `sky_render::BACKGROUND_SPAN` (0.8185 rad = 60° horizontal at 4:3) to
  match the background gradient. `TrackRoot` carries bounds; `TrackLights` (resource) the track's lights.
- **Car** (`car_render`): write `SpawnCarRender { request: u64, spec: CarSpec::new("LA/350z", colour), transform }`
  (`CarSpec.parts` = part index per `PART_SLOTS` bonnet/f_bumper/r_bumper/exhaust/spoiler/trim/scoop; `pearl`;
  `wheel` model stem). The root gets `CarRender { request, spec }` + `CarRenderParts { body, wheels: [FL, FR, RL,
  RR], bounds, wheel_radius }`. Each frame set the root `Transform` (chassis pose, game space) and each wheel
  entity's `Transform` (relative to the root, e.g. from the physics wheel render matrices; initial placement is
  only estimated from the body bounds). `DespawnCarRender { request }` removes it. FE car-select can use the same.
  Insert `CarLightInput { braking, reversing, damage }` on the root each frame for brake/reverse lights (LODs and
  lights run by themselves; `CarSpec.frontend` = `_high` models, one LOD).
  **Paint**: build the spec with `CarSpec::new(dir, _).with_colour(&CarColour::from_record(c.unk_0c, c.colour_a,
  c.colour_b))` from the setup's body colour record (`VehicleSetup.colours[0]`, COLOURS.BIN record) — that gives the
  original paint, pearl and paint type (0 gloss / 1 pearl / 2 metallic).
- **Skinned characters** (`skinned_object_render`): write `SpawnSkinnedModel { request, pack: CharPack::Chars |
  FlagGirl, model: "<xmf stem>", anim: Some("<gaf stem>"), transform }`; the root gets `SkinnedModelRender`.
  Move/hide it through the root `Transform` / `Visibility`; `SkinnedModelAnim { request, anim, time }` switches the
  anim (`time: Some(t)` holds normalised time t, `None` advances and wraps); `DespawnSkinnedModel` removes it.
  Flag girl models: `flag_girl_model("LON_FLAGGIRL_A0") = "lon_flaggirl_a"` (FG_M.vcd), anims = timeline BoneAnim
  item names = FG_A.vcd stems ("Walk", "Start"). Foyer records: Name = CHARS_M stem, STANDANIM/IDLE*ANIM = CHARS_A
  stems, CharPos + `Quat::from_rotation_y(Rotation.to_radians())` (models face +Z).
- Viewer: `trackview --track ExecTest --car LA/350z,LA/RX8 [--colours 3,6] [--chars fg:lon_flaggirl_a:Start,
  ch:dj:dj_idle1] --shot …`; `--foyer hangout` (instead of `--track`) loads the foyer + its Crowd/Dance characters.

**Screenshots** (`research/screens/`): `exectest_01.png` (ExecTest overview), `exectest_signs.png` (street level,
text check), `lacoast_01.png` (LACoast overview + sky), `lacoast_street.png` (street level: sky gradient, lit
instanced palms/lamps), `car_350z.png`, `car_colours.png` (5 cars, COLOURS.BIN 3/6/8/5/1 on LADown),
`characters_lineup.png`, `gfx_specchars.png` (see "Character specular"), `gfx_carshader.png` (see "Fixed-function car sets"), `flag_girl_grid.png`, `foyer_hangout_chars.png` (foyer is dark as lit), `gfx_fx.png` (LADown:
FSE present into an image, impact/glass particles of the damage demo, Race01 arrow barriers with a red hit wobble;
`--track LADown --car LA/350z --damage --particles Circuit\Race01\ --barriers Circuit\Race01 --barrierhit
190,-8.6,14.4 --cam 197,-5,34,0,-8`), `gfx_mist.png` / `gfx_wheelblur.png` / `gfx_rain.png` / `gfx_flares.png` (see "Puddle mist" / "Wheel blur" /
"Camera rain" / "Track flares"), `gfx_beams.png` / `gfx_beams_debug.png` (car headlights, see "Track flares"), `gfx_beamlights.png` (headlight pool on
instances, see "Beam lights"), `gfx_carflares.png` (see "Car light flares"), `gfx_neon.png` / `gfx_neon_debug.png`
(car shadow and neon), `gfx_envmap.png` (see "Dynamic envmap"), `gfx_spray.png` (tyre spray, see "elParticle and tyre spray"), `gfx_wetroad.png` / `gfx_wetroad_game.png` (see
"RainSurface road layer"), `gfx_stencil.png` (car stencil under the zone ring: `--track LADown --car
LA/350z --colours 3,6,8,5,1 --cam 197,-5,34,0,-8`, `VELOCITY_ZONE_FORCE=40`). `VELOCITY_FSE_DEBUG=1` also saves the FSE back buffer (`<shot>.bb.png`). Re-render: `cargo build -p visos_render --bin trackview`, then
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
- **Shader programs** (stage 1): `renderer::xgraphics_shader` replaces XGAssembleShader — parses every
  `Shaders\*.vxu|*.pxu` (vs/xvs 1.x, ps/xps 1.1; `#define`, `;`/`//` comments, CR-only files, instructions split
  over lines, `+` co-issue, `c[n + a0.x]`) and emits WGSL with NV2A semantics: vertex constants c-96..c95 (array
  index = reg + 96), D3D `lit`/`rsq`/`m4x4`, scalar ops on `.w` without a swizzle; pixel side = register
  combiners (rgb and alpha portions, values clamped to [-1,1], co-issued pairs read before either writes,
  `r0.a = t0.a` at start, `texcoord` = clamped (s,t,r,q), alpha reading `.rgb` takes blue, xmma/xmmc/xdm/xdd,
  `_bias/_bx2/1-` sources, `_x2/_x4/_d2/_sat` results, output clamped 0..1). All 34 files translate and pass naga
  validation (test). `game_shader_program_manager` = the 8 vertex / 6 fragment program tables (names 0x6875D8 /
  0x6875F8, input tables 0x15D250 / 0x15DB90); `global_mat_input_resolver` = the 30 global inputs (0x12C980, ids in
  `gmi`, `set_vector`/`set_texture` = 0x12D090/0x12D130, defaults texcoord scale (8,8,8,1), minTextureAlpha
  (1,1,1,0), compartment fade 1). `renderer::shader_program_manager::ProgramMaterial` = BindShader 0x2DF5A0:
  `bind_inputs(material inputs, ObjectInputs{lights, base}, globals)` resolves each table entry by name (shader
  input -> global -> table value, 0x2A08B0) into the uniform (vs/ps constants, stage texture sizes + linear flag);
  matrices (matWorldViewProj/World/WorldView/ViewProj/View) and cameraDir are computed in the vertex wrapper from
  bevy's mesh/view (register k = row k of the column-vector matrix = the transposed d3d upload); lightingDirs rows =
  D3DLIGHT direction with w = Range = 0 (so `lit` powers from c22..24.w are 0), lightingCols rows 0-2 diffuse, row 3
  ambient a=1 (GFXHelper 0x12C670/0x12C770); textureNSize = (w,h,0,1). Xbox **linear** textures (D3DX loads with
  non-pow2 size, e.g. ENVMAP1.TGA 132x132) are addressed in texels: the wrapper divides by the size, sampler clamps.
  Programs are generated at startup from the raw files (`ProgramMaterialPlugin::finish`) and picked in
  `specialize`; stage blend/z state via `shader_manager::apply_stage_pipeline_state`.
- `car_render` + `car_palette_pc_xbox` — chassis `<car>_base` + parts, high LOD from VehicleS.vcd, wheels from
  wheels_m.vcd. Car shaders get a shader type (GameDataLoading 0x9EAF0, `car_shader_type`) and a replacement set
  (0x157BA0, `shader_set_for`): type 0 -> ShinyCar = ShinyCarPaint.vxu + Default.pxu, types 4/5/7 -> CarDecal =
  Decals + Decals (blend 3, no z write); both are `ProgramMaterial`s rebound every frame (`bind_car_programs`):
  paint/pearl colours (0x1573E0 with CarRender tables 0x6866D0/0x6866F0/0x686710 {pearl power, paint scale/128,
  pearl scale, y offset}; `CarSpec.paint_type` = car data +1780, `CarSpec.frontend`), pearlescentMatrix row0 =
  normalize(camera dir - (0, 0.7, 0)) w = power, row1 = (2,2,1,1) = noise uv scale (0x1570F0), envmapMatrix =
  car rotation x basis around normalize((0,1,0) + dir to camera) x scale 0.9 about 0.5, transposed (0x114D80 +
  0x10CE90), refmap = `EnvMapTest\ENVMAP1.TGA` with envmapIntensity (0.2,0.2,0.2,0.1), noise = glitter with
  noiseIntensity 0.6 for paint types != 0 (else 0), neon scale (global 29) 1 in the showroom else 0. With the
  dynamic envmap on (in game `byte_6F7157 == 0`) the game instead binds its "Envmap" target at 0.6/0.8 with noise 0
  and the 0x1218A0 matrix (see "Dynamic envmap").
  Shot: `research/screens/car_350z_program.png`. CarShader/ShineyWheel: see "Fixed-function car sets"; BrakeLight
  is ported (see "Car LOD + lights").
- `model_builder` — shared XMF→mesh/material building (`Builder`, `SpawnOpts`, `TexSet`; `Builder.records` = every
  spawned primitive in GFXHelper 0x12B4C0 search order).
- **Car LOD + lights** (`car_render`; stage "car rendering details"). In game the car is built from VEHICLEM.VCD
  `<stem>`, `<stem>_med`, `<stem>_low` (chassis `<car>_base…`, parts, wheels `_wheel`, `_wheel_Med`, `_wheel_Low`;
  VehicleEntity 0x1C08B0 / 0x1C0D60 — `_high` is front-end only, FEAlloyEditor/FE3DVehicles), a missing LOD falls
  back to the better one. `CarLod { lods, wheel_lods, lod }`; per frame `select_lod` = 0x111920: d² = |cam − car|² ·
  (fov°)² / 8100, LOD 1 from 400, LOD 2 from 1600 (thresholds +55756.. set in 0x10D7E0, side/front pairs equal).
  Lights (0x115110 / 0x1152D0 / 0x117AA0 lookups, drawn by 0x112A50 when LOD ≠ 2 and the car type ≠ 2):
  `left_brake_l2_1` else `left_brake` (+right; `_l2_1` = flag 0x200 → v' = 0.7v + 0.3), `left/right_headlight` in the
  chassis (all set to prim type 9 = excluded by the car mask 0x200 and redrawn from LOD 0 at LODs 0/1 — reparented to
  `CarLights.group`), `left/right_reverse` (off_6867F4) chassis then parts. Every `_brake` shader (also discs/pads)
  uses the per-car **BrakeLight** set (0x156A60 → `brake_material`, stage mode 1): COLOROP MULTIPLYADD(arg0 TEXTURE
  + TFACTOR·TEXTURE), ALPHAOP MODULATE(TFACTOR, TEXTURE), blend 1, alpha test ALWAYS, the prim's texture; TFACTOR =
  (v,v,v,v) via 0x1589F0 → 0x158000 (`_ftol(v·255)`), v = brake level +1828 (±0.25 per rendered frame by VE+168 > 0,
  scaled here to dt·60; 0.5 without a vehicle), 0 for a broken side (+2250 bits 4/8; `_l2_1` cars skip a broken one).
  Reverse level +1832 (±0.25 by gear < 0, 0x1B74A0) redraws the reverse prims with **"revlight"** (0x10D7E0: blend 2,
  z test no write, rgb = texture, alpha = TFACTOR = min(2·_ftol(r·128), 255), stage mode 2). Headlight damage (bits
  1/2) only changes `dword_70657C` +12 (w of `dword_7181D4` = (128,128,128,64)), which nothing reads on xbox — so has
  no visible effect; the same dead vector gets the brake colour lerp (80,64,64,18)→(128,128,128,64).
  Stage-op decoding used for this (ShaderManagerBase setters → stage bytes, CompileStage 0x2A3320 code 12 →
  SetStageOps 0x2A7120): colour op +228, alpha op +244, args +260 → ARG1, +276 → ARG2, +292 → ARG0; arg enum
  0 TEXTURE, 1 DIFFUSE, 2 CURRENT, 3 SPECULAR, 4 TFACTOR (0x2A71B0); op enum `MapTextureOp` 0x2A3230 (0 DISABLE,
  1 SELECTARG1, 2 SELECTARG2, 3 MODULATE … 15 MULTIPLYADD, 16 LERP); code 16 = TFACTOR (raw D3DCOLOR, 0x2A2D60);
  0x2A0F40 = alpha func + 1, 0x2A1140 = stencil byte (glow), 0x2A2100 = all three args. Input from core:
  `CarLightInput` (see core.md request). Shot: `research/screens/gfxlights_1_start.png` (LADown dusk, 0.5 brake glow).
- **The "every car is lilac" bug** was not the palette: ShinyCar (blend mode 1 = no blend, RGBA write) wrote its
  bloom alpha (`t1.a * c12.a`) into the target and bevy's camera compositing then showed the sky cameras through the
  car. `apply_stage_pipeline_state` now masks alpha writes whenever blending is off (framebuffer alpha only fed the
  original's bloom; dest-alpha blend modes 5/6/8/10/13 read 1 now). Root cause found later: bevy alpha-blends every
  camera after the first on a target; FSE cameras now write with REPLACE and the glow-stencil views write the bloom
  alpha on purpose (see "Full-screen effects"). High-LOD bodies are open at the doors: the
  `<car>_{f,r}_{l,r}_door.xmf` models in VehicleS.vcd are added when `<car>_base_high` is used.
- `car_palette` — CarPalette.cpp: COLOURS.BIN record -> paint. CVehicleDatabase::Init 0x00184860 fills record
  +0x34/+0x38 with `VISOS_ColourXBOX::Set` 0x002B1370 (a=1, rgb = float/255, `_ftol(x*255)`, 0xAARRGGBB) from the
  floats at +0x1C/+0x28 (zero in the file); SetCarColour 0x0010C960 copies paint kind `unk_706260[+0x0C]` (static
  init 0x004245A0: 3 entries {(1,1,1,0.12), 0.5/0.53/0.825, 0.3/0.5/1.2, 0/2/1, paint type 0/1/2}) to car +1752 (paint
  type = +1780) and hands +2236/+2240 to 0x00157F80 -> ShinyCar paint/pearl via 0x00015ED0 (`byte * 0.003921569`).
  Kinds 3..5 (rims/neons slots) are not body paint. 0x001561C0's 32x32 "CarColour" gradient: see "Fixed-function
  car sets". 0x0010D0B0's 0x706544.. globals are debug tweak mirrors.
- **Skinned models** (`bone_skeleton_model`, `bone_anim`, `bone_anim_blend_tree`, `skinned_object_render`).
  XMF2 `+0x2C == 1` = skinned: `+0x30` -> `{u32 checksum, u32 numBones, bones}`, bone 240 B {+0 name, +4 parent,
  +8 u16 depth, +0x10 quat, +0x20 scale, +0x30 local (row 3 = bind translation), +0x70 world, +0xB0 inverse world},
  depth-first. Each part's primitives are followed by `{u32 count, u32 offset}` per primitive = palette of skeleton
  bones; vertices: pos FLOAT3, weights PBYTE4 (v1), normal NORMPACKED3, uv, palette slots SHORT4 (v15; BoneBlend4
  `v15 * 4` into 33 palette matrices). `.gaf` ('GAF0', count, absolute offsets): info {name[128], +0x80 skeleton
  checksum, fps, duration, frames, +0x90 vec, +0x9C loop, +0xA0 compressed, +0xA4 nbones, +0xA8 bone infos {name,
  active, nkeys}, +0xB0 key ptrs, +0xB4/+0xB8 positions, +0xBC/+0xC0 scales}; keys 16 B {u16 euler xyz (65536 =
  2pi), pos idx, scale idx, start frame, run, pad}; compressed = runs, interpolated (signed) towards the next record
  (0x002E17A0, binary search 0x002E1710); sample 0x002E11C0 (frame = n*t looping / (n-1)*t, slerp 0x002B7530, euler
  quat 0x002B7100). Blend tree: bind QTS (quat +0x10, scale +0x20, translation +0x60) blended by layer weight
  (0x002DBB40), local = rotation by the conjugate (Renderer 0x002AC2C0), rows scaled, translation row; world by
  depth stack; skin = invBind * world. Test: every skinned model rebuilds its stored bind matrices (identity skins),
  every .gaf parses/samples. Bevy side: joints = model-space bone worlds under the root, inverse bindposes = +0xB0,
  `NoFrustumCulling`; time advances by `frame_inc` (0x002E0FF0) and wraps (SkinnedObjectRender 0x0013AF80). Skinned
  prims draw with GameDataLoading 0x0009EF30's "CharacterSpecular" override (0x001065A0): **correction** — its only
  compiled record is code 2 = alpha test ALWAYS ref 0 / z LESSEQUAL, so the own cut-outs (eyes, mouth, `FG_alpha_*`
  GEQUAL 0x8D) draw uncut as with the car sets; the own blend stays (the old "blend 2 → 1 for all" stand-in is gone,
  SpecularChars now rewrites it, see "Character specular").
  **BoneBlend4 lighting** (done, `skinned_object_render::relight_skinned`, every rendered frame after posing):
  Characters 0x79C90 binds BoneBlend4.vxu through ShaderProgramManager; 0x79EC0 builds the object set from the
  scene's dirs (+4) / colours (+52) / ambient (+100) unless the caller passes others; the shader lights the normal
  through the **major bone only** (palette slot 0, `v15.x`), then the world matrix (unnormalised), oD0 = c-87 +
  Σ (n·L>0)·(n·L)·col (= `VsLights::light_vertex`). Skinned meshes keep their main-world copy (`RenderAssetUsages::
  all()`) for the colour rewrite; no anim = bind pose, still relit with the root rotation. Shot `gfx_skinlit.png`.

**Requests (graphics -> others)**: core — `car_spec` should also call `spec.with_setup_underbody(setup, db)` (shadow
size + neon, see "Car shadow and neon"), and `HeadlightView` is still unwritten (core.md); `race_cars::car_spec` should use `CarSpec::with_colour` (record kind
`unk_0c` + float colours, not `colour_a / 255` with paint type 0) and map `fitted_bodyparts` to `CarSpec.parts`;
flag girl: spawn a `SpawnSkinnedModel { pack: FlagGirl, model: flag_girl_model(name) }` per `FlagGirlModel`, apply
`FlagGirlEvent::ModelMatrix/ModelVisible` to its root `Transform`/`Visibility`, and emit the intro timeline's
BoneAnim items (kind 1, `animation` "Walk"/"Start") as `SkinnedModelAnim` (the flag girl sequencer does not surface
them yet). Frontend — foyer characters (Crowd/Dance/Chars.txt) through `SpawnSkinnedModel` (`CharPack::Chars`),
idle cycling via `SkinnedModelAnim`; `foyer.rs` should also use `with_colour`.

### Tweaks (`gfx_tweaks`, the 21 graphics AddTweaks; registered tree done, menu hook-up = core request)
- `GraphicsTweaks` (resource, `GraphicsTweaksPlugin` in RenderPlugin) records every registration 1:1 as
  `GfxTweakCall`s (createSystem path, then addTweak float / S32 / enum / bool with the binary's names, min, max,
  step, enum text; S32 step 0 passed raw) and one `GfxValue` slot per tweak (297 tweaks, 52 systems).
  `PHASES` = the original's call sites (CommonInit 0xB503F FSE, 0xB50A2 Spec (via CarDamageFX 0x79C30 → 0x143140),
  0xB51BC Bloom, 0xB51E5 Fog, 0xB5212 TrackFX, 0xB5268 Fullscreen Blur, 0xB5295 Reflection; FrontEndInit 0xB5829
  ZoneRender 0x163770 → CarPalette 0x10D7E0 [own, 0x10E3F0, 0x1214A0 → 0x122AE0], 0xB5854 DOF, 0xB587D MTV;
  InGameInit Spec (0xB4170 at 0xB4BD1), Fog, Beams, Slipstream (0x13F380), Rain (0x133AB0), DOF (0x120B20), CatsEye,
  Sky (0x13B540), Barrier ×2, CarPalette chain again, FlareRender 0x128100 → ZoneFX (0x162B60) then NOS FX; police
  car ctor 0x18000 → Graphics.Damage). Core replays them into `CTweakManager` (request in core.md).
- `sync_graphics_tweaks` (First, exclusive): slot ≠ last synced (menu edit) → written to the live variable, else the
  live value is pulled in. **Live** slots: Spline Barriers (`BarrierTweaks`), Point Light Intensity / Force Car LOD /
  Force LOD Level (`CarRenderTweaks`, new: 0x111920 returns the forced level outside the front end; 0x119EF0's point
  light scale is now a `car_light_set` argument), DOF (`DepthOfField`), Dynamic Envmap (`DynamicEnvmap.enabled` +
  `EnvmapTweaks`), Fullscreen Blur (`FadeRender.blur`), NOS FX, Rain (`CameraRain`), Reflection (`RainSurface`,
  `FeReflection`), Sky (`SkySetupRes`, cloud layers by index; "Num Cloud Layers" read-only: layers are built at
  load), Slipstream (`RaceEffects.slipstream`: new `distortion_step/amplitude` (+3764/+3768) and `globals`
  {trail length, test vehicle, brightness, alpha} kept across the in-game re-init; `MotionBlur` slip min/max), Spec
  "Enabled", ZoneFX (`ZoneFx.tweaks`, ZoneData index = (addr − base)/4), Fog (`FogRender` is now a resource inserted
  by the track build; materials bake its states, so edits show on the next build), Bloom (`GlowTweaks`),
  3DLightBeams (`BeamTweaks`, new resource replacing the consts in `beam_texture` / `beam_pose`; "Redraw Texture"
  rebuilds the volume in `beam_update` and clears itself as 0x1618D0; `ProjectedLightBeam.on`).
- **Slot-only** (the slot is the variable; reader not in the port): Graphics.Damage (CarDamage debug draws),
  WheelPenetration (CarRender 0x1134A0), most of Paint / Lights / LOD ranges / AlloyColour (mirrors applied by the
  debug override 0x11AD30; LOD ranges = √ of 400 / 1600), CatsEye Display, Envmap hi-contrast / greyscale /
  desaturate, MTV Effect (0x1300D0), Reflection Additive, Sky save / load cfg, Slipstream Test Vehicle (0x13F770
  debug set), Spec editing / display / file tools, TrackFX detail road / stretch, FullScreenEffects mip bias /
  reload. Defaults = the .data value, bss 0; `this`/local targets without a port value default to 0.
- Byte variables under S32 tweaks (original bug: the menu writes 4 bytes): `byte_6870C0` trail length, `byte_686950`
  envmap fresnel. Test `registrations_match_the_binary` compares every call against `research/gfx/tweak_regs.tsv`
  (`tools/ida/gfx_tweak_regs.py`: decompiled statements, name strings resolved, initial .data value or "bss"); also
  `slots_and_phases`, `sync_pushes_edits_and_pulls_live_values`. Readers of the slot-only variables:
  `research/gfx/tweak_xrefs.txt`, `tweak_readers.txt`; callers `tweak_callers.txt`, `tweak_site.txt`.

### Character specular (`specular_chars`, SpecularChars.cpp 0x143140..0x146D50; done)
Dumps `research/gfx/SpecularChars.c`, `spec_dc{1,2}.c`, `spec_dis1.txt` (0x143F80), `spec_res.c` / `spec_callers.c`
(resolver vtable 0x62B0D8 = 0x79A10, callers 0x79EC0 FE / 0x7A460 game), `spec_vals.txt`, census `spec_census.txt`
(test `spec_census`, ignored); scripts `tools/ida/gfx_spec_*.py`.
- **Init** 0x143140 (CarDamageFX 0x79C30 ← CommonInit, arg = front end; here on the first `SpawnSkinnedModel`: pack
  `Chars` = front end, `FlagGirl` = game → resource `SpecularChars`): textures `dword_736478` (all in `\VFX\misc.vcd`)
  [0] Spec2, front end [1..11] spec_t_gk_001..011, game [1] 005 [2] 011; settings 0x1442F0: front end
  `\foyers\hangout\SpecChar.bin` ('10PS', v6, 30 × 172 B, +12 offset), game `\chars\FGlit<byte_6F7130>.bin` then
  `FGlit.bin` (neither ships) → defaults: 5 records {rot 0, elev 45, rgb 1, intensity 1, bloom 0.2, materials
  `off_6871B8` fabric/skin/leather/plastic/metal/hair (power, intensity 0.7/0.25, 1/0.5, 1.5/0.75, 2/1, 1/1, 1/0.5,
  texture 0, uv stretch 0.5)} + '20PS' 72-byte diffuse sets {3 colours 0.5, ambient 0.15, rotations 0/120/240,
  elevation 30}. Count > 30 → pass off. Record: +0 rot, +4 elev (deg), +8 rgb, +20 intensity, +24 bloom, +28 6 × 24 B
  materials {+4 power, +8 intensity, +12 texture, +16 kind, +20 uv stretch}. Tweaks "Frontend.CharacterSpecular" 0x145F20
  (Enabled `byte_6871A4` = 1; All-Skin / Show glossmap / Display / editing / save: debug, not ported).
- **Kinds** 0x143540: shader name contains "spec_" and (lower case) a material name, prim kind 0 or 2 → kind 8 + m.
  Data: 121 `spec_*_0` prims (CHARS_M, FG_M), all blend 2 + stage +0x0C bit 0x10 ("gloss"); others have no blend
  record and an alpha test.
- **Per draw** 0x143A40 (from 0x79EC0 / 0x7A460): active = Enabled, camera distance to (root bone world + model
  sphere radius up) ≤ 9 (`dword_6871AC`), ≥ 1 spec material; light dir = normalize(sin r cos e, sin e, cos r cos e),
  colour = rgb · intensity · (9 − d)/9, w = bloom. 0x143740: list ≤ 10 materials by first use; gloss spec prims get
  blend 1 (`dword_687248`) while active, 0 while not (0x2A2BE0 rewrites the blend record in place). 0x143C70: base pass
  = prims not in the list (kind mask `(1<<kind)<<16` = exclude, Renderer 0x2A9720), material index 0xFFFF; then per
  list entry i its prims with 6D6D9C = colour · (H·V) · entry intensity. 0x143F80 (ecx this, 6 stack args: world,
  out, power, light, uv stretch, pos; returns H·V in st0 — **world, power and uv stretch are unused**): V =
  normalize(cam − pos), H = normalize(L + V), up = (0,1,0) (or (−H.y,0,0) when H.x = H.z = 0), side = normalize(up × H),
  u = side × H with **y negated** (not orthogonal, original), normalized; matReflection rows (H, u) after the double
  transpose. Resolver 0x79A10 (`dword_6D6D58` +4 matrix, +68 colour, +84 char, +88 material): SpecularLightColour =
  (colour.rgb · min(2 · material[i].intensity, 1), bloom), i = **the list position, not the material** (original
  bug kept), base pass rgb 0; SpecularTexture = front end textures[material[i].texture] else Spec2.
- **Shaders**: BoneBlend4.vxu oT1 = 0.5 · (n_world · row)³ for rows 0/1 (no offset, wrap: pass 1 0x2A1BE0(1,1));
  Characters.pxu r1 = t1·c10, rgb = t0·v0 + r1.rgb, a = t0.a · r1.a. Port: `XboxStageMaterial` gained `params.spec`
  (c10), `spec_texture` (bindings 4/5) and `arg.w` = 1 Characters mode reading `uv_b`; each skinned primitive gets
  its own material copy; `relight_skinned` writes `ATTRIBUTE_UV_1` with the colours; `update_skinned_models` rebinds
  c10 / texture / blend per frame (only on change). Game characters (flag girls) are lit by the record's diffuse set
  (0x144BC0 → 0x144C10: dir (sin r cos e, sin e, cos r cos e), colours, ambient) instead of the scene lights.
- Character index: `SpecCharIndex(u8)` on the root (0 without). FE = character element +764 (no direct writer found:
  `research/gfx/spec_764.txt`), game = flag girl index (0x7A460 a11) — requests below. Camera = `TrackCamera`, else the
  lowest-order active Camera3d. `VELOCITY_SPEC_CHARS=0` = Enabled off (A/B).
- Shot `research/screens/gfx_specchars.png` (`trackview --foyer hangout --size 640x480 --cam -0.4,1.5,0.6,90,-8`; vs
  `VELOCITY_SPEC_CHARS=0`: 1112 px differ, max 43 — highlights only where n·H and n·u both near ±1, as the original's
  uv maths gives). Tests `hangout_settings_parse`, `light_and_reflection`, `list_blends_and_draws`, `default_diffuse_set`.

### Vertex animation (`vertex_anim`, VertexAnim.cpp / VertexModel.cpp / VertexAnimBlendTree*.cpp; data + maths only)
Dumps `research/gfx/VertexAnim.c`, `VertexModel.c`, `VertexAnimBlendTree.c`, header dump `va_files.txt`.
- Files (only `\Chars\CHARS_A.VCD`): `DJ_Select.gvf`, `DJ_Select2.gvf` ('GVF0', +0x10 count, +0x14 info offsets; info
  +0 name[128], +128 checksum, +132 fps, +136 duration, +140 frames, +144 loop, +152 targets, +156 12-byte targets
  {name, active}, +160 frame count, +164 frame pointers → one weight byte per target; DJ_Select 20 targets, 55
  frames, 2.3 s) and `CharacterAnims.gvs` ('GVS0', +0x10 name, +0x90 count, +0x94 64-byte anim names).
- Vertex model = XMF +0x34 (`VertexModel` 0x2E2670..: +0 checksum, +4 count, +8 24-byte subs {+4 object, +8 vertex
  count, +12 base positions (16 B), +16 target count, +20 20-byte targets {name, delta count, index count, s16x4
  deltas {x, y, z, vertex count}, u16 indices}}); written back per object part 0 in VertexIterator order (0x2E2C70).
  **No shipped CHARS_M / FG_M model has one** (+0x34 = 0 everywhere, test `no_shipped_vertex_model`), so nothing is
  ever deformed (0x79EC0's a18 = FE char +448 slot; the checksum assert at VertexAnimBlendTree.cpp 360 would fire).
- Ported: parse + `sample` 0x2E2390 (frame n·t looping / (n−1)·t, t ≤ 0.99999, bytes/255 lerped, last frame wraps or
  holds), `frame_inc` 0x2E2230, `update_leaf` 0x2D8540 (weights zeroed, anim layers lerp active targets towards the
  sample, mode-1 layers lerp a named target towards 1), `limit_active` 0x2D8370 (drop the smallest until ≤ max),
  `apply` 0x2D8C10 (base + Σ delta·w/4096 over the index runs). No render hook (no data). Tests `shipped_vertex_anims`,
  `no_shipped_vertex_model`, `blend_limit_apply`.

### Beam lights / "Graphics.3DLightBeams" (`projected_light_beam_pc_xbox`, `model_builder::{Builder::beam, BeamCtx, beam_override, gdl_override, part_vertex_flags, beam_material}`; done)
- **Port** (`ProjectedLightBeamPlugin` in RenderPlugin): texture 0x161520 = `beam_texture` (64 × 64 × 32 L8 volume:
  outer loop slice t = i/31, then rows (v), inner x (u) — the linear order before XGSwizzleBox 0x1610C0; CreateVolume
  Texture(64, 64, 32, 1, 0, fmt 0 = L8)), uploaded as an Rgba8Unorm 3d image (l, l, l, 255), clamp u/v/w, linear;
  a 1³ black image until the first in-game car exists (front end = null texture). Pose 0x161F10 = `beam_pose` (player =
  lowest request of the in-game non-traffic `CarRender`s; head z from `CarHeadBeams`, in-car from `HeadlightView`,
  damage = `CarLightInput.damage | CarLightDamage`, width = `CarShadow.size[0]`); transform 0x1618D0 = `beam_transform`
  (LookAtLH(pos, pos + dir + (0, −0.2, 0), up) · scale(1/c, 1/c, 1/s) · trans(0.5, 0.5, 0), transposed; rows → c10..c13
  via `GlobalMatInputResolver.beam_matrix`, gmi `LIGHTBEAM_TEXTURE_TRANSFORM`). `ProgramMaterial.t1_volume` (bindings
  9/10, `texture_3d`) holds a volume-typed stage-1 input (`mat_input_type::TEXTURE_VOLUME`), `params.tex[1].w` = 1
  makes `xps_tex1` sample it with (s, t, r).
- **Instance override** (CompartmentManager 0x8AC20 → 0x8ABB0 on every instance model, `SpawnOpts.beam_lights` set by
  `build_track` for instances): part flags from the first draw (`part_vertex_flags`, GDL 0xA4CF0), primitives GDL
  0x9EF30 already overrode (`gdl_override`: packed uv, trafficbody_0, Overlay_, windowenv_ with 0x4A0 & 0x804) keep
  theirs; else 0x804 → "HeadLightBeam" (BeamLights.vxu, oD0 = v3 · fade), 0x4A0 → "HeadLightBeamLit" (_Lit.vxu; its
  three `mad`s all read r0.x — original bug, translated as is). Material = own stage records + code 2 (alpha test
  ALWAYS, z LESSEQUAL) + BeamLights.pxu, one per (texture, records, lit, fog) (`BeamPrograms`, 143 in London),
  rebound every frame (`bind_beam_programs`). Census (test `beam_census`, ignored → `research/gfx/beam_census.txt`):
  LA/London/London2 instances are all 0x6 (beam) or 0x22 (beamlit); only 44 London prims keep a GDL override.
- **Lights of the _Lit program**: GFXHelper 0x12C670/0x12C770 read the device lights back (0x12D2E0 = GetLight) and
  D3DRS_AMBIENT. The instance set has count 0, so Renderer 0x2A8BE0 calls SetLight for nothing (only LightEnable off)
  — SetLight's only other caller is the boot init StringManager 0xC8030 — so the lights are the last ones set the
  previous frame; ambient = the instance set's (`TrackInstanceLights.sky_ambient`; flag-0 stages leave the last
  AMBIENT, which is the same value). Dumps `research/gfx/beam_setlight_{xr.txt,dc.c}`.
  **Verified against the XBE** (`research/gfx/beamlit_chk{.c,2.txt}`, `tools/ida/gfx_beamlit_chk*.py`): 0x2A8BE0 is
  reached only through 0x2A8FD0 (DrawList 0x2C2A00 records, FuryInstance 0x12A7D0 / GFXHelper 0x12B0E0); it SetLights
  slot i < count whenever |dir|² > 0.01 (else only LightEnable off, data kept). In GameManager 0xB9010 every instance
  primitive is drawn by the draw-list flushes 0xB9243..0xB953B (solid, 6D5338, 6D5324), **before** the cars (CarRender
  0x119900 at 0xB975D/0xB9760 + their lists 0xB9773..0xB97A9); the last light-set draws of a frame are the 3d hud,
  OSD3dInterface 0xEA2B0 (0xBA172 / 0xBA1A3) flushed by the "Post 3d interface2" lists 0xBA1C8..0xBA1E9. The hud set
  is the player car's set (slots 0-1 point lights, free slots black with dir (0,0,−1), slot 2 scene light 0: all three
  have |dir| = 1, so all three are SetLight) with colours 0/1 × 1.5 — exactly what `bind_beam_programs` binds. Only
  when the hud is not drawn would the last car's set stand instead (not modelled).
- **A/B** (`gfx_ab` phases 7 `beam_off` = black volume texture, 9 `override_off` = plain stage materials, each between
  plain frames 6/8/10; count a pixel only when it differs from both neighbours and they agree). London grid, car
  standing (`SMOKE_DIRECT=1 SMOKE_AREA=2 VELOCITY_GFX_AB=20,gfx_beamab`): override off changes only instance props
  (lamp posts / railings on the bridge, 531 px); beam off changes nothing beyond the blinking chevron barriers and
  HUD (no instance within the pool: road = sector, barriers = BarrierRender, flag girl = skinned). Viewer A/B
  (`VELOCITY_BEAM_LIGHTS=0` = black texture, `=all` = every point mapped to the centre of slice 0 to show which prims
  carry the override): `trackview --track LACoast --car LA/350z --caryaw 150 --size 640x360 --cam
  322,-55.6,-392,-45,-12` → only the street lamp in front of the car brightens (211 px > 8, max 26) —
  `research/screens/gfx_beamlights.png` (on | off, below the ×8 diff). The pool is orthographic (w = 1): a 27.6 m
  square cross-section, 33 m deep, ring radius 0.4..4 m, fading to 0 at the far end. Fixed in passing: trackview
  never added `ProjectedLightBeamPlugin`, so its beam programs were never bound (instances drew without a base texture).
Dumps `research/gfx/beam_dc.c`, `beam_dc2.c` (callers, texture, transform, part flags), `beam_xr.txt`,
`beam_vals.txt`; scripts `tools/ida/gfx_beam_*.py`. The player car's headlight pool projected onto **instance**
geometry — not the beam quads of TrackFX (those are done).
- Shaders (SaveFileData 0x1065A0, object `unk_705F20`, 0x106A40(i) = shader i+1): 4 "HeadLightBeam" = vp 5
  BeamLights + fp 4, clamp (2A1CC0(3,3,3)); 5 "HeadLightBeamLit" = vp 6 BeamLights_Lit + fp 4, 2A1BE0(3,3). Installed as
  the +0 override by CompartmentManager 0x8ABB0 (from 0x8AC20, instance load) on every instance primitive without
  one: part flags & 0x804 → 4, else & 0x4A0 → 5. Part flags GDL 0xA4CF0 from the first draw's vertex desc: 4 = diffuse
  stream, 0x20 = normal stream, 8 = packed uv (format 33); 0x80/0x400/0x800 set elsewhere (not traced).
- Programs: BeamLights.vxu oD0 = v3 · c14 (compartment fade), oT1 = world pos · c10..c12 (LIGHTBEAM_TEXTURE_TRANSFORM,
  gmi 2), w = 1; _Lit adds per-vertex lighting from c20..c27 (dirs/cols/ambient). BeamLights.pxu: xmma r0 = t0·v0 (+
  t0.a += c10.a MIN_TEXTURE_ALPHA), r0.rgb = t1·r0 + r0, a = t0.a · v0.a; t1 = LIGHTBEAM_TEXTURE (gmi 5).
- Object (+0 texture 64×64 fmt 32, +4 matrix, +68 initialised, +69 "On/off" = 1): init 0x161C40 (InGameInit 0x0B62C3):
  texture 0x161520, tweaks 0x161190 (MinOffset 0.03 `68762C`, MaxOffset 0.3, MinSigma 0.02, MaxSigma 0.56, FOV 1.39
  `687640`, Extent 43, DamagedFOV 0.96, DamagedExtent 30, FallOffStart `7486A8` bss 0, BeamDip 0.35, Intensity 140
  `687650`, Redraw), 0x15D070(1, 29, 0). Texture 0x161520: 32 rows t = i/31, offset o = lerp(min, max, t), sigma s =
  lerp; g(x) = exp(−(x − o)²/(2s²)), normalised by g(0) − g(1); per texel r = √(u² + v²) (u, v = 2i/63 − 1), value =
  clamp((g(r) − g(1))/(g(0) − g(1))), × (1 − (t − F)/(1 − F)) past FallOffStart, byte = _ftol(140 · value) (row
  layout of the 64 × 64 vs 32 profiles to confirm). Per frame 0x161F10 (GameManager 0x0B904E, player car): pos = car +
  z·(head.z [+0.5 in car]) − 0.35 up; broken lights (+2250 bits 1/2) use the damaged FOV/extent and shift by
  ∓ x · width/5.5 (both broken: −100 y, dir (0,−1,0)); 0x1618D0 builds the transform (look-at from pos along dir with
  up (0,1,0), offset `unk_748968` (0, −0.2, 0), scale cos/sin((π − fov)/2) · extent, transposed) into +4.

### Sector streaming, sun/moon, flag girl height
- `compartment_manager` — CompartmentManager.cpp streaming (indexedUpdate 0x0008A270, priorities 0x000897C0, slot
  setup 0x000875B0). `index.idx` (v2; Tokyo's v1 index is refused, as LoadIndex does): `'_IDX', 2, count = sectors
  + 1` then 36-byte entries from +12: entry 0 common, then {high, low} per sector; `+4` size, `+0xC` centre (z stored
  negated: box z = -cz ± half), `+0x18` half x, half z, half y. Positive control: every live track's `_high` models
  lie in their boxes (London2's index is from another export: 8/35 off). **`_low` sectors are never loaded** (64-128
  byte stubs, empty index entries; no "_low" string in the exe) — residency of high sectors is the LOD. Slots =
  min(128, (20 480 000 − common − Instance − uInstance) / (largest sector + 64), count) (LACoast 19 of 28).
  Priority per sector = xz squared distance from camera to box; ×20 when the box is outside a camera plane, else
  ×((1−dot)·0.3+0.7) when ahead (dot > 0.01); raceline points j (40 m apart) with d²/900 < 1 pull it down by
  (1−g)(1−w)+g, g = (1−min(0.15(j+1),1))f + min(...), w = weight from the driver's raceline lateral offset +0x2AC (|l| > 16: (44−(|l|−16))/44); min over
  cameras; the sector number is stored in the float's low byte, insertion-sorted, first N = wanted. Each update:
  if every slot is busy, unload the resident-not-wanted sector with the largest priority, then load the
  wanted-not-resident one with the smallest (loads are instant here; the first pass after a load fills all slots).
  `track_render`: each `Sector_<n>_high.cpt` (+ its instances) sits under a `TrackSector` entity shown while
  resident; `TrackStreaming` resource; views from `TrackStreamViews` (core fills it; empty = the `TrackCamera`, bevy
  frustum planes, no raceline). Shot: `stream_lacoast.png`.
- Sun/moon (`sky_render::{SunMoon, update_sun_moon}`, `CSkyRender::RenderSunMoon` 0x0013C390, called from
  GameManager 0x000B9010 after the world): quad at camera + dir·1000, dir = `DirectionFromAngles` 0x0013E380
  (sin r·cos e, sin e, cos r·cos e), right = dir × up unnormalised, height along world up; sun = "sunb" (shader index
  `dword_6870AC` = 1): `\VFX\misc.vcd` sun_poly, blend 14, z test, no z write, colour Sun RGB; moon = sky's Moon
  texture, blend 3, z test, colour Moon RGBA, only when the texture exists. Flag `dword_6870A0 & 2` (0x3F, on).
  `ShaderManagerBase::2A15F0(1)` on "sunb" is not identified. Shot: `sun_lacoast.png`.
- Flag girl height: not a render bug. TimelineSequencer 0x00174A70 composes base·path·rot + full grid slot (core
  has it right); GameManager 0x000B9010 then drops each flag girl model onto the ground by a ray ±5 m around the
  grid y before drawing — requested from core (docs/core.md).

### Race effects and particles (stage: effects)
**Use from core / physics** (`RaceEffectsPlugin`, `ParticlePlugin`, both added by `RenderPlugin`):
- `LoadParticles { area: "\\Tracks\\LADown\\", sub: Some("Circuit\\Race01\\") }` at race load (foyers:
  `"\\Foyers\\hangout\\"`, no sub), `UnloadParticles` at shutdown. `SpawnParticle { name, matrix }` = StatelessParticleMgr
  Spawn 0x2D3240 (one-shot emitter; matrix columns = axes, translation in `w_axis`), `ParticleGroup { group, start }`.
- `EffectCars` (fill every game update): cars in race-manager order with `vehicle` (physics id), render `matrix`,
  FastCar velocity, VE+20 speed, VE+4788 heading, VE+3691 smoke off, 4 `EffectWheel`s (FastCar order, 0/3 front:
  centre, radius, contacts, any-contact sliding, surface, VE+24+i no marks), `ground_plane` (VE 0x1BAF60); plus
  `dt`, `time`, `start_line`, `professional`, `go`. Then run `race_effects::race_effects_update` in
  `InGameUpdateSet::Effects` (skid marks, tyre smoke, start light state).
- Messages: `RaceEffectsInit` (InGameInit, after the first `EffectCars`), `RaceEffectsReset` (RestartRace),
  `RaceEffectsShutdown`, `SparkCollisionMessage(SparkCollision)` per collision message, `SlipstreamStrength
  { vehicle, strength }`, `StartLineFire(pos)` (flag girl "startlinefireeffect"). `EffectClock { paused, scale }`
  stops sparks / slipstream / particles.
- Drawing: every effect writes triangles into `effect_draw::EffectDraw` (per code-built shader: texture from
  `\VFX\misc.vcd` / `\Particle\PARTTEX.VCD`, blend, z, bias, wrap, alpha test; two-sided) in `EffectSet::Build`;
  `flush_effects` writes one dynamic mesh per shader (`NoFrustumCulling`). `StageState.two_sided` was added for it.
- Viewer: `trackview --track ExecTest --particles --effects skid,smoke,sparks,slip,start --fxat 70,-13.4,133.6
  [--fxfollow k]` (two cars circling the point), `--car traffic:car01`. Shots: `fx_skid.png` (marks + smoke),
  `fx_sparks.png`, `fx_slip.png`, `fx_start.png` (green start lights), `particles_hangout.png` (placed steam),
  `traffic_cars.png`.

**StatelessParticles** (`stateless_particles::*`, `particle_manager::*`): types = 256-byte records (260 from version
0x6B) in `.slp` (PARTDEF.VCD) and `\Particle\PartFX.bin` (v0x6A, 12 types); layout in `stateless_particle_type.rs`
(+36 count, +40/+44 first copy alpha/size, +48 copies with +52/+56/+60 steps, +64..+72 size, +76/+92/+108 colours
0..128, +124 spin, +132 box, +148/+164 velocity, +180 accel, +196 emission window, +200 life, +204 duration (<0
endless), +208 frames, +212/+216 anim, +220 texture, +251 flags (2 fixed start angle, 4 billboard round the emitter
y), +252 blend, +256 sort key). Name crc = case-sensitive crc32 (`VsStringAnsi::Hash`). Pool of 384 emitters (52 B:
matrix, seed from Schrage 0x2D5BA0 without fix-up, time, scale, type, flags 1 run / 2 loop / 4 kill / 8 visible,
stop time); update 0x2D32C0 frees finished one-shots (total = ceil(duration/life)·life + window). Particles are
stateless: 0x321FC0 regenerates every particle each frame from the emitter seed with the 15-bit LCG 0x322880
(`20077·s + 12345`), ages in 1/600 s ticks, colours doubled twice (type 0x2D6F10 and vertex 0x2B11C0, saturation
kept), random colour = bytewise wrapping add. Quads 0x321930 (45° + spin, size·0.7071, uv frames 0x3224C0), camera
billboard = inverse-view rows 0/1. Drawn sorted by sort key / texture; z test, no z write.
**ParticleManager**: `particle.pld` = `PLDDATA` + version byte 3 + count, 48-byte records (+0 pos, +16 quat through
its conjugate, +32 type crc, +36 id, +40 group crc, +44 manager, +45 enabled); LACoast's v1 file is refused (original
too). Finite types fire once at load, endless ones are dynamic emitters. Many pld crcs are of lower-cased names and
miss the case-sensitive type crcs — those instances are skipped, as in the original.
**Collision debris** (`particle_manager`): `collision_particles` = 0x1005C0 (PhysicsManager 0x1A3A30/0x1A7060, same
record as the collision message: material +76, point +8, impulse +40 ≥ 1/140), pair table per eSURF from static init
0x423DE0 (`MATERIAL_DEBRIS`: ColDustGrey, ColGrass/ColGravel + dust); run from `SparkCollisionMessage` before the
sparks. `object_collision_particles` = 0xFFC30 (impulse ≥ 10/7, kind = object type +52, table 0x4239C0 `OBJECT_DEBRIS`:
splinters/leaves/chunks/glass/fence, 14 and 23 PSXATMsmoke + AnimFireSmall; at the first pld instance of the type
carried by the object matrix, else the hit point, 0xFFEA0; kind 23 hydrant dynamic groups 0x100B40 not ported) via
message `ObjectCollisionParticles`; `fire_off_particle_trail` = 0x1000E0 (event 24 propane_trail, 26 wheel_fire +
wheel_smoke every metre) via `ParticleTrail`. Spawns are `SpawnParticle` 0x100010 → 0x2D3210 (identity axes at the
point). Not ported: dynamic groups (0x100B40..). elParticle: see "elParticle and tyre spray".

**Effects** (one module per .cpp; the idb's region labels are shifted, a file's init/update often sits in the
neighbour's range):
- `skid_mark_render` — SkidMarkRender.cpp (init 0x1382F0 in the RainSurface range): 30 chunks x 20 vertices ring,
  strips (u = vertex & 1, v = vertex & 2), half width 0.3, 0.08 above the wheel's ground point, a segment every 2 m
  (squared 0x40808080), colour 0x40808080, "skidmark" blend 3 depth bias 0x40; front wheels (mask 9) crossing the
  start line in contact lay 1.5 s of "skid_flame" marks (blend 7, half width 0.15, alpha 0.4·255 fading over the last
  0.5 s; per-update fade r/g 0.5, a 0.2 of dt·255, static init 0x425DB0). `last` = the right edge of the newest pair
  (0x1396B0 copies before raising). Restart clears only the start line records (0x1398A0).
- `skidsmoke` — skidsmoke.cpp 0x139D50: per-wheel cooldown 0.02 s (0.05 on a slow flame), fires on sliding, a flame
  record > 0.5 ("TrailBlaze") or surfaces 12/13 spinning above 2 m/s; type by surface (11 GrassSmoke, 12/13
  GravelSmoke, else SkidSmoke / SkidSmoke_stop under 5 m/s) at wheel centre − radius/2, 1 m back above 5 m/s.
- `spark_render` — SparkRender.cpp (init 0x141730 / collision 0x141980 sit in the Slipstream range): 100 sparks,
  8-point ring, 4 styles round robin (width 0.025, ring 3/1/2/1 — `rand() >> 16` is always 0), materials −1/19/5/7/14/16/21
  with impulse ≥ 0.714, count = strength·0.2 ≤ 25, gravity 20, bounce 0.5 off the player's wheel plane, life =
  vy·0.175, alpha life·712 + 64 when dead (912 for 5/7), colour 0x4080FF / 0x2050FF in engine format (`CatsEyeRender`
  0x11DE40: low byte red, rgb ×2, alpha ×4). Positions are relative to the player's car and travel with it.
- `slipstream_render` — SlipstreamRender.cpp (init/update 0x13F380/0x13F770 in the SkyRender range): two sets of 8
  "streek01" ribbons (blend 7, alpha test ≥ 0x20) for the vehicles physics reports, origins `dword_731E00` (4 sides,
  4 roof), wobble/spin/rise tables in the module, alpha −0.1 per 60 Hz frame. Screen distortion: see stage 3.
- `start_line_effects` — StartLineEffects.cpp: professional races only, two rows of five "LensFlareSecondaryBlob"
  (blend 12, bias 256) 3.14 m above the line centre, red, then green and 0.44 higher on go; alpha = view · line
  normal. Fire: "startline_fire_segment" every metre of the fire point's path (type not shipped: warns).
- `car_render` traffic: `CarSpec::traffic("car01", colour)` = VehicleEntity 0x1C0D60 no-setup branch (TrafficM/T.vcd,
  body `<name>`, four `<name>_wheel`); core's `traffic_render` stand-in can go.
- Fixed in passing: a track without a sky (ExecTest) left the world camera uncleared (moving objects smeared, holes
  showed stale frames); the world camera now clears when no sky camera exists.

### Track flares and light streaks (`track_fx`, `light_streak`; `TrackFxPlugin`, automatic with the track)
- TrackFX object `dword_736958` starts with a LightStreak manager (its +0 = 150 is also the near/far split). Light
  info 32 B at +2204: +28 reflection id, +29 flare id (-1 none), +30 streak slot, +31 flags (2 reflects). Ids from the
  second PtLights name (0x9D860: from the second-last `_`, first number = flare, second = reflection, `w` = 99 none),
  minus 1, clamped (0x14AEC0). Setup.txt `"FLARES"` block (0x14D140): brightness ×128 → `byte_6872A4`, flare
  textures + sizes (`flt_687308`), reflection textures; defaults lightflare_orange/blue/orange_ray01 (10), spec3/glowtest.
- Render 0x14A5B0 per frame: lights with a flare whose sphere (r 3) is in view; depth = fwd·(p − cam) ≥ split → quad
  c = p − (R − U)/2 − fwd·0.2 (R/U = camera right/up (+88/+104) × size), "lightflare" blend 12, z test, bias 0x20,
  colour VISOS(0x80, b, b, b); nearer → streak record (≤ 16; when full the farthest is replaced if farther, and the
  split drops to that depth, 0x12FA80) + near flare pair. **Camera vectors**: R = VsBaseCamera +88 `m_right_RENDER` =
  up × dir = **bevy's left** (the far quad used bevy right before: texture was mirrored; fixed), +104 up, +120 dir,
  +72 pos; camera +8 = the view matrix (0x5D2E0 builds the world matrix with row 0 = −(+88), then copies the view).
- **Near flares** (`track_fx::near_flare`, 0x149970, done): x = (+88)·d, y = up·d, n = (x, y)/|(x, y)|, s = (−n.y,
  n.x); sizes lerp(10, 9, |n.x|) / lerp(5, 9, |n.x|) (`flt_6872B4`/`6872AC`/`6872B0`, CRim 0x452C0 = (b−a)t + a);
  axes A = W(n)·size, B = W(s)·size with W = transposed view (0x2ABD60 = D3DXMatrixTranspose, 0x2ABF70 row-vector
  3×3) = x·(−(+88)) + y·up — so the first quad points to the **mirrored** side (kept); quad c = p − (B − A)/2 −
  fwd·0.2: c, c+B, c−A, c−A+B, uv corners, colour VISOS(0x80, 0x40×3) (`byte_6872A8`); then n.x, s.y negated, second
  quad VISOS(0x80, 0x80×3) (`byte_6872A9`). Same stream = the light's "lightflare" shader. Light on the view axis
  (1/0) is skipped. Test `near_flare_pair`.
- **Road reflections** (`TrackFx::{setup_reflections, render_reflections}`, `reflection_strip`; done). Setup 0x149020
  (PhysicsManager::Init 0x19CE4B after the quad tree; else from the first 0x14A010 via +2212): front end (+2232 =
  CommonInit's arg) keeps the light positions, heights (4, 1) (`dword_687298/9C`), width scale[0] 0.25; in game
  heights (10, 5) (`687290/94`) and per light: flare ≥ count → 0, ray p → p − 50 up (MaterialProperties 0x19D310 →
  RayCast flags 17 = track + normal), hit: y −= t·50, flags |= 2, reflection id > 0: y −= t·50 again (mirrored below
  the ground); miss: id −1, y −= 10; id −1 clears bit 2. The resulting y goes into the point light's colour w
  (0x14DEC0, every light, `TrackFx::ground_heights` on `TrackPointLights`), then VsRender 0x5A040
  (`vs_render::reduce_light_overlaps`, game only): light records are 32 B {pos, w radius; colour, w}; distance
  0x5B200 = √(dx² + d(colour w)² + dz²) (0x5B1D0 = colour w: the ground height, not the light y); per light i with
  r > 0, 0x5A3B0 finds the first j < k overlapping i (r_i + r_j > d) that also overlap each other (0x59FB0 = r_a +
  r_b − d) → records (i,ja),(i,jb),(ja,jb), sorted ascending (0x5B2E0, predicate 0x5B780), both lights of the
  smallest lose 0.51 (`dword_629238`) of it, repeated until no triple (MAX_PT_LIGHT_SOURCES 2). LACoast: radii sum
  28805 → 9707, so car point lighting is much smaller in game than from the raw file. Here the setup waits for
  `PhysicsWorldRes` to hold a world.
  Render 0x14A010 (GameManager 0x0B93A9 after CarRender 0x1127F0, `byte_6872C1` = 1): bit 2, d² < 100², fwd·d ≥ 0;
  intensity 1, from 50 m 1 − (d − 50)/50; id 0: h = cam y − y, h ≥ +2216 skipped, h > +2220 × 1 − (h − min)/(max −
  min). Strip 0x149480: colour bytes _ftol(col·intensity·128) → CatsEyeRender 0x11DE40; p1 = camera with y = p.y,
  at most cam y − 1; side = up × normalize(p1 − p); p ∓ side·3·scale, p1 ∓ side·3·scale (`dword_6872C4/C8`,
  `dword_687330`), uv (0,0),(1,0),(0,1),(1,1). Shader "lightref" (147DC0, `byte_687283` = 1): blend 13 (rgb += src ·
  dest alpha), no z test/write, stencil EQUAL ref 1 mask 1 (glow bit; road textures such as asphalt_tile/c_pave1/
  flyover_roadbase carry `eMS_Stencil`) — not modelled: on glow views dest alpha is already 0 off glow surfaces. The
  64-strip counter (`v9[17]`) has no effect. Viewer `--lightref` loads the track collision; `VELOCITY_LIGHTREF_DEBUG=1`
  draws the strips with blend 12. Test `road_reflections`. Shot `research/screens/gfx_flares.png` (`trackview --track
  LACoast --lightref --size 640x360 --cam 322,-55,-392,-10,-5`: lamp flare pair, faint orange reflection on the road;
  the debug blend shows full strips from each lamp's foot to the camera). Dumps `research/gfx/nflare_*.{c,txt}`,
  scripts `tools/ida/gfx_nflare_*.py`.
- **Car headlights 0x14B030** (`track_fx::{render_headlights, headlight_reflection, headlight_beam, beam_shader,
  BeamCar, CarHeadBeams, HeadlightView, render_neons}`; GameManager 0x0B93C1 right after 0x14A010, render flag bit 2;
  done, pass 4 see "Car shadow and neon"). Shaders (init 0x147DC0, `byte_687283` = 1, misc.vcd textures; dumps `research/gfx/TrackFX.c`,
  `hb_dec*.c`, `hb_vals*.txt`, scripts `tools/ida/gfx_hb_*.py`): +2128 "hb" Hdlights2 blend 13 no z, +2132 "hb2"
  blend 12 no z bias 100 (`dword_687284`), +2136 "hbZT" 13 z test bias 100, +2140 "hb2zt" 12 z test bias 100,
  +2148 "headlightRef" headlight02 13 no z, +2156 "OneHeadlight" OneHdLgt 13 no z; blend 13 ones carry stencil EQUAL
  ref 1 mask 1 (glow roads, as "lightref"). (The `byte_687283` = 0 set "headlightbeam"/blend 7 is unused.)
  Car data used: +0 matrix (= the root `Transform`, nose +Z), +768 l_head, +1820 headlight level (1 from init; only
  CarRender 0x111B20 moves it), +1828 brake level, +2250 damage, +2251 culled (CarRender 0x1127F0: inactive or the
  chassis sphere outside the view, **or the camera is in this car**), +2328 bit 4 in-car (camera modes 1/2 on this
  car). Pass 1 (cars not culled): depth d = camera plane 4 (0x5C9C0: normal fwd through the near point) · pos; d < 80;
  dir = norm(pos − cam), dot = −z·dir; > 0 front (fade 40..80, damage 1/2 hide left/right), else rear (15..50, dot
  negated, damage 4/8, rear = 1 + brake·2, the **head locator with z negated**, original); a = (dot − 0.5)/0.5 ·
  (dir.y + 1)⁴ · fade, ≤ 1; points = car · (±head.x, head.y − 0.6, ±z); 0x14BE40: strip from p to the camera on p's
  plane (≤ cam y − 0.25), normalised (CSpoke 0x47840 = v /= len), length d·a + 1 ≥ 2, half width 0.3, colour
  (255,255,232)·a or red·min(a·rear, 1) → CatsEyeRender, far end bss 0. Pass 2 (`byte_687282`; shown = not culled ||
  in car || camera mode 3): intact cars, `dword_6865F0` (= `ZoneFx.zone_mask`) bit i set, p = pos + z·(head.z [+0.5
  in car]) − 0.35 up; 0x14C430 when level > 0: quad p → p + z·9 (12 in car), side = up × z · 4.5, colours
  VISOS(0x20,48,56,56)·l → 0 (in car (0x80,112,128,128)·l → (0x30,48,48,48)·l); car index 0 (front end −1) is flushed
  alone with hb + hb2, the rest with hbZT + hb2zt (a car skipped by the mask or damage also skips the flush check).
  Pass 3: exactly one broken headlight, not culled || in car: same quad, u mirrored when the left is broken,
  "OneHeadlight". Pass 4 (`byte_687280` = 1): the neon cars' quads (see "Car shadow and neon"). Input: `HeadlightView { camera_mode, inside: Option<request> }` (core request in core.md). Debug:
  `VELOCITY_LIGHTREF_DEBUG=1` also draws the blend-13 beam shaders with blend 12. Test `headlight_reflection_strip`,
  `headlight_beams`, `vs_render::overlap_tests`. Shots `research/screens/gfx_beams.png` (`trackview --track LACoast
  --lightref --car LA/350z,LA/RX8 --caryaw 0 --size 640x360 --cam 322,-55.6,-392,-45,-12`: faint, the road under
  the cars has no glow alpha) and `gfx_beams_debug.png` (same with the debug blend: beam cones and the white
  headlight streaks in front of both cars).
- **Car shadow and neon** (`car_render::{CarShadow, ShadowVehicle, shadow_quad, shadow_pass, CarSpec::
  with_setup_underbody}`, `car_palette::{carshadow_shader, car_neon_shader}`, `track_fx::{render_neons, car_passes}`;
  dumps `research/gfx/neon_*.{c,txt}`, scripts `tools/ida/gfx_neon*.py`). Quad 0x111B20(car, shader, colour): rows of
  car +0: x · −W (+1808), z · L (+1812) (`dword_6865F4/F8` 0.8 / 0.9 for any shader but "carshadow"); c0 = pos + z/2 −
  x/2, c1 = c0 + x, c2 = c0 − z, c3 = c2 + x, uv (0,0) (1,0) (0,1) (1,1), strip, identity matrix. With a vehicle (+1788):
  corner i over FastCar wheel `dword_686780` [3,0,2,1], ground point 0x1C68F0 = centre (0x361390) − body up
  (0x1BB200(1)) · radius (0x35FB40); race cars (+1784 0) with that wheel off the ground (0x364BB0) cast 0x19D350 (flags
  17) from 0.1 above it to 25 below (`flt_68677C`), y = from − t·25 (a miss returns: no draw, no level change); y +
  0.01 (`dword_686778`). Then VE+4388 ≠ 0 → +1820 −0.2 (`dword_6865E8`), else +932 = norm(c0 + c1 − c2 − c3), +1804 =
  (c0.y + c3.y)/2, +1820 +0.2 ≤ 1 when every tested wheel touched, else −0.2 ≥ 0 (no reader of +932/+1804 ported).
  Without a vehicle (front end) y = pos.y − +1816/2 + `dword_7184B0` (bss 0); +1816 = FE3DVehicles 0x25C890's wheel
  value (not decoded; the wheel diameter stands in). `dword_7065A4` (garage mode, bss 0) gates it to 0 / ≥ 6.
  +1808/+1812 = vehicle data +0x254/+0x258 (`VehicleData.unk_254`), 2.9 / 4.8 without a database entry (0x110220).
  Neon (0x115920 from 0x110220 when setup +0xCC ≠ 0): setup colour slot 1 (+0xC8) id ≠ crc "OFF" 0x425AC6DB
  (COLOURS.BIN 168, kind 3) → +2328 bit 1, +928 = slot +0x34, +2232 = slot +0x38. Shaders (CarPalette 0x10D7E0):
  CarRender +16 "carshadow" misc `shadow` blend 3 clamp, z test no write, bias 100 (`dword_686658`; 82 `68665C` in the
  showroom), stencil write mask 0; +4 "CarNeonDA" misc `neon1` blend 13 (dest alpha: faint off glow surfaces, as the
  beams), same z/bias, 0x2A17D0(1,0) unknown. Users: TrackFX pass 4 per neon car not culled: carshadow VISOS(0x60)
  (`dword_737490`) full size, then CarNeonDA in +2232 at 0.8/0.9; **CarRender 0x11A6E0** (car draw 0x119900, GameManager
  0x0B975D after TrackFX): a car whose `dword_6865F0` bit is clear counts as culled; culled + in car (bit 4): +1804 =
  y − 0.5, level ±0.2 by all wheels down && !VE+4388 (+932 = z); visible, initialised (bit 2), no neon: carshadow in
  `dword_706604` = VISOS(0x90) in game (`71823C`; front end `709D58` VISOS(0x60)), car 0 skipped under the
  "Graphics.Shadow.Silhouette" tweak `byte_6F5B7C` (off; its draw 0x1638F0 is empty). Every car (race, traffic) has
  `CarShadow` except front-end builds; the level steps scale by dt·60. Physics read directly (`PhysicsWorld::slot`).
  Viewer `--neon 169,172` (COLOURS.BIN records as neon). Tests `shadow_quad_corners`, `shadow_quad_rays`,
  `shadow_pass_gates`, `neon_off_record`. Shots `research/screens/gfx_neon.png` (`trackview --track LACoast --lightref
  --car LA/350z,LA/RX8 --neon 169,172 --caryaw 0 --size 640x360 --cam 322,-55.6,-392,-45,-12`: faint red glow under
  the 350z) and `gfx_neon_debug.png` (`VELOCITY_LIGHTREF_DEBUG=1`: red / green underglow bars).
- LightStreak (init 0x12E5C0 "spec2" blend 7 z test, width 192 TrackFX / 128 DangerZone; update 0x12E800, slot 120 B:
  8 samples {x·512, y·512, alpha 0.7}, +96 max(w,4), +100 z, +104 depth, +108 intensity, +112 light, +114 head, +115
  count, +116 colour = min(flare, 4)): matched records sample (0x12F0C0, fails for w < 1), new ones take free slots;
  drawn (0x12EDC0 → 0x12F250 ribbon, ≥ 8 screen units long, (1 − depth/150)·intensity, ramp to 64; colours
  head/tail table 0x4257E0 mixed by alpha⁵) before aging: every 0.1 s a sample is kept, else the head is undone;
  unseen slots shrink then free; alphas −dt. Screen points are unprojected at the head's eye depth into the world
  camera (the original draws pretransformed into a 512² space). Flares need lit (night) tracks to show.

### Full-screen effects (`full_screen_effects_manager_pc_xbox`, `motion_blur_pc_xbox`; added by `RenderPlugin`)
- **Correction**: 0x15B2F0..0x15D250 *is* FullScreenEffectsManager_PC_XBOX.cpp (asserts at lines 286/355/1206), not
  mislabelled. Glow 0x15EF70..0x15F9B0, lens spray 0x15F9F0..0x160100, motion blur 0x160120..0x161050 follow
  GameShaderProgramManager alphabetically (no asserts, file names unknown). DangerZoneRender.cpp holds the DOF
  (0x120B20..0x1212D0, "DOFBlur1/2" in CarShaderManager 0x158F90), the danger ring warp 0x11F860, the "Danger"
  overlay 0x120850, a screen colour fade 0x123250 (`flt_7190FC`: alpha, rate, colour; "Graphics.Fullscreen Blur").
- Manager `dword_738570`: own 8 program pairs (vertex `off_687590` Copy/Blur/BloomStart/Blur/Copy/PointBlur/
  PointBlurWithAlpha/GrayScale, fragment `off_6875B0` Copy/Blur/BloomStart/Bloom/Colour/PointBlur/PointBlurWithAlpha/
  GrayScale; input bindings in `vertex_bindings`/`fragment_bindings`), 2 half + 2 quarter "OffScreenBuffer"s (fmt 18 =
  linear A8R8G8B8, `0x15C920` get / `0x15CA70` release), quad draw `0x15C240(tex, shader, prog, srcRect, dstRect, z,
  colour)` -> 0x15C3C0 (clip-space dst with a D3D half-pixel shift — dropped —, uv = src rect × texture size, i.e.
  texels of a linear texture; z via 0x15E740). Globals: 11 SourceTexture, 13 AlphaTexture, 14 fIntensity, 15
  fFocusPoint, 17 ConstantColour, 18 fBlurAmount, 19 fSourceTextureSize = (w,h,1/w,1/h) of global 11 (0x12BD80), 20/21
  fViewportSize / inverse of the current render target (x,y only, 0x12BE80/0x12BF40).
- Bevy side: the `TrackCamera` (and its sky cameras) is redirected into a "BackBuffer" image (`Rgba8UnormSrgb`, size
  of the final target); each queued `FsePass` gets a camera (orders 2..8, layers 24+) + clip-space quad with
  `FseMaterial` (translated FSE programs; sampled render targets re-encoded to gamma, `tex[s].w`); the last pass
  copies the back buffer to the final target at order 9 (HUD from 10). Passes into the back buffer use the world
  camera's msaa + `depth_load_op: Load`, so they share its colour and depth textures (bevy keys both by target+msaa) —
  that is how the z-tested composite works. `FseCameraPlanes` = original near/far (default 0.5/600) to place D3D-z
  quads in bevy reverse z.
- **Camera orders**: sky passes `WORLD_CAMERA_ORDER - 4..-1` (-44..-41), world camera `track_render::WORLD_CAMERA_ORDER`
  (-40), up to `MAX_PASSES` = 39 passes at -39..-1, present at 9, HUD 10+. While FSE is active, `redirect_world` gives
  the world and sky cameras the world's `Msaa` (one shared main texture) and **every** camera on the back buffer or an
  off-screen buffer `CameraOutputMode::Write { blend_state: Some(REPLACE) }`: bevy alpha-blends each camera after the
  first on a target by its main-texture alpha (the real cause of the old "lilac car" / sky-through-world bugs and of
  ping-pong passes onto the same buffer). The present pass is blend 0 (rgb) over a `Color::BLACK` clear so the HUD
  cameras composite over alpha 1. `FsePass`/`FseShader` gained `z_func` and `ffp` (fixed-function stage 0:
  MODULATE(diffuse, texture), alpha = diffuse; `ProgramParams.alpha.x` flags it in the FSE wrapper).
- **Glow stencil in alpha** (`shader_manager::{apply_glow_stencil, GLOW_STENCIL_VIEW}`): SetShaderStage 0x2A4960
  keeps STENCILPASS = ZERO (xbox op 0; 0x2A6FA0: 0 REPLACE, 1 ZERO, 2 KEEP; 0x2A6EA0 func map 0 ALWAYS, 1 NEVER, 2 LESS,
  3 LESSEQUAL, 4 EQUAL, 5 GREATEREQUAL, 6 GREATER, 7 NOTEQUAL; xbox render states 67 colour write, 69 pass op, 70 func,
  71 ref (OR-accumulated), 72 mask, 73 write mask), so every draw clears the stencil it covers. Bit 1 = glow: data
  `eMS_Stencil` (code 6; all 607 records carry the flag → ref |= 3, REPLACE, write mask 0xFF; `_glow`/`_bloom`
  textures: car lights, wheels, track `_bloom`, sky, chars — census test `stencil_census`, ignored, writes
  `research/gfx/stencil_census.txt`) and the code-built ShinyCar 0x156C00 / BrakeLight 0x156A60 (ref 7 + 0x2A12C0(0) =
  REPLACE; CarShader 0x156520 uses ref 4 = not glow). Blend 2 without stencil: rgb write only + KEEP; blend 0 with
  stencil → blend 1 (rgba). No dest-alpha blend mode (5/6/8/10/13) occurs in the data. Emulation: on views carrying
  `GLOW_STENCIL_VIEW` (= `ShadowFilteringMethod::Temporal`, the only free view-key bits; world, sky and back-buffer pass
  cameras while FSE is active, marker `FseGlowView`) non-glow pipelines write alpha 0 (alpha blend factors zero), blend 2
  masks alpha, glow pipelines write their alpha per the blend table. `StageState.stencil` is in `XboxStageKey` /
  `ProgramKey`. Track without sky: world clear colour alpha 0 (the cleared stencil).
  **Correction**: LightStreak 0x12E5C0 sets **no stencil** (SetTexture spec2, SetBlendMode 7, SetZState(1, 0, 0),
  CreateShader — disassembly checked, `research/gfx/ls_stencil*.{txt,c}`); its draw 0x12EDC0 uses shader +1932 +
  4·`dword_71FF10` (bss, never written = +1932, the init's). So streaks are not road-only; the old "EQUAL ref 8" note
  was wrong and nothing is missing. Bit 4 (cars):
  see "Car stencil"; bit 8 (RainSurface, RefPaste, puddles): see "RainSurface road layer".
- **Car stencil (bit 4)** (`FullScreenEffects::car_stencil`, `CarStencilCamera`, `shader_manager::{apply_car_stencil,
  CAR_STENCIL_VIEW}`). Stage fields (ShaderManagerBase setters → stage +28.. → CompileStage 0x2A3320 codes 15/19):
  0x2A10C0 func (+60), 0x2A1140 ref (+68 byte), 0x2A11C0 mask (+72), 0x2A1240 write mask (+76), 0x2A12C0 pass op
  (+64; 0 REPLACE, 1 ZERO, 2 KEEP), 0x2A1340 = z func (+80), reset 0x2A0AA0: ALWAYS, ref 1, masks −1, ZERO. Writers
  of bit 4: ShinyCar / BrakeLight (ref 7), CarShader 0x1564A0 (ref 4, clears glow), ShineyWheel 0x156520 (ref 4,
  write mask ~3: glow bits kept) — every car, not only the player's. Readers (NOTEQUAL ref 4 mask 4): "ZoneWobble"
  (zone ring) and "MotionBlurStencilToScreenBlit" (0x160BA0 flag 8, write mask 0). The danger zone's shaders set no
  stencil (none of its effects is kept off the car); its overlay copy-back (Default shader, ALWAYS/ZERO, whole screen)
  clears the stencil (`clear_car_stencil`). GlowStencil/GlowFinal have write mask 0, so the mask survives the bloom.
  Emulation: `StageState.stencil_car` (keys of both materials; CarShader/ShineyWheel prims get a stage-material copy
  via `Builder::car_stencil_material`). The first stencil-tested pass of a frame queues an alpha-only clear of the back
  buffer (`FSE_STENCIL_CLEAR`; the bloom has consumed the glow alpha) and a scene slot whose order goes to
  `CarStencilCamera` (child of the world camera, same projection and layers, back buffer target, shared msaa
  colour/depth, depth Load, `CAR_STENCIL_VIEW` = `ShadowFilteringMethod::Hardware2x2`, never the default). In that view every pipeline keeps
  its z test, writes no z / colour, and writes alpha 1 (shader def `CAR_STENCIL_WRITE`, after the alpha test) for car
  sets, 0 for other draws, nothing for blend 2 without a stencil record (KEEP) — the original's last-writer stencil
  replayed at equal depth (same vertex path, LESSEQUAL). `FSE_STENCIL_NOT_CAR` passes `textureLoad` the back buffer
  image (= the previous camera's output) at the fragment and discard on alpha 255 (1..254 = RainSurface bit 8). Costs a second world draw on
  frames with zone FX or in-game motion blur. Assumed: off-screen targets have no shared stencil. Shot:
  `research/screens/gfx_stencil.png` (LADown, five cars, `VELOCITY_ZONE_FORCE=40`: grey ripples stop at the outer
  cars' outlines).
- **Bloom** (`glow_pc_xbox`, region 0x15EF70..0x15F9B0, name guessed; `GlowTweaks` = "Graphics.Bloom" 0x15F820, defaults
  0x15F360: on, 5 passes, intensity 0.3, display blur off, stencil on read on; `VELOCITY_BLOOM=0` turns it off). 0x15F4B0
  (GameManager 0x0B9010 after the barriers, flag 0x2000): half buffer (0x15F2D0), global 14 = intensity, 0x15F600:
  GlowStencil (emulated, no draw), BloomStart (rgb·4 · a·a·0.3) back buffer → half ("GlowScreenCopy": blend 1, alpha test
  dropped — equal to the original's cleared buffer), `blur_passes` 0x15C770 (manager shaders +280 "GlowBlur1" / +284
  "GlowBlur2", both blend 1; per pass half → quarter with Bloom = keep the max-alpha of 4 taps at ±0.0035·size, quarter
  → half Copy), "GlowFinal" (blend 14) half → back buffer with Copy (v0·t0, colour `dword_77FD9C` = white). 12 passes.
  The glow object's own GlowBlur1/2 shaders (+8/+12) are never used. Shot: `research/screens/gfx_bloom.png` (LADown,
  350z: wheels, paint, lights, window `_bloom` textures).
- **Depth of field** (`danger_zone_render`: object `dword_718BA8` 0x120B20 init / 0x120BF0 set_focus / 0x120CF0 set /
  0x120D20 gate / 0x120E80 update / 0x120E30 coc; draw 0x158F20 → 0x159720 → 0x159550 in the CarShaderManager range;
  "Graphics.DOF" tweaks 0x1212D0; tables 0x6868B8.. dumped to `research/gfx/dof_consts.txt`). Gate: enabled, active and
  (force +5, cut scene +4, replay, camera mode 3, `dword_749110 == 2`, `byte_718E70`) — so never in plain racing. Focus
  d (mm) = car distance (+3) / replay type 3 4500 / (+48 − offset)·1000, ≥ 1200; f = 90/tan(fov); coc = |(d/A)·f²/(d−f)|
  ·blur·100 (A = 2000 near, 250000 far); 3 layers each side, spacing tables 0x68690C (game) / 0x686918 (cut scenes),
  alphas 128/80/48 · sqrt(coc). Draw: back buffer → quarter Blur → half (one 0x15C770 pass), then planes at view depth
  (D3D z from the camera planes, clipped outside 0..1): before the focus "DOFNear" (blend 2, z GREATEREQUAL), behind it
  "DOFFar" (LESSEQUAL), colour VISOS(a/2, 0x80, 0x80|debug 64, 0x80), fixed-function stage. Input
  `DepthOfFieldInput` (core request); `VELOCITY_DOF_FORCE=<focus m>` forces it (checked with 1.5: far planes at 2.3 /
  3.8 / 13.5 m).
- Motion blur: amount `0x12FB60` (view +20 factor = mph ramp 65..120, +24 amount = mph/130 × 0.0075 capped, +28 depth
  20; VE+0x44 over 0.25..0.75 raises the factor, depth += 50·g; factor × (1 − max(2·fury, 2·zone))), drawn when the
  factor > 0 by 0x160FF0 → 0x160BA0(flags 8): focus 0x160920 = car pos + last grounded velocity × 1000 projected,
  x/y clamped [-0.5, 1.5]; intensity = amount·120·(1−1.2|fx−0.5|)·(1−1.2|fy−0.5|); c4 = (−0.5, −0.02, −0.04, −0.06),
  c5 = (1, −1, 0, 0); back buffer → half buffer PointBlur ("MotionBlurBlit" blend 1), half → screen
  PointBlurWithAlpha with "spec2" (misc.vcd) as AlphaTexture, alpha = (1−spec2.a)·intensity, blend 2, z test at
  0.995, not over the cars ("MotionBlurStencilToScreenBlit", see "Car stencil"). Input: `MotionBlurInput` (core, see core.md);
  `VELOCITY_MOTION_BLUR_FORCE=1` = the `byte_71FF38` force flag (focus stays at screen centre without input).
  Shots: `research/screens/fse_before_driving.png` / `fse_after_driving.png` (forced).
- **Fixed: black headless captures.** bevy's `camera_system` recomputes a camera's target size only for new cameras,
  projection changes and image asset events — not when the `RenderTarget` component is swapped. The pass cameras
  are spawned on the (absent) window and then moved onto images; onto the new back/off-screen buffers that worked
  (their `Added` asset event), onto the viewer's existing final image it did not, so the present camera had no
  viewport and drew nothing. `execute_passes` now touches the pass camera's `Projection` until its viewport matches
  the target. The FSE slots also get a degenerate 4-vertex strip instead of an empty mesh (the 8 boot-time
  `slab_allocator` errors core reported are gone). trackview runs with the FSE plugins again.
- **Fixed: black FE hangout (fecapture career/teams, every foyer shot).** `CAR_STENCIL_VIEW` was
  `ShadowFilteringMethod::Gaussian` = bevy's **default**, which `MeshPipelineKey` also gets for cameras without the
  component. So every plain Camera3d (the frontend's `FoyerCamera`, and a world camera whose FSE is off — the FSE
  removes the marker) was keyed as the car-stencil view: no colour, no z. Present since the car-stencil stage (in
  8e3c615 already, so no build bisect was needed). Now `CAR_STENCIL_VIEW = Hardware2x2` (only ever inserted
  explicitly, on `CarStencilCamera`); plain = default Gaussian, glow = Temporal. Test
  `shader_manager::view_marker_tests::default_view_is_plain`. Also `lib.rs::refresh_retargeted_cameras` (PostUpdate,
  before `CameraUpdateSystems`): touches the projection of any camera whose `RenderTarget` was swapped or that sits
  unsized on an existing image — the same bevy limitation as above, generalised (fecapture retargets the foyer camera
  a frame after spawning it); test `retarget_tests` (positive control asserts bevy still needs it). Shot:
  `research/screens/gfx_hangout.png` (fecapture career "THE SCENE": Tory, the 300ZX, crowd; dark as authored).
  Capture check requested from frontend (frontend.md). Puddle mist: done (see "Puddle mist").
- GameManager 0x0B9010 FSE order per view: "Danger" overlay 0x120850 (early), bloom 0x15F4B0, motion blur 0x160FF0
  + slipstream distortion 0x13FDB0, streaks 0x123E60, DOF 0x120D20, 0x123510/0x1238A0/0x1235F0, flag 0x4000000:
  0x162D10 (zone FX) + 0x1205E0, stateful particles, (bloom here when `byte_6F5B7F`), flares, 0x15FB30 lens spray,
  fades 0x127080 / 0x123250. Bevy order: danger overlay → `GlowSet` → `MotionBlurSet` → `DofSet` → `ZoneFxSet` → danger
  blur/warp inside `FseSet::Build` (trackview adds `ZoneFxPlugin` / `DangerZonePlugin` too).
  Dumps: `research/gfx/zone_dec.c`, `zone_dis.txt`, `danger_xref.txt`, `danger_dis.txt`, `tb_dis.txt`, `rtc_desc.txt`
  (debug-build `_RTC` frame descriptors name stack locals: e.g. 0x162D10 "ZoneData", 0x1205E0 "renderSetup").
- **Screen strips** (`FullScreenEffects::draw_strip`, `FseVertex`): GameShaderProgramManager 0x15E780 begin / 0x15E990
  vertex (x, y in 0..1 of the target = ×512 into the 512² pretransformed space, u, v, D3DCOLOR) / 0x15EA60 (stream of
  ≤ 1000 = 0x3E8 vertices, the rest dropped, `MAX_STRIP_VERTICES`) / 0x15E7B0 draw (one D3DPT 6 triangle strip, shader
  texture word = a render target). `FsePass.verts` replaces the quad; uvs are the stream's as written (texels when the
  program passes v9 through). `prog::DANGER` (8) = the game's Danger.vxu/.pxu pair run through the FSE layer (c0..c3
  identity = matWorldViewProj of a screen strip, c4 texture0Size, c10 intensity, t1 = global 11 carried in the alpha
  slot). `bind_pass` now writes device constants first and the program's bindings over them (BindShader order).
  `motion_blur_pc_xbox::render_from(current)`: 0x160BA0's source/composite is the *current* render target.
- **Zone FX** (`zone_render_pc_xbox`, ZoneRender_PC_XBOX.cpp, object `flt_737AB0`; init 0x162B60 and draw 0x162D10 sit in
  the ProjectedLightBeam range, level/tweaks 0x154BD0..0x155070 in tyrespray's). Tweaks 0x155070 "Graphics.ZoneFX"
  (FadeRateA/B 0.25 per frame `dword_687524`/`68752C` (+[1] negated copies), Min Dilation 0.4, Max Dilation Speed 75,
  "On/off" +20 and "Invert Colour" +40 unread), ".MinZone" `unk_6874F0` / ".MaxZone" `unk_6874BC` = 13-float ZoneData
  (WaveSpeed, WaveSize, Wave Frequency, Blur, GreyScale W/H, Wobble Ring Size, Ripple Width, R, G, B, Brightness, level;
  GreyScale/RGB/Brightness read by nothing), ".OneOff" speed +48 = 0.005/ms. Level 0x154BD0 (per player +24, 0..2): in
  zone = vehicle +89 (0x155990) and FuryFX 0x128680 ≤ 0; one-off VsAIRaceVehicle 0x23310 starts +41/+44 (ms clock),
  its end zeroes the level; < 1: data = MinZone with level, ±FadeRateA; ≥ 1: lerp Min→Max by level−1 (0x154F00),
  in zone move by FadeRateB towards min(speed/(75·0.448)·0.6 + 0.4 + 1, 2), else −FadeRateA; ≤ 0 → 0 and +44 = −1.
  0x154FC0 = level·0.5 (`ZoneFx::amount`, read by motion blur 0x12FB60, FuryInstance, LightStreak; `dword_6F5B70`).
  0x162D10: +44 += frame ms when ≥ 0; level step; 0x163740 writes the in-zone flag to the camera manager
  (`dword_6D55C8`+44p+41 → `ZoneFx.in_zone`); level ≤ 0: out → `dword_6865F0` = −1, done; in → bit cleared, StartZone
  unless one-off; out of zone → StopZone (every frame) (`ZoneFxAudio` message); one-off amount = sin(+44·0.005) (< 0
  ends it). Amount > 0: half buffer ← back buffer "GrayScale" (blend 1; GrayScale.pxu = 2·red, a 1), motion blur
  0x160BA0 flags 0 at Blur/2 *on the half buffer*, ring 0x162FF0 over the back buffer with "ZoneWobble" (blend 2, stage
  MODULATE = the Copy pair; stencil NOTEQUAL ref 4 mask 4 = not over any car, see "Car stencil"), release.
  Ring 0x162FF0 (RTC names nTextureWidth/Height, RGBA): clock +32+4p += WaveSpeed; outer 0.7, level < 1 → outer·level,
  ring size·l² + (1−l²)·0.5, fade; inner 0.5 − ring size, n = (int)((outer − inner)/ripple + 0.5) rings of 16
  (`dword_687670`) segments + 1, alphas (`_alloca(n+1)`) 0, 32, … capped 255 (`dword_687678`), fading = ×min((1−i/n)·4,
  1) (`dword_687674`); vertex pos (sin a·r + 0.5, cos a·r + 0.5), uv the same at angle a + sin(phase)·WaveSize, phase
  starts at inner/ripple·freq + clock, += freq per ring; innermost edge unwobbled (`byte_68766C` = 1); `byte_748C00`
  (bss) would pin the last edge at 0.7. Full MinZone = 31 rings = 1054 vertices, the stream keeps 1000.
  Order: GameManager calls 0x162D10 after the DOF when `byte_685E8C` = 1 (it is), else after the flares (`ZoneFxSet`
  after `DofSet`). Input `ZoneFxInput` (core request); `VELOCITY_ZONE_FORCE=<speed m/s>` forces it. Frame rate: the
  per-frame fades run as whole 60 Hz sub-steps of the real delta (no overshoot on long frames), the ring clock
  scales by dt·60. Shot: `research/screens/gfx_zone.png` (LADown, forced at 40 m/s: grey rippled rim, colour centre).
- **Danger zone** (`danger_zone_render`, object `unk_718AC0`, TrailBlazer's effect; init 0x1202F0 from InGameInit:
  "spec4Inv" (`\VFX\misc.vcd`), shaders +4 blend 7, +8 blend 1 + LinearCopy, +12 blend 1 + Danger vp 7 / fp 5; params
  0x11F590 at +72 = `DangerParams`, offsets in the struct). TrailBlazer 0x0C5F90 per update: on → +64 = &TB+96 (prev[5],
  cur[5]), +28 = 1, +48 = TB+92 success; off → +28 0, +52 1, +36/+32/+44/+48 0. 0x1205E0 (render, after the zone FX):
  record lerped by the timer lerp (`dword_74A430`+32): +36 = L3, +52 = clamp(1 − L3), +40 = L1, +32 = L2 (overlay), +44 =
  L4 (amount); L0 unused; blur = max(lerp(0, 0.012, +36), 1e-5); `dword_6868AC` = 7: bit 2 motion blur 0x160BA0 flags 4
  (no z), bit 4 warp 0x11F860. Warp: half ← back buffer copy; amount t (`dword_6868B0` override −1): inner radius
  lerp(0, 0.35) (success lerp(0.6, 0.35)), band lerp(0.1, 0.3) (scaled by t/0.1 below 0.1; success 0.3), uv push
  lerp(10, 6), freq lerp(1, 2.4); clock A += clamp((t−0.9)/0.1)·(−20)·dt, clock B += 0.5·dt (frame ms·0.001, not when
  paused); 16 rings (i/17), w = sin(f·2π·freq + A)·sin(f·π)² (0x120A60), colours 0xFFFFFFFF minus _ftol(32·tint·w)
  per byte (wrapping; tint 0.8/1/0.8); 31 points: radius + f_i·(sin(2.5·2πf)·sin((f + B)·2π))²·0.4·amount, uv = pos +
  w·band/16·push·(sin, cos); blend 1 over the back buffer. Overlay 0x120850 (early, before the bloom, render flag
  0x4000000): active and +32 > 0: colour 0x120100 = hsv 0x11F3A0 (hue 200°+180, sat 0.45·(1 − clamp((1−L1)/0.3)), val
  0.15; L1 < 0.7: hue 25°+180, sat 0.55·k, val lerp(0.15, 0.3, k)) → Armco 0x13940 `_ftol(x·255)`; half ← back buffer
  through Danger (intensity = +32), copied back with Copy (output alpha 0 — clears the glow mask, as the original).
  Input `DangerZoneInput` (core request); `VELOCITY_DANGER_FORCE=<amount>` forces every record value.
- Not ported: the danger blur's `a6` pass count (0x160BA0 ignores it), `GameShaderProgramManager__15E760` after the
  overlay, 0x127080 (FlareRender records, not a fade). Lens spray, slipstream distortion and the fade: see below.

### Fury FX — nitrous (`fury_fx`, `fury_instance`; `FuryFxPlugin` in RenderPlugin)
FuryFX.cpp = 0x001273B0..0x00128E40 (the idb labels it FlareRender__ / FuryFX__), FuryInstance.cpp =
0x00128E40..0x0012AE10. Dumps `research/gfx/fury_dec{,2..8}.c`, `fury_xr{,2,3}.txt`, `fury_vals.txt`,
`fury_gm_calls.txt` (GameManager 0x000B9010 call order); scripts `tools/ida/gfx_fury_*.py`.
- Object `byte_71A0D0` (init 0x00128100 from InGameInit = `RaceEffectsInit`; RestartRace 0x00128530 re-runs each
  instance's 0x00128A40 without releasing the old emitters — leak kept; 0x00128DC0 kills them = `RaceEffectsShutdown`):
  textures spec3/spec4Inv/spec4 (misc.vcd), shaders "bits" (unused), "beams" (spec3 blend 12 clamp, z test no write),
  "streak" (spec4 blend 2 wrap, z test no write, alpha func 7 ref 1), "mb_mask" (spec4Inv blend 4, z off; bound by
  0x00129580 then replaced by the blur's own); six 1844-byte instances at +36, **index = vehicle index** (view +28);
  beam records `dword_71CEB0` (40 × 156 B: 8 points, +128 base, +144 head, +148 count, +152 max 8), styles
  `dword_71A078` (+4 colour 48, +20 width 0.2; the only one `dword_71A0CC` points at). Tweaks "Graphics.NOS FX"
  0x001287B0: Fov Change 20 (`dword_686B5C`, read by the camera), Max Nitro Speed 65 (`dword_686B60`).
- Instance: +0 30 streaks × 52 B {pos (car-relative), +16 dir, +32 speed, +36 length, +40 alive, +48 u}, +1560 own beam
  (only reset), +1716 vehicle, +1720 views ≤ 1, +1724 active, +1725, +1728 nos = VE+88 ? `flt_75B27C` : 0, +1732 speed
  factor = clamp(fwd speed·2.25/65), +1740 streak count, +1744/+1748 ≤ 4 exhaust emitters {handle, started, offset}.
  **Amount 0x00128680** = active ? +1728·+1732 : 0 (`FuryFx::amount(vehicle)`, `view_amount(p)`).
- Per frame (GameManager, after RainSurface): 0x00128E40 per human player: speed factor; unpaused: nitrous on, zone
  amount ≤ 0 and +1728 ≤ 0 → that car's two beams reset; +1728 updated. 0x00129090: "nitrous" (PartFX.bin) emitters
  start while +1728·+1732 > 0, stop (0x002D4EF0 = stop + 1) after; matrix = car with translation car·(x, y, z − 0.1)
  of the exhaust part's (EXHAUST = part 10, car +1208 +440) LOD 0 `l_nos` / `r_nos` (`CarFlareLocators.nos`, z
  negated). 0x00128A40 also ors car +2252 bits (1 left, 2 right, 4 centre, 8 both; no reader found).
- Per view 0x001293E0: v7 = VE+88 && zone ≤ 0, v8 = +1728·+1732; unpaused, !v7, +1728 ≤ 0 → StopNos + off
  (0x001298E0, `dword_6F5B70` = 0); v7 && off → on (0x001297F0: +1728 = 0, all 40 beams, own beam and streaks
  cleared); v7 && !(`dword_6D4558` & 0x20) → StartNos (`FuryNosAudio`, flag mirrored as `nos_playing`); then
  **0x00129910** car beams; on && v8 > 0 → +1740 = (int)(30·+1732), 0x00129670 streaks.
- 0x00129910: every CarRender car not culled (+2251), +2304 = 0, not police (type 1 + 0x4000; the type-1 instance
  branch reaches no other car), amount = v8 > 0, i·2 < 40; traffic shown by traffic vehicle +32, +33 resets its
  beams (not surfaced: taken as shown / no reset); per side unless brake light broken (+2250 4/8): only vehicle 0's
  instance unpaused adds a point at `l_brake` (+896, x mirrored for side 1) — race cars car-relative 0x00127600
  (stored points move by (pos − base)·`flt_686B64` −0.25, base = pos), traffic world 0x00127560 — then 0x00127740
  draws with alpha +1728·v8. Ribbon 0x00127740 (≥ 2 points): 2n + 2 vertices oldest → newest, side = norm((p −
  next) × (p − cam))·0.2, caps norm(v × side)·±0.2, alpha byte (int)(128·a·(i/(n−1))²) with colour 48 →
  CatsEyeRender (red 0x60, alpha ×4), uv (1,0) / pairs (0,0)(1,1) / (0,1).
- Streaks 0x0012A2B0 (unpaused): pos += dir·speed, length += 6 while < 6, u += `word_71CEA4` (bss 0); behind the
  camera's plane 4 (near plane, n = view dir) or dead → speed rand(1, 2), length 0, pos = car rotation ·
  (norm(rand(−1,1), rand(0,1), 0)·1.1 with z = 3), dir = −car z, u 0 (VsRandRange order y then x; own CrtRand).
  0x00129E10: alpha byte (u8)_ftol(a·8) (so ≤ 16 after VISOS doubling — faint, original), colour pairs
  VISOS(a, 80/60/40/40, 40, 40), 8-vertex strip p ∓ (Y × dir)·0.25, p += dir·length·0.33, u steps 0x555, v 0/1.
- Late 0x00129580 (GameManager 0x000BA023, after the second bloom): on && amount > 0 → motion blur 0x00160BA0 flags 4
  at amount·0.0075 with the view's focus (`fury_blur_system`, after `danger_zone_render::DangerWarpSet`, before the
  lens spray). `byte_71FF60` = blur object +32 has no reader.
- Frame rate: the original steps per rendered (60 Hz) frame; streaks run as 60 Hz sub-steps of the real delta, beam
  points are added per due 60 Hz step at poses lerped from the previous frame, and a frame with no step due draws the
  ribbon as if one were added at the current pose. Motion blur / zone now read this frame's amount (the original's
  blur reads the previous frame's: 0x00128E40 runs after it).
- Inputs: physics read directly (vehicle = the view player's race car = p-th non-traffic `CarRender.request`);
  `ZoneFx::amount(vehicle)`; `EffectClock.paused`. `VELOCITY_FURY_FORCE=<fade>` forces player 0's VE+88 on with that
  fade. Core requests (core.md "Requests from graphics (FuryFX / nitrous)"): forward `FuryNosAudio`, camera to use
  `FuryFx::amount`, `dword_6F5B70` into physics. Tests `fury_fx::tests`, `fury_instance::tests`.
  Seen: smoke race `VELOCITY_FURY_FORCE=1 SMOKE_PREFIX=gfxfury SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=10` (exit 0, no
  emitter warnings) → `research/screens/gfxfury_3_driving.png`: radial fury blur, red brake-light ribbons behind the
  player car and the car ahead; streaks too faint to pick out (alpha ≤ 16). Log `research/gfx/fury_smoke.log`.
- **ZoneRender 0x00154FC0** (zone amount = level·0.5) was already ported (`ZoneFx::amount`, core fills
  `MotionBlurInput.zone` from it); only its fury input was 0.

### Driver, fade, lens spray, slipstream distortion (stage 3)
Dumps: `research/gfx/s3_*.c` (driver 0x119900/0x118DD0, fade users, lens 0x12D400..0x12D960 / 0x15F9F0..0x15FC60,
slip 0x13F380 / 0x140990 / 0x1629B0, sprite state 0x2AA770). Viewer: `trackview --track ExecTest --car LA/350z --caryaw
90 --cam 70,-10.8,142,0,-14 --lens 16 --fade 0.15` → `research/screens/gfx_driver.png` (driver in the cabin, drops,
fade).
- **Driver** (`car_render::{CarDriver, CarDriverInput, DriverSway, driver_locator, DRIVER_MODEL}`): CarRender 0x110220
  loads `driver01` from wheels_m.vcd (renderer +52) and, when the LOD 0 chassis has a "driver" locator, stores its
  translation with z negated at car data +912 and sets +2328 bit 0x800 (22 of 28 chassis; test). 0x119900 draws it per
  car (FuryInstance 0x12A7D0, no object lights → built like the wheels) when bit 0x800, +2304 == 0 (unknown; taken as
  0), not front end, (+1784 car type == 0 race car or `byte_7065A9`) and LOD 0. Matrix = car matrix with translation
  car·+912 = a hidden child of the root. `byte_7065A9` (CarPalette 0x10D580 = front end || single player in camera
  mode 3 || `dword_749110 == 2`) also sways car 0's driver, 0x118DD0: Δv = velocity (VehicleEntity 0x1C6480 =
  VsAIRaceVehicle 0x24200) − last; side = ±|z × Δv| (sign of its y), along = z·Δv (z = car row 2); targets pitch −0.5·
  along (≤ 0) / −0.1·along, roll −0.2·side; lerp rate 0.5 towards a larger |target| else 0.2; clamp pitch [−0.25,
  0.75], roll ±0.35; local = RotX(pitch)·RotZ(roll) (0x2AB7E0). Per rendered frame in the original → 60 Hz sub-steps
  of the real delta here. Traffic cars are type 1 here (`spec.traffic`; their locators are not looked up).
- **Fade** (`fade_render::FadeRender`, `g_FadeRender` = `flt_7190FC`): init 0x122FB0, reset 0x123030, start 0x123070
  (rate 1/t, t < 0 → alpha 0.999999), set 0x1230C0, update 0x123100 (clock delta, not paused, stops at 0/1), draw
  0x123250 (after the lens spray, before the OSD): VISOS(_ftol(a·128), colour) 512² sprite under 0x2AA770's state
  (SRCALPHA/INVSRCALPHA, z off) = FSE pass `prog::COLOUR`, blend 2. The alpha moves per game update and is drawn
  unlerped, as the original. Callers (core): CommonInit init, RestartRace reset, PostRaceSequence start(1)/(−1),
  respawn 0x22C90 start(delay), 0x18A40 start(−t), attract mode 0x0B3170 start(0.5)/set(1), 0x0BDEF0 start(1.25)
  when not fading. Front-end camera use (FECK2ElementCamera 0x2999B0) not wired.
- **Lens spray** (`lens_spray_manager::{LensSpray, LensSprayInput}`, `unk_71EC50`: 128 drops × 32 B {pos 512² +0,
  life +16, full +20, radius +24, active +28}, +4096 on, +4100 splash): update 0x12D520 per game update: speed =
  |cam − last|·0.5 (0 if the camera car's forward speed < 0), drops −1/24 life and slide out from (256,256) by speed;
  strength = speed/2.5 clamped [0.2, 1] from speed 1 (else 0); splash level > 0 → min(speed/2.5·40·level, 10) drops
  (radius 3..4, life 1), level −0.25; wet → every > 8 − _ftol(5·strength) updates _ftol(3·strength + 0) drops
  (radius 2..10, life 3; `flt_725E20` is never written). Draw 0x15FB30 (render flag 0x10000000, assumed set):
  back buffer → **quarter** buffer ("lensscreencopy" blend 1, Copy), then per drop 8 triangles (rim a, rim a + π/4,
  centre; 512² pos ± sin/cos·r; rim uv = clamp((x − sin·r·35)/512), (y − cos·r·20)/512 of the copy, centre uv = own
  spot) with "lensspray" (blend 2, alpha = diffuse = VISOS(life/full·128)). The 1024 three-vertex draws are one
  `FullScreenEffects::draw_triangles` pass (degenerate joins, cull off).
- **Puddles / lens splash trigger** (`puddle_render::{PuddleRender, PuddleInput, PuddleRoadLines}`, object
  `unk_720840` "Puddle Render Init", functions in LightStreak.cpp's range; dumps `research/gfx/st_puddle*.c`): init
  0x131690 ("puddle1"/"tyremist" shaders, +12 = 1 unconditionally, +13 = +12 || "ladown"; puddles only for wet
  "london" tracks) → 0x131AC0: from 25..50 m along race section 0's race line, ≤ 100 records of 48 B {+0 pos = lerp
  between the section's +8 / +12 lines by rand t (+8 · t), +12 w radius = (a+b)/4, +16/+20 random unit dir xz, +24
  race distance, +28 rand & 1, +32/+36 sides 15..25}, next += 25..50 (10 %: +200..400; VsRandRange = unit·(hi−lo)+lo).
  0x1326A0 (from the puddle draw 0x131F50 when +13): per car (car data +2251 == 0, +2249 ≠ 2, type ≠ 2): race section
  → 0x132A00 at the driver's race distance (+680): the last record whose race distance (`this`+44 at 0x132A69 =
  record +24; records start at `this`+20) < distance, else 0, or the next one (wrapping) when the car is not
  within radius + 2; wheels in contact
  within the radius → tyrespray 0x152860(car, wheel, 2.0), camera car (camera manager +0x60) → `LightStreak__133910
  (1.0)` = lens +4100 splash, mist 0x1331D0, VE +4872+w flag; no race section → wheels on surface VE +4868 == 1 do the
  same. Ported: generation, the test and the splash (`PuddleRender::update`). Seen in game (wet London,
  `SMOKE_WET=1 SMOKE_AREA=2 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=25`): log `lens splash: car 0 wheel w in puddle k at
  race distance d` (first hit after the splash level ran out) for puddles 0..4 in 25 s; frame
  `research/screens/gfx_splash.png` (drops are faint at 640x480).
- **Puddle quads** (`PuddleRender::draw`, `draw_puddles` in `EffectSet::Build`): 0x131F50 is called only from the
  RainSurface render 0x137090 → 0x162400 (before the wet-road reflection), which runs when RainSurface +19/+18 (wet
  track, or "LONDON" in the track name: CommonInit 0x136C50) or +16, `byte_6F7157 == 0`, view 0 camera mode ≠ 3 and
  0x162920 gets a half buffer (`PuddleInput.rain_surface_off` = the negation, core). Per puddle (sphere test 0x5E070
  skipped): corner = pos − A/2 − B/2, A = (dir.x, 0, dir.z)·+36, B = (−dir.z, 0, dir.x)·+32; strip corner, +A, +B, +B+A
  (VertexStream 0x2B95D0 with strip joins, format 14 = xyz + colour + short uv); uv templates (OSDManager 0xEFD30 u /
  0xEFD50 v, 0x42DD0 sets uv, 0x42E00 colour, 0x43FB0 position) = texture quarter `kind & 3` (u 1024/4096 wide), v 0..1;
  colour `byte_721B34` = VISOS(0x80, 0, 0, 0) (static init 0x425B70) — black, full alpha: the puddles darken the road
  by the texture alpha (puddle1 256×64 white, alpha means per quarter 32/28/98/123 — kinds 0/1 only, so faint). Type
  2 (never generated): alpha = _ftol(((1 − dy/|d|)²·`dword_686F0C`(1) + 0)·128), d = camera − pos. Shader = +4
  (`byte_72639B` is never written): PARTTEX "puddle1", blend 2, SetZState(0,0), alpha 0x2A0F40(6)/0x2A0FC0(1) (+0:
  ref 8), clamp, stencil EQUAL ref 8 KEEP; 0x2A1040(3) = stage dword +56, 0x2A17D0(1) = stage bytes +86/+87,
  0x2A15F0(1) = stage float +104 (1.0) — unidentified. **Stencil bit 8** = RainSurface's "roaddetail" layer (see
  "RainSurface road layer"): the quads are drawn untested (SetZState(0, 0), no bias — the old z-test + bias 0x40
  stand-in is gone) by the puddle road camera, kept to the marked road pixels (`EffectShader::road_stencil`).
  Viewer: `trackview --track London --puddles x,y,z,yawdeg[,len,halfwidth]` (straight stand-in race line,
  `StraightLine`). Shot: `research/screens/gfx_puddles.png` (`--puddles -644,6.62,60,180,300,8 --cam
  -644,25,60,180,-40`: faint dark patches on the square; A/B diff 505 darker pixels). Dumps `research/gfx/pud*_*`.
- **Runtime puddles** (`PuddleRender::{add, restart}`, messages `PuddleAdd(Puddle)` / `PuddleRestart`, system
  `puddle_messages` in Update; dumps `research/gfx/pud_add_dc.c`, `pud_restart_dc.c`, `pud_funcs.txt`): the only
  caller of 0x133750 is PhysicsObjectManager 0x1E1A20 (friction spot: kind 2, radius 15, 30×30, life 480, flags
  0xCC…CD). 0x133750 appends at +20 + 48·count (no bound check), qsorts by +24 (0x1337C0, comparator 0x133800: −1 /
  (a > b)), then 0x133A20 sets **RainSurface** (`byte_726388`) +18 = 1 — not a dirty flag: it turns the RainSurface
  render (wet road + puddle draw) on for the rest of the race (`RainSurface.wet = true`). Sort = stable insertion
  (CRT qsort may order equal distances differently). Record +40 life is never read; +44 flags bit 1 marks runtime
  records (generated ones: +40/+44 = 0). Restart 0x131EC0 (only from RestartRace 0x0BE20C; InGameInit 0x0B6EC5 is
  the init): from the last record down, while flag bit 1 is set clear it and drop the record; stops at the first
  without (original: runtime puddles sorted before a generated one survive). Kind 2 makes the draw's view-angle alpha
  (and the mist's fresnel colours) live for the first time: alpha 0 seen from straight above. `From<&PhysicsPuddle>`
  converts physics' record. Core wiring requested (core.md "Requests from physics (explosions, …)"). Test
  `runtime_add_and_restart`. Viewer: `trackview --track London --addpuddle x,y,z[,distance]` (also inits RainSurface
  and PuddleRender dry without `--puddles`); shot `research/screens/gfx_addpuddle.png` (`--addpuddle
  -644,6.62,85,100 --size 800x450 --cam -644,9,60,180,-6`: faint dark patch on the road ahead, 1022 darker pixels in
  the band y 207..284 vs no puddle).
- **Slipstream distortion** (`slipstream_render::{render_distortion, distortion_strip, DistortView}`): motion blur
  0x160FF0 = ribbons 0x13FDB0, blur flags 8, then 0x1629B0 with the blur's half buffer (`dword_748430`, now returned by
  `motion_blur_pc_xbox::render_from`). Drawn if trail 0 or 8 has ≥ 2 points; per such trail 0x140990: head p0, p1 =
  p0 + (point[head−4] − p0)·count/4, ± width in y (roof: +start height / RISE[4]·rise + start), both edges clipped
  against the camera planes (0x140820, bevy frustum sides + near), projected (0x15EC80: x,y 0..1, D3D z), stretched
  ×1.1, n = min(_ftol(len·32) + 1, 16) segments, uv = pos·size with v + sin(i + phase)·0.2·0.01 (phase −1 per update).
  "SlipStreamDistort" = CreateShader defaults (it resets after creating): blend 1, z test LESSEQUAL no write, alpha
  ALWAYS, MODULATE. Per-vertex z: `FsePass.vert_z` / `draw_strip_z`. 0x1407C0 (distortion instead of ribbons for
  single player when `byte_6870E0`) has no known caller. The ribbon update 0x13F770 now runs as 60 Hz sub-steps
  (`update_dt`) — it was stepped once per rendered frame.
  **Render check fix**: 0x13F770 / 0x13FF20 read the car matrix through `VsAIRaceVehicle__23F40` (car data +0, as
  CarDamageFX 0x781C0: model space, nose +Z — origins x = ±0.9 lateral, ribbons spin in the car's x/y plane), but
  `EffectCar.matrix` is the FastCar body (nose +X): every trail stood 90° off. `slipstream_render::model_matrix` =
  body · RotY(90°) (core's `MODEL_TO_BODY`) is now applied in race_effects. The trackview fx demo now turns its car
  roots by the same rotation (the cars drove sideways). Rendered (`trackview --track ExecTest --effects slip --fxat
  70,-13.4,133.6 --fxfollow 1`, `VELOCITY_MOTION_BLUR_FORCE=1`, strips tinted for the check): side bands from the
  rear wheels and roof bands from the bonnet stream back along the path, z-tested; the uv push (0.2 % of the height)
  is barely visible, so untinted the band shows mainly as the blurred half buffer.
  **Frame rate**: 0x13F770 / 0x13FF20 run inside the render 0x13FDB0 and read car data +0, which the car bridge
  0x96E20 rewrites every rendered frame by `timer.lerp`. `race_effects::render_car_matrix` now gives them the car's
  `CarRender` root `Transform` (request = vehicle id; core's `place_cars` writes the lerped pose there in Update),
  falling back to the 24 Hz `EffectCars` pose only without a root; the update moved into `draw_race_effects`
  (PostUpdate, `EffectSet::Build`), so trails sample interpolated positions. The trackview fx demo now mirrors
  the game: `EffectCars` + `race_effects::game_update` at 24 Hz of real time, roots lerped/slerped per frame,
  demo vehicles 20/21 (= requests, clear of `--car`).

### elParticle and tyre spray (stage: elParticle)
**elParticle** (`el_particle::*`, `ElParticlePlugin`; dumps `research/gfx/ts_el*.c`, fixed-type re-decompiles
`el_dec_out.c` / `el_dec_helpers.c`, disassembly `el_dis_out.txt`, scripts `tools/ida/gfx_el_*.py`). Of the ~520
functions most are std::map / list / vector / elSmartPtr templates (replaced by Vecs and handles); the computing
ones are ported 1:1. Naming: `elParticleType` = the 0x78-byte parameter object (vtable off_661408, ctor 0x2D0890,
effect block 432 B at +8 = the SFP record), `elParticleSystem` = an emitter instance (0x9C, ctor 0x2CECC0, vtable
off_66104C). **No SFP file ships** (PARTDEF packs hold only `.slp`), so every type is built in code: tyre spray,
puddle mist (LightStreak range 0x132D20..0x1341A0, adds a collision plane 0x133D20), CarDamage bonnet smoke.
- Effect block (`EffectData`): +8 max particles, +12 lifetime (0.5), +16 gravity, +32/+48 velocity/spread, +64 80 B
  default shape, +144 velocity mode (1 = vel + ball·spread per axis, 0 = ±spread.x along vel), +148 rotation mode
  (0 billboard, 1 velocity-stretched, 2 trail ribbon = segments + 1 nodes), +152 angle/spin ranges (degrees),
  +168 segments, +172 trail {time 0.1, end size factor 1, colour delta}, +196 emission (0 rate, 1 burst +236),
  +200 rate curve + jitter +232, +240 size curve + +272 (= 1 − randomness; scale = rand(1, it)), +276 colour curve
  (80 B) + random colour +356, +372 drag, +376/+380 random walk, +384 blend, +388 texture, +392 anim {u, v, rate
  lo/hi, +408 random start, +410 random jump}, +412.. planes, +424 flag 1 = velocity in the shape frame. Curves
  (`elParticleControlCurve_T`, 32 B / 80 B): 4 keys at 0, t1, t2, t3 (+loop, fmod); eval 0x2D0520 / 0x343E80; ctors
  constant 0x79750, keys 0x791F0, colour two-key 0x792E0 (inlined in CarDamageFX's range).
- Particle = 48-byte header {+0 age, +8 u16 released nodes, +10 dead nodes, +12 angle, +16 spin, +20 frame, +24
  frame rate, +28 walk (0.5), +32 colour scale, +36 size scale, +40 size step} + nodes ×32 {pos, +12 dead, +16 vel}.
  Emit 0x2CF310(s, pos, vel): frees the oldest while ≥ max; random draws in order frame, rate, colour, size, shape
  (0x320F30: 0 point, 1 ball, 2 disc, 3 box, through the shape matrix), velocity, angle, spin; pos −= system vel·s,
  pos += v·s, v += g·s; every node = node 0. Tick 0x2CFA50: release node k at age > k·nodetime (aged by the rest),
  angle/frame step, death when age − (released − 1)·**trail time** ≥ lifetime (kept), random-walk kicks, nodes
  0x2CFEC0 (move, planes: kind 0 bounce v += n(−1 − √k)(n·v), kind 1 kill, others report-only — the report
  callback type +32 has no user, gravity, drag 1 − dt·k, kick). System tick 0x2CEEB0: age += dt, accum += dt·rate,
  emit at `s = (accum − next)/v·dt`, next = rand(c ± jitter/2). Free 0x320950 copies the oldest into the freed slot
  (iteration order kept). Start 0x2CF100, stop 0x2CF170, finished 0x2CF190.
- Pools: shared buffer 0x32000 B (VsCommonMain 0x16E111) = 412 blocks of 480 B; a system appends to its last block
  else takes a block; out of blocks → `SacrificeParticle` 0x2C6840 frees the first block of the first system with
  the most blocks (`ParticlePool`, `BlockBuffer`). Random: `dword_6C2430` = 0x39D4C6BA, `s = (−24883·(u16)s − 3561)
  & 0x7FFF` (0x320BB0), range 0x320C60 = r·(hi − lo)/32767 + lo; frame jumps use CRT rand (own `CrtRand` here).
- Manager 0x2C5B70 tick: each system ticks, then is erased when (removed and empty) or (kept-for-removal and
  finished); add type 0x2C5D20 refuses a duplicate name crc, remove 0x2C5DC0; add system 0x2C6380(keep), remove
  0x2C63E0(immediate | stop + remove when empty). Map order (by pointer) → insertion order. The second container
  (+40, 0x319xxx) has no game user: not ported.
- Renderer 0x343360 per node k (one extra, unreleased node in trail mode): age − k·nodetime, colour = curve + random
  colour·scale (+ k·colour step), size = size curve · (scale + k·step); bytes 0x352010 `_ftol(x·255.99)`; Xbox quads
  0x318060 (6 verts, a→(u0,v0) b→(u1,v0) c→(u1,v1) d→(u0,v1), frame table 0x343840 ≤ 16 frames), billboard 0x317C80
  (right·sin a + up·cos a), velocity 0x317E90, ribbon 0x317AA0 (statics carry over: `TrailState`). Systems with a
  transform (+20, a pointer the owner keeps current) are drawn in it (0x343B40: transform·view): here the corners are
  built with the camera basis taken into the transform and moved to world. Shader per type (setShader 0x2D0D70 =
  "particleshader" `Particle\<tex>`, z test no write; 0x2D0F30 = caller's shader).
- Frame rate: GameManager 0x0B9F6A ticks with the frame ms·0.001 → the real frame delta (`EffectClock`, 0 paused).

**Tyre spray** (`tyrespray::{TyreSprayRender, TyreSprayInit, TyreSprayShutdown}`, `TyreSprayPlugin` in RenderPlugin;
object `byte_73750C`: +0 wet, +12 records (436 B per car = 4 × 108 + car), +16/+20 race/TFT cars, +28 init). Init
0x1523D0 (InGameInit) only when wet or the track contains "ladown"; per wheel 0x152B90: shader "mistspray" (PARTTEX
`tyremist`, blend 2, z test no write, stencil write mask 0), rear wheels 1/2 → "mistsprayworld%dx%d" (rate 8) +
"mistsprayback%dx%d" (20), front 0/3 → "mistsprayfront%dx%d" (20): type(150, 1, 0), no trail, jitter 0.3, started.
Update 0x1527F0 per rendered frame (before the particle tick): 0x153560 strength ·0.9 per frame (here
0.9^(dt·60), not while paused); wet → ≥ 1, else < 0.01 → 0 and skip (the last rates stay, kept); 0x154870 speed
factor = min(1.75·mph/240 + 0.1, 1) from 0.1 mph (mph = m/s·2.25), > 1 strength: ·(1 + 0.33(s−1)), size 0.5 + 2(s−1).
Front 0x153630 / back 0x153CB0: frame x = fwd × up, y = x × fwd (fwd = VE+4788 heading, flipped when reversing),
system transform = translation at wheel + 0.1·fwd, box shape across the wheel, throw −fwd·10 (back: −fwd·7 + 3 up,
gravity −10), sizes/colours/lifetimes in the module constants (`dword_687428..6874B8`, static init 0x426560..0x4268A9;
`dword_7374D8..E8` are never written = 0). World 0x154310: still mist 2 m ahead of the rear wheel, box ±1 along fwd,
sizes 1 → 20 over 4 s·speed, alpha 0.2 → 0. `set_strength` = 0x152860 (puddle hit 2.0, read from
`PuddleRender.wheel_wet` each frame). Wheel positions = `EffectWheel.centre` moved onto the lerped `CarRender` root.
Core: `TyreSprayInit`/`TyreSprayShutdown` requested in core.md. Viewer: `trackview --track ExecTest --effects spray
--fxat 70,-13.4,133.6 --fxfollow 0` (wet). Shot: `research/screens/gfx_spray.png` (grey mist around and behind the
circling 350z; hard horizontal lines = big world quads cutting the road, as on Xbox without soft particles).
Tests: curves, rand/pool, emission/ageing, planes, manager + sacrifice, uv/colour, billboard, spray factors, wet
spray lifecycle, dry decay.

**CarDamage bonnet smoke** (`car_damage_smoke::{CarDamageSmoke, CarDamageSmokePlugin}`, in RenderPlugin; done):
**never emits in the original** — no caller of `elParticleSystem::start` 0x2CF100 in the CarDamage range (callers:
LightStreak 0x132D20/0x1341A0, tyrespray, VsPoolAllocator only) and +156 ("running", checked by stop 0x76870) is only
zeroed (0x76730), so the systems are created and re-tuned every frame but stay silent. Ported that way;
`VELOCITY_BONNET_SMOKE_FORCE=1` starts them and counts 4 headlights. Lifecycle: `RaceEffectsInit` = create 0x766B0,
`RaceEffectsReset` = 0x766F0, `RaceEffectsShutdown` = 0x76780. Static init 0x421CE0..0x421E51 (`research/gfx/
bs_init.txt`, VsVector4 x,y,z): velocity `6D6998` (0.4,0,−0.2) → `6D67E0` (0.4,0,0), spread `6D6C78` (0.1,0.1,0.1) →
`6D6988` (0.2,0.2,0.4), gravity `6D67C0` 0 → `6D69A8` (0,0,−6), colours `6D67D0` (0.7,0.7,0.7,0.5) / `6D67F4`
(0.3,0.3,0.3,0), box `6D69F8` (0.1,0,0.4); `6D67A8/AC/B0/B4/B8` are bss never written (0); `6D67F0` = 43·0.0464 has no
reader here. 0x78040 = min(2·mph/240 + 0, 1). Per frame (z of velocity/gravity × sign of forward speed, left side x
negated, all through the frame x = fwd×up, y = x×fwd, z = fwd): system transform = translation, system velocity
(+104, 0x79190) = particle velocity, box shape; rate lerp(30,50) jitter 0.2, life lerp(0.7,1.2), size keys(0, 0.7,
1, life, lerp(1,1.5)) randomness 0.2, colour two(c0, life, c1), rotation (0, 360, −45, 45). Dumps `bs_dec.c`,
`bs_start.txt`, `bs_vals.txt`. Tests: level gate, tuning, silent until forced.

**Next (elParticle users not ported)**:
- (done, see above) CarDamage bonnet smoke — the systems belong to the CarDamageFX object `byte_6D6808` (CarDamage 0x769E0 create
  from 0x766B0 (CarDamage 0x6B8B0 / reset 0x766F0), 0x76E80 per frame from 0x768D0 = GameManager 0x0B9037 first thing,
  stop 0x76870, shutdown 0x76780; dump `research/gfx/el_cardamage_smoke.c`, constants + bss users
  `el_cardamage_smoke_data.txt`, inputs `bs_lvl.c` / `bs_obj.txt`): shader "BonnetSmoke" (PARTTEX `enginesmoke`,
  blend 2, z test no write, stencil write mask 0), "bonnetSmokeRight"/"Left" types (150 = `dword_6852E4`, rate 20
  `dword_6852E8`, jitter 0.5); per frame car 0 only: centre of the front wheels (+128/+320) ± model x·(−0.5), y + 0.5,
  z + `dword_6D67AC`; frame from VE 0x1C66C0 (heading) × up; `level` = 0x78090 = **a speed factor**, not damage:
  mph = VE+20·2.25 ≥ 0.2 → 0x78040(mph/240), else 0; scaled by (count +8 clamped 1..4)/4 as an **integer** division
  (so 0 until count 4, kept). Count +8 = 0x76970 = physics' `CarDamageManager.stats.headlights` (bones 0/7 breaking a
  headlight at the full shell; reset 0x766F0) — available, no physics request needed. Shapes, velocities, sizes,
  colours (two-key curve 0x792E0) and angle ranges (0x79120: 360, −45, 45) are lerped between bss vectors by the
  level: static init 0x421CEF..0x421E3F still to dump (`dword_6D6998`, `6D67E0`, `6D67C0`, `6D69A8`, `6D6C78`,
  `6D6988`, `6D67D0`, `6D67F4`, `6D67B0/B4/B8`, `6D69F8`), helpers 0x77F30 (float lerp) / 0x77F50 (vector lerp).
- (done) Puddle mist, see "Puddle mist" below. **Correction**: 0x1341A0(dt) / 0x135690 are not the mist but a
  camera **rain** effect (GameManager 0x0B9F43/0x0B9F4D): `byte_721DC8` = elParticle rain around the camera (box shape
  ahead, rate × +16464 level, collision plane 0x133D20 / 0x133CF0 0.25), `byte_721DC9` = ground splash sprites (1024
  ring of points around the camera, ground height RainSurface 0x136090, GenRenderer 0x2A7AE0 billboards, two textures
  +16396/+16400); dump `research/gfx/mist_dec.c`. Ported: see "Camera rain".

### Puddle mist (`puddle_render::{PuddleMist, MistHit, MistCar}`, in `PuddleRenderPlugin`; done)
Dumps `research/gfx/mist_dec.c`, `mist_vals.txt`, `mist_init.txt` (static init 0x4259C0..0x425B8A), scripts
`tools/ida/gfx_mist_*.py`. Object `unk_720840`: types `dword_7207FC` / systems `unk_7207DC`, 2 × 4 slots at
2·wheel + k, k = 0 car 0, 1 **every other car** (they share one system per wheel; the last car tested wins the tuning).
- Build 0x132D20 (from 0x131690 when +13, i.e. always): "puddle%dx%d" (k, wheel) types (100, 1, 0), velocity (0,2,0)
  spread (1,0.5,1), gravity (0,−10,0), shader +8 **"splash"** (PARTTEX `tyremist`, blend 2, z test no write, alpha
  func 7 ref 1, stencil write mask 0), rate 0, size 0.1, colour two((1,1,1,0.3), 0.3, (0.8,0.8,0.8,0)), life 0.22;
  systems added (keep 0) and started. Shutdown 0x1330D0 (destructor list 0x131AA0). Here rebuilt when
  `PuddleRender.generation` (bumped by `init`) changes.
- 0x1326A0 first zeroes every type's rate, then each wet wheel (puddle hit or surface 1) calls 0x1331D0(car, wheel,
  slots, puddle +28 == 2): from |v| ≥ 0.5, shape disc r 0.05 (0x1339B0: identity + translation, kind 2) at the wheel
  matrix translation (car data +128 + 64w) − 0.5 y, system velocity = v·2; y = min(0.2|v| + 1.5, 5), x =
  min(0.15|v|, 5), z = min(0.1|v|, 100); particle velocity = model x·(odd wheel ? −x : x) with y set, + v·z/|v|;
  scale s = min(0.02|v| + 0.05, 1) (type 2: min(0.03|v| + 0.1, 1.25)); spread (2,2,2)·s; rate 50; trail (0.5, 1);
  size keys (s, life/2, 3s, life, 4s) randomness `flt_7207C8` (bss 0); life = y·0.22; colour two((1,1,1,0.3) — type 2
  (1,1,1,0.75) —, life, (0.8,0.8,0.8,0)).
- Frame rate: 0x1326A0 runs in the render path; core calls `PuddleRender::update` per rendered frame, which records
  the calls (`mist_hits`, `tested`); `puddle_mist_frame` (PostUpdate, before the particle tick) applies them with the
  wheel centres moved onto the lerped `CarRender` root (as the tyre spray). Test `puddle_mist`.
- Viewer: `trackview --track ExecTest --effects mist --fxat 70,-13.4,133.6 --cam 70,-8,145,0,-25` (demo wheels on
  surface 1, no race section). Shot `research/screens/gfx_mist.png` (grey mist thrown up and out behind both cars).

### Wheel blur (`car_render::{CarWheelBlur, CarWheelSpin, wheel_blur_levels}`; done)
Picked after the mist as the next most visible item: it shows on every race car's wheels at any speed, every race.
Dumps `research/gfx/wblur_dec*.c`, `wblur_dec5.txt`, `wblur_112a50.c`; scripts `tools/ida/gfx_wblur_dec*.py`.
- Setup: CarRender 0x117980 (per LOD 0 wheel model): first "wheel_spokes" prim → prim type 10, its texture handle
  +2268 (car 0's name → `dword_7065AC`); first "blur_0" → type 11. CarRender 0x110220: `<dword_7065AC>_blur` from
  wheels_t.vcd (`byte_68661D` = found) → "wblur" (+32): stage flags 1, blend 3, z test no write, stage 0
  MODULATE(DIFFUSE, TEXTURE) / alpha SELECTARG2 TEXTURE, stage 1 SELECTARG1 CURRENT / alpha MODULATE(CURRENT,
  TFACTOR). Spoke fade shader +36 (CarPalette 0x10D7E0) has the same state, texture swapped to +2268 per draw.
- Level 0x116E30 (from 0x1134A0 when (`byte_7065A9` or car type 0), not front end, `byte_68661D`, +2304 == 0): v =
  |FastCar +1432 (wheel 0 ω)|·0.31830987; +2260 spokes = 1 (v ≤ 5) else 1 − min((v−5)·0.1, 1) and mask 1024 (normal
  spokes left out); +2264 blur = 0 (v ≤ 2) else min((v−2)·0.1, 1); mask 0x800 (blur prim never in the normal draw).
- Draw 0x116F60 (0x112A50 when spokes < 1 or blur > 0, needs both records): blur prim on the 4 wheels with "wblur" at
  TFACTOR VISOS(blur·128, 0,0,0); spokes prim with +36 at VISOS(spokes·128) while 0 < spokes < 1. Both use LOD 0
  records at every LOD (here: copies in the LOD 0 wheel group only). `byte_7184E0` debug override off.
- Extra pass (`byte_68661C` = 1, while 0 < spokes < 1): **draws nothing on Xbox** — 0x114C10 (wheel model, wheel
  matrix, ctx, `dword_686618` = 8) is an empty `__stdcall` stub (the PC multi-draw compiled out); 0x114A90 only fills
  the draw context (wheel · camera 0x11BA30 · scale(`dword_686600`) matrix, `unk_7181E4`) and sets renderer +52052 mask
  −1025 / +52048 = `byte_68660C`; `byte_7181FC` = colour from car +964 ·`flt_686614` (1.0) with w 0.1 (`dword_686610`).
  Not ported (dump `research/gfx/wblur_dec6.c`).
- **Shared texture (original quirk, kept)**: 0x110220 runs once at CarRender init and loads `<dword_7065AC>_blur`,
  which 0x117980 latches from car 0's (`this + 304`) spoke texture only; every car's blur discs use it and
  `byte_68661D` (it exists) gates every car. `latch_wheel_blur` (`WheelBlurTexture { car0, image }`): car 0 = lowest
  `CarRender.request` with blur records (race car 0 in game; viewers start at 1 / 20); its own `_blur` image goes into
  every car's blur material; no image → no car runs the blur. Blur copies are built without their own texture too.
- Input: `CarWheelSpin { omega }` on the root (core request in core.md); without it the wheels look stopped.
  Viewer: `--effects blur[=ω]` spins every car's wheels. Shot `research/screens/gfx_wheelblur.png` (ω 300: spokes
  gone, dark blur discs). Test `wheel_blur_thresholds`.

### Camera rain (`camera_rain::{CameraRain, CameraRainInput, CameraRainInit/Reset/Shutdown}`, `CameraRainPlugin`; done)
Object `unk_721DD0` (ctor 0x135EF0: +0 shader "rainq", +4 type, +8 system, +12 1024 × Vec4 splash points (w = life),
+16396/+16400 textures rain1/rain2, +16404/+16420 cover pos/dir, +16436 index, +16456/+16460 cover distances, +16464
level, +16468 under cover); functions 0x133AB0..0x135F70 in LightStreak.cpp's range. Dumps `research/gfx/mist_dec.c`
(0x1341A0 / 0x135690), `rain_dec.c` (helpers, tweaks 0x135120, constants 0x686F10..0x686FA8, RainSurface 0x136090),
`rain_dec2.c` (init, sprites), `rain_dec3/4.c` (projection, sprite draw); scripts `tools/ida/gfx_rain_dec*.py`.
- Init 0x133AB0 (GameManager::InGameInit): PARTTEX `RainQuad` / `rain1` (128×64) / `rain2` (128×128); "rainq" = blend 2,
  z test no write, alpha func 7 ref 1, stencil write mask 0. 0x133D20: type "RAIN" (256, 1, 2 = trail ribbons):
  velocity (0,−20,0) spread (1,1,1), gravity 0, trail (0.2, 1), rate 250 jitter 1, size 0.2, colour constant 1, life 20,
  collision plane y = −2 "splash" kind 2 (report only, no effect); system box (3) at (−5,10,10) size (5,0,5), transform
  = `dword_725E28` (identity). Systems added kept, life randomised (0x135630), `byte_721DC8` rain / `byte_721DC9`
  splashes = `VsAudio::IsTrackWet`, level +16464 = 0.25 (0x133CF0, also RestartRace). Shutdown 0x134120.
- Update 0x1341A0(frame ms·0.001) (GameManager 0x0B9F43, render flag 0x800000, not paused): start/stop the system by
  the rain flag; dt < 5 → level += dt·0.01 ≤ 1; camera travel since last frame (`dword_7260E4`). Rain: box at
  normalize(dir with y 0.6)·(15 + travel·5) + 1 up, axes (up × d, d × side, d), size (13,5,15) (cover: at cover pos −
  cam, y 0.6·ahead + 1, axes (−d.z,0,d.x)/up/d, size (19,0,6.5); cover distances shorten it); rate level·250, trail
  (0.25, 1), size 2, colour (1,1,1, 0.1·level), transform = camera translation, velocity −travel·7 with y −20, spread
  (16,2,2). Splashes: near 3 − 1.5, far travel·7.5 + 28; per point life −0.1; dead → follow camera (mode 1):
  fwd = rand·rand·(far−near)+near (skewed tweak on), side ±min(fwd/2 + 2, 12); else fwd −6..35, side ±12; p = cam +
  dir·fwd + (+88 = up × dir)·side, y = ground + 0.25, life 1. Ground = RainSurface 0x136090 type `dword_686F5C` = 4
  "Rays": ray (flags 9 = track, first hit) from p + up to p − 100 up, y = start.y − t·101; miss → driver 0's y
  (0x135FB0 +20). Frame rate: travel and the life decay are per rendered frame in the original → taken per 60 Hz
  frame of the real delta.
- Draw 0x135690 (always): BeginSprites 0x2AA770 (blend SRCALPHA/INVSRCALPHA, clamp, MODULATE, z test, z write off),
  points from 6 m on with rain1 (frames i & 1, 92×23 px = 46·(2, 0.5)), nearer ones with rain2 (i & 3, 140×35);
  GenRenderer 0x2A7AE0 → 0x2A7F90 → 0x2A9D50: pretransformed quad in the 640×480 back buffer, **top-left at the
  projected point**, fixed pixel size, uv bytes (u 0..width, v = _ftol(h·f/n)), colour VISOS(_ftol(128·level²·0.07·
  life), 0x80, 0x80, 0x80); each drawn point also calls CRT rand(). Here: the quad is unprojected at the point's view
  depth into the world camera (640×480 scaled to the target, `SpriteView`).
- Cover trigger 0x134EF0 (TriggerUpdate 0x181260) = `CameraRain::cover` (not wired: the trigger's 0x135D30 length /
  0x23D30 position are taken as given). Tweaks "Graphics.Rain" = `RainTweaks`.
- Viewer: `trackview --track LADown --effects rain[=level] --fxat 197,-8.6,20 --cam 197,-6,34,0,-8`. Shot
  `research/screens/gfx_rain.png` (level 1: streaks across the view, splash sprites over the road). Tests
  `camera_rain::*` (build/level, splash respawn range and ground, sprite size/uv, ray ground). Core wiring requested
  (core.md "Requests from graphics (camera rain)").

### TrailBlazerRender — zone trails (`trail_blazer_render`, `TrailBlazerRenderPlugin` in RenderPlugin)
TrailBlazerRender.cpp is 0x14DF70..0x152380 (the idb's "TrackFX__14DF70..150140" are its first half; 0x1523D0..
is tyrespray). Dumps `research/gfx/tb_dec.c` (init/update/draw), `tb_dec2.c` (helpers, constants 0x6873D4..),
`tb_uv*.c` (uv anim), `tb_misc.txt`, `ts_trailblazer_render.c` (build), scripts `tools/ida/gfx_tb_*.py`.
- Object `dword_7374CC` (singleton 0x0C04F0/0x0C0730, ctor 0x14DEF0): +0 trails (≤ 16 × 36 B), +4 count, +8
  intensity (TrailBlazer 0x0C6240), +12/+16 shaders, +20 enabled. Trail {+0 blocks, +4/+8 sampled points, +12 road
  line, +16 state 0 off / 1 growing / 2 shown / 3 fading, +20 next (0x151ED0: 1→2, 3→0), +24 lerped, +28 timer, +32
  previous (−1)}. Init 0x14DF70 (InGameInit) → 0x14E480: one trail per lane of every trailblazer section (0x151DC0,
  lines with > 1 point). Reset 0x14E2A0 (RestartRace). Calls: 0x1521E0(line, state, queue) — settled trails (0/2,
  0x151F10) switch at once via 0x151F50 (prev = timer if the state repeats, else −1; timers 0), others only queue
  `next`; the trailblazer follows its section's **lane 0** (`section + 8`), so `Render { spline }` resolves the crc to
  that line. Update 0x14E530 (InGameUpdate, not paused, +20): prev = timer; state 1 += dt/0.5, 3 += dt/3; ≥ 1 → next.
- Build 0x1501E0: n = `_ftol(ceil(len/2))` samples along the line (`len/n` apart, interpolated between points,
  plus the last point). Per sample: fade 0..1 over the first/last 3, two alphas rand(190..255)·fade (VsRandRange,
  own CrtRand here); ground probes = PhysicsManager::RayCast flags 17 (VsAITools 0x28460) c→c+2up, c→c−2up, nearer
  wins; both miss → top→bottom ray whose t is stored as 0 (ground = c, bug kept); a miss clears the alpha. Centre
  normal n → side = norm(dir×n)·0.85, up' = norm(side×dir); sides p∓side probed along ±2up'; the two ground points
  squeezed to 1.7 apart about p's projection, lifted 0.16 along their normals; u = triangle wave over 20 samples
  (×4096); each ribbon = ±norm(dir×n_side)·0.4. Blocks 0x151880: 16 segments, a new block repeats the last segment
  (strip break mask +960 never set; bounds +968/+984 never read). Colour = `dword_7374C4` 0x002040FF | alpha.
- Draw 0x14E710 (GameManager 0x0B9721, every frame): +24 = lerp(prev, timer, `dword_74A430`+32) unless prev = −1;
  base rgb = FuryInstance 0x12A680 bytewise lerp(0x002040FF, 0x00FF0000, intensity in 128ths); strips (> 3 vertices)
  edge0 v 0 / edge1 v 1. State 2 0x14F3A0: as built. State 3 0x14F960: alpha − _ftol((1−(1−t)²)·255), edges pushed
  apart by s = 9t + 1 (p0 = e0 + (e1−e0)s, p1 = e1 + (e0−e1)s). State 1 0x14E940: head = ((n−1)·2 + 100)·t, samples
  up to `_ftol((50 + n)·t)`; inside the 100 m head f = (d − head + 100)/100 → alpha ×(f/0.8 + 1) rising to ×2 at
  0.8 then to 0 at 1, s = 9f + 1, rise `flt_7374C0`·(cos((1−f)π)/2 + ½) (bss, 0); the sample index steps back one
  per block (original). `byte_7187E0` (+0.2 y) is only read (false). The stream is drawn twice: "zonetrailone" /
  "zonetrailtwo" = misc.vcd `streek01` (`\textures.vcd` does not exist; the name lookup finds misc's), blend 7, wrap u
  clamp v, z test no write, stage flags 4 and 0x2A15F0(1) unmodelled, each with an `eMS_UvAnim` (code 22) record
  {speed, 1, offset, type 1 = u}: offset = seconds·speed (0x3015F0; 0.25 / −0.47) added to u (SetShaderStage case 22).
  Anim clock = real frame time, not paused.
- Viewer: `trackview --track LADown --zonetrail Circuit\Race02[,intensity[,section]] --cam 339,-4.5,-64,-58,-15` reads
  the race's AISpline.spl trailblazer sections itself (`track_view_trail`; velocity_ai cannot be linked), builds over
  the track's physics collision and switches the section's trail on (24 Hz updates). LADown Race02: section 0
  0xA777883A (661 m), section 1 0x3E7ED980 (186 m, the smoke's spline). Shot `research/screens/gfx_trail.png`
  (section 1: two blue ribbons along the road, faded in at the start). Tests: flat-ground geometry, miss → hidden,
  state machine + queued fade, draw passes, colour lerp. Core wiring requested (core.md "Requests from graphics (zone
  trails, danger particles)"); not yet seen in a smoke run.

### Danger particles (`danger_zone_render::{DangerParticles, DangerParticleCall}`, in `DangerZonePlugin`)
- Object `byte_719168` (ctor 0x123450: +0 level, +4, +8 five handles, +28 active, +29 enabled; dumps
  `research/gfx/el_danger_particles.c`, `dp_dec.c`, `dp_ctx.txt`). **Stateless** types `off_6869A4` chase_debris /
  chase_explosion / chase_sparks / chase_fire_column_L / _R (LADown partdef.vcd). Start 0x123510: add_emitter
  (0x2D4A70; a failed add returns 385 and the original indexes past the pool — no emitter here), start (0x2D5800),
  looping (0x2D59B0); +28 = 1, level 0. Level 0x1234A0 (TrailBlazer 0x0C4FF0 per update): level > 0 and +29 → start if
  inactive, level = CRim 0x452C0 lerp(level, √l, 0.25). Stop 0x1238A0 (TrailBlazer 0x0C60E0): running → 0x2D5A10 stop
  (handles kept). +29 = TrailBlazer 0x0C63E0: 0 at the intro 0x0C6080, 1 from "TriggerBigParticles" (TriggerUpdate
  0x180640) / timeline item 0x17A750. GameManager 0x0B9DE4/0x0B9DF9 debug starts (`dword_6F6F3C`) not ported.
- Place 0x1235F0 (GameManager 0x0B9E24, per frame before the particle update): camera matrix rows +88 right = up ×
  dir (VsBaseCamera: bevy's **left**), +104 up, +120 dir, +72 position; offsets `dword_719118` (static init 0x4250F0:
  (0,−1,0) ×3, (−1,0,2), (1,0,2)) with x · lerp(7, 3.25, level), y · lerp(7.5, 3, level), z = `dword_719110` if ≠ 0
  (bss: never); emitter matrix = camera rotation at offset·M (0x2ABDE0, 0x2D5960); +28 = any running. Test
  `danger_particles_on_the_camera` (LADown types, gate, level, placement, stop).

### Dynamic envmap (`danger_zone_render::dynamic_envmap`, `car_shader_manager_pc_xbox`; `DynamicEnvmapPlugin`)
Dumps `research/gfx/envmap*_{dec,main,help,consts}.*`, scripts `tools/ida/gfx_envmap{1..7}.py`. **It is not a
scene render**: the back buffer as it stands before the cars (world + sky) is polar-warped into a 256² sphere map.
- Object `byte_709F00` = CarRender +56 (init 0x1214A0 from CarPalette 0x10D7E0): +0 target "Envmap" (0x2EEA10, fmt 18
  linear, 1 << `dword_686924`: 512 at init, 0x1598A0 remakes it 256² (camera mode 3 or "hi-res" tweak, on) else 128²),
  +4 saved target, shaders +8/+12 (blend 1/14, no tex), +16 Spec2 blend 14 z off, +20 Spec2 blend 21 z off, +24 blend 9
  (z default), +28 blend 1 z off colour op 4 = MODULATE2X (MapTextureOp 0x2A3230; CreateShader reset 0x2A0AA0 = colour
  and alpha MODULATE(texture, diffuse)), textures +32 Spec2 / +36 lightglow_white / +48 fresnel4 (misc.vcd), +56 hi-res
  state, +57 enabled (= `byte_6F7157 == 0`; front end 0), +58 drawn this car pass (init sets 1, so the first pass
  skips). Tweaks 0x122AE0 = `EnvmapTweaks`; constants `dword_686924..6869A0`, `687540..68757C` in the module.
- Draw: CarRender 0x119900 (start of the car pass, before any car) → 0x1598A0 if +57: once per run (`byte_687558`)
  0x121BB0 bends the player car's chassis + parts normals (all LODs) 0.15 towards the position direction (0x121C80);
  if +58 == 0: 0x159D00 = save target, clear colour 0, horizon y (camera dir with y = car +932's y, x/z × (1 −
  y²)/(x² + z²), car + 10000·d, 0x15EC80), rings 0x122400 (32 segments, disc 0..0.3, band 0.3..0.53; uv = screen
  circle of radius 0.5 in texels of the saved back buffer, v squashed by max(2h, 0.4), colour VISOS(0, b, b, b),
  ramp top/mid/bot 0.75/0.5/0.1 by height) with +28; blur 0x15C770 (1 pass); flags `dword_686948` 0x7F: 0x10 subtract
  (0x15C340 = Colour quad, VISOS(16, 0x80…), blend 9), 0x20 fresnel (fresnel4, Copy, VISOS(0x70), blend 21), 8 spots
  0x15A090 (0x121ED0: point lights ≥ 1 m above and within 60 m xz of the car, dir with y·1.5 through D3DTS_VIEW
  (GetTransform 0 via 0x12D2C0 = sub_439FB0), sphere map u = x/2s, v = y/2s, s = √((−z + 1)/2); colour ·fade·lerp(1,
  1 − s, 0.8)·0.75, alpha fade·1.2·(1 − s)²; quads 0.25·size, wider/flatter below; lightglow_white blend 14).
  0x159C10 at the end clears +58 ("display" draws the map at (0,0,0.3,0.3): `VELOCITY_ENVMAP_DISPLAY=1`).
- Sampling: CarPalette 0x10CE90 uses 0x1218A0 while +57: car · D3DXMatrixLookAtLH(camera +72, car pos − 0.16 y,
  camera +104), translation row cleared, · [−0.5, −0.5, 0; offset 0.5, 0.5], transposed → envmapMatrix (ShinyCar
  t1 rgb·0.6 added, alpha·0.8 = bloom; CarDecal t1 at 1). `byte_7190F0` (car · view) is bss, off. 0x1218A0's shader
  +12 texture swap has no visible user (skipped).
- Bevy side (since perf stage 3, see "Performance"): in-game cars go to `CAR_LAYER` 20 and draw in the transmissive
  phase (`StageState.car_pass`); the world camera's main texture is copied after its opaque pass into the
  full-size source (`EnvmapCaptureView`, `capture_envmap_source`; no extra view; one frame late). Was: a source
  view (sky copies + world child without cars, half size). The envmap passes are **early FSE passes** (`FsePass.early`, `set_early`, orders
  −57..−46, pool slots after the present slot; layers ≥ 64 need `RenderLayers::none().with`) — so the cars sample
  the map. FSE additions: `FseImage::Target` (`add_target`/`set_target`: own targets and plain textures;
  `srgb` false = gamma stored as is in `Rgba8Unorm`, the wrapper skips its decode via `prog.alpha.z`; non-linear
  sources get 0..1 uv as 0x15C3C0 does), `FseShader.ffp` is now `FFP_*` (DIFFUSE_ALPHA = the old bool, MODULATE,
  MODULATE2X). Deviations: opaque shadows/beams are in the source (the original draws shadows after 0x1598A0); car damage rewrites bent normals (the original's
  deformed vertices lose the bend too); two players: decals get no map (original: an undrawn target).
- Input: `DynamicEnvmapInit`/`Shutdown`, `DynamicEnvmapInput` (core request in core.md); trackview sends the init
  (`--noenvmap` off). Shot `research/screens/gfx_envmap.png` (`VELOCITY_ENVMAP_DISPLAY=1 trackview --track LADown
  --car LA/350z,LA/RX8 --colours 3,6 --size 800x450 --cam 197,-5,34,0,-8`: map inset top left, sky/buildings in the
  car paint). Tests `dynamic_envmap::tests` (rings, spots, matrix, bend).

### Fixed-function car sets (`car_palette_pc_xbox::{fixed_car_set, FixedCarSet, car_colour_gradient}`, done)
Dumps `research/gfx/cs_dec{1..10}.c`, `cs_dis8.txt`, `cs_tab.txt`, census `fixed_set_census.txt`; scripts
`tools/ida/gfx_cs_*.py`.
- **Override mechanism**: 0x001577F0 builds 5 set objects (vtable +4 = 0x00156440 for CarShader / ShineyWheel); 0x00157BA0
  (from CarShaderManager 0x001587D0 = car data +1092 chassis LODs, +1208 parts, **+1104 the 4 wheels per LOD**, and
  SetCarColour) writes the set's shader into **shader +0** of each primitive's shader. ShaderManager 0x002A4240: with
  +0 set it runs SetShaderStage 0x002A4960 on the own stage with a4 = 0 (no closing reset), then on the override's
  stage with a4 = 1 — the override's compiled records land on top, the own records stand where it emits none. The
  pass-0 texture comes from the own stage (+1556 from stage +4); code 10 never applies pass 0, so the sets' default
  texture (0x0011F260 = texture manager +4, white) is never bound. ShaderManagerBase stage = 340 B at manager +28
  (setter dword d → CompileStage `a2 + 4·(d − 7)`), passes 0..3 (MAX_PASSES 4): 0x002A1AC0(tex, pass, stage), 0x002A2100
  (arg1, arg2, arg0, pass, stage), 0x002A1DC0 (+212, code 17), 0x002A1990 TFACTOR (code 16), 0x002A1A00 pass count,
  0x002A12C0 stencil pass op ("A1::ShaderManagerBase"), 0x002A13B0 SetZState (+56/+57).
- **CompileStage 0x002A3320 emits only non-default records** (code 1 blend ≠ 0, 3 clamp, 2 when z off / an alpha func /
  z func ≠ 2, 15/19 stencil, 4 no z write, 5 bias, 16 TFACTOR ≠ white, 10/17/27 per pass, 12 ops when > 1 pass or
  not MODULATE(TEXTURE, DIFFUSE)). CarShader 0x001564A0 / ShineyWheel 0x00156520: reset, 0x002A0F40(2) (alpha test on,
  table 0x65CE30[1] = ALWAYS 0x207, ref 0), stencil ref 4 REPLACE (ShineyWheel write mask −4, stage flags 1, TFACTOR
  VISOS(1, 0.35, 0.35, 0.35)). So the only modelled change is code 2 = alpha test ALWAYS ref 0 + z LESSEQUAL over the
  own stage (`compile_alpha_test`, `StageState::apply_commands`; `Builder::car_stencil_material(mat, set)`): blend,
  clamp, z write and bias stay the model's. Census (per primitive, with +4 kinds): 8210 fixed-set primitives, 1630 with a cutting alpha test (GREATER
  0x28; mostly `vl_dummy`, the wheel rims `wheel0xx`, `seem01`) now draw uncut, as on the Xbox.
- Front end (CarShaderManager +12) ShineyWheel adds pass 1 (0x00156150): texture = 0x001577F0's argument 0 (no
  texture), LERP(TFACTOR; TEXTURE, CURRENT), alpha DISABLE, 0x002A1DC0(5) — not drawn (`FixedCarSet.passes`).
  Stage flags 1 is not modelled (also on ShinyCar, CarDecal, wblur).
- **"CarColour"** (`car_colour_gradient`, `car_render::{CarColourTexture, car_colour_image}`): 0x001577F0 registers
  32x32 format 6 and fills red alpha 0; 0x00157F80 (SetCarColour → 0x001589A0) refills with paint +2236 (top/bottom)
  and pearl +2240 (middle row): t = 2·|0.5 − row/32|, `(u8)_ftol(a·t + b·(1 − t))` per A8R8G8B8 channel, alpha 0.
  **Nothing samples it**: no car data stage names it (only HIN01 has a "CarColourSwach"), ShinyCar's fixed-function
  fallback that binds it (0x00156F86) is behind a constant `if (1)`, FEHangoutProgression's "CarColour" is an FE
  element crc. Built per car root anyway.
- **Override re-checked** (garage white interior / white seam lines): SetShaderStage 0x002A4960 code 2 writes the
  render states directly (143 ZENABLE, 57 ZFUNC, 60 ALPHATESTENABLE, 58 ALPHAFUNC `0x65CE30[(v>>1)&7]`, 61 ALPHAREF),
  CompileStage 0x002A3320 encodes 0x002A0F40(2) as `(2−1)<<1` = index 1 = ALWAYS, and 0x002A4240 runs the override
  last: the 1607 cut shaders really draw uncut on the Xbox (no LOD/hide flag; colour = own texture MODULATE diffuse,
  CarColour unused). In the 300ZX FE models they are only on optional parts (`vl_dummy` on bonnet1/spoiler/rear
  parts), not the stock body. Not the cause. **Cause: the interior pack.** FEAlloyEditor_Basic
  0x002590A0 loads `<car>\VEHICLEI.VCD` (seats, dash, dials, door jambs `<car>_interior[_gloss]`, engine, boot) before
  VEHICLET while FE object +15694 is set (1 from ctor 0x00258770 / reset 0x00259580; CarCustomisation's view cycle
  0x002527A0 / 0x00252830 sets it to `dword_7065BC == 3`, also gating the door models in FE3DVehicles 0x0025D600).
  The port never loaded it, so those prims drew untextured white (the door jambs = the white lines). `spawn_car` now
  adds it for `spec.frontend`. The colour is the texture's: no palette interior slot exists (no setup/COLOURS field,
  no "interior" string). Census `research/gfx/fe_texture_census.txt` (test `frontend_textures_resolve`): 278 → 77
  missing; the rest are data gaps (Supra, RX7, M3, MR2 ship no VEHICLEI; `4x1_0_off`, `bonnet_off`) — missing in the
  original too. Per-car shader dump of the 300ZX `research/gfx/int_300zx.txt`; scripts `tools/ida/gfx_int{1,2}.py`.
  Shot `research/screens/gfx_interior.png` (fecapture garage). In-game cars do not load VEHICLEI (one xref only).
  The garage's thin door-outline lines are the door jambs (`base_paint_matt_0` / `car_matt_0` with
  `<car>_interior_gloss`, type 1/3 → CarShader) seen through the modelled door gaps (hiding `interior_gloss` removes
  them): before ~200-225 grey (untextured white × light), now ≤ 120 (the texture's grey × light) — faithful as far as
  the code shows (157BA0 gives type 1 no paint; 0x00157D40 / 0x00157710 only clear the overrides on release). The
  thin light lines along the car-select sill are not identified yet.
- **Primitive +4 = the starting car shader type** (`Primitive.kind`, `PrimRecord.kind`): GDL 0x0009EF30 calls 0x0009EAF0
  with the primitive record, whose +4 holds the file's layer index (0/1/2 on the decal / seam / logo layers), so the
  `decal_seams` switch and the `logofw/logorw` test start from it: `side_decal_seams_1` (1) → 4 CarDecal,
  `side_decal_seams_2` (2) → 0 → "seams_2" → ShinyCar, `logofw_alpha_1` (1) → 5 CarDecal. The port passed 0 (→ 6 /
  2: CarShader) before; `car_shader_type` now gets `kind` everywhere. 0x00157BA0 only walks part 0 of each object.
- Tests `car_colour_gradient_rows`, `fixed_sets_override_alpha_test`,
  `fixed_set_alpha_census` (writes the census). Shot `research/screens/gfx_carshader.png` (`trackview --track LADown
  --car LA/350z,LA/RX8 --colours 3,6 --caryaw 60 --size 800x450 --cam 197,-6.5,30,0,-10`). Plain smoke race runs.

### RainSurface road layer + wet road reflection (`rain_surface`, `projected_light_beam_pc_xbox`; done)
- **Draw lists** (DrawList.cpp; dumps `research/gfx/rd_xref.c`, `rd_dl.c`, `rd_gdl.c`, scripts `tools/ida/gfx_rd_*.py`):
  VsRender 0x58140 creates 8 lists in the renderer (`dword_7806C8` +16 + 36·j, count +424; list = {+0 mask, +4 size,
  +8 count (1 = sentinel), +12/+13 bytes, +28 far cut, +32 records of 28 B}): 0 solid (mask −1), then masks 4, 8, 32,
  512, **16 = `dword_6D5328`**, 128 = `6D5330`, 256 = `6D5334`. 0x2C2650 files each primitive by its shader's +4
  flags: (u16) 0 → solid, else the *last* list whose mask matches; record {+0 prim, +4 stage, +8 object, +12 instance,
  +16 matrix slot, +20 flags (0x8000 = beyond the list's +28 cut, 0 here), +22 sort key, +24 stage index}. Flags come
  from GDL_RelocateModel 0x9EF30 (data flags are all 0, census test `shader_flags_census` → `research/gfx/
  shader_flags_census.txt`): name contains **"c_road" / "Specular_Road" → 0x10** (London asphalt_tile/london_road_dark/
  cobbles, LA roads, Scenery c_pave1, ExecTest c_road2), "_reflect" → 0x80 (only London's dustbin), "windowenv_" →
  0x100, "Overlay_" → 0x20 / 0x200. So `dword_6D5328` = every visible road primitive (`model_builder::
  is_road_detail_shader`); GameManager draws it normally after the solid compartments and keeps its count for the
  late redraw. (`dword_736940`, a texture override for road shaders, is only ever cleared.)
- **Init** 0x136C50 (in game; `RainSurface::init` from `TyreSprayInit`): +19 on / +18 wet / +0 brightness 128; wet →
  "mtest4", dry LONDON → "mtest4_dark", else "black" (+4/+8 = 16/8 instead of 32/7; tweaks 0x137610 "Graphics.
  Reflection"); misc.vcd textures. "roaddetail" (+44): blend 4 (alpha only), z test no write, bias 20, wrap, stencil
  ALWAYS ref 8 REPLACE write mask 8 (`road_detail_state`). The front-end "FErefl" branch is not ported.
- **Render** 0x137090 (GameManager after the bloom and motion blur; `RainSurfaceSet` after `MotionBlurSet`, before
  `DofSet`): when +19 || +16 || +18, `byte_6F7157 == 0` and view 0 mode ≠ 3 (`PuddleInput.rain_surface_off`): slope
  +20 = 0.9·+20 + 0.1·(car +932).y per 60 Hz frame (`0.9^(dt·60)` here; the ray branch `byte_7266D8` is dead), camera
  +120 dir with y = slope and x, z ×(1 − y²)/(x² + z²) (original, not a normalisation), point = car pos + dir·10000,
  horizon +24 = clamp(0x15EC80 y, 0, 1). Player car = `dword_74BA70` +4 → here the first `EffectCars` car.
- **0x162400** (`render_reflection`): quarter buffer (0x162920 = 0x15C920(**2**): slot {used, +4 target, +8 texture}),
  else nothing. TrackFX 0x14D490 → 0x14D6C0 redraws each list record with the prim's shader handle *and* stage
  swapped for roaddetail (not the +0 override merge), Renderer 0x2A9060; puddles 0x131F50; RefCopy (blend 1, z off,
  Copy program, fBlurAmount 1) of the band y = 0.35·h .. h (`dword_687658`) into the buffer; 0x15C770 one blur pass;
  RefPaste (`dword_74897C` +8: blend 8 = src·dest alpha + dest, z off, stencil EQUAL ref 8 mask 8, no program →
  MODULATE) strip of `paste_strip`: bit 1 the band mirrored from y = h + (1 − h)·0.025 (`dword_68765C`) to the bottom
  (v (1 − 0.025)·H → 0, VISOS(0x80, +0 ×3)), bit 2 the fade from black at y = h; all 8 vertices one strip (the 2 → 3
  join triangles are drawn too, original). So the reflection = the blurred mirrored sky line added in proportion to
  the roaddetail alpha (mtest4.a × prelit alpha, then the puddles' blend-2 alpha). `byte_748988` (paste with RefCopy)
  is never written.
- **Bit 8 emulation** (`StageState.stencil_road`, `RoadCamera`, `FSE_STENCIL_ROAD`): the road copies (same mesh,
  roaddetail material, `ROAD_DETAIL_LAYER` 21, spawned by `Builder::add_model` for world models) and the puddles
  (`PUDDLE_LAYER` 22) are drawn by two `RoadCamera`s (children of the world camera, plain views, back buffer, shared
  msaa colour/depth, depth Load) at the order of their `FullScreenEffects::road_scene` slots. Before them
  `road_stencil_clear` zeroes back-buffer alpha (skipped when the car stencil is already in it: car = 255, rest 0).
  Roaddetail writes its alpha clamped to 1..254/255 (`ROAD_STENCIL_WRITE`) = "bit 8 set"; puddles and RefPaste test the
  back buffer for 1..254 (`ROAD_STENCIL_TEST` with `XboxStageMaterial.stencil` / FSE `prog.alpha.y = 2`), RefPaste
  writes rgb only. The car-bit test (`FSE_STENCIL_NOT_CAR`) now discards on alpha 255 only, so a car stencil built
  before the road pass survives it. Deviations: roaddetail alpha 0 / 255 stored as 1 / 254; the road pass draws after
  every world draw here, so transparent world draws that in the original came later and cleared bit 8 do not.
  Tests `init_flags`, `horizon_projection`, `paste_stream`, `reflection_passes`. Viewer: `--puddles …` also inits
  RainSurface wet; `VELOCITY_RAIN_SURFACE=0` turns the render off (A/B). Shots `research/screens/gfx_wetroad.png`
  (`trackview --track London --puddles -644,6.62,60,180,300,8 --car LA/350z --size 800x450 --cam -644,9,60,180,-6`:
  the lit windows mirrored and blurred in the wet asphalt, pavement dry, puddle bottom left) and `gfx_wetroad_game.png`
  (smoke race `SMOKE_WET=1 SMOKE_AREA=2 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=25 SMOKE_RECORD=…`, building lights on the
  road right; the grey haze is the tyre mist). Plain smoke race runs.
- **Wet London haze A/B (confirmed: tyre mist, not a bug)**. Harness `gfx_ab` (debug, `GfxAbPlugin` in RenderPlugin):
  `VELOCITY_GFX_AB=<s>[,<prefix>]` waits `s` real seconds after the RainSurface init, then runs six consecutive frames
  — 0 on, 1 RainSurface off (= `VELOCITY_RAIN_SURFACE=0`), 2 on (frame-to-frame noise), 3 `tyremist` effect batches
  off (tyre spray "mistspray" + puddle "splash"), 4 every `EffectDraw` batch off, 5 both, then 6 on / 7 beam texture
  off / 8 on / 9 beam override off / 10 on (see "Beam lights") — captures each
  (`research/screens/<prefix>_<phase>.png`), logs stats against frame 0 and writes ×4 diffs (`<prefix>_diff_*.png`).
  Same run = same seed, adjacent frames (a separate `VELOCITY_RAIN_SURFACE=0` run is not frame-matchable: the smoke
  drive runs on real time). Wet London (`SMOKE_WET=1 SMOKE_AREA=2 SMOKE_FOLLOW_LINE=1 SMOKE_DRIVE=30
  VELOCITY_GFX_AB=35,gfx_ab_wet`, 96 mph behind the car): RainSurface off = mean |d| 10.1 / 148k px, *less* than the
  plain next frame (11.6 / 174k), luma 44.9 → 39.4 (noise 42.4) — the wet road layer adds only the local mirrored
  lights; mist off: luma 44.9 → 19.1 (268k of 307k px), all effects off 15.7. So the screen-wide grey veil is the
  tyre-mist particles, mostly the rear-wheel "mistsprayworld" quads (0x00154310: 1 → 20 m ·speed, alpha 0.2, 4 s,
  rate 8/s, left in the world for every car, so the chasing camera drives through them), with the constants checked
  against the dump. Shots `gfx_ab_wet_0_on.png` / `_1_rs_off.png` / `_3_mist_off.png`.

### Car light flares (`flare_render`, FlareRender.cpp 0x123A20..0x1273B0; `FlareRenderPlugin` via TrackFxPlugin)
**Correction**: "DangerZoneRender's own streak manager 0x123E60" is FlareRender.cpp (assert at 0x1271B0 names the
file; the idb labels it DangerZoneRender/FlareRender). Object `unk_719450` (init 0x123A20 from CommonInit, +0 = front
end), drawn by GameManager 0xB9D04 (render flag bit 16, right after TrackFX 0x14A5B0; `draw_flares` runs after
`draw_track_fx`). Dumps `research/gfx/DangerZoneRender.c` (0x123A20..0x1270E0), `cf_dec.c` (helpers + constants
0x6869C0..0x686B50), `cf_dec2.c` (callers), `cf_dis*.txt` (static inits 0x4251A3..0x425567), `cf_vs.c`
(VertexStream); scripts `tools/ida/gfx_cf_*.py`.
- Shaders (0x123A20, misc.vcd, blend 12, stencil write mask 0): +4..+16 "flare" = LensFlareMiddle (z test no write,
  bias 128 = `dword_6869D0`/2) / LensFlareSecondary / LensFlareSecondaryBlob / LensFlareStreak (no z); +20 = +24 "rearL"
  LensFlareSecondaryBlob, z test, bias 128, colour op 4 = MODULATE2X (`EffectShader::colour_op`, stage mode 3 =
  `STAGE_MODE_SCALED`: rgb × 2 saturated); +28/+32 "FS0"/"FS1" Spec4 MODULATE4X (flash records, not ported).
- Per frame 0x123E60 (dt = frame ms · 0.001): traffic phases `dword_686A3C[10]` += dt·2·2π; `dword_7192A8` = camera
  mode 3 && in game (halves ghost/streak alpha). Per CarRender car i (request order) not culled (+2251: hidden /
  outside the view / camera inside, as TrackFX) and +2304 == 0: flash scale = 0x12BC40(1.75 − fmod(+1824, 1.75), 8
  points `unk_686A88`) while +1824 > 0 (+1824 −= dt; 0x123930 = traffic hazard call sets 4.0 / 0); d = camera − car,
  |d| < 200: d·z ≤ 0 → rear 0x1248F0, else head flares at car·l_head and x-mirrored (damage bits 1/2) via 0x124340 +
  streak records; then 0x125190 (emergency traffic) and 0x1257B0 (top lights, immediate). Then 0x125720 (+= phase,
  deferred list) and, in game, LightStreak 0x12E800 on the manager at +196 (width 128) with the frame's records.
- 0x124340 head: L = |cam − p| ≤ 200, c = dir·z; size = lerp(1.3, 3.5, L/150) (beyond 150: 3.5, fade 1 − (L−150)/50)
  · c² · flash, alpha c⁴·0.4·fade, quad a = up·size, b = (+88)·size·2, c0 = p − (b − a)/2, c0+b, c0−a, c0−a+b, white;
  within 150 m (`byte_686A10`; the ray test `byte_7192AC` is never set): g = 1 − L/150 → ghosts 0x126340(p, c⁴·g²,
  g²·192·flash) = 3 screen quads (`dword_719358`: {0.66, at 0, size 0.8, Blob, white}, {0.6, 0.7, 0.3, Secondary,
  (1A,19,10)}, {0.6, 0.72, 0.25, Secondary, (0D,17,1C)}) on the line light → mirrored point, size min(s·a5·a4, a5) ×
  4:3, alpha min(a·((0.5 − r)/0.5)²·a4, 1)·mode; streak 0x125E60(p, c⁸·g⁴, g⁴) = 6-vertex strip 320 × lerp(8, 24) in
  the 512² space, colour (10,10,20), alpha a4·mode. Screen quads (Renderer 0x2A9B00) are unprojected at eye depth 1
  (`SCREEN_DEPTH`, z off).
- 0x1248F0 rear: both rear lights broken (4 and 8) → none; traffic: alpha 0.75, base 1.1; race: alpha lerp(0.5, 1,
  brake), base lerp(1.1, 1.5, brake) (front end 1.1); t = L/200: size lerp(base, 7, t), alpha c·(1 − t²)·k; quads at
  car·(l_brake + (0.1, −0.05, 0)) then x mirrored (the offset is not, original), red (C0,0,0); streak types 2/3;
  reverse (+1832 ≠ 0): 0.75 quads at `l_reverse`/`r_reverse` (+784/+800, zero if missing; part 0's model fills a
  missing one, 0x110220) for sides with reverse prims (+2184/+2200 = `CarLights.reverse`), alpha c·reverse, white.
- 0x125190 emergency traffic (vehicle +5080 ≥ 0 and VsAITrafficVehicle 0x2B5D0 `is_triggered_type`): ten lights
  (`unk_7193A0`, a 9 m vehicle) facing `unk_7192B8`, size lerp(2, 20, t), b = (+88)·size·1.5, alpha max(0, dir·n) ·
  max(0, cos(index·2 + phase_i))⁴ · (1 − t²) · 1.5, colour (0, 50, B0) → light blue.
- 0x1257B0 top lights (`TopLight_1/2` locators +816, count +1748): +2336 += dt·2·2π; size lerp(3.5, 25, t), a =
  up·size·0.35, b = (+88)·size·0.6, alpha max(0, cos(i·π + phase))·(1 − t²)·1.75, colour = 0x123970's raw
  D3DCOLOR (VsAIPoliceVehicle::SetSiren 0x179B0: a, r, g, b; +2332 on).
- Streaks 0x1271B0: not front end, car ≠ player (`dword_74BA70` +4 = first race car); ≥ 16 records → 0x12FA80
  (farthest, the manager's max depth drops to it); record {pos, depth = distance, intensity = c, light = car + 22·type,
  colour index type/2 + 2 (head 2 white-blue, rear 3 red)}. Gate: manager +0 (150) > distance.
- Bytes: every alpha is `_ftol(x·255)` stored into a byte — values > 1 wrap (traffic/top lights, original).
  VertexStream 0x2B95D0 joins appended chunks with repeated vertices (degenerate joins), so each light is its own strip.
- Input: `CarFlareMessage::{HeadlightFlash, TopLight, TrafficLights}` by `CarRender.request` (core request below);
  `CarFlareLocators` / `CarFlareState` are added by `spawn_car`. `VELOCITY_CAR_FLARES=0` turns the pass off.
  Tests `flare_render::tests` (flash curve, head quad + ghosts + streak, rear levels / damage, alpha wrap, streak ids).
  Shot `research/screens/gfx_carflares.png` (on | `VELOCITY_CAR_FLARES=0`; `trackview --track LACoast --lightref --car
  LA/350z,LA/RX8 --caryaw 0 --size 640x360 --cam 322,-55.6,-392,-45,-12`: headlight glows, the blue horizontal
  streaks and two ghost rings below the screen centre; 11.4k px differ, max 161).
- **Dummy police car** (`car_render::{DummyPoliceCars, DummyPoliceCar, render_dummy_police_cars, dummy_wheel_offset,
  police_car}`, `flare_render::dummy_top_lights`; dumps `research/gfx/s9_dc{,2}.c`, `s9_vals.txt`). Caller: only
  PhysicsObjectParticleTracker 0x1E3B70 (PhysicsObjectManager's draw 0x1E0710 / 0x1E0EE0): a visible (+152 & 2) object
  whose type record +56 == 1 (`PhysicsObjects__1E5F30`; physics tags `propld_dummycopcar`, hash 0xF7847E20, with 1 —
  shipped in `LADown\Circuit\Race02` and `Race09` race.vcd) is **not drawn as itself**: CarRender 0x1192F0 copies the
  first car with +2328 bit 0x4000 (0x119250; set by 0x110220 for car index ≥ race + traffic count = police,
  `CarSpec.police`) into one of 6 scratch slots (0x11BA70), puts it at the object matrix, the 4 wheels at (±0.8, −0.1,
  ±1.3) (x negated for wheels 1/2, z for 2/3, car rotation for all — no left/right turn), draws LOD 0 chassis + wheels +
  parts (DrawList 0x2C23C0; no lights/shadow/LOD); within 200 m (d² < 40000) top lights at fixed (∓0.4, 1.4, −0.3), on,
  blue 0x800070FF / red 0x80FF0000, phase fmod(`flt_7192B4` + (addr >> 6), 2π) → 0x1257B0(a6 = 1) queues the stream on
  FlareRender +2256 (≤ 20, count +2336), flushed by 0x125720 with "rearL"; count = the police car's +1748. No police car:
  "couldn't find an index to a police car" and nothing at all. Port: clones of the police car's LOD 0 group and wheel
  groups (bevy `clone_and_spawn` linked) under one root per key; `key` (object index) stands in for `addr >> 6`.
  Requests: core (fill `DummyPoliceCars`, skip those objects, `CarSpec.police`), physics (expose the +56 tag).
- **Front-end flares** (FECK2ElementCamera 0x2999B0 → 0x299C5A `0x123E60(…, 0)`, not garage `dword_7065A4`, only while
  `byte_7065C0` (CarPalette 0x10CE80) is 0): `draw_flares` falls back to a `rain_surface::FrontEndCamera` view when no
  `TrackCamera` exists, sets +0 front end (no streaks, no mode-3 halving, rear size 1.1) and draws the front-end cars
  only; gates from `rain_surface::FrontEndRender { garage, flares_off }` (frontend request).
- **Flash records** (`FlashRecord`, `FlareFlash`, `FlareRender::add_flash`, `flash_alphas`, `draw_flash`; dumps
  `s9_dis.txt` (0x1270E0 disassembly), `s9_vals3.txt`): 0x1270E0 searches a free slot (timer +36 + 4i < 0) and then
  **overwrites the index with 0** (original bug: every flash lands in slot 0), copies the 36-byte record (+0 rise, +4
  fall, +8 intensity, +12 full screen, +16 pos, +32 size), fall ×10 (`flt_624A90`), timer 0. 0x127080 (GameManager
  0xBA275, in game) → 0x1267C0 per slot: timer += frame ms · 0.001; layers k = 0..2: rise `dword_686B40` (1, 1, 1.5) ×
  rise, fall `dword_686B34` (0.5, 2, 1) × fall, squared `byte_686B30` (1, 0, 1); a = clamp(… · intensity); timer > 0
  and all 0 → −1 (no draw); full screen quad or `size`·512 around the projected point if 0x125D90 passes (in front of
  the camera, ray camera → point with flags 0 = tests nothing); uv 0.25..0.75 (`dword_686B2C`); shaders `dword_686B1C`
  (FS0, FS1, FS0) = Spec4 blend 12 no z MODULATE4X; colours VISOS(0x40, FF, 40, 20) / VISOS(0x80 ×4) with alpha
  `_ftol(a·255)`. Callers (requests): VsAIRaceVehicle 0x19740 event 1 (ai), TimelineCameraShaker 0x17A490 (core).
- Motion blur's amount 0x12FB60 was already ported (see "Full-screen effects", Motion blur).

### Front-end floor reflection "FErefl" (`rain_surface::{FeReflection, FrontEndCamera, FrontEndRender, FeReflectionSplit, fe_reflection, fe_reflection_transform, fe_paste_strip}`)
Dumps `research/gfx/RainSurface.c` (0x136C50 init, 0x137610 tweaks), `s9_dc.c` (0x1378E0 / 0x137B40 / 0x137C50,
FECK2ElementCamera 0x2999B0), `s9_dc4.c` (LoadFoyerLights 0x298CD0, CarRender 0x1190D0), `s9_dc5.c` / `s9_vt.c`
(VsBaseCamera 0x5D2E0, view build StringManager 0xC8810), `s9_vals2.txt`.
- Data: FECK2Element3D::LoadFoyerLights 0x298CD0 reads after the lights (and the optional FOG / GRADIENT FOG block) a
  `"REFLECTION"` block: Reflect Environment (`byte_7263CD` = RainSurface +69), Blur (`_ftol` → +72, read by nothing),
  Intensity (+76, 0..255); none → +68 (`byte_7263CC`) = 0. Shipped: `Foyers\carSel\lights.txt` (env 0, 16, 64) and
  `modArea\lights.txt` (env 1, 16, 255). Init 0x136C50 (front end: CommonInit arg 1): "FErefl" target 256² (1 <<
  `dword_686FB4`/`FB8` = 8, fmt 30 linear X8R8G8B8, +64) and four shaders (+48.. blend 0, 0 + 0x2A17D0(1, 0), 6, 13; z
  off, no texture); the paste uses +48 + 4·`dword_686FE0` (3) = **blend 13** (rgb += src · dest alpha).
- 0x2999B0 when +68 and not garage (`dword_7065A4 == 0`): 0x1378E0 flushes three draw lists, targets FErefl (cleared to
  `unk_7263E0` colour, bss 0, z 1), plane y = car 0 (CarDamage 0x761B0) y − +1816/2 + 0.02 (`flt_686FDC`), reflection
  R (m[1][1] = −1, m[3][1] = 2y); `byte_686FC0` = 1: the camera position/right/up/dir are reflected (saved in
  `dword_726428..458`) and 0x5D2E0 rebuilds the view **from the vectors as given** (0xC8810 rows −right, up, dir: a
  mirrored, pixel-aligned view; nothing changes the cull mode). Then the reflect-flagged elements (+292 + i) if +69,
  CarRender 0x1190D0 (every car not culled: 0x1173B0 / 0x119E60 / 0x1134A0 + flushes; licence plate 0x12E480(0/1)
  around it), 0x137B40 restores camera + target. After the foyer elements and the 0x6D531C flush: 0x137C50 (when +68)
  pastes the 6-vertex strip (`fe_paste_strip`): 512² positions (0,0)..(512, 512·0.6) VISOS(0x80, c, c, c), to 512·0.85
  black, uv = the same fractions in texels, z 0.1; c = `_ftol`(((1 − clamp(−dir.y, 0, 1))·1 + 0) · +76) as a byte —
  then TrackFX 0x14A010 / 0x14B030, characters, cars + shadows, bloom, flares, fade, particles.
- Port (front end only, `FrontEndCamera` view, no `TrackCamera`): split active when `FeReflection.on`, not
  `FrontEndRender.garage`, and a front-end car exists. Front-end car meshes go to `FE_CAR_LAYER` (13), front-end
  characters and the `EffectDraw` meshes to `FE_POST_LAYER` (14) (layers ≥ 64 panic in `RenderLayers::layer`; 24..62
  are the FSE slots). Reflection camera (order −5, `FE_REFL_SIZE`² srgb
  image, clear 0, fixed-aspect copy of the foyer projection, layers 13 (+ `FE_REFL_ENV_LAYER` 16 when env)) at R · W
  · S_x: x flipped in view space so bevy's back-face culling stays right (the paste flips u back). The foyer camera gets
  `FE_ALPHA_VIEW` (Tonemapping::Reinhard as a view-key bit; `shader_manager::apply_alpha_write`: unblended draws write
  their alpha as the original RGBA masks do — the paste reads it). Paste camera (order 1, orthographic 512², same
  target / msaa, depth load, REPLACE output, layer 15): the strip (blend 13, `params.arg.w = 2` = sample a render
  target and re-encode to gamma) then a blend-14 quad adding (0,0,0,1) so alpha is 1 again for the 2D cameras. Post
  camera (order 2, foyer pose / projection, depth load, REPLACE) draws 13 + 14. Viewer (`--fecam` = `FrontEndCamera`
  instead of `TrackCamera`; `--foyer` now also loads `FeReflection` from the Lights.txt): `trackview --foyer carSel
  --fecam --car LA/350z --frontend --size 640x480 --cam 5,1.4,0,0,-6` → `research/screens/gfx_ferefl.png` (the 350z
  mirrored in the dark floor, faded towards the camera; front-end headlight flares on; the viewer's car floats a little
  above the floor). Tests `fe_reflection_block` (shipped files), `fe_reflection_geometry`.
  Frontend hook requested (frontend.md "Requests from graphics (FErefl …)").

### White car surfaces in races (fixed; user priority)
- **Player car's white bonnet / sides** (every in-game race car with decal shaders): the decal prims
  (`bonnet_decal_1`, `side_decal_1`, `logo0N_alpha_0`, …; 179 + 24 + 348 prims) name `bonnet_off` / `side_off` /
  `4x1_0_off` (16² white, **alpha 0**), which only `\Vehicles\DECALST.VCD` holds. CarRender 0x00110220 (from CarPalette
  0x0010D7E0, after the vehicles loaded) loads DECALST via 0x001163A0 and 0x00116510 then sets the decal textures —
  so on the Xbox the decals draw transparent. Port bug: `CarProgramCtx::program` caches one CarDecal program per
  (set, texture name); the model build made the `bonnet_off` program first with **no base** (DECALST not loaded yet),
  and the decal pass's request for the same key hit the cache and kept the base-less entry → bound without a texture
  = opaque white over the bonnet. The cache hit now fills the missing base. (Front-end cars had the same path.)
- **Traffic white windows / undersides / tyres**: TrafficM models use `win01_envmap`, `bottom03..05`, `traff_tyre01`,
  which live only in `\Vehicles\CommonT.vcd` — loaded for every model by GameDataLoading::LoadGameData (0x0009D170;
  VehicleEntity 0x001C0D60's traffic branch pushes only TrafficM / TrafficT). `spawn_traffic_car` now adds COMMONT
  before TrafficT. Traffic paint is the textures' own (no setup, no paint set); unchanged.
- In-game packs re-checked (VehicleEntity 0x001C08B0 / 0x001C0D60): race cars VEHICLEM + VEHICLET + wheels_m/t (+
  CommonT globally) — **no VEHICLEI in game** (FE only). Census (test `ingame_textures_resolve`, ignored →
  `research/gfx/game_texture_census.txt`): race cars 729 of 11524 stage textures not in their packs — 580 are the
  DECALST `_off` names above, the rest (152) exist nowhere (M3 `_gloss` parts, CopTO/CopLO bodies, some exhausts:
  data gaps, the original's missing-texture path returns the texture manager's white default too, AddTexture
  0x0030AEE0); traffic 4 of 396 (`car02_damage`, `glass01`, `plate02`: nowhere).
- Shots: `research/screens/gfx_white_before.png` (smoke look-back on the 300ZX, `SMOKE_DIRECT=1 SMOKE_CAMS=1
  SMOKE_DRIVE=30`, recorded frame: white bonnet) / `gfx_white_after.png` (same view, black bonnet). Viewer debug env:
  `VELOCITY_VIEW_PARTS=b,f,r,e,s,t,sc`, `VELOCITY_VIEW_CARDROP=<m>` (cars lower than the camera, to see bonnets).

### Split screen — two players (`view_render`; stage "split screen")
Dumps `research/gfx/ss_*.c|txt` (0x0B3EC0 / 0x0B3960 views, 0x0B9010 render loop `ss_b9010.c`, 0x0B88B0, 0x0B3C30,
camera manager 0x05ECC0 / 0x05F8F0 / 0x05D1D0, projection VsRender 0x0C89F0 `ss_vt.c`, FSE quad 0x15C3C0 / bloom
0x15F600 `ss_dc11f.c`, per-view users `ss_dc10f.c`, values `ss_vals.txt`); scripts `tools/ida/gfx_ss_vt.py` + the
`gfx_dc/xr/peek` helpers.
- **Views** (core ported 0x0B3EC0 / 0x0B3960 as `velocity_game::game_mode::game_views`): two players = view 1 kind 2
  (top half, +28 player 0), view 2 kind 3 (bottom, player 1), view 0 inactive; `byte_6F9180` (side by side, kinds 4/5;
  core: options +0x24, only a 0 store has a direct xref) works the same through the rects. 0x0B88B0 per frame: every non-mirror view gets render flags `dword_685E84` = **−1** (all
  effects), the mirror (kind 1, single player only) `0x08000620`. HUD views (+988): two players [0] top / [1] bottom.
- **0x0B9010 per view**: 0x0B3C30 = viewport (RenderTargetManager 0x2EEF10 stores it with the current target;
  0x2EEE20 re-applies a target's own viewport when it is set again) + the view's camera, then the whole world
  pass (sky, sectors, cars with CarRender 0x1127F0(camera) = cull / in-car per camera, LOD per view, TrackFX
  reflections / headlights, skid marks, sun, characters, barriers, bloom, flares 0x14A5B0 / 0x123E60, zone FX
  0x162D10(view, player), particles — the stateless / elParticle / camera rain **updates run inside the loop, i.e.
  twice per frame for two players** (not reproduced)), each with that camera. After the loop 0x0B3C30(view 0 = whole
  screen): flash records 0x127080, 0x1315D0, fade 0x123250, then the 3D HUD per HUD view (camera mode 5,
  OSD3dInterface 0xEA2B0(view i)) and the 2D OSD once.
- **Gated off for two players** (`byte_6F7157`): motion blur (flag 0x10 && !split, the forced flag sits inside
  0x12FB60 so it is off too; the slipstream ribbons draw instead), RainSurface 0x137090 (wet road + puddles),
  dynamic envmap (DangerZoneRender 0x1214A0 +57), CarPalette 0x10D580 `byte_7065A9` (driver sway), StartLineEffects
  0x147DC0, PhysicsObjectManager 0x1E0710/0x1E0EE0, CarRender 0x110220 (not checked in detail). Slipstream ribbons
  update only in player 0's view (`*(view+28) == 0`), sparks only for camera 0 (both updated once here anyway).
- **Projection**: VsBaseCamera 0x5D1D0 = tan(fov/2)·near (+232) and ×aspect (+228), aspect +144 = 4/3 (16/9
  widescreen, 0x5F8F0) regardless of the viewport; VsRender 0x0C89F0 builds the matrix from those extents (far fixed
  3000). So each 640×240 half shows the full-screen projection **squashed vertically** — kept (`fix_view_aspect`:
  after `camera_system`, before the frusta, the perspective aspect of every view / sky / car-stencil camera is reset
  to the target's and `clip_from_view` recomputed). 0x5ECC0 also scales both cameras' far planes by 0.5 (600 → 300,
  200 → 100): only the camera planes (culling) use it (`SPLIT_FAR_SCALE`, core request).
- **FSE in a view**: 0x15C3C0 maps the quad's dst rect in clip space, so with a view's viewport every back-buffer pass
  lands inside the view; sources are sampled whole unless the caller passes a rect. Bloom 0x15F600 passes the view
  rect (÷ back-buffer size) as the source → each view blooms its own half. Zone FX 0x162D10 / danger 0x120850 copy the
  **whole** back buffer (no rect) into the half buffer and draw it back inside the view (original quirk, kept by the
  mechanism). Port: `FullScreenEffects::set_viewport` / `viewport()`, `FsePass.viewport` (back-buffer passes get the
  current viewport, off-screen buffers the whole surface; `quad_mesh` maps quads and strips into it). Per frame the
  viewport starts at player 0's view (`fse_view_viewport`), the bloom loops over the views, the fade switches to the
  whole screen.
- **Bevy side** (`view_render`, added by `TrackRenderPlugin`): `TrackView { player, rect }` on each view's world
  camera; `TrackCamera` = player 0's view only — a camera with `TrackView.player ≥ 1` gets `SecondaryTrackCamera` and
  graphics removes its `TrackCamera` (so every single-view effect keeps working on player 0). `SplitScreen` resource
  (views by player; `on()` = `byte_6F7157`). `setup_view_cameras`: viewports from the rects × target size (player 0's
  too while split), player 1's camera copies player 0's target (the FSE back buffer), msaa, output mode, glow-view key
  and layers. Orders: player 1 first — sky copies −70..−67 (the background clears the shared target), world −66 —
  then player 0's sky −44..−41 (background clear saved in `SavedClear`, off while split) and world −40, FSE passes,
  present. `sync_view_sky`: one copy per sky camera for player 1 (`ViewSky`), background quad + material copied on
  `VIEW_SKY_BACKGROUND_LAYER` 19 (`ViewSkyBackground`, v range from player 1's pitch), rotation from player 1's camera.
  Cars: `update_car_lights` picks a LOD per view (`CarViewLod`), groups are shown to the views that picked them via
  `VIEW_CAR_LAYERS` [17, 18] (player p's camera sees layer 17 + p; lights group = views with LOD ≠ 2). Gates: motion
  blur, RainSurface. Car stencil camera copies the world camera's viewport.
- **Per-view effects** (stage "split leftovers"): `view_render::ViewCams` (SystemParam) lists the views in loop order
  (`ViewCam { player, camera, transform, projection, frustum, rect, tag }`); camera-built producers call
  `EffectDraw::set_view(tag)` around each view's pass, `flush_effects` puts a tagged batch on `view_layer(p)` =
  `VIEW_CAR_LAYERS[p]` (player p's camera only) while `SplitScreen::per_view()`. Per view now, each call stepping its
  timers by the whole frame time as the original's loop does (twice a frame for two players): TrackFX 0x14A010 /
  0x14B030 / 0x14A5B0 (`track_fx::draw_track_fx` → `track_fx_view`: car passes, reflections, track flares, near
  flares, light streaks), car flares 0x123E60 (`flare_render::draw_flares`), StatelessParticleMgr tick + 0x2D3460
  billboards (`stateless_particles::draw_particles`: view 0 ticks in `update_particles`, every further view ticks
  again), camera rain 0x1341A0 / 0x135690 + elParticle tick 0x2C5B70 / render 0x2C6660 (`el_particle_frame` →
  `camera_rain::camera_rain_view`; the rain's camera travel then spans the two cameras, as `dword_7260E4`), zone FX
  0x162D10(view, player) (`zone_fx_system`: viewport = view rect, player 1's input = `ZoneFxSecondInput`, core
  request; blur focus per view, see below) and lens spray 0x15FB30 (one drop set drawn in each view's
  viewport, each copying the whole back buffer). Flash records 0x127080 stay once, with player 0's camera (the
  original draws them after the loop full screen with view 0 = player 0's camera; here they land in view 0's layer).
- **Per view since the "mirror effects / per-view leftovers" stage**: DOF 0x120D20 (flag 0x20000000; each view's
  camera 0x5ACD0 and viewport, focus car stays `VsAIRaceVehicle__24F90() + 4` = player 0's), danger overlay 0x120850
  and 0x1205E0 (flag 0x4000000; view argument unused, viewport per view, warp steps by the frame time per call),
  DangerParticles Place 0x1235F0 (ungated, before each view's stateless tick: `draw_particles` places on view i ≥ 1's
  camera before its extra tick), barriers 0x109080 (flag 0x08000000, per view + mirror; a3 = mirror skips the wobble /
  scroll step, so the step runs once per player view; the no-entry sign models follow view 0's list). In-car hide
  (CarRender 0x1127F0 per view: the car the view's camera sits in is +2251 culled — body, wheels, lights): `CarLod.
  in_car` / per-view LOD selection skip it (`update_car_lights`); `HeadlightView` per view = `track_fx::HeadlightViews
  { player1, mirror }` (`for_view`), used by TrackFX / shadows / car flares / the in-car hide. Core fills them
  (`game_cameras::headlight_view`: `player1` = (1, mode(1)) while > 1 player, `mirror` = (0, mode 2 = VehicleEntity
  0x1C57C0) while `MirrorView.on`; `inside` = `PhysicsCar.id` = `SpawnCarRender.request`, checked); the stand-ins
  (player 1 outside, mirror = mode 2 inside view 0's car) only apply outside races. `TrackStreamViews` are per player
  (core: 0x8A270 passes 2 players while split; `update_track_streaming` takes every view). Still single: the
  projected beam pose 0x161F10 (player car only, as the original).
- **Zone blur focus per view** (`research/gfx/mb_focus_dc.c`): 0x162D10 builds a view copy whose +28 = the view's
  player and calls 0x160BA0(.., flags 0, Blur/2) → 0x160920(view +28) = VehicleManager 0x1DDCC0(n) = the n-th
  vehicle with +5084 == 0 (the player's car), projected by 0x15EC80 with the device view / projection = **the view's
  own camera**. The +44 last-grounded velocity lives in the one blur object `unk_71FF40` (shared latch: a view whose
  car is airborne keeps the velocity the previous view stored). Only reached when the zone draws (amount > 0, half
  buffer got). Port: `zone_fx(.., focus)` updates the focus right before the blur; `zone_fx_system` passes view p's
  car (player 0 `MotionBlurInput`, player 1 `zone_render_pc_xbox::MotionBlurSecondInput`, core request) and that
  view camera's `clip_from_view` · inverse transform. Without player 1's input the view keeps the last focus.
  Core fills it (`race_effects_bridge::fill_motion_blur`, slot-1 race car, None outside two-player races; checked).
  (0x160FF0's own blur passes the view too, but it is off for two players.)
- **Flash records after the loop** (0x127080, GameManager 0x0BA275 after every view and the mirror, 0x0B3C30 on view 0
  = whole screen with player 0's camera): `view_render::PostViewCamera` (player 0's transform / projection / target /
  msaa / output, whole target, no colour clear, depth load) draws `POST_VIEW_LAYER` 23 at FSE slot `ROAD_SCENE_POST`
  (queued by `post_scene` after the mirror's slots, before the fade; `POST_FALLBACK_ORDER` without the FSE); batches
  tagged `POST_VIEW_TAG` go there. So the flashes now sit over the bloom / zone FX / lens spray of every view and over
  the mirror, and cover the whole screen while split.
- Test: `VELOCITY_ZONE_FORCE=40 trackview --track LADown --car LA/350z --size 800x600 --cam 197,-5,34,0,-8 --split
  197,-5,14,180,-8 --effects rain --fxat 197,-8.6,20 --lens 16` → `research/screens/gfx_split_fx.png` (rain streaks /
  splashes, zone ripples and lens drops built for each half's own camera).

### Rear-view mirror (`view_render::{MirrorView, setup_mirror_camera, mirror_scene, update_mirror_orders}`)
Dumps `research/gfx/mirror_dc.c` (0x2EF0B0 clear, 0x0B3C30, 0x5FFE0), `mirror_dc2.c` (0x5FB30, 0x5F580, VsRender
0x0C89F0), `ss_dc3f.c` (0x0B88B0).
- One player: view 1 = kind 1 (0x0B3960: x W/3, y 0, w W/3, h H/6 = 213×80 of 640×480). 0x0B88B0 per frame: the
  mirror's flags = `dword_685E88` 0x08000620; +36 (drawn) = `byte_685808` (1) && `dword_749110` != 2 && the player's
  camera mode == 1 && follow camera +524 ([14]) == 0 && !+585 (looking back) → only the static bumper / in-car views.
- 0x0B9010 for it (`HIBYTE(v77[58])` = 0x0B3EA0 = kind 1): `RenderTargetManager 0x2EF0B0(0, 1.0, 0, 5)` = D3D Clear
  z + stencil only (flag 1 → ZBUFFER, 4 → STENCIL, 2 → TARGET 0xF0; still view 0's full-screen viewport), camera
  mode 2 (0x5FFE0 → 0x5FB30: the mirror camera = bridge record [1]), 0x0B3C30 viewport + camera, then the loop body
  under 0x08000620: sky + sun/moon (0x20; `SkinnedObjectRender__13B5A0` there is CSkyRender's render), cars (0x200,
  CarRender 0x119900(…, !mirror)), fog (0x400), barriers (0x08000000); sectors, characters and trails always; no
  road detail / reflections / skid marks / catseyes / physics objects / bloom / sparks / flares / particles / rain /
  zone FX / DOF / lens spray; the "Decal2" and fading-compartment lists are skipped. It runs after view 0's whole
  pass (incl. lens spray), before the post-loop flash records, fade and HUD. No x flip anywhere (projection 0x0C89F0
  as for any camera, aspect 4/3 → squashed into 213×80).
- Port: core spawns the camera with `MirrorView { player, rect, on }` (request in core.md); `update_views` sets
  `SplitScreen.mirror` (only without a split; `per_view()` true, `on()` false so no two-player gates).
  `setup_mirror_camera` (Update, after `setup_view_cameras`): `is_active` = on, player 0's target / msaa / output /
  glow key, layers = player 0's without 17 with 18, viewport = rect, colour clear None, depth Clear. `sync_view_sky`
  copies the sky passes for it (`ViewSky.player = MIRROR_SKY`, no clear, mirror's rotation and background pitch).
  `mirror_scene` (FseSet::Build, after `LensSpraySet`, before `FadeSet`) queues five scene slots
  (`ROAD_SCENE_MIRROR` + 0..4); `update_mirror_orders` (FseSet::Execute after the passes) gives the 4 sky copies and
  the world camera those orders (`FullScreenEffects::scene_order`; `MIRROR_FALLBACK_ORDER` −39.. without the FSE).
  Cars pick a LOD for the mirror as view 1 (`update_car_lights`; groups back to no layers when the mirror goes,
  `clear_view_layers`); while it is drawn untagged `EffectDraw` batches go to layer 17 (player 0 only). Envmap source
  cameras drop layers 17/18 too.
- **What the mirror's pass draws besides the world** (0x0B9010 under 0x08000620, `ss_b9010.c`): ungated calls
  CarRender 0x1127F0(camera) cull / in-car, TrackFX 0x14DB30(1.0), StartLineEffects 0x1475B0, CatsEyeRender flush
  0x11E4D0 (its producers — headlight strips 0x14B030 flag 4, catseyes 0x11CF70 flag 0x100 — are off, so empty),
  TrailBlazerRender Render, SlipstreamRender 0x13FDB0 (flag 0x10 is off, so the ribbons rather than motion blur; the
  view's +28 = player 0, so the ribbon update 0x13F770 **runs a second time** in the mirror), DangerParticles Place(0)
  (no stateless tick follows in the mirror: no visible effect, not reproduced), SpecularChars 0x144DF0, the OSD map
  0x0EA160 (frontend); under its flags: cars 0x119900(…, a4 = 0) = shadows 0x11A6E0 + car lights 0x1129F0 but **no**
  dynamic-envmap update 0x1598A0 and **no licence plates** (0x12DF40 / 0x12E400 gated by a4), barriers 0x109080(1).
  Port: `EffectDraw` tag `effect_draw::ALL_VIEWS` (world-space, layer 0: player 0 and the mirror) for the zone trails
  and — without the mirror — the slipstream ribbons; with the mirror the ribbons are tagged view 0, stepped again and
  drawn again tagged `MIRROR_VIEW_TAG` (layer 18); `ViewCams::mirror()` gives the mirror camera to the shadows
  (`car_passes(.., shadows_only)` after the views), barriers and start lights. **Licence plates (checked, port
  unchanged)**: a4 = 0 skips only 0x12DF40 (redraw of the 64×32 "LPlate" target: background + text) and 0x12E400
  (its debug quad `byte_71FEE0`; 0x2EED60 / 0x2EEDC0 begin/end are assert-only no-ops, `research/gfx/
  RenderTargetManager_X.c`). The plate prims themselves (car 0's REARBUMPER, customPlate shader → the target) are
  drawn by 0x1134A0 like any prim, so the mirror shows the plate with the target view 0 drew this frame — which is
  what the port's cpu-composed, change-driven target gives. (The mirror camera usually sits in car 0, so car 0 and
  its plate are in-car hidden there anyway.) Shot `research/screens/gfx_mirror_fx.png` (`trackview --track LADown --car LA/350z --size
  640x480 --cam 197,-5,34,0,-8 --mirror 197,-5,34,0,-8 --barriers Circuit\Race01`: the mirror, looking the same way,
  now shows the Race01 arrow barriers; before this stage it showed none).
- Test: `trackview --track LADown --car LA/350z --size 640x480 --cam 197,-5,34,0,-8 --mirror 197,-5,14,180,-8` →
  `research/screens/gfx_mirror.png`. Tests `view_render::tests::view_viewports` (mirror rect 213×80).
- Test: `trackview --track LADown --car LA/350z --size 800x600 --cam 197,-5,34,0,-8 --split 197,-5,14,180,-8` →
  `research/screens/gfx_split.png` (top: player 0 view of the 350z, bottom: player 1 looking back at it, both
  squashed 2:1, sky in both halves). Tests `view_render::tests` (viewports, background uv).
- **API for core**: put `TrackView { player: p, rect: GameViews::fraction(view) }` on each player's world camera
  (player ≥ 1 need not be a `TrackCamera`; graphics strips it), and let graphics set the viewports (core's
  `player_viewports` sizes them against the window; with the FSE the target is the back buffer of the same size, so
  both agree — one writer is enough). Set `PerspectiveProjection.far` × `SPLIT_FAR_SCALE` for two players
  (0x5ECC0). `DynamicEnvmapInit.two_players` and `PuddleInput.rain_surface_off` stay as they are (graphics also gates
  on `SplitScreen`). Frontend: the 3D HUD per HUD view (rect = the player's half, camera mode 5) and the OSD.

### Multi-layer blend trees (checked; the one shipped user ported)
Callers outside BoneAnimBlendTree*.cpp (`research/gfx/s9_bt_xr.txt`, script `tools/ida/gfx_s9_bt.py`): AddChildLayer
0x2DB300 only from Characters 0x79EC0 (twice), SkinnedObjectRender 0x13AF80 and TimelineSequencer 0x2B5320 (once
each); the nested-layer nodes (0x2DC7C0, 0x2DDB40, 0x2DDBC0) are reached only from the layer ctor / dtor
(0x2E0270 / 0x2E02C0 = vector bookkeeping) — **nobody builds a child tree**. The one multi-layer use: Characters
0x79EC0 adds two root layers when a13 < 1 and a previous anim a3 exists (layer a3 weight 1 time a15, then a2 weight a13
time a14), i.e. the front-end character element's 1 s cross-fade (0x285350: weight = (now − +132) / `dword_6C1780`
(1.0) while the previous anim +116 ≥ 0; its time +2564 advances by its frame increment, wraps when +2569 else holds 1).
Port: `skinned_object_render::SkinnedModelBlend { request, prev: Option<(anim, time)>, weight }` → two `update_leaf`
layers (prev first). Frontend request filed (the element's clock is theirs). Game flag girls and the timeline use one
layer.

### Swapchain guard (`swapchain_guard::SwapchainGuardPlugin`, in RenderPlugin; platform layer, no original)
- The intermittent sweep panic "Trying to destroy a SwapchainAcquireSemaphore that is still in use by a
  SurfaceTexture" (wgpu-hal-29.0.4 `vulkan/swapchain/native.rs:359`) is always a **teardown** panic, never an acquire
  failure: wgpu-core `Surface::drop` (instance.rs:666) unconfigures without releasing the surface's `acquired_texture`.
  It fires when (a) the device is lost (present/discard fail `check_is_valid` before taking the texture; bevy's
  default error handler then AppExits) or (b) anything panics while a frame is acquired (`wgpu::SurfaceTexture::drop`
  skips discard while panicking) -> second panic during unwind -> abort, exit 127, first panic hidden
  (`research/fe_smoke_career2.log`). The sweep's 12 s LONDON2_CIRCUIT_RACE09_wet log was overwritten by a rerun, so
  which of the two it was is unknown; earlier SPRINT_RACE04_wet the same.
- Fix: `Texture::destroy` on a surface texture drops the hal texture with no device check (resource.rs:1451), which
  frees the semaphore Arc. `handle_render_error` (our `RenderErrorHandler`) destroys every window's swapchain texture
  then keeps bevy's policy (log, `AppExit::error`, StopRendering); `SwapchainDropGuard` (render-world resource
  inserted at build, i.e. before `WindowSurfaces` from RenderStartup, so it drops first) holds clones of the frame's
  swapchain textures and destroys them on drop while panicking. Rendering unchanged.
- Acquire errors are already survived by bevy `prepare_windows`: Outdated -> reconfigure + retry, Timeout / Occluded /
  Lost -> log + skip frame. A lost *device* is not survivable without `RenderErrorPolicy::Recover` (render resources
  built in plugin `finish` would need RenderStartup re-init) — not attempted.
- Debug env: `VELOCITY_GFX_SWAPCHAIN_GUARD=0` (A/B off), `VELOCITY_GFX_LOSE_DEVICE=<render frame>` (destroy the
  device), `VELOCITY_GFX_PANIC_AT=<render frame>` (panic in the render world after acquire).
- Checked (2026-11-07): `PANIC_AT=200` -> guard releases 1 texture during unwind, single panic; with the guard off
  no abort either (render-thread panic: the world is dropped from a non-panicking thread, discard works), so the
  guard only matters for main-thread unwinds like fe_smoke_career2. `LOSE_DEVICE=200` (`Device::destroy`) does not
  reach the handler: bevy_pbr's cluster readback panics first (`Buffer::get_mapped_range` fatal, then PoisonError
  `cluster/gpu.rs:1156`), no semaphore panic. 6 hidden LONDON2_CIRCUIT_RACE09 wet runs (3 parallel, drive 20 s):
  all pass, no repro. No upstream wgpu issue number confirmed (offline).

### Stubbed / not yet
RainSurface types 0-3 of 0x136090 (from LightStreak 0x1341A0) stay unported: the shipped tweak is 4 = rays, the others
are debug-menu only. Physics-object instances (types 1/3, core's `physics_object_render`) still draw without the
instance set (requested in core.md).
**SkinInst.bdl**: all 24 shipped `Tracks\*\{Circuit,Sprint}\RaceNN\skininst.bdl` are 105-byte bundles with an empty
Instances chunk (test `skinned_object_render::tests::skininst_bundles_are_empty`) — nothing to place. (Character
specular: done, see "Character specular"; vertex animation: data + maths, no shipped vertex model, see "Vertex
animation".)
Foyer particles: the frontend writes `LoadParticles { area: "\\Foyers\\hangout\\", sub: None }` (ready here).
Depth bias (code 5) uses bevy `depth_bias`, not the original projection-z shift (-value/1024).

**Answered by core (2026-11-06, wiring checked)**: `MirrorView` + `ZoneFxSecondInput`, `HeadlightViews`,
`TrackStreamViews` per player, `MotionBlurSecondInput`, `PuddleAdd` (`race_effects_bridge::forward_puddles`, GameTick;
read by `puddle_messages` in Update) / `PuddleRestart` (on `RaceRestart`). `PlayerRaceStatus` is HUD data (frontend);
graphics reads none of it. Seen: `SMOKE_SPLIT=1 SMOKE_DRIVE=14` (`research/screens/gfxsplit_3_driving.png`, both
halves with each player's car, lights, sky) and `SMOKE_DIRECT=1 SMOKE_CAMS=1` (`gfxmir_3c_view2.png`, bumper view:
mirror at the top centre, player 0's car in-car hidden in it); both exit 0, no panics.
**Requests to others (open)**: frontend — 3D HUD / OSD per view (frontend.md "split screen HUD
areas"); core — `DummyPoliceCars`, `CarSpec.police`, `FlareFlash` from TimelineCameraShaker
(core.md "Requests from graphics (dummy police car, flashes)"); physics — the object type tag +56 in
`PhysicsObjectInfo` (physics.md); ai — `FlareFlash` from VsAIRaceVehicle 0x19740 event 1 (ai.md); frontend —
`FrontEndCamera`, `FrontEndRender`, `FeReflection`, reflect-flagged elements on `FE_REFL_ENV_LAYER`, `SkinnedModelBlend`
(frontend.md "Requests from graphics (FErefl …)"); core — `CarFlareMessage` from the ai's hazard / police light hooks (core.md "car light
flares"); core — `SpecCharIndex` on flag girls; frontend — `SpecCharIndex` from the FE character
element +764 (both "Character specular"); frontend — 3d hud lighting (`set_lit` with `hud_light_set`, flash from ai's
`PoliceFlash`) and `with_setup_underbody` on front-end cars (frontend.md "Requests from graphics (3d hud lighting…)");
core — `MotionBlurSecondInput` (core.md "Requests from graphics (player 1 zone blur focus)"); frontend — name tags
per player view (frontend.md "Requests from graphics (name tags per view)"; graphics draws no part of them); core —
`PuddleAdd` from `PhysicsPuddleMessage` and `PuddleRestart` at RestartRace (core.md "Requests from physics
(explosions, …)"); core — `FuryNosAudio` → StartNos/StopNos, camera fury from `FuryFx::amount`, `dword_6F5B70` into
physics, traffic +32/+33 (core.md "Requests from graphics (FuryFX / nitrous)").
**Requests from others**: none open (physics' runtime puddles done in the runtime-puddle stage; re-checked in the plates / player-1 focus / name tags stage and in the flare-leftovers / FErefl stage; earlier, in the requests stage: audio's zone heartbeat answered, LightStreak
stencil corrected, wet haze A/B done; earlier: core's puddle wiring + `rain_surface_off` requested in
core.md). **Requests from core**: `CarLightInput` is now written (core.md "Car lights"). Original render rate = TV refresh
(60 Hz NTSC/PAL60, 50 Hz PAL50; core.md "Original render rate"), so `car_render::XBOX_FRAME_RATE = 60` is right for
the NTSC reference — kept, its doc line cites it (done). No other render-rate assumption found in visos_render.
**Request from core (slab_allocator errors)**: done — FSE slots spawn with a degenerate strip (see "Full-screen
effects").

### Light sets, 3d hud lighting, front-end car shadows (`vs_render`, `car_render`, `model_builder`)
- **Start-up scene lights** `VsLights::vs_render_defaults()` = VsRender 0x583F0 (ctor 0x58140; also
  LoadFoyerLights 0x298CD0 when a foyer has no lights.txt): sources (−6,1,3), (0,1,0), (3,1,−6) normalised (xyz /
  length, `VsVector4::Normalise` 0x135B0), dirs = negated + normalised, colours 0.2/0.3/0.1 grey, ambient 0.4 grey
  (bit-exact floats). Used by `track_render` / `track_view` when Lights.txt is missing (Tokyo, Tokyo2; foyers
  CutScene, Network, Test, empty, garage, hangCE, plynth, props). `VsLights::default()` stays the neutral value
  (ambient 1) for scratch sets. Shot `research/screens/gfx_defaultlights_garage.png` (`--foyer garage`).
  Request to frontend: `foyer.rs` should fall back to `vs_render_defaults` instead of `unwrap_or_default()`.
- **Light set** (32 B, ctor Armco 0x13EA0): +0/+4/+8 three packed directions, +12/+16/+20 three packed colours
  (VISOS colour, Armco 0x13CA0 clamps to 0..1), +24 ambient, +28 count. Applied per draw by Renderer 0x2A8FD0 →
  0x2A8BE0 **only when the stage has flags bit 0** and the draw's render struct has a set: D3DLIGHT directional
  (diffuse = colour, direction = −unpacked; lights with |dir|² ≤ 0.01 or i ≥ count disabled), D3DRS_AMBIENT = +24,
  LIGHTING on; no set → LIGHTING off (vertex colour passes). Xbox render-state numbers (from CFogRender: FOGENABLE 92,
  FOGTABLEMODE 93, FOGDENSITY 96, FOGCOLOR 138): LIGHTING 102, DIFFUSEMATERIALSOURCE 111 (= 1 COLOR1, Renderer init
  0x2A81C0), AMBIENTMATERIALSOURCE 112 (= 0 MATERIAL), AMBIENT 115; material diffuse/ambient/specular 1, emissive 0;
  no other caller changes 111/112. So lit rgb = saturate(ambient + Σ vertex diffuse · colour_i · max(0, n·−dir_i)),
  alpha = vertex alpha; **a primitive without a normal stream gets the ambient alone** (nv2a default normal 0 —
  assumed, not hardware-checked). `VsLights::light_fixed_function`; `SpawnOpts.light_set` makes the builder apply it
  (per vertex with normals, else `Builder::lit_material` / `XboxStageMaterial::set_lit` = params `arg.z` 1, `lit`).
  Track **sectors** are drawn without a set (their 9242 flag-1 prims keep the prelit colours).
- **Instance set** (CompartmentManager 0x87D00 relocation, done; `VsLights::{instance_light_set, instance_own_set}`,
  `track_render::TrackInstanceLights`; dumps `research/gfx/instlit_*`, scripts `tools/ida/gfx_dis_range.py`,
  `gfx_dc_range.py`): visual/collision records (128 B, render struct +64, set ptr +76, own set +92) and physics/
  immovable records (160 B, set +64, ptr +120). cpt version ≥ 6: own set from VsRender 0x598D0 (slots count..1 black
  dir (0,0,1), slot 2 = scene light 0, count 3, ambient = scene ambient (+100) × 0.5). **Every shipped pack is < 6**
  (5, stale 3) → the global `unk_718218` (static zero, count 0, ctor 0x424B50). Its only ambient writer is
  CSkyRender::Render 0x13BF30 / RenderClouds (it doubles as the sky's TFACTOR scratch; colours + count 0 also by
  OSD3dInterface 0xEAD80, the hud/CarRender 0x1154E0 copies are restored): last write before the world = unk_730DE8
  (1,1,1, static init 0x426070) × "Buildings Brightness" (`flt_68700C`) when the sky loaded (+0x94 = its shaders
  exist, 0x13B85C). So with no light enabled every flag-1 instance prim draws rgb = that ambient (LA 0.25, LondWet
  0.31, London 1.0; no sky = 0), vertex rgb dropped, vertex alpha kept. Ambient not quantised to the D3DRS byte
  (`_ftol(x·255)`). Test `instance_light_sets`. **Since the beam lights stage** nearly every instance prim draws
  through the "HeadLightBeam(Lit)" programs instead (see "Beam lights"); the fixed-function path stays for the GDL
  overrides and as the A/B fallback. Shot `research/screens/gfx_instlit.png`.
- **Car set** (CarRender 0x119EF0 per car, `VsLights::car_light_set`, every frame in `bind_car_programs`; result in
  resource `CarLightSets` by request): front end (+49) = scene lights 0/1 + ambient, light 2 off ((0,1,0), black); in
  game ComputeObjectLights (0x59140, search unless LOD 2) then light 2 = scene colour 0 (+880), ambient = scene ambient
  (+864) and **both point light colours × 0.25** (`flt_6866A0`; before this they were full strength). The +2328 bit
  0x1000 second set (+1060) and the debug tweak override 0x11AD30 are not ported.
- **3d hud set** (OSD3dInterface 0x0EA2B0, `VsLights::hud_light_set`): copy of the player car's set (car index =
  the view player), colours 0/1 × 1.5 (`flt_686000`), ambient = lerp(white, flash.rgb, flash.w) (FuryInstance
  0x12A4F0) with flash = +2912 = VsPoliceManager::Flash 0x3A1C0 (ai's `police_flash`, zero without a chasing police
  car). Used by the five slot models (render struct light set) and the map screen layers (the set is copied into
  `unk_718218` around COSDMapRender::Render with lighting arg 1); LCD_Movie (GFXHelper 0x12AE10 arg 0) is unlit. All
  OSDM prims have stage flags 1, a diffuse stream and no normals → rgb = that ambient: **the vertex rgb is dropped**
  (white normally, tinted red/blue by the police flash). Wiring requested from frontend (frontend.md).
- **Front-end car shadow** (FECK2ElementCamera 0x2999B0 → CarRender 0x119900(…, 0, 1) → 0x11A6E0,
  `car_render::draw_frontend_shadows`): every front-end car root now has `CarShadow`; no vehicle → full "carshadow"
  quad at y − +1816/2 (+ `dword_7184B0` 0), VISOS(0x60) (`dword_709D58`), bias 82; `dword_6865F0` = −1 (set by
  CarPalette 0x10D7E0 / CarRender 0x110070 at car init — `ZoneFx::zone_mask` now defaults to −1 too). +1816 =
  CarRender 0x1100B0's a6 from FE3DVehicles 0x25C890 = the setup's WHEELS.BIN record +0x0C (setup +0x74 = wheel
  record) → `CarSpec.wheel_height` (`with_setup_underbody`; 2·wheel radius without). Culled = root not visible
  (0x1127F0 sphere test not modelled). Viewer: `trackview --track ExecTest --car LA/350z --frontend --caryaw 200
  --size 640x360 --cam 70,-12.35,142,0,-14` → `research/screens/gfx_feshadow.png` (dusk: the quad is faint). Dumps `research/gfx/hudlit_*`,
  `feshadow_*`, `rs_calls.txt`; scripts `tools/ida/gfx_{xr,names,peek,grepdis,rs_calls}.py`.

### Car damage rendering (`car_damage_render`; done — was physics' request below)
- **Source**: every in-game non-traffic car root gets `CarDamageSource { head, brake, meshes }` at spawn (core turns it
  into `SetupCarDamage`, see core.md "Requests from graphics"). head/brake = LOD 0 chassis locators `l_head` /
  `l_brake` translation with z negated (CarRender 0x00110220; locator table = XMF flags bit 0, `+0x34` -> {count,
  64 B matrices, name offsets}, `Model::locator`, GameDataLoading 0x000A3CB0). Locator z is stored negated relative to
  the geometry, so after CarRender's negation it lies on the body: damage model space == XMF model space (nose +Z).
  Test: every chassis' head locator is within 0.35 of a body vertex.
- **Groups** (VertexIterator order, CarDamage 0x0006D920/0x0006DA40): chassis LOD 0 then each fitted part LOD 0; per
  object only `*(obj+84)` = **part 0**; every primitive, every draw; vertices k = 0..=max-min (index count if not
  indexed) read from the draw's stream offset (VertexIterator 0x002C4DA0); matrix = `*(part+0)` (= the object matrix
  in all car data, `Part.matrix_at`); element 0 = register 0 (position), element 1 = register 2 (normal, read as
  FLOAT3; car data is FLOAT3), element 2 = uv, 3 = diffuse (`Performance__2C4C70` desc offsets +8/+40/+152/+56).
  `Builder.damage` collects them and a per-bevy-vertex (group, k) map; those meshes are built `RenderAssetUsages::all`
  so `MeshChanged` rewrites positions/normals in place (test: writing every group back reproduces the meshes).
- **Lights** 0x00073700 (light 0 left head, 1 right head, 2 left brake, 3 right brake — by name in the binary; physics'
  comments call 0 "right"): in LOD 0 chassis + parts, per object the first primitive whose shader name contains
  `left_headlight`+`left_head_surround` / `right_…` / `left_brake`+`left_surround`+`left_reverse` / `right_…`
  (0x00073600) gets the broken shader, repair restores its own. Broken shaders (0x0006C290): blend 2, clamp, texture
  = stage texture of the first `left_headlight` (… `right_brake`) primitive with `_headlight`/`_brake` overwritten by
  `_headlight_dam`/`_brake_dam` (`350z_headlight_glow` -> `350z_headlight_dam`); surrounds/reverse take the brake
  side's, head surrounds the head side's plus z state (1,0,0) = no z write (guess). Head surrounds' prim type 3/2
  not modelled. CarRender 0x001157B0 sets bit `1 << light` of car data +2250 **of car 0 always** (assert nCar == 0;
  kept: applied to request 0) -> `CarLightDamage`, OR-ed into `CarLightInput.damage`.
- **Windows** 0x000730C0 (car flags +7636 bit 2): window 0..3 = stage texture `glass_f/b/l/r` (case-insensitive,
  every stage, every LOD of chassis and parts, 0x00072F80) -> `glass_?_damage{stage+1}` (COMMONT.VCD), repair ->
  `glass_?`; prim type 3 on stage 2 (else 2) not modelled. Done by cloning the primitive's stage material.
- **Car damage FX** (`car_damage_fx`, CarDamageFX.cpp 0x00078320..0x00079C90, object `byte_6D6808`; `CarDamageFxPlugin`
  added by `CarDamageRenderPlugin`, runs before `CarDamageRenderSet` so light bits are read before the break):
  - Input: `CarDamageRenderEvent` — new `CollisionFx { vehicle, point (world), impulse (halved) }` (core request in
    core.md), `LightBroken`, `WindowCrack`. Car matrix = root `GlobalTransform` (car data +0 = model space, nose +Z),
    locators from `CarDamageSource`, paint bytes from `CarSpec.colour` (+2236 = b, g, r).
  - Impacts 0x00078880 (from CarDamage 0x00070F40): damage car 0 only; cooldown `dword_6D67BC` = 5 (`dword_685370`)
    rendered frames, counted down in 0x00076E80 (GameManager 0x000B9010 = per render frame) — kept as 5/60 s; local
    hit = rigid inverse (0x002AB330) of the car; halved impulse ≥ 20 (`dword_685394`); d = z of the normalised local
    hit, |d| < 0.5 nothing (cooldown still set), ≥ 0.866 `Car_impacts_90_*` else `_45_*` (Plast_rear, smoke, spots;
    PARTDEF.VCD `car_impacts.slp`). Types are changed for good: Plast_rear count 32, spots colours (0,0,0,100)/(0,0,0,128),
    accel (0,-20,4,1), Plast_rear colours = VISOS(0x60 / 0, paint/2 doubled) (`byte_685384`, `dword_685380` = 2);
    matrix = car · RotY(±acos(-d), sign = -sign(local x)) at the world point. No free emitter: logged, the ones
    already taken leak (original).
  - Glass 0x000781C0 (light 0..3 -> slot 4..7, only if bit `1<<light` of car data +2250 is clear, i.e. `CarLightDamage`
    — set for car 0 only) / 0x00078270 (window 0..3, `Car_Glas_Front` for 0/1 else `Car_Glas_side`; its gate
    `byte_6D63A0[0]` in 0x000730C0 is assumed set) -> 0x00078320: emitter one-shot, table +164 (6 cars × {index, 8
    slots}). 0x000784A0 each frame moves live emitters: lights at `l_head` (4/5) / `l_brake` (6/7), x negated for 5/7,
    rotation `dword_685350` (0,∓2.3)/(0,∓0.5); windows at `dword_6D69B8` (0,0.784,0.35)/(0,0.784,-1)/(±0.7,0.65,-0.2),
    rotation `dword_685330` (0.503,π)/(1,0)/(0.3,∓1); 0x002AB7E0(a,b,c) = RotX(a)·RotY(b)·RotZ(c) (d3d order; D3DX
    X/Y/Z assumed for 0x002AB540/560/580); world = car · rot, translation = car · pos. Stopped emitters drop out.
  - Glass sound (audio's request, done): `car_damage_fx::GlassFxFired { request }` is written once per glass emitter
    started, after the "no free emitters" early-out, where 0x00078320 calls `VsAudio::PlayPhysicsImpact` (crc
    0x3B5B1A82, rand%700+100); core forwards it as `AudioCommand::GlassImpact` at that car's position (core.md).
  - Bonnet smoke of 0x00076E80: see "CarDamage bonnet smoke" (never emits). Not ported: debug tweaks `dword_685374/78`.
- **Particle fix (stateless_particle_type)**: 0x00322680 is `_ftol(ceil(v/step))·step` (0x000863B0 -> CRT 0x003E2A80,
  FPU control word 0x1B3F = round up; 0x003E27F5 with 0x173F is `floor`), not a truncation: every one-shot whose
  duration is shorter than one life (all the Car_impacts types) was freed at once. `round_up` now, also in 0x002D3C80.
- Viewer: `trackview --track ExecTest --car LA/350z --damage --cam 70,-10.8,142,0,-14` runs the physics model on car 1
  with four scripted hits (frames 104..128, 8 apart; CollisionFx sent with the world point; add `--particles` for the
  damage FX types) (`track_view_damage`, the only reason visos_render depends on velocity_physics). Shot:
  `research/screens/gfx_damage.png` (dented nose, broken headlights, cracked windscreen).
- Decompiles: `research/gfxdmg/lplate.c` (0x12DC60..0x12E5C0), `plate_ctx.c` (callers, GamePlayer init, setters),
  `carrender_decals.c` (0x115BB0..0x116700), `damagefx.c` (next: CarDamageFX), `vertexiter.c`, `locators.c`.

### Licence plates and decals (`licence_plate_render`, `car_render` decals)
- **Car model slots** (CarRender data): +1092.. chassis LODs 0-2; +1208 -> 14 × 44 B entries by BODYPART type
  (0 FRONTBUMPER, 1 REARBUMPER, 2 BONNET, 3 SKIRT = our `trim`, 4 SPOILER ...), entry +0/+4/+8 = LODs 0-2; +1184[6]
  = front-end-only extra models (FE3DVehicles 0x25D600 via 0x110190; zero in game -> our FE doors). Prim +4 = shader
  type from GameDataLoading 0x9EAF0 (`car_shader_type(name, 0, fe)`). `CarModels` (stems + per-LOD record ranges)
  is built in `spawn_car` for these lookups.
- **Decals** (`CarSpec.decals: CarDecals`, `CarDecals::from_setup(setup, db)` = setup +0x194..+0x228 as
  `a3[101..138]`): CarRender 0x1163A0 (car init, after "Car Decals") opens `\Vehicles\DECALST.VCD`, 0x116510 per car
  (skips vehicle type 6/7): `bonnet_decal` (0x115BB0: BONNET part LODs 0-2) and `side_decal` (chassis LODs 0-2), then
  +1184 models: every **type 4** shader whose name contains it gets the texture (0x115CC0). Logos 0x116040: name
  `logo%02d` (slot + 1), model by kind (0x116AD0: 0 chassis, 1 f_bumper, 2 r_bumper, 3 skirt, 4/5 chassis with the
  digits overwritten -> `logofw`/`logorw`), first **type 5** shader containing it (FuryInstance 0x12AC80); not found:
  kind 5 retries the spoiler, kind 0 the +1184 models. Slots: chassis 10 (kind 0), f/r bumper and skirt 2 each,
  front/rear screen 1 each. Texture 0x116280 = record name (+0x18) + `_white` (flag 1) / `_black` (flag 2); empty
  slot = "bonnet_off". Setting a shader's texture hits every prim using it; here: a CarDecal program material per
  texture (`CarProgramCtx::program`). Quirks kept: `logofw/logorw_alpha_*` classify as type 2, so the screen strips
  never change; **240 of 245 DECALS.BIN records have no texture in the shipped DECALST.VCD** (vinyl_* names vs
  bonnet001/side001 files) — only ML_4x1__HKS01/HKS02/Blitz01/Injen01 and VT_window_HKS01 resolve (test). No shipped
  setup carries decals, so in game every decal shader shows bonnet_off. Missing "side_off" texture warns (in data).
- **FE wheel orientation (2026-11-07)**: FE3DVehicles 0x25B4D0 uses the f_l/r_l locators at +x (car left), left wheels (a4 0/1) unrotated, right wheels (a4 2/3) a -1 x mirror matrix (`Renderer 0x2AC070`). The port had rotated left wheels by PI (race convention, left at -x), so FE rims faced inward. Now `wheel_base_rot` = identity for all and the right wheels' LOD groups carry scale -1 x (`spawn_car`). Shots `gfx_decals_fixed.png`, `gfx_wheel_crop_fixed.png`.
- **Foyer decal invisible (2026-11-07)**: not a render bug; `foyer.rs` never sets `CarSpec.decals` nor keys the respawn on it (request in docs/frontend.md). Render path unchanged (`trackview --decals`, `gfx_plates.png`).
- **Slot counts** `CarRenderParts.decal_slots` = CarRender 0x116BC0 (`decal_slot_count`): per kind, kinds 0-3 count `logo01..logo10` that `find_shader` (type 5) finds in the kind's LOD0 model (kind 0 also the `_door` extras); kinds 4/5 count the single `logofw`/`logorw` shader (kind 5 + spoiler's, so up to 2). Because those classify as type 2 (see above), 4/5 are normally 0 here as in the original's classification. Traffic: 0.
- **Licence plate** (LicencePlateRender.cpp; object `unk_71FEE4`: +0 background shader, +4 "customPlate", +8 current
  background, +12.. the three, +24 render target, +28 enabled, +29 front-end flag = GameManager::CommonInit's arg):
  init 0x12DC60 loads `la_plate`/`london_plate`/`tokyo_plate` (`\VFX\misc.vcd`; in game only the style at init);
  0x12E1E0 (CarRender 0x110220 and the part setters 0x10F9C0/0x10FD20, **car 0's REARBUMPER LOD 0 only**) makes the
  64×32 "LPlate" target (fmt 18) and swaps the shader of the first prim whose stage texture contains `plate01` (every
  prim using it, 0x12AD70) for customPlate (texture = target, stage flags 1, blend 1, vertex program 1 LinearCopy:
  oD0 = v3 = white without a diffuse stream -> `arg.x = 0`). 0x12DF40 (from CarRender 0x119900 each frame when car 0
  has parts): background quad (GFXHelper 0x12B720 at `dword_71FED8/DC` = 0, 64×32, colour 0x80 = white) then the
  text: font helper id 6 in game / 2 in the front end, width = GetStringWidth at the font's scale (+0x40); > 58 ->
  scale·58/width, width 58; x = (64 − width)·0.5, y = 16 − 5; colour per style `dword_71FF04` (VISOS a,r,g,b: LA
  (0x80,0x0C,0x12,0x21), London (0x80,8,8,8), Tokyo (0x80,0,0x2E,0)). Text `unk_6F71E0` = "OCTANE", style
  `dword_6F71DC` = 0 (GamePlayer 0x0C1930; FEOnlineSignInScreen 0x23AE10/0x23AEF0 set them). 0x12E480 (CarRender
  0x1190D0) swaps customPlate to the plain background during another pass, 0x12E400 = debug draw (`byte_71FEE0`);
  neither ported. Here: `LicencePlate { style, text, front_end }` resource, the target is composed on the cpu
  (`render_plate`) when it changes; cars with `CarSpec.licence_plate` or request 0 get `CarPlatePrims`. **Text needs
  `PlateFontHook`** (trait `PlateFont`: scale, string_width, draw_string_at -> glyph quads, page) from the frontend
  — provided now (`velocity_frontend::plate_font::PlateFonts::hook()`, inserted at frontend startup; not shot-checked)
  (fonts are theirs); without it only the background shows.
- Viewer: `trackview --track ExecTest --car LA/350z --decals 0 --plate VELOCITY[,style] --caryaw 200 --cam
  70,-10.8,142,0,-14` (`--decals n` = usable decals per kind, `--caryaw` = car turn vs camera). Shot:
  `research/screens/gfx_plates.png` (rear plate background, white logo decals on the side).
- **track_view.rs was rebuilt in this stage** (an edit script truncated it, no copy existed): same args; foyer
  loading (`--foyer`: `Model\<f>.xmf` + `<F>T.VCD`, Lights.txt, Crowd/Dance.txt Name/CharPos/Rotation/STANDANIM) and
  the default overview camera are rewritten, not verified by a shot. The viewer adds the full-screen effects plugins
  again (the black capture is fixed, see "Full-screen effects").

### Requests from physics (CarDamage, see physics.md "CarDamage") — done (see "Car damage rendering")
- When a race car's model loads (CarRender 0x00110220 -> 0x0006AE70), write `velocity_physics::plugin::SetupCarDamage
  { vehicle, model: DamageModel { head: l_head locator, brake: l_brake locator (both z negated, car data +768/+896),
  meshes }, model_to_root, deform }`. `meshes` = every vertex group of the body model (+1092) then each model of the
  +1208 list, in VertexIterator order, positions in mesh space + element 1, `matrix` = mesh -> model (row-vector);
  `model_to_root` = damage model space (x lateral, y up, z front) -> the car root (+X forward); `deform` true for the
  player car. Max 8 cars.
- On `PhysicsCarDamage(MeshChanged { vehicle, mesh })` re-upload that mesh's positions (and element 1) from
  `Res<CarDamageManager>.get(vehicle).model.meshes[mesh]`.
- `LightBroken { light }` / `LightRepaired`: CarDamage 0x00073700 shader swaps (`broken*` light shaders) + CarRender
  0x001157B0(light, broken) + glass particles 0x000781C0 (light 0 right head, 1 left head, 2 right brake, 3 left brake).
- `WindowCrack { window, stage }` / `WindowRepaired`: 0x000730C0 glass crack texture stage (window 0 front, 1 rear,
  2/3 sides; `glass_*_damage`). `CollisionFx { msg }`: CarDamageFX 0x00078880 impact particles (`Car_impacts_*`).
- Car flags (`CarDamage::car_flags`, car data +2256: 1 front / 2 rear lights intact) gate the light rendering 0x0006B1D0.

### Professional races: armco rails and posts (fixed 2026-11-09; the first reading "no barriers" was wrong)
- Professional = `VsAIWorld::Init(barriers = !professional, armco = professional)` (InGameInit 0xB6773) and `BarrierRender 0x107560(!professional)`
  draws no arrow / no-entry barriers, but Armco.cpp draws the loaded `armco01.ar` itself: GameManager 0xB9A83 (per view, before the
  barriers) calls `sub_12160` (0x12160) whenever armco lines are loaded; the render flag is irrelevant. Mesh `Armco::BuildMesh` 0x12440
  (FCMesh + FCQuadTree) is collision only; the visuals are built in `sub_11C50` (0x11C50, from `Armco::Load` 0x11090).
- Rails: lines of kind 2 / 3 -> `CStripFromSplines 0x810F0` strip of (base, base + 1.2 m) vertex pairs (kind 3: top first), all points (closed
  lines repeat the first), u = side 0/1 (kind 3 flips it except for the first pair), v = distance along each side scaled to whole `trunc(len/32 + 0.5)`
  repeats (0x81C80), lit per vertex (0x81940: ambient 0.1 + lights 0.7/0.3/0.2 along (0,66,-115), (100,66,57), (-100,66,57), progressive normals
  0x81780), texture `Armco05` of SCENERYT.VCD (`off_684E40[0]`, the random index arg is 0), ArmcoShader: blend 3, wrap, z test+write.
  Posts: the `armco_post` model (SCENERYM.VCD) translated to every point with w == 1 (>= 5 m apart), 0x12290.
- Port: `visos_render::barrier_render::{ArmcoLineDraw, armco_strip, BarrierRender::draw_armco}` (`LoadBarrierRender.armco`, filled by the bin's
  `barriers.rs` from `AiWorld.armco`), `EffectTexture::Scenery`. Frustum-culled in 8-segment chunks (the original culls per primitive chunk by
  sphere, 0x7D4E0/0x7D3C0). Light sign / packed-direction rounding assumed (N.L > 0 with the stored directions). Seen: LADown Circuit/Race05,
  189 chunks + 527 posts, red/white rails both sides (`research/screens/armco_3_driving.png`).
- Shipped data: pro races ship `armco*.ar` (LADown Circuit/Race05-06, Sprint/Race04-05, London Circuit/Race03,07,10, London2 Circuit/Race02,04,05,07,10,
  Sprint/Race02); underground races `Barier*.br` / `NoEntr*.br`; LACoast, Tokyo*, ExecTest, Cruise, Circuit/Race11-15 ship neither.

### Requests from core (collision listener) — done, see "Barrier render"; core's wiring requested in core.md

### Barrier render (`barrier_render`, BarrierRender.cpp 0x00106B30..0x0010B740; `BarrierRenderPlugin` in RenderPlugin)
- **API**: `LoadBarrierRender { arrows, no_entry, render }` (AI world `barriers[0]` / `[1]` lines' points; InGameInit
  -> 0x00107560), `UnloadBarrierRender` (0x00107CE0), `BarrierHit { car_a, material, point }` (0x0010B590; core sends
  while `byte_6F70E9 == 0`). `BarrierTweaks` = the "Graphics.Spline Barriers" tweaks (frequency 6, colour 1,
  amplitude 2, speed 0.3, duration 1, UV wobble 256, red (48,0,0) from static init 0x004244B0). Drawn in
  `EffectSet::Build` through `EffectDraw`. Collision mesh 0x00108030 stays velocity_ai's.
- **Render data** (72 B at +12, 0x00106B30): +0 bounding sphere (centre of the min/max box, radius to max), +16 line,
  +20 u per point in 4096ths (u += seg · 1/8 m, `dword_705F84` = 1/(4·2); then scaled so the total is whole thirds:
  `floor(3u)/(3u)`, CRT floor 0x003E27F5), +24 d² to the nearer end, +28 kind, +30/+31 caps, +32 length, +36 hit
  (−1), +40/+56 cap points 3 m in from each end (only if the end segment is longer, w = u ± int(3·u/m)) — 0x00107100,
  arrow barriers only. Object: +44/+48 scroll clock / quantised scroll (−0.000667/ms, `int(6·clock) % 6 / 6`), +52
  current barrier = the nearest drawn, +56 wobble clock (+= 0.3 · ms · 0.025, < 0 off). Frame time taken in ms.
- **Draw** 0x00109080 (GameManager 0x000B9010, render frame): frustum-visible barriers (0x00108E60), sorted far to
  near (0x0010B840; +52 = last), skipped from d² ≥ 62500; alpha 128, below 2500 lerp to 92, beyond 40000 fading to 0
  (`dword_705F98` = 1/22500). Shaders built at init: "Arrows" = misc.vcd `BarrierArrows`, blend 2, z test no write,
  wrap, bias 100 (0x002A1760 = depth bias), `ShaderManagerBase::2A15F0(1)` unknown (as on "sunb"); "bit" =
  `UndergroundDash` same state; "X" = `UndergroundX` (z write; never drawn, texture not shipped).
  - Arrows: strips of ≤ 43 segments / 7 repeats (`dword_686578`), vertices p0,p0,p1..pn,pn alternating bottom/top
    (toggle runs on across batches), u = (u16)(U − base) /4096 read back signed, v = top; ends transparent; caps
    inserted at j == 2 of the first batch / j == count−2 of the last. Height 4 (`dword_68656C`).
  - Hit wobble 0x0010AAC0 (kind 0 with +36 ≥ 0): 1.5 m pieces (1..24 per segment), per vertex x = 2|d/len − hit|,
    phase = clock − 6x, live while 0 < phase < 2π·duration: w = clamp(sin(phase)(1 − phase/2π)·2, ±1), u += int(256w),
    rgb = red·|w| + 128(1 − |w|); first/last two vertices alpha 0 / half; strips of ≤ 50 vertices; when nothing is live
    the hit and clock reset. 0x0010B590: car a 0, material 21, current barrier kind 0, wobble idle or no hit → hit =
    clamp(dot(p − first, dir)/len, 0, 1) of the first-to-last chord, clock 0.
  - No-entry 0x00109D70: from each end up to length/2 − 2.5 (cut point interpolated), u × 2, dash strips 0x0010A610
    (≤ 47 segments, bottom y+0.5 / top y+4, u scrolls +/− by direction and is negated on the first half; the toggle
    starts from the debug build's 0xCC stack = top). `barrierStopMainOn` (SCENERYM.VCD, SCENERYT textures) between
    the cut points: x = along, y = up, z = x × up (FuryInstance 0x0012A7D0) — one hidden entity per barrier, shown
    while drawn. `barrierStopMainOff` is loaded but never drawn.
- Viewer: `trackview --barriers Circuit\Race01 [--barrierhit x,y,z]` (reads Barier01.br / NoEntr01.br itself; hit at
  frame 140). LADown Race01: 25 arrow + 24 no-entry barriers. Tests: u thirds, hit position, wobble run-out, sign gap.

### Requests from core (trailblazer render calls) — done: see "TrailBlazerRender — zone trails" and "Danger particles"; the wiring back is requested in core.md
- The trailblazer now runs in game (LADown Circuit Race02; danger overlay/warp seen through `DangerZoneInput`). Its
  other render calls arrive as `TrailBlazerMessage { vehicle, event }` (velocity_game::game_mode::trail_blazer) and
  have no consumer yet: `Render { spline: Some(crc) }` = TrailBlazerRender 0x001521E0(spline, 1, 0) on
  `GameManager__C04F0()` (`dword_7374CC`), `Render { spline: None }` = 0x001521E0(old spline, 3, 1);
  `Intensity(level)` = 0x000C6240 (`dword_7374CC` +8); `DangerZone(Some(level))` = DangerZoneRender 0x001234A0 on
  `byte_719168` (danger particles, gated by its +29, which 0x000C6080 zeroes at start and the TriggerBigParticles
  timeline trigger sets) and `DangerZone(None)` = 0x001238A0. Also 0x000C6120 writes `unk_718AC0` +32 = 0 (C6290).
  The spline points are the ai's trailblazer road line (`VsAIWorld::find_trail_blazer_line_mut(crc)`). Read the
  messages in the same GameTick (Effects set or later) or keep your own cursor.

### Requests from ai (puddle test never splashes — `puddle_render::puddle_test`) — done (fixed, seen in wet London)
- 0x00132A00 at 0x132A69 reads `*(float*)(this + 48*k + 44)`: `this`-relative, records start at +20, so that is the
  record's **+24 race line distance** (`Puddle.distance`), not a never-written record +44. The loop is "last puddle k
  with `d > puddles[k].distance`" (j = 0 if none). The port tests `d > 0.0`, so every car always tests only the last
  and the first puddle and wet London never splashes the lens (`SMOKE_WET=1 SMOKE_AREA=2`). The ai inputs are fine:
  `PuddleInput.distance` = the race vehicle's +0x2A8 raceline distance, set every update for players and opponents.

### Coverage audit (`tools/coverage_gfx/cov.py` -> `research/graphics_coverage.txt`; 2026-11-07, reconciled same day)
Regions: Render\*, Renderer\*, elParticle, StatelessParticles, Vertex/BoneAnimation, VsRender, ParticleManager (69 files).
Strict match = function start address in a `/// File.cpp 0xA[ / 0xB + 0xC]` origin line of `crates/visos_render`
(separators `/ , +`; every address of a merged port goes on that one line).
- First audit: 217/3289 fn, 174858/651558 B (26.8 %), with 451 fn "mentioned" (address only in a comment).
- **Reconciled: 447/3289 fn, 290010/651558 B (44.5 %); 50.1 % of the non-platform bytes** (header line 3).
  `tools/coverage_gfx/mentions.py` -> `research/gfx_mentions.tsv` lists where each mentioned address appears
  (kind doc-own-first = first line of a fn's doc block starts with the address = the port of it);
  `tools/coverage_gfx/plan.py` -> `research/gfx_origin_plan.tsv` = the 226 doc lines that got a `File.cpp` prefix
  (by hand, Edit tool). File = the objregions region, except where the region boundary is off (SkyRender
  0x13BE20, slipstream 0x13F380/0x13F770, spark 0x1418D0/0x141980, TrackFX 0x148CF0..0x149970, flares
  0x123A20..0x1270E0 cite their real file). Not converted (kept "mentioned" = **not ported**, the address is only a
  reference: a caller, a callee, a constant's source, or the doc line is on a const/field): 221 fn / 67 KB; the
  biggest are 0x11BA70 (police car data copy), 0x147DC0 (start-line init, tweak site only), 0x108030 (ai's barrier
  render), 0x13C390 `RenderSunMoon`, 0x131F50 (puddle draw), 0x1134A0 (wheel blur gate, partly in `update_wheel_blur`).
  Second copies of one helper keep no origin line (0x2B11C0 cited once in `barrier_render`).
- Tags re-done honestly (`tag()` in cov.py): **platform-replaced** now only the pure D3D/XGRPH wrappers of Renderer\
  (TextureManager, VertexStream, PrimitiveManager, RenderTargetManager, HighLevelTextureManager_XBOX,
  TransformationManager, VertexIterator, RenderShaderProgramSupport, GenRenderer): 553 fn / 77 KB unported;
  **renderer-logic** (ShaderManagerBase, ShaderManager, Renderer, DrawList, ShaderProgramManager, VISOS_ColourXBOX —
  stage/blend state, draw order, program binding, maths; what a draw looks like) 247 fn / 38 KB; Render\Pc_xbox\
  *_PC_XBOX, elParticleRenderer_Xbox, StatelessParticleRenderInst_XBOX are game code -> visible-rendering
  (1167 fn / 202 KB unported). getter 853 / 20 KB, debug 6, template-dup 6, mis-attributed 9.
- The first audit's "purpose" guesses for the top gaps were wrong in places (IDA-checked):
  **0x2D6700** = `CType` **save** (writes every field through the file writer 0x2E4020, the one CTweakSystem::Export
  / CSkyRender::SaveConfig use; +256 only from version 0x6B; caller 0x2D4747 = the editor's save) — no rendering;
  the reader is `CType::parse`. **0x10F330** = `CarRender::AddCar` (m_nCars < 22, defaults, `l_head`/`l_brake`
  locators z negated, +1808/+1812 = 2.9/4.8 without a vehicle, light lookups) — merged into `spawn_car` (origin
  line added). **0x583F0** = VsRender's start-up light set (ported, see "Light sets"). **0x117770** = TopLight
  locators (`CarFlareLocators::from_chassis`). **0x1181C0** = the per-car light-prim draw of CarLightInputs 0x112A50
  (brake/reverse/head prims +2012/+2028/+2044/+2060/+2076/+2092 through GFXHelper 0x12B0E0 with the car's light set
  at +1028, damaged sides (+2324 bits) with a second, darkened set; front end (+49) also draws +2108.. per part).
  **0x1123E0** (CarLightInputs end, also Characters 0x7A460) = the car's point-light set for the next frame: over
  VsRender's point lights (+0x124 count, +0x128 32-B records; radius 0 skipped) t = 1 − d²/r² on the **xz plane**
  (t > 0.001), Σt, Σ pos·t, Σ colour·t; colour clamped to 1.99 (`flt_686798`), w 0; dir = normalise(Σpos/Σt − car
  pos) = the car's light set at +1028 (32-B set, see "Light sets": +0 direction 0, also `unk_718208`), colour 0 =
  colour · `flt_706534` · 1.0, colours 1/2 black, ambient (+24) = colour · 0.75 · `flt_706534` with a 0.25·Σt
  (0.25 when Σt > 1); colour also to `unk_7181C4`. **Ported** (`VsLights::car_point_light_set`, test
  `car_point_light_set_xz_weights`): `flt_706534` = the "Brightness" tweak (CarPalette 0x10E3F0, registered before
  0x10D7E0 stores it, so its tweak init is the BSS 0), written by CarPalette 0x10D7E0 = `dword_686668` (game) /
  `dword_68666C` (front end), both 1.0 (`CAR_BRIGHTNESS`); also by the debug override 0x11AD30. Call site: end of
  CarLightInputs 0x112A50 (from 0x1129F0, after the car draw lists are flushed; skipped for the car the view sits in,
  +2251; 0x1129F0 itself skipped when car +2088 == 2, not modelled), pos = car matrix row 3 (0x5C820(3)); directions
  1/2 and the count stay. The next frame's 0x119EF0 rewrites the whole set before the cars draw, so the car body never
  sees it: what does is everything that copies +1028 later in the frame — the 3d hud set (OSD3dInterface 0xEA2B0) and
  `bind_beam_programs`' player set. So `CarLightSets` now holds the post-pass set (`bind_car_programs`). Readers of
  `unk_718208` / `unk_7181C4`: Characters 0x7A460 (only when SpecularChars is not loaded, byte_736530 == 0 — in game it
  always is, so not ported) and two **unreferenced** functions 0x113B70 / 0x114680 (no code or data refs; created in
  the IDB, decompiler gives nothing). Σt = 0 → 0/0 NaN direction in the original (black colour); zero here.
- **0x10F9C0** (861, VehicleEntity 0x1C08B0 / FE3DVehicles) = `CarRender::SetPart` (part slot a4 of 14, 44-B records
  at +1208 from the m_nCarsWithParts pool, +28 = car +1028 light set, matrix copy for part 10 below
  `dword_6F5C00`, carbon prims 0x1174F0, licence plate on slot 1) — the parts are built by `spawn_car`; not cited.
- **ClosestLights / MTV stage 2026-11-07**: → **472 fn / 304491 B (46.7 %, 52.6 % excl. platform)**: VsRender
  0x5A710 + CarRender 0x118D70, LightStreak.cpp 0x1300D0 / 0x130310 / 0x130340 / 0x130380 / 0x1306E0 / 0x130C00 /
  0x130DC0 (IDB: `MTVEffect::*`, `VsRender::ClosestLights`, `CarRender::ClosestLights`). Left: 0x1181C0, 0x10F9C0.
- **Porting stage 2026-11-07**: 450 fn / 293733 B (45.1 %, 50.7 % excl. platform) → **463 fn / 299536 B (46.0 %,
  51.8 %)**: CarRender 0x1123E0 (above), CatsEyeRender (12/15, see "Cat's eyes"). Gaps checked and left:
  - **0x5C220** (GameManager::InGameUpdate) = a debug orbit camera (pad axes 24/40/25, buttons 2/3 cycle vehicles,
    toggled by 0x5C1E0 / `byte_736554`; `dword_6D5340`) — debug, no port.
  - **0x599E0** (from 0xB9D83) = debug draw of the point lights as dot spheres (FontManager 0x2B0C20) when
    `dword_6D533C` != 0; nothing writes it → never runs.
  - **0x5A710 ClosestLights(pos, dirs, colours, ambient)** — **ported** (see "Light sets" → ClosestLights): with null outputs it writes
    **VsRender's own scene set** (+4 dirs, +0x34 colours, +0x64 ambient = itself): pos.y += 0.75 (`dword_685074`),
    3D d² < r² → atten = 1 − d/r (assert 0..1), insertion into 3 slots by atten (init 0 / −1), slot with a light:
    dir = normalise(pos − light) (negated, since outputs were null), colour = light colour · atten; empty slot: colour =
    Armco 0x136B0 (zero), dir kept. Skipped when the point-light count (+0x124) is 0. Caller 0x118D70 (gate
    `byte_6865E4` = 1, FeLightsAddTweaks tweak) on the car's position (`VsAIRaceVehicle__23DD0(car +304)`), called from
    CarRender::AddCar 0x10F98A and FECK2Element3D::LoadFoyerLights 0x2993BF.
  - **LightStreak 0x1300D0..0x1310E3** — **ported** (see "MTV glitch overlay"); first audit notes: = the front end's
    "MTV" glitch overlay, not light streaks: init 0x1300D0 (FrontEndInit; `MTVLines`, `FE_Glitch_%s` from
    `off_686C40`, shader +160 stage flags 128 blend 2 no z, +164 "MTVTint" blend 23), start 0x130310(tex, flag)
    (QuickRace car select ChangeCar 0x225820 / 0x225E84 / 0x227744, FE3DCamera 0x275EF5), stop 0x130340 (FE3DCamera
    0x275FBE), drawn by FECK2ElementCamera 0x2999B0 last: 5 scrolling bars (x 0..512, spawn chance `dword_686E7C`·0.004),
    frame sequences from 4 tables at 0x686E50 (8 B: frames ptr, u16 frames, u16 per frame; 16-B quads x, y, size, tex),
    a timed tint (+92, fade in `dword_686E98` / out `dword_686E9C`) through the full-screen effects (0x15C340).
  - Still open from the list: 2D3680 1019 StatelessParticleMgr, 13AB80 853 skidsmoke, **0x1181C0** (decompiled to
    `research/gfx/lightprim_dc.c`: per car with prims at +2024 and LOD model +1092[+2249]: light set = car set +1028
    dirs with colours from 0x11B7B0(+864 scene, +980 car matrix) and 3× 0x5AF20 point lights through 0x11B7B0; a second
    darkened set (`flt_686818` mix) when +2324 has damage bits; `dword_70657C`+12 = car +980.w for the draws; GFXHelper
    0x12B0E0 on the brake +2012 / +2076 (if +2088), reverse +2028 / +2092 (if +2104), head +2044 (bit 4), +2060 (bit 8)
    prim lists with the damaged set per side bit; front end (+49) also each extra part's +2108 + 16k with model +1184[k]
    — needs a look at how `model_builder`'s BrakeLight / revlight stage modes split these prims first), **0x10F9C0**.

### MTV glitch overlay (`mtv_effect::{MtvEffect, MtvEffectPlugin}`, LightStreak.cpp 0x1300D0..0x1310E3; in RenderPlugin)
- Object `byte_720208` (init 0x1300D0 at GameManager::FrontEndInit 0xB587D; tweaks 0x131130 "Frontend.MTV Effect",
  wired in `gfx_tweaks`, "Enabled" = +0). Layout: +0 running, +1 no-glitch flag, +4 five 16-B bars (x, alpha (0..128,
  > 0 = live), width, speed), +84 frame (float), +88 sequence ptr, +92 tint clock (−1 off), +96 time left, +100
  "MTVLines" texture, +104 14 × "FE_Glitch_%s" (`off_686C40`; all in `\VFX\misc.vcd`), +160 shader (stage flags 128,
  blend 2, no z), +164 "MTVTint" (blend 23 = DESTCOLOR·src, rgb only).
- start 0x130310(seconds, flag) / stop 0x130340(immediate); callers: FE3DCamera 0x275E50 (camera move, time = the
  move's), FE3DCamera 0x275F10 (stop(0) on an interrupted move), quick-race car select ChangeCar 0x225820 / 0x225E84 /
  0x227744 (start(0.1, 1)). Hook requested in frontend.md.
- update 0x130380 per drawn frame (dt = FE timer ms · 0.001; spawn chances are per frame — kept): time −= dt; bars move,
  die off 0..512; tint clock runs to in + out; while time > 0: (no flag) each dead bar spawns with chance Line Freq ·
  0.004 at x 512, r = rand(0.33, 1), speed −Line Speed·r, width Line Width, alpha Line Alpha·128·r; (no flag, no sequence)
  sequence rand() % 4 with chance Image Freq · 0.004; tint with chance Tint Freq · 0.004; else nothing alive → +0 = 0.
- draw 0x1306E0 (FECK2ElementCamera 0x2999B0, last): update; bars = 0x130DC0 quad (x0, y0, x1, y1, tex, colour, u0, v0,
  u1, v1; Renderer 0x2A9B00 pretransformed, 512² space) x..x+width × 0..512, OvColour(alpha, 0, 0, 0) (doubled);
  sequence (`0x686E50`, 4 × {quads, frames, per frame}, 16-B quads x, y, size, tex) frame = (int)+84: quad size ·
  0.01 · `dword_686C78`[tex], width size·3/4, colour OvColour(128·Image Alpha, 0x80 ×3), seq 3 frame 2 mirrored
  (u 1→0); +84 += dt · Image Speed (15/s); tint k = clock/in rising, 1 − (clock − in)/out falling, colour =
  lerp((128)×4, (1, 0.35, 0)·128/0.475, Tint Intensity·k) (FuryInstance 0x12A4F0; luma weights 0.3/0.5/0.2, used if
  > 0.1), each /128 capped at 1 → VisosColour → FSE 0x15C340 screen quad in "MTVTint". "Desat Intensity" (flag 2 of
  `dword_686EA4` = 3) is computed and passed to 0x130C00, which ignores it.
- Port: `MtvEffect::frame(dt)` returns the quads + tint (tested); `mtv_draw` (PostUpdate) draws them with an
  orthographic 512² camera (order `MTV_ORDER` 3 after the FE reflection passes, layer 17, the FE camera's target and
  msaa, output replace) as pooled `XboxStageMaterial` quads, later draws nearer. Only while a `FrontEndCamera` exists.
  Viewer: `trackview --track ExecTest --fecam --mtv --size 640x480` → `research/screens/gfx_mtv.png` (thin dark bars,
  a mirrored glitch image bottom left, the orange tint; over the dark foyers the black bars barely show).

### Light sets: ClosestLights (VsRender.cpp 0x5A710; `VsLights::closest_lights`, `car_render::fe_closest_lights`)
- Only caller CarRender 0x118D70 (gate `byte_6865E4` = 1, `FE_CLOSEST_LIGHTS`) with car 0's matrix row 3 (CarRender
  +304), called from AddCar 0x10F98A when the first car is added (AddCar's only caller FE3DVehicles 0x25B310 → front
  end only) and FECK2Element3D::LoadFoyerLights 0x2993BF after lights.txt / PtLights.txt. Null outputs → VsRender's
  own scene set: p = pos + (0, 0.75, 0), 3D d² < r² → atten 1 − d/r sorted into 3 slots (init 0 / −1); a filled slot:
  dir = −normalise(light − p), colour = light rgb · atten (w 1); an empty slot black, dir kept; ambient unchanged;
  nothing without point lights. Port: `fe_closest_lights` (Update, before `update_car_lights`) applies it to
  `TrackLights` when the front-end car count goes 0 → >0 or when `TrackLights` is replaced while a `FrontEndCamera`
  exists (the foyer reload, `foyer.rs` inserts both resources); car 0 = lowest `request` among the newly added, its
  last position kept after it goes (the original's record outlives the car). The FE removes cars one by one, the
  original only on a full clear. Test `closest_lights_sorted_slots`; `track_view` `--foyer` loads PtLights.txt too.

### Cat's eyes (`cats_eye_render`, CatsEyeRender.cpp 0x11CDB0..0x11E080; `CatsEyeRenderPlugin` in RenderPlugin)
- **API**: `LoadCatsEyes { race_dir }` (with trailing `\`; reads `<dir>CatsEyes.ced`, GameDataLoading 0x9D055 → 0x11CDB0
  on object `unk_7184E8`), `UnloadCatsEyes`; wiring requested in core.md. Only LADown Circuit Race02 / Race09 ship a
  `.ced` (the 6 LACoast `CatsEyes.txt` are editor sources, never read). 0x11E080.. is CollisionMeshRender (debug, misfiled).
- **File**: "CatEye", +6 u16 version 1 (`CATS_EYE_VERSION`), +8 u16 sectors, +10 u16 sector table offset / 4; sector
  20 B = centre xyz, radius, +16 u16 string offset / 4; string = u16 kind (0 red, 1 green: `dword_71851C`, static init
  0x424D40, OvColourXbox(0x80, …)), u16 n, start xyz, n × i16[3] mm deltas → n + 1 points. **Original bug kept**:
  strings are packed at 2-byte steps, so the 2 strings per file that do not start on a dword are read 2 bytes early
  (kind 0x60DC / 0x3920, n 1, start garbage or ≈ origin) and never come within draw range (test `every_cats_eye_file`).
- **Draw** 0x11CF70 (GameManager 0xB9435, views with render flag 0x100 — not the mirror): sectors with (r + 60)² > d²
  to the camera and sphere in the frustum → 0x11D450: depth d = camera plane 4 · p; 0 < d < 60 only. d ≤ the player
  car's depth: glow of size 0.3. Else fade = c⁵ · (1, or 1 − (d − 20)/40 from 20 m), c = car nose (row 2) ·
  normalise(p − car pos), size = 0.3 + 1.1 · d/60; fade > 0.01: glint, colour alpha = _ftol(255 · fade) (0x11DB40:
  camera +88 R · size · 0.5 by +104 up · size · 0.2, uv (0,0)(1,0)(0,1)(1,1) as a 4-vertex strip, colour through
  0x11DE40); d < 15 and fade < 0.5: glow of that size. kMAX_CATEYES 256 (glints + near glows). Glows 0x11D960: flat xz
  quad ± 0.3·w, colour 0x80808080 → white. Order: glows ("catseye2" = `catunlit`, blend 3) when `dword_68687C` bit 0,
  then glints (+12 + 4·`dword_7184E4` (BSS 0) = "catseye" `spec_t_gk_002`, blend 7) on bit 1; both z test no write,
  bias 128 (`dword_686868`); "catseye" also 0x2A15F0(1) (unmodelled); `spec3` "catseye2" at +16 never drawn. The car is
  the race's player vehicle in every view. Constants `dword_686858..686884` (no writers, no tweaks).
- **Seen faint, as in the data**: `spec_t_gk_002` averages rgb 41 / alpha 44 (`dump_textures`, ignored test), so the
  additive glints are nearly invisible. Viewer `trackview --track LADown --car LA/350z --caryaw 180 --catseyes
  Circuit\Race02 --cam 467.6,-14.3,-31,180,-6` (`VELOCITY_VIEW_CARDROP=2.9`, lower car-park road): 4 glints drawn
  (trace log), `research/screens/gfx_catseyes.png` shows nothing obvious; the same frame with the texture forced white
  (one-off experiment, removed) `gfx_catseyes_white.png` shows the green glints along the string. Test
  `glints_need_the_headlights`.

### Requests from frontend — done (second race load crash fixed)
- **Second race load panics** (render thread): every flow that loads a race again after returning to the front end
  (`SMOKE_CAREER=1` with a fresh `VELOCITY_SAVE_DIR`: hangout restarts UT0EVENT1; `SMOKE_ATTRACT=1` when a second
  attract race starts): first `Validation Error: depth attachment (160, 120) vs colour attachment (640, 480)` in
  Queue::submit, then `attempt to add with overflow` in bevy_pbr-0.19.1 `cluster/gpu.rs:1169` and a wgpu-hal swapchain
  semaphore panic; exit 127. Backtrace log `research/fe_smoke_career2.log` (RUST_BACKTRACE=1).
- **Root cause** (not leaked lights — nothing spawns bevy lights): the FSE pass-slot cameras are pooled and survive
  the race; they are retargeted (`execute_passes`, PostUpdate after `camera_system`) whenever a slot gets a pass with
  another target, and bevy recomputes a camera's target size only on projection/image changes — so that frame the
  camera extracts with the *old* target's size. bevy shares one depth texture per (target, msaa) and one main texture
  set per (target, usage, hdr, msaa), each sized by the first camera seen: a slot moved from the 160x120 glow buffer
  onto the 640x480 back buffer handed the back buffer's views a 160x120 depth. The invalid submission is dropped and
  bevy_pbr's GPU cluster readback then reads garbage (`next_power_of_two` overflow). Race 1 spawns fresh slots (sized
  on their first target); race 2 starts with slots still on race 1's last pass layout (pause menu / fades), so the
  first frames retarget across sizes. Fix: `visos_render::sync_camera_target_sizes` (Last, after every retarget
  command) sets `computed.target_info` + projection/clip of any *sized* camera whose target size changed (unsized
  cameras are left alone: bevy does not extract them, and sizing them early extracts a view with no clusters —
  zero-width "clustering dummy texture"). Registered with `refresh_retargeted_cameras` by `add_camera_target_fixes`
  (RenderPlugin and trackview). Test `retarget_tests::late_retarget_extracts_the_new_size` (fails without the sync).
- **Two-load check**: `trackview --track LADown --car LA/350z --cam 197,-5,34,0,-8 --reload 2 --size 640x480 --shot …`
  tears the scene down like `despawn_race` (camera despawned, track unloaded, cars removed) 60 frames after placing
  and loads it again 10 frames later, alternating glow passes 4/5 so the pass layout changes across loads.
  Without the sync it dies on reload 1 with the same 160x120-vs-640x480 error; with it: 3 loads, no validation error,
  `research/screens/gfx_reload.png` (third load). `SMOKE_CAREER=1` now reaches the second UT0EVENT1 load with no
  validation error (exit 0); SMOKE_ATTRACT (with frontend's guard) runs one attract race, exit 0.

### Requests from audio (zone heartbeat never starts) — answered: original behaviour, port unchanged
- Checked in the debug XBE (dumps `research/gfx/zone_start*.txt`, scripts `tools/ida/gfx_zone_start*.py`): the only
  caller of `VsAudio::StartZone` 0x0004B950 is 0x00162DED, and 0x00162D10 reads `+0x18+4p` (0x00162D95) **after**
  0x00154BD0 stored the stepped level at the same `this + 4p + 24`. In the zone 0x00154BD0 adds `dword_687524[!v15]`
  = +FadeRateA (0x3E800000 = 0.25) whenever GameBridge +545 (paused) is 0, so the level is ≥ 0.25 on the entry frame
  and StartZone is skipped; it can only fire on a *paused* frame in the zone at level 0 (or with FadeRateA ≤ 0 by
  tweak). So in the original the heartbeat practically never starts (StartZone: `byte_6D4A94` latch, preset 6 at
  0.2, event crc 0x678C58E1); StopZone 0x0004B9E0 has a second caller, `VsAudio::StartNos` 0x0004B4DF.
  StopZone does repeat every frame while out of the zone with level > 0 (fading); at level ≤ 0 out of the zone the
  function returns before it (mask −1). `zone_fx` matches all of this 1:1 (paused = frame_ms 0 → no step); kept.

### Requests from core (split screen world cameras, 2026-11-05)
- In a two-player race core now spawns **two** `TrackCamera`s (each with `velocity_game::game_mode::race_cars::
  PlayerView(p)`, same order `WORLD_CAMERA_ORDER`, player 1 `ClearColorConfig::None`) with `Camera.viewport` = the
  player's view rect of GameManager 0x000B3EC0 (`velocity_game::game_mode::game_views::GameViews::player_view(p)`:
  top / bottom halves, or left / right when options +0x24 `byte_6F9180` is set) scaled to the window. One player keeps
  one camera, no viewport. Please make everything that assumes a single `TrackCamera` per-camera (sky passes, FSE,
  DOF, zone FX, lens spray, camera rain, flares / `HeadlightView` per view, car layer 20, camera order enforcement),
  with the per-view render flags where the original has them. Player 1's view state = `PlayerView(1)`.
  **Answered** (see "Split screen — two players"): graphics needs `visos_render::view_render::TrackView { player: p,
  rect: GameViews::fraction(view) }` on each of those cameras (it cannot see `PlayerView`); then it keeps player 0's
  as the `TrackCamera`, strips it from player 1's, sets orders / clears / viewports / targets itself. Per-view render
  flags are all −1 (0x0B88B0) — what differs per view is listed there; the camera-built effects, DOF, danger FX and
  the in-car hide and the zone blur focus are per view now (only `TrackStreamViews` stays single).
- Done for your requests: `DummyPoliceCars` filled every frame (`physics_object_render::dummy_police_cars`, key = object
  index, matrix = lerped object pose; those objects' own models hidden), `CarSpec.police = true` on police specs,
  `FlareFlash` from `TimelineObjectMessage::ScreenFlash` (`race_effects_bridge::timeline_screen_flash`).

### Requests from physics (friction-spot puddles, 2026-11-06) — done (see "Runtime puddles"; core wiring requested)
- PhysicsObjectManager 0x001E1A20 (a hydrant hit, or the AI's VsAIRaceVehicle 0x00019DD0) appends a record to
  `unk_720840` = your `PuddleRender`: `LightStreak__133750` (+16 count, record copied to +20 + 48·n, no bound check),
  then qsort 0x001337C0 by +24 race distance (comparator 0x00133800) and 0x00133A20 (`byte_726388` +18 = 1). The
  record comes out of physics as `velocity_physics::plugin::PhysicsPuddleMessage(PhysicsPuddle { position (+0), radius
  (+12 w = 15.0), direction (+16/+20, random unit xz), race_distance (+24), kind (+28 = 2), size (+32/+36 = 30, 30),
  life (+40 = 480), flags (+44 = uninitialised | 1, 0xCCCCCCCD in the debug build) })`; core forwards it. Please add an
  `add` (= 0x00133750 + sort + dirty) to `PuddleRender` for it, and clear the list on `GameManager::RestartRace`
  (0x000BE20C) if not done already.

### Requests from frontend (black FE hangout background: D3DVSDT_NORMSHORT3 positions, 2026-11-06)
- **Root cause of the near-black main menu / THE SCENE**: `Foyers\hangout\Model\hangout.xmf` has one vertex desc,
  stride 16: register 0 position **format 0x31 = D3DVSDT_NORMSHORT3** (3 × i16 at +0), diffuse D3DCOLOR at +6, uv0
  NORMSHORT2 (0x21) at +10. `renderer_types::Model::read_attr` has no 0x31 arm (`_ => None`) and the builder's
  `unwrap_or([0.0; 4])` puts every vertex at the origin, so each of the 37 objects collapses onto its matrix
  translation: 67 meshes spawned and view-visible, zero-size boxes (measured in fecapture, `VELOCITY_FOYER_DEBUG=1`).
  The objects' matrices carry the de-quantisation (diagonal = half extent, e.g. Box1 (0.263, 0.205, 0.351), last
  row = centre); NORMSHORT3 `i16 / 32767` (the NV2A's normalisation, as your NORMSHORT2 arm) times that matrix
  gives the room (-14.3, 0.0, -42.4)..(20.2, 5.1, 18.0), inside the header sphere (2.96, 2.55, -12.21; 34.8).
- Please add `vsdt::NORMSHORT3 = 0x31 => [x, y, z as i16 / 32767.0, 1.0]` to `read_attr` (check whether any other
  shipped xmf uses a format that falls to `None`: a reader test over all models would catch it). Control, done on
  the frontend side only (`VELOCITY_FOYER_DEBUG=3`: `foyer.rs::expand_normshort3` rewrites the positions as FLOAT3
  before `Model::parse`): `research/screens/fe_bg_menu_normshort3.png` (main menu: the whole hangout set) and
  `fe_bg_scene_normshort3.png` (THE SCENE), vs `fe_bg_scene_before.png`.
- **Done (2026-11-06)**: `read_attr` decodes the NV2A normalised/short/byte families by component count (high nibble):
  NORMSHORT1-4 (0x11/21/31/41, i16 / 32767, missing = 0, w 1), SHORT1-4 (0x15..45, raw i16), PBYTE1-4 (0x14..44,
  byte / 255); FLOAT*, D3DCOLOR, NORMPACKED3 as before (format values are the XDK D3DVSDT constants; scale is the
  hardware's, not an exe table). `parse_every_model` now asserts every declared (non-NONE) input of every draw decodes;
  all 8102 models pass. The frontend stand-in is removed: `research/screens/fe_bg_menu_fixed.png` (fecapture main menu).
- **Dark vs bright foyer (checked 2026-11-06): dark is right.** The bright control `fe_bg_menu_normshort3.png` was
  shot with `VELOCITY_FOYER_DEBUG=3`, and `foyer_debug`'s `mode >= 2` branch also turned on the full-bright override
  (TFACTOR white, texture alone). Same fecapture frame (300), and the stand-in only appended FLOAT3 positions: diffuse
  and uv bytes were unchanged. hangout.xmf: every shader is program 0, stage flags 1, alpha test GREATER 40 (VHLRoad01
  ref 0, lights_bloom stencil). The background render struct has no light set, so Renderer 0x2A8FD0 → 0x2A8BE0(0)
  turns LIGHTING off and the prelit diffuse (avg 0.19/0.20/0.16, alpha 0x80 on every vertex) times the texture
  (MODULATE, the reset op) is the colour. Mean max-rgb of the 3D area: 47 full bright vs 14 → 22 lit (after the fix below).
- **Fixed: black patches in the foyer = back-face culling.** Every hangout.xmf draw is a D3DPT_TRIANGLESTRIP, and its
  degenerate stitching flips parity between segments: the flat floor triangles come out ~50 % up / 50 % down with no
  duplicate faces (`hangout_dump` test, `--ignored`, → `research/gfx/hangout_dump.txt`). The original never culls:
  D3DRS_CULLMODE (xbox index 0x93; setter 0x440E80) is only set to 0 = D3DCULL_NONE, in Renderer init 0x2A83CF,
  per shader in 0x2A4240 (0x2A4488), the reset 0x2A65CC and 0x2AA978 (`research/gfx/cull_push93.txt`, scripts
  `tools/ida/gfx_cull*.py`). Xbox complex-state indices confirmed from the data: ZENABLE 0x8F, STENCILENABLE 0x90,
  CULLMODE 0x93, TEXTUREFACTOR 0x94, ZBIAS 0x95. `apply_stage_pipeline_state` now sets `cull_mode = None` for every
  stage/program/FSE pipeline (was Back). Shot `research/screens/fe_bg_menu_final.png` (3D area non-zero 74 % → 98.5 %).
- **Fixed (2026-11-08, user report vs real Xbox `research/screens/ref_xbox_hangout_profile.jpg`): flat / washed-out
  hangout = uv not de-quantised.** hangout.xmf uv0 is NORMSHORT2 (format 33): per shader the uvs span 0..0.125 (±0.125
  boxes, ±1 pipes), i.e. the authored 0..1 / 8 (`hangout_uv_dump` test, `--ignored` → `research/gfx/hangout_uv.txt`
  + raw level-0 textures in `research/gfx/hangout_tex/`). We sampled 1/8 of each texture, so surfaces were one smeared
  texel area (grey pipes, no wall/ceiling detail). Original: GDL_SetPartVertexFlags 0x000A4CF0 sets part flag 8 when
  register 9 has format 33; GDL_RelocateModel 0x0009EF30 then overrides the shader with "DefaultPacked"
  (SaveFileData 0x1065A0 slot 7 = vertex program 2 only, own stages kept), `Shaders\default_packed.vxu`:
  `m4x4 oPos, v0, c0 / mul oT0, v9, c4 / mov oD0, v3`, c4 = fTexCoordScaleFactor (8, 8, 8, 1) (0x0015D250).
  Port: `model_builder::{packed_uv, TEX_COORD_SCALE_FACTOR}` scale uv0 by 8 per draw in `Builder` (oD0 = v3 unlit, as
  before). Stage ops checked: SetStageOps 0x2A7120 / MapTextureOp 0x2A3230 (enum 3 = D3DTOP_MODULATE 4, 4 =
  MODULATE2X), the only code-built op is MODULATE; no second stage / lightmap in hangout.xmf (1 stage per shader).
  Scripts `tools/ida/gfx_stageops_callers.py`, `gfx_uv8.py`, `gfx_gamma_refs.py` (no SetGammaRamp caller; the
  GameManager "Brightness ( Multiply - free)" tweak +0x84 is core's). Census (`parse_every_model` writes
  `research/gfx/packed_uv_models.txt`): no model in any VCD / compartment has packed uv, only the loose foyer xmfs, so
  race scenes cannot change. Shots (fecapture career, profile select = the ref's Game Profile view):
  `research/screens/fe_uv8/{before,after}_profile.png` and `*_sbs.png` (ref left, ours right). Region means vs the ref
  (640x480, ref 4:3 crop): ceiling pipe 21.7 → 28.0 (ref 28.4), left wall 38 → 44.5 (35), back wall 12.9 → 15.2 (20.6);
  the overall level was already right (unlit prelit × texture), only the texture detail / colour was wrong. Our floor is
  brighter than the ref only under the foyer car's headlights (no car on the real boot screen).
  Golden (`research/golden/uv8` vs `ref`, `research/golden/uv8_compare.txt`): changed only the hangout views, as
  expected: menus_00/01 (main menu over the hangout, mean 4.6), career_00..03 / 11 (profile select, plate, hangout,
  mean 0.8..4.7), tv_hangout (4.2); tokyo 0..7 and tv_fe_carsel bit-exact, the rest of menus / career noise level
  (menus_07 max 12, 0.017 %). London / split / particles not re-run (no packed-uv model outside the foyers).

### Requests from core (GameDataLoading lighting loaders, 2026-11-07)
- Core ported the lighting loaders 1:1: `velocity_game::game_data_loading::gdl_lights::load_lights(scale, track_dir)`
  (GDL_LoadLights 0x000A0B40: lights.txt transposed rows, then `PtLights.bin` 0x0009DA10 else `.txt` 0x0009DE90) and
  `load_point_lights_txt(out, foyer, true)` (FE foyers, 0x00298CD0's call), returning `GdlLights { lights: VsLights,
  point_lights: Vec<PointLight>, flare_ids: Vec<(flare, reflection)> (GDL_LightNameIds 0x0009D860 → TrackFX 0x0014AEC0),
  from_bin }`. Test `gdl_lights::tests::every_track_lights`: identical to your `parse_lights_txt` / `parse_ptlights_txt`
  on all five tracks. velocity_game depends on visos_render, so you cannot call it: if you want the 1:1 reader on
  the "Load Lighting" step, add an input (e.g. a `LoadedLights` resource / message in visos_render with those fields)
  and core will fill it from `load_lights` there; then `track_render` / `track_fx` can drop their own parsers.

### Requests from frontend (front-end door parts: per-part transforms, 2026-11-07)
- The car showroom opens / closes the shown car's doors (option 1, and automatically when the camera reaches the car);
  the frontend now keeps the door state 1:1 (`fe_3d_vehicles::FoyerVehicle.door_t[8]`, eased by FE3DVehicles 0x0025BF80's
  state part, window 0x0025D9A0) but `CarRenderParts` exposes no door entities, so nothing moves on screen. What
  0x0025BF80 needs (dump `research/fe_doors.c`, called per part from SetRotation 0x0025DB50 into CarRender data
  +384 + 64 k for k = 0..5, and the bonnet / top boot matrices into slot +96 / +160 → CarRender +1208 +100 / +188):
  1. each front-end door model as its **own child entity** of the car root (the `<name>_<d>_door` stems you already
     load as "extras", CarRender data +1184 set by FE3DVehicles 0x0025D600), in a fixed order: f_l_door, f_r_door,
     r_l_door, r_r_door, m_boot, b_boot, bonnet, t_boot (absent ones `None`), e.g.
     `CarRenderParts.doors: [Option<Entity>; 8]`, drawn with the car-space transform the frontend writes;
  2. the **locator** of each part in the car model (GDL `GameDataLoading 0x000A3CB0(model, "f_l_door", 0)` etc.,
     0x000A3C10 = model loaded): its position (the hinge) and rotation (the open pose), e.g.
     `CarRenderParts.door_locators: [Option<(Vec3, Quat)>; 8]` in car space.
  The frontend then sets each door entity's `Transform` = T(hinge) · slerp(identity, locator rotation⁻¹, t) · T(−hinge)
  (0x0025BF80: `CompartmentManager 0x0008DD10(0,0,0,1)`, conjugate w, `TimelineSequencer 0x002B7530` slerp) every frame.
  Nothing else is needed from graphics (the turntable now turns the car root's `Transform` directly, see
  `foyer::foyer_car_poses`; please don't overwrite the root transform of `frontend` cars).
- **Wheels (same request, added 2026-11-07)**: the front end steers the shown car's front wheels
  (`FoyerVehicle.steer`, FE3DVehicles +1256 via 0x0025E040: 28° in the hangout / garage list, 35° in a garage
  screen) and lowers the wheels under the wheel-change bounce (+1264). Its wheel / brake-disc matrices 0x0025B4D0 /
  0x0025BA40 (dump `research/fe_wheels_dc.c`; wheel k: 0 f_l, 1 r_l, 2 f_r, 3 r_r) are: position = locator
  `f_l_wheel` (k 0/2) / `r_l_wheel` (k 1/3) of the car model (`0x000A3CB0(model, name, 1)`, `VsRender 0x0005C820(loc,
  3)` translation, **z negated**; locator missing -> car +64 (front) / +80 (rear) x,z with y 0, halved), right wheels
  (k 2/3) mirrored by a scale(-1,1,1) matrix; rotation = RotY(-steer) for k 0, RotY(+steer) for k 2, none for the
  rears; then the car's RotY(+1248), translation car pos + (0, -bounce, 0). What the frontend needs: the frontend
  car's wheel and brake-disc entities as `CarRenderParts.wheels: [Option<Entity>; 4]` / `discs: [Option<Entity>; 4]`
  in that k order with a car-space pivot (the locator position), so the frontend can set each one's local
  `Transform` = T(pivot) · RotY(∓steer) · T(−pivot) and a y offset of −bounce. Rolling spin is not used in the FE.
- **ANSWERED (stage 11, 2026-11-07)** in `car_render::CarRenderParts` (frontend cars only for doors; `spec.frontend` + `_base_high` in VehicleS.vcd):
  - `doors: [Option<Entity>; 8]` in `FE_DOOR_NAMES` order (f_l_door, f_r_door, r_l_door, r_r_door, m_boot, b_boot, bonnet, t_boot); each is a child of the root holding the model `<car>_<name>` of the pack (vertices in car space, so `Transform` = T(h)·R·T(-h) works); absent models `None`. The old "extras" no longer merge into the LOD group (they were drawn static before).
  - `door_locators: [Option<(Vec3, Quat)>; 8]` = chassis locator of that name, position z negated (like `l_head`), rotation = 3x3 rows conjugated by the z flip (S·M·S).
  - `fe_wheels: [Entity; 4]` in FE3DVehicles order (0 f_l, 1 r_l, 2 f_r, 3 r_r; `wheels` stays f_l, f_r, r_l, r_r), `wheel_pivots` (locator `f_l_wheel`/`r_l_wheel`, z negated, right wheels x mirrored; no locator = estimated placement, not the 0x25BA40 car +64/+80 fallback) and `wheel_base_rot` (π about Y for left, identity right: the wheel model faces +x, we rotate instead of the original's scale(-1,1,1)). Frontend cars start with `Transform` = T(pivot)·base_rot; set `T(pivot)·RotY(∓steer)·base_rot` (translation y − bounce) per frame. Left sign: k 0 → RotY(-steer), k 2 → RotY(+steer) as in the request.
  - `discs: [Option<Entity>; 4]` always `None`: the disc is the brake part model (BRAKE_DISC strings, car +2476 "disk" locator) which no spawn path builds yet.
  - Unverified visually (no door model in some cars; frontend must pose them).
- Done on the frontend side: `foyer.rs` falls back to `VsLights::vs_render_defaults` when a foyer has no lights.txt
  (your 2026-11-07 request).


### Stage 10 (2026-11-07): StatelessParticleMgr cull, light-prim audit
- **0x2D3680 / 0x2D3A80 / 0x2D3BB0 ported** (`CType::bounding_sphere`, `axis_extent`, test `bounding_sphere_covers_every_type`): emitter-space sphere of a type = per axis the extremes of v·t + a·t²/2 over the life for both velocity bounds (+ the turning point, 0x2D3BB0 solving t = -v/(a/2) when a ≥ 1e-6), clamped to include the origin, ± half the spawn box, centre = (min+max)/2, radius = max(1, |max-centre|) + copy_size·max(size_max, size_end)·emitter scale. Used by 0x2D4080 (cull = 1 from every 0x2D3460 caller): centre through the emitter placement (0x2D5830), `Renderer::SphereVisible` 0x2AAC40 (6 planes, outside when n·c+d < -r) = `Frustum::intersects_sphere` on the view's camera (`draw_list(frustum, ..)`; no frustum -> no cull). Unwritten out value of 0x2D3BB0 (true without writing) taken as 0.
- **0x1181C0** (per-car light-prim draw): only binds the car's light set (+1028 dirs, 3 point lights via 0x11B7B0, darkened copy for +2324 damage bits) around GFXHelper 0x12B0E0 of the brake/reverse/head prim lists. The BrakeLight / revlight materials (`model_builder::brake_material`, `car_lights_setup`) are unlit stage-mode shaders, so the light set has no visible effect; the draws themselves are `update_car_lights` (0x112A50 light pass). Not cited. **0x10F9C0** `SetPart`: parts are built by `spawn_car`; not cited.
- **0x13AB80** = `SkinnedObjectRender::loadResources` (IDA name skidsmoke__13AB80 is a mis-attribution): loads `\Scenery\SknObj_M/T/A.VCD`, animations (max 31) and per-instance ComputeObjectLights; **0x13A810** validates the `F3SkinnedAnimatedObject` instance bundle. The install has no SknObj_*.VCD, so both are no-ops in practice; not ported.

### Stage 11 (2026-11-07): FE door/wheel parts, CarRender prim lists
- Door / wheel API for the frontend: see "Requests from frontend (front-end door parts)" ANSWERED.
- **0x1174F0** (`CarPrimLists::build`, carbon): first `_carbon` prim of the chassis, then of each fitted part (max 3; a 4th asserts at CarRender.cpp 5109/5126). **0x117BB0**: the six `off_6867FC` prims (glass_f/b/l/r_alpha_0, logofw/logorw_alpha_0) looked up chassis-then-parts; one of the first four missing clears flag +2328 & 0x8000 and slot 0 (`glass_complete`); door glass from the door models (+1184..): `glass_l_alpha_0` else `glass_r_alpha_0`; all found get prim type 7 (alpha pass) which our materials already do by blend, so the lists are lookups only. Carbon prims otherwise reach shader type 8 via `car_shader_type`.
- Not ported: **0x11B1E0** CarRender::CarData ctor (zero/identity init of the +0..2352 record, +2236/+2240 colours = dword_77FD9C, +2347 = 0x96 alpha; Rust defaults), **0x10FD20** SetPart (asserts CarRender.cpp 1469-1475 index < 22 cars, < 14 parts, < 6 cars with parts; copies model/flags into part slot 44·k of +1208 and calls 0x1174F0; plate on car 0 slot 1) = `spawn_car`, **0x12B2D0** GFXHelper draw of a prim with a sort key = n·(camera pos) − w of its bounding sphere (view-space depth sort + Renderer 0x2A95F0 queue; Bevy sorts transparents itself), **0x12C450** caches the view-projection matrix (GlobalMatInputResolver 1/0/6) into `unk_71EB78` for a draw context (platform).

### Stage 12 (2026-11-07): FE brake discs, coverage triage
- **Brake discs built** (`car_render::spawn_brake_discs`, `CarSpec::brakes`, `CarRenderParts::{discs, disc_pivots}`). FE3DVehicles record: +2476 / +2480 = front / rear wheel models, **+2484 / +2488 = brake disc / pad models**, loaded by `FEAlloyEditor::LoadBrakeModel` 0x25A7F0 (IDB; wheel twin 0x25A5A0) from the setup's brake parts (`BrakePart +0x10` name, file `<name>_high.xmf` in `\vehicles\wheels\wheels_m.vcd`: `disk001_1`.., `caliper001_1_high`..). Drawn per wheel k by 0x25C890 (`DrawWheelsAndBrakes`) → CarRender 0x110140 with the 0x25BA40 matrix: car locator `f_l_wheel`/`r_l_wheel` + the front wheel model's `disk` locator, z negated, right side x negated and scale -1 in x. `CarSpec::with_setup_underbody` fills `brakes` from `setup.brake_parts` (so the frontend gets discs for free); the disc and pad of a slot share one entity (`discs[k]`, FE slot order f_l, r_l, f_r, r_r) at `T(disc_pivots[k])`. The frontend steers them like the wheels (RotY about the pivot). Not drawn in races (the in-game wheel models carry the brake prims). `0x411B0` (idb VsTrafficManager, really the alloy editor) is the same loader for the default disc/pad.
- Viewer: `VELOCITY_VIEW_BRAKES=disk001_1,caliper001_1 trackview --track ExecTest --fecam --car LA/350z --frontend` → `research/screens/gfx_brake_on.png` vs `gfx_brake_off.png` (red calipers / white discs behind the spokes; the FE camera is fixed, so only seen small).
- Coverage triage (the remaining "visible-rendering" list is mostly not visible): 0x163E80 ZoneRender camera update (fixed-camera zone, core), 0x599E0 debug sphere draw, 0x14DB30 debug drawlist, 0x10D200 car unload, 0x5B470 / 0x10B9C0 `std::sort` internals, 0x1154E0 OSD-map damage draw, 0x136840 ray/quad test helper, 0x1449A0 loads `FGlit.bin` (unused here), 0x118BD0 / 0x118A40 setup paint-CRC checks (flags +2328 0x8000 / 0x1000), 0x115A70 / 0x116BC0 / 0x1173B0 shader name searches, 0x11AAB0 / 0x11A8F0 per-LOD shader flag masks (bits 0x103 of material +8: 1 / 256 / 512), 0x11E850 cat's-eye sector sphere pre-test (port uses the sector sphere: same result).
- **Open (needs physics + camera input, not ported): 0x11ABB0 engine shake.** Per rendered frame (GameManager 0xBB1C0 draws `flt_6F6C58/5C/60` = three `VsRandRange(-10, 10)`), for the car the player camera follows (car +0x4B8 camera +0x1C4 == car +0x40, +0x6FC vehicle, GameBridge 0x97E90 false): render matrix (+64) = matrix (+0) with its translation += (r0·0.0005, r1·0.001, r2·0.0005) · min(rpm / 8000, 1) when +2304 is 0 (`dword_68684C` = 5e-4, `dword_686848` = 1e-3). At most ±5 mm / ±1 cm; the car's pose is set by core, so core would apply it (`CarLightInput`-style input of rpm).

### Tyre texture (CarSpec.tyre, 2026-11-07)
CarRender 0x00118910 loops the 4 wheels calling 0x00118850 = FindShader(wheel, "_tyre") + texture rebind. Port: `apply_tyre_texture` swaps the TexSet entry of each "_tyre" wheel shader's stage 0 texture for the TyresT.vcd stem (`VS_soft`, `VS_medium`, `VS_hard`, `VS_super_soft`, `VS_super_hard`) before the wheels are built. Per-car TexSet, so other cars are unaffected. The decal request from frontend had not appeared.

## Performance (stage 1 2026-11-07: safety net + instrumentation; stage 2 2026-11-08: measurements, no optimisation yet)

**Stage 3 (2026-11-08): dynamic envmap without a second scene view.**
- Original (IDA, GameManager 0x0B9010 call order `research/gfx/perf3_order.txt`, `tools/ida/perf3_order.py`): one 2D sphere
  map per car pass (CarRender 0x119900 → 0x1598A0 gated by +58, cleared by 0x159C10 at the end of the pass), 128²
  (256² in camera mode 3 / hi-res), **source = the back buffer itself** (`dword_7A5780`, 0x159D00) as it stands after
  sky, compartments, TrackFX, cat's eyes, light streaks, trail blazer, start-line FX — before cars, characters,
  sparks, barriers, particles. No cube faces, no scene re-render. Ours rendered 5 extra views per frame (4 sky
  mirrors + a half-size world copy without cars) — more than the original.
- Port: in-game car parts (`CAR_LAYER` / view car layers) get `StageState.car_pass` (`mark_car_pass`), so both
  materials report `reads_view_transmission_texture` and draw in bevy's Transmissive3d phase (after opaque + mask,
  before transparent; depth-tested, so the frame is unchanged: envmap-off A/B shots 02-06 bit-exact).
  `capture_envmap_source` (Core3d, after `main_opaque_pass_3d`, before `main_transmissive_pass_3d`, views with
  `EnvmapCaptureView`) copies the world camera's main texture into the full-size source image. `ScreenSpaceTransmission.steps`
  = 0 on every Camera3d (no material samples it). Deviations: the early envmap passes run before the world camera, so
  they warp the **previous** frame's copy (one frame late; static golden shots unaffected); world transparents (flares,
  streaks) are no longer in the source (the original has TrackFX/streaks, but not particles); full back-buffer
  resolution as the original (was half).
- Gate: race images are not comparable to `ref` across builds (see core.md request: per-frame `gm.rand` draws while
  paused, and order-ambiguous smoke shot timing move shots 07+ by one GameTick even with envmap off on both builds);
  harness gained `VELOCITY_GOLDEN_SETTLE=<n>` (min paused frames, logs "settled N frames"). A/B with the baseline
  built from HEAD in a worktree, both `SETTLE=60` (`research/golden/p3base` vs `p3new`): 51/64 pass; fails = car paint
  only at matched moments (tv_mirror 0.28 %, career_06 0.27 %, menus_06 0.21 %, london 02/03/06; crops
  `research/perf/p3/*.png`, envmap insets `envmap_inset_base_new.png`: same sky/buildings, sharper) and the
  one-tick-shifted london 01/07-12 (same shift with envmap off: `p3base_off` vs `p3new_off`).
- fps (same session, PC slower than stage 2; `research/perf/p3_base`, `p3_new`, table in `research/perf/baseline.md`):
  london_wet_cams run 12.6 → 16.5 (+31 %), tokyo 31.0 → 31.5, split/menus/career unaffected (envmap off there).

**Stage 2 results (full tables: `research/perf/baseline.md`).**
- Noise floor (two captures, golden clock live): 64 images, 58 bit-identical; the rest max 33 / mean <= 0.0006 /
  over8 <= 0.003 % (london_wet_cams 06-09, career 13 max 4). Default tolerances (mean 0.5, 0.2 %) stand. Only harness bug:
  stale images from an earlier run made compare report "missing" (`golden.py capture` now deletes the scene's old
  images first). `research/golden/ref/` = copy of run a (64 PNGs).
- Baseline run fps (uncapped, RTX 5070): menus 39, london_wet_cams 17 (59.9 ms mean, p99 77), tokyo 45, split 22,
  FE/garage 45. GPU pass spans are all < 0.06 ms: CPU / submission bound, not GPU bound.
- Suspects on london_wet_cams (17.0 fps): envmap off +49 % (25.4), mirror off +16 %, road off +6 %, stencil ~0.
- CPU trace (race): the Core3d graph runs ~25 times per frame (22 ms of 46 ms render-app time); per view the
  cluster pass (`cluster_on_gpu` 0.17 ms, `prepare_clusters_for_gpu_clustering` 3.7 ms/frame total) and bin
  unpacking / preprocess run again. Then main_loop 6.6, submit_pending_command_buffers 5.2, run_frontend 4.6,
  update_cameras 4.6 (per tick), physics_tick 5.2 (per tick), visibility culling 2.9. FE/menus: run_frontend 4.3,
  audio apply_commands 2.2, allocate_and_free_meshes 1.9, prepare_clusters 1.9 (all ms/frame, trace-inflated ~2x).
- Top 3 optimisations (none applied): (1) envmap: cut the number of cameras/views it renders (cube faces to at most 1-2 per
  frame in round-robin, or lower resolution, no clustering/bin passes on envmap views: disable `ClusterConfig` on
  envmap, mirror and road cameras): expected +25-45 % in races; risk medium (the original updates the envmap faces
  over several frames anyway, but reflections change visibly: gate by golden compare with tolerance for the envmap
  car images). (2) Remove clustering from cameras that need no lights (`ClusterConfig::None` on sky, envmap, road,
  stencil, mirror): ~3-4 ms/frame, +5-8 %; risk low (their materials are unlit/custom: verify with golden). (3) Mirror
  camera: render it at reduced rate/resolution or only when the in-car/bumper view shows it (+16 % measured when off, so up to
  ~+10 % obtainable); risk low-medium (mirror image content). Further: frontend `run_frontend`/`apply_commands`
  (audio) per-frame cost in menus (~4+2 ms) is the menu bottleneck.
- Caveats: perf runs reach fewer shots than capture runs (script clock stretches); chrome traces are 5-20 GB for 10-60 s
  (delete after summarising); hidden desktop present path was used for all numbers.

**Harness (`visos_render::golden`, in RenderPlugin and trackview; inert unless `VELOCITY_GOLDEN` is set).**
- `VELOCITY_GOLDEN=<dir>`: `TimeUpdateStrategy::ManualDuration(1/60 s)` (`VELOCITY_GOLDEN_HZ`), so the smoke script,
  front end, particles and anything on Bevy `Time` advance per frame, not per wall second. Every smoke-test
  screenshot (an `Added<Screenshot>` it did not spawn; two in one frame = one capture) queues a capture: `Time<Virtual>`
  pauses, it waits ≥ 6 frames and until `PipelineCache::waiting_pipelines()` (render world, `RenderSystems::Cleanup`)
  is 0 for 3 frames (900-frame cap, logs a warning), saves `<dir>/<VELOCITY_GOLDEN_PREFIX>_<nn>.png`, resumes.
  trackview in golden mode pushes a `GoldenRequest { image, exit: true }` at frame 150 instead of its own shot
  (and its headless runner has no 1/60 wait).
- `golden::clock_ns()`: the frame-locked virtual clock for `GameClock::now`. **Core request (core.md OPEN)**: until
  `GameClock::now` returns it first, race images follow the wall clock (24 Hz game ticks per real time) and differ
  run to run; menus and trackview scenes are deterministic already. Expect race-scene noise until it lands.
- `VELOCITY_GOLDEN_PERF=<n>`: after each capture, n frames timed frozen (render + Update only, no GameTick) then n
  running (deterministic workload: 1/60 s of game per frame, i.e. 0.4 GameTicks per frame, more logic per frame than a
  real uncapped run) → `<dir>/perf.csv` (fps, mean / p50 / p99 ms, wall clock `Instant` in `Last`); adds Bevy's
  `RenderDiagnosticsPlugin` (wgpu timestamp queries: adapter features are requested in full, Vulkan/DX12) →
  `<dir>/gpu.csv` (`render/<pass>/elapsed_gpu|elapsed_cpu` averages; only Bevy's own passes carry spans, our FSE /
  custom nodes do not).
- `VELOCITY_PERF_OFF=envmap,road,mirror,stencil` (any subset): dynamic envmap source + sky-mirror cameras
  (`update_envmap_source` want=false), RainSurface `RoadCamera`s, `MirrorView` camera, `CarStencilCamera`. A/B timing
  only (images change).
- CPU per system: `--features visos_render/trace_chrome` (= `bevy/trace_chrome`), built in `target/perf-trace`; the run
  writes `trace-<ts>.json` in the cwd (big: keep runs short); `tools/perf_trace.py <json> [N] [--from SEC]` sums
  inclusive span time per name{args.name} (systems, schedules, render nodes) and per frame.
- Present mode: `--fps 0` = `PresentMode::AutoNoVsync` (immediate/mailbox), no limiter (`crates/velocity/src/frame_cap.rs`,
  core); without `--fps` the window keeps default vsync (Fifo) = capped. `golden.py perf` passes `--fps 0`. trackview
  scenes render to an offscreen image, no present at all.

**Tool `tools/golden.py`** (pure-stdlib PNG reader/writer; scenes in its `SCENES` table, `golden.py list`):
game scenes via `target/perf/release/velocity.exe --smoke-test`: `menus` (main menu, profile, quick race, flag girl,
LADown start + HUD, driving, braking, sparks, pause, quit confirm; 13 images), `london_wet_cams` (wet London, 4 car
views incl. bumper / in-car mirror, look back; 14), `tokyo` (8), `split` (two-player LA; 10), `career` (profile, plate,
hangout garage); trackview scenes via `target/perf/release/trackview.exe`: `tv_fx` (smoke / skids / sparks / start
lights), `tv_lights` (LACoast night: lamp flares, road reflections, beams, neon), `tv_mirror`, `tv_split_zone` (zone
ring / car stencil, `VELOCITY_ZONE_FORCE=40`), `tv_fe_carsel` (front-end car + floor reflection), `tv_hangout`.
compare: per image max / mean abs RGB error and % pixels off by > 8; fail if mean > 0.5 or % > 0.2 (defaults, set
from the noise floor once measured), or a missing / extra image; `--diffs` writes ×8 diff PNGs; exit 1 on failure.

**Commands to run (PC idle, sweep finished = 164 lines in `research/core_sweep_run.log`), one game at a time:**
1. Build: `CARGO_TARGET_DIR=target/perf .venv-ida/Scripts/python tools/lowprio.py cargo build --release -p velocity
   -p visos_render --bin velocity --bin trackview` (and the trace build: `CARGO_TARGET_DIR=target/perf-trace ... cargo
   build --release -p velocity --features visos_render/trace_chrome`).
2. Noise floor: `.venv-ida/Scripts/python tools/golden.py capture research/golden/a`, same into `research/golden/b`,
   `tools/golden.py compare research/golden/a research/golden/b --diffs`; record per-scene max/mean here, set the
   tolerances; then copy run a to `research/golden/ref/`.
3. Baseline fps: `tools/golden.py perf research/perf/base --frames 300` → `perf.csv`, `gpu.csv`; write
   `research/perf/baseline.md` (fps per scene, top 10 costs).
4. Suspects: `tools/golden.py perf research/perf/off_<x> --frames 300 --off <x> london_wet_cams split menus` for x in
   envmap, road, mirror, stencil and `envmap,road,mirror,stencil`; deltas vs base.
5. CPU breakdown: `hidden_run.py --timeout 300 --env SMOKE_DIRECT=1 --env SMOKE_DRIVE=8 --env VELOCITY_GOLDEN=research/perf/trace
   -- target/perf-trace/release/velocity.exe --smoke-test --fps 0 --game <dir>` (cwd = repo; trace json lands there,
   move it to research/perf/), then `tools/perf_trace.py research/perf/trace-*.json 40 --from 20`.

**Gate for every later optimisation stage (user requirement: no regressions):** one optimisation per commit; before
and after each: (a) `golden.py capture research/golden/new` + `golden.py compare research/golden/ref research/golden/new`
must pass within the recorded noise floor (any failing image = revert or justify as a pre-existing bug fix, then
re-baseline the ref in the same commit with the reason); (b) `cargo test -p velocity_game camera::vs_vehicle_follow_camera_golden`
(golden chase camera) green; (c) `golden.py perf` before/after on the same idle PC, delta recorded here. Never
re-capture refs to make a compare pass.

### Performance stage 4 (2026-11-08): determinism + clustering off
- Order: `race_effect_messages` runs `.before(particle_messages)` (race_effects.rs) and `car_damage_fx` too (the other unordered
  `ResMut<Particles>` writer in Update; `danger_particles_system` was already between `particle_messages` and `update_particles`;
  fury_fx is PostUpdate). Refs re-captured from this build: two full captures were bit-identical (64 images, max 0, noise floor 0)
  -> `research/golden/ref`. After the `car_damage_fx` ordering, shot london_wet_cams_06 (sparks) changed to a second, now stable
  image (max 143 / mean 0.44 / 1.4 % over 8, sparks only); ref's copy of that one image was replaced by the stable one (pre-existing
  ordering bug, not caused by the optimisation; the same image came out with the optimisation toggled off).
- Optimisation A (`lib.rs::no_cluster_on_new_cameras`, Update, `Added<Camera3d>` -> `ClusterConfig::Single`): bevy light clustering
  was running for ~25 views/frame although no bevy lights exist (the original's light sets are shader constants). `ClusterConfig::None`
  does NOT work in bevy 0.19 with GPU clustering: validation error "clustering dummy texture: Dimension X is zero" -> swapchain guard
  quits the app. `Single` = one cluster, image unchanged (63/64 images identical, the 64th is the ordering above). Debug toggle:
  `VELOCITY_PERF_OFF=clusteropt`. fps: london 16.7 -> 22.4 (+34 %), tokyo +13 %, split +10 %, career +8 %, menus +26 % (noisy);
  `research/perf/baseline.md` "Stage 4".
- Not done: optimisation B (mirror camera cost). Candidates: skip the mirror's FSE/post passes it does not draw, one fewer scene slot
  (5 slots/frame in `mirror_scene`); needs an A/B with `VELOCITY_PERF_OFF=mirror` shares (mirror +16 % in stage 2).

### Performance stage 5 (2026-11-08): fewer graph runs, mesh churn
- Census tool: `VELOCITY_GOLDEN_CAMS=1` (golden mode) appends every active camera per capture to `<dir>/cameras.csv` (order,
  target, tags fse/road/stencil/sky/viewsky/track/mirror/post/second/fe, layers) and logs the next frame's FSE pass list ("fse
  pass k: ..." in the scene log). London wet shot 3 = 35 graph runs: 8 early FSE (envmap: clear, 2 ring strips, blur pair,
  subtract, fresnel, spots), 4 sky, world, 18 FSE (bloom = 10 ping-pong passes OffScreen 0<->2, each reads the previous), 2 road,
  post, present, HUD 3d + 2d; mirror shots add 4 view-sky + mirror world. FSE quad cameras are the bulk (27); next step would be
  one render node drawing consecutive FSE passes (the original's quads on one device), or merging same-target passes that do
  not read their target (early envmap 21-23 and 26-28: 8 -> 4 cameras; needs Transparent3d + per-pass depth_bias ordering).
- Kept A, `VELOCITY_PERF_OFF=skyopt` = old path: one sky camera with layers 1-4 instead of 4 (`sky_render::sky_one_camera`);
  background quad is the only opaque sky item, blended passes ordered by `StageState.depth_bias` = (pass-4)*10000 (Transparent3d
  sorts by view z + bias, all sky shaders z off). View-sky copies keep the blended layers (`layers.without(1).with(19)`).
  -3 runs per view (-6 with mirror / split). Images identical.
- Kept B, `postopt`: `PostViewCamera` active only in frames whose flush put a batch on `POST_VIEW_LAYER`
  (`effect_draw::PostViewUsed`, `setup_post_camera` after `EffectSet::Flush`; 0x00127080 draws nothing without flash records).
  Found: POST_VIEW_LAYER was 23 = frontend `HUD_3D_LAYER_P2`, so the post camera drew player 1's 3D hud models into player 0's
  view (small icon at the top view's right edge, split_01-04/07-09, max ~200, < 0.03 % pixels): moved to layer 11 (free); refs
  for those split images carry the old artifact (re-baseline when convenient). Also MTV_LAYER 17 == VIEW_CAR_LAYERS[0] (unchecked).
- Kept C, `meshopt`: `crate::set_mesh` skips unchanged meshes (FSE quads, MTV quads, FE paste strip) and marks the entity's
  Mesh3d changed when written. NOT applied to `effect_draw` batch meshes: skipping equal ones changed london 00 (a light-beam quad
  drew with hard edges; always-write + re-extract matched ref, cause not found). Skipping unchanged `FseMaterial`s was reverted
  (split 02/03/05 changed, over8 0.4 %; likely a re-created image behind an unchanged handle).
- Gate: all-on full capture vs ref: pass except (a) split icon fix above, (b) pause-menu "Rumble ON/OFF" text differing run to
  run (menus_09/10, tokyo_05/06, split_07/08; game state, not rendering: core/frontend determinism), (c) one capture run closed
  its window early ("No windows are open", no error; rerun). london_wet_cams_06 (sparks) flips between two states depending on
  the build's system graph (stage 4's ordering is not complete; another `Particles` message writer order). Camera golden test
  and visos_render tests green.

### Performance stage 6 (2026-11-08): FSE runs on one camera, MTV layer, pinned profile
- Harness: `golden.py` runs every game scene with `APPDATA=<outdir>/_profile` = a fresh copy of
  `research/golden/profile_seed/Velocity` (made once from the user's `%APPDATA%\Velocity`, `Rumble=1` forced). The smoke runs
  write options back, which flipped the pause menu's "Rumble ON/OFF" between runs; now menus_09/10, tokyo_05/06, split_07/08
  are stable. Delete the seed to re-seed.
- Refs re-baselined from a HEAD build with the pinned profile (`research/golden/s6head`, all 64 within noise of ref otherwise):
  split_* (stage 5's POST_VIEW_LAYER icon fix) and menus_00/01, career_00-03/11, tv_hangout_00 (hangout packed-uv x8 fix of
  3e5d6d1, same numbers as `research/golden/uv8_compare.txt`; refs had not been updated then).
- Kept A (`VELOCITY_PERF_OFF=fseopt` = one camera per pass): `fse_runs` gives each queued pass a lead slot; a pass joins the
  previous pass's camera when both write the same target, neither is a scene / road slot (those slots' orders belong to the car
  stencil / road cameras) and it reads nothing of the run's target (source, alpha texture, or the back buffer as the
  NOT_CAR / ROAD stencil source). The lead camera's RenderLayers = all the run's quad layers, joined cameras inactive. All passes
  of a run go to Transparent3d (`FseMaterial.in_run` -> AlphaMode::Blend; the pipeline state is still the pass's own via
  `specialize`) sorted by `FseMaterial.bias` = 1000 x position in the run (Material::depth_bias; Opaque3d has no queue order).
  London wet shot 3: 35 -> 28 graph runs (early envmap 21-23 and 25-28 merge; bloom ping-pong and back-buffer passes reading the
  back buffer cannot). Golden: 63/64 within noise (most bit-exact, a few max 1-20 / mean <= 0.01); london_wet_cams_06 (sparks)
  flips with the build as in stage 5 and is identical with `fseopt` off on the same binary (pre-existing, not this change).
  fps (idle PC, same binary, `--off fseopt` vs on, 300 frames; `research/perf/p6_base`, `p6_new`): london 22.5 -> 24.7 (+10 %),
  split 25.1 -> 25.9 (+3 %), tokyo 49.2 -> 54.0 (+10 %).
- Kept B: MTV_LAYER 17 -> 10. 17 was `VIEW_CAR_LAYERS[0]`: (a) the MTV overlay camera (FE only, active while glitch quads
  draw) would also draw race car parts on 17, (b) `mark_car_pass` (envmap on) marks anything on 17 as a car part, so the MTV
  quads could be moved to Transmissive3d. Not visible in any golden shot (FE has no race cars, envmap off there); layers 5-10
  are now: 10 MTV, 5-9 free.
- Remaining big costs: bloom = 10 ping-pong cameras (each reads the previous: needs a custom render node that runs several
  passes with target switches in one graph run), early envmap 3 cameras, 4 back-buffer composites.

### Performance stage 7 (2026-11-08): effect order determinism, GPU clustering off, program rebind
- Sparks flip (london_wet_cams_06) root cause: every `EffectDraw` producer sat unordered in `EffectSet::Build`; batches draw in
  submission order (merged per shader + view, mesh slots reused), so the scheduler's order changed blending where sparks overlap
  particles. Now `effect_draw::EffectOrder` (chained sets in Build, original GameManager 0x000B9010 order): Shadows (FE),
  RaceFx (skid/sparks/slipstream/start lights), CatsEye, Trail, Barriers, TrackFx (fury, track_fx, flares), Puddles, Particles,
  ElRain, ElDamage, ElMist, ElSpray (the 4 `ElParticleSystemManager` feeders were unordered too), ElParticle. Also
  `car_damage_fx.after(race_effects::RaceEffectsMessages)` (both wrote `Particles` in Update). Verified with bevy's ambiguity
  report: `VELOCITY_AMBIGUITY=1` (golden.rs; names need the trace_chrome build, which enables bevy/debug) lists no remaining
  Update/PostUpdate pair on Particles / EffectDraw / ElParticleSystemManager / RaceEffects. Left for core (core.md request):
  `effect_clock` / `mode_messages` vs `race_effect_messages`. Ref london_wet_cams_06 replaced from `research/golden/s7a`; two later
  builds with different system graphs (s7b, s7d) reproduce it.
- Trace (`research/perf/baseline.md` "Stage 7 trace"): render thread bound (37.9 vs 20.5 ms/frame traced), 23.1 Core3d runs per
  frame, `cluster_on_gpu` 4.2 + `prepare_clusters_for_gpu_clustering` 2.7 ms the largest single item. `tools/perf_trace.py`
  gained `--to SEC` and counts frames by bevy 0.19's "update" span.
- Kept A (`VELOCITY_PERF_OFF=gpucluster` = old): `lib.rs::cpu_clustering` (Startup) sets `GlobalClusterSettings.gpu_clustering = None`
  (inserted by `PbrPlugin::finish`, before any pipeline): bevy falls back to CPU clustering, free with one cluster and no lights.
  fps same binary (`research/perf/p7_base` vs `p7_new`): london 24.3 -> 29.5 (+22 %), tokyo 54.0 -> 73.2 (+36 %), split 28.6 -> 34.5
  (+21 %). Golden: s7b 63/64 (london 07 lap timer 8.54 vs ref 8.58 = one GameTick shot shift, game state), rerun s7c passes 07 but
  01 differs slightly; `gpucluster` off on the same binary passes: the faster build exposes the smoke script's timing nondeterminism
  (not rendering; see core.md request). Camera golden test green.
- Kept B (`bindopt`): `ProgramMaterial::rebind` writes car (`bind_car_programs`) and beam (`bind_beam_programs`) program materials
  only when params / textures changed (each `get_mut` re-prepared a 3.4 KB uniform + bind group: `prepare_erased_assets<ProgramMaterial>`
  2.1 ms/frame traced). Golden 64/64 (`research/golden/s7d`). fps (`p7b_base` vs `p7b_new`): london 33.3 -> 35.2 (+6 %), tokyo +1 %,
  split +3 % (session faster than p7's; compare within a pair only).
- Next: bloom chain in one render node (10 graph runs), per-view preprocess nodes (unpack_bins / build_indirect ~3 ms over 23 views),
  frontend `run_osd_3d` + `run_hud` 6.7 ms on the main thread (hidden while render-bound).

### Performance stage 8 (2026-11-08): FSE chain node (bloom ping-pong in one graph run)
- Refs re-captured for core's game-timer / fly-camera changes: two captures with this build and `--off fsechain`
  (`research/golden/s8r_a`, `s8r_b`) are 64/64 bit-identical -> `research/golden/ref` (old refs kept in `ref_pre_s8`; they differed
  in london 01/06 only). `golden.py capture` now takes `--off x,y` too.
- Kept (`fse_chain.rs`, `VELOCITY_PERF_OFF=fsechain` = old path): `execute_passes` marks passes for a chain (`chain_mask`): off-screen
  target, no z test, no stencil, not a scene / road slot, and no camera-drawn pass of the frame writes the same image. Consecutive
  chained passes go to one `FseChainCamera` (render graph `FseChainGraph`: no Core3d, `CameraOutputMode::Skip`, 1x1 dummy target) at
  the first pass's order; its `FseChainView` (extracted) holds per pass target, program, blend, `ProgramParams`, stage textures and the
  `quad_mesh` strip. Render world: `prepare_fse_chain` (PrepareBindGroups) builds one vertex buffer, a uniform + bind group per pass
  (FseMaterial's layout: uniform 0, t0-t3 + samplers 1-8, stencil 9; None = fallback image), pipelines specialised by (program, blend,
  target format) with `apply_stage_pipeline_state`; `run_fse_chain` draws each pass in its own render pass straight into the target
  image (load / store). Vertex shader = the FSE vertex programs with `CHAIN_VERTEX_WRAPPER` (no mesh / view bindings). Bloom and the
  other off-screen ping-pong runs become one graph run each.
- Determinism trap: spawning the chain cameras lazily (only with the chain on) shifted later entity ids, and london_wet_cams_06 (barrier
  debris) came out with different particle positions (max 132, 1.6 % > 8) although the frame timing was identical: a game-side
  entity-order dependence, not rendering. The pool now spawns `PRESPAWN` = 4 inactive chain cameras with the FSE slots in both
  modes; then on vs off (`s8x_on` / `s8x_off`) agree (06 max 30 mean 0.0006).
- Gate: `s8x_on` vs ref 64/64 (max 17, mean <= 0.0002); camera golden test and visos_render tests green.
- fps (idle PC after the sweep, same binary, `--off fsechain` vs on, 300 frames, mean run fps over the shots; `research/perf/p8_base`,
  `p8_new`): london 26.7 -> 42.5 (+59 %), tokyo 69.9 -> 106.2 (+52 %), split 34.9 -> 48.2 (+38 %); frozen fps london 32.6 -> 53.1.
  Trace (`research/perf/baseline.md` "Stage 8 trace"): Core3d runs 23.1 -> 9.0 per frame; render and main thread now nearly
  balanced. Next: `prepare_erased_assets<ProgramMaterial>` (car per-object constants in the material uniform), check_visibility
  over views x entities, frontend `run_hud` / `run_osd_3d`.

### Performance stage 10 (2026-11-08): effect batch meshes uploaded only on change
- Kept (`VELOCITY_PERF_OFF=effmesh` = old always-write): `effect_draw::flush_effects` writes a reused batch mesh through `set_mesh`
  (skips equal contents) and `set_changed` on its `Mesh3d` when written (the stage 5 "beam quad drew harder" was the missing
  re-extract after a slab move). Golden on vs off same binary: london 13/13, tokyo, split all ok (max 12, mean <= 0.0008).
  fps, interleaved (`research/perf/p10_*`): london run fps sum over 3 shots +3.5 % (+8/-4.5 %, +12/+1 %, +13/-7 %), i.e. noise-level
  gain; most batches change every frame (particles), so the saving is small.

### Performance stage 9 (2026-11-08): car program writes after visibility
- Pre-existing at this tree (not from stage 9): london_wet_cams_06 (barrier debris) differs from ref (max 132, mean 0.48, 1.58 % > 8)
  with every stage 9 switch off on the same binary (`research/golden/s9a_off`); s9a / s9b vs s9a_off: 06 max 12, rest within noise.
  Someone's game-side change since 031f792 (entity order / debris); re-baseline 06 once core confirms.
- Kept (`VELOCITY_PERF_OFF=carvis` = old immediate `rebind`): `bind_car_programs` (Update) only computes each car program's
  `ProgramBinding` (params + textures) into `PendingCarPrograms`; `flush_car_programs` (PostUpdate, after `CheckVisibility`) writes
  it only when some entity drawing that material is visible in a view this frame and it differs. Must use `get_mut_untracked` +
  a hand-written `AssetEvent::Modified`: the flush runs after `AssetEventSystems` (cannot be before: AssetEventSystems ->
  CalculateBounds -> CheckVisibility), so `get_mut` would reach extract a frame late (first try: identical images by chance, london
  -11 %). Side effect: no `AssetChanges` entry, so the meshes are not re-specialised (the key `ProgramKey` is not in the binding;
  `mark_car_pass` still uses `get_mut` for `state`). Golden s9a 63/64 (06 above), camera test + visos_render tests green.
  fps, same binary, two interleaved A/B samples (`research/perf/p9a_*`, `p9c_*`, no concurrent games): london 40.0 -> 40.6 and
  40.6 -> 44.5 (+1 / +10 %), split +2 / +4 %, tokyo 0 / -4 % (noise; few cars). Session noise is about +-5 %.
- Tried and reverted (`chainvis`): switching the 4 `FseChainCamera`s inactive around `CheckVisibility` (they draw no entities but
  each costs a full entity walk). Images identical (s9b), fps london +4.5 %, split -0.5 %, tokyo -4.5 % = noise: the main thread is not
  the bound, so check_visibility savings do not show.
- Not done: frontend `run_hud` / `run_osd_3d` (10.7 ms main thread, request updated in frontend.md); render thread remains the bound
  (Core3d runs, `allocate_and_free_meshes` for per-frame effect batch meshes, queue / specialise).
