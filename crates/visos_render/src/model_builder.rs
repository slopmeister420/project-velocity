//! mesh/material construction shared by tracks, skies and cars (not an original module): turns XMF2 models into
//! bevy meshes with `XboxStageMaterial`s — the work the xbox does in `CRenderer::DrawPrimitiveData` 0x2A9160 per draw.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::renderer::renderer_types::{vreg, Model, Shader};
use crate::renderer::shader_manager::{StageState, XboxStageMaterial, XboxStageParams};
use crate::car_palette_pc_xbox::{car_shader_type, shader_set_for, CarShaderSet};
use crate::game_shader_program_manager::{fp, vp};
use crate::renderer::shader_manager::d3dcmp;
use crate::renderer::shader_program_manager::{ProgramMaterial, ProgramTexture};
use crate::renderer::texture_manager::DecodedTexture;

/// `XboxStageParams.arg.y` stage modes: 0 MODULATE(diffuse, texture) * colour; 1 the BrakeLight set
/// (rgb = tex + colour * tex, a = colour.a * tex.a); 2 "revlight" (rgb = tex, a = colour.a); 3 MODULATE2X/4X
/// (mode 0 with the scale in `colour`, saturated)
pub const STAGE_MODE_BRAKE_LIGHT: f32 = 1.0;
pub const STAGE_MODE_REVLIGHT: f32 = 2.0;
pub const STAGE_MODE_SCALED: f32 = 3.0;

/// game space to bevy. the data is authored right-handed (text on signs reads correctly unmirrored and the
/// front faces are counter-clockwise), so positions and winding are used as stored.
pub fn to_bevy(p: [f32; 3]) -> Vec3 {
    Vec3::from_array(p)
}

pub fn make_image(t: &DecodedTexture, au: ImageAddressMode, av: ImageAddressMode) -> Image {
    let mut data = Vec::new();
    for l in &t.levels {
        data.extend_from_slice(l);
    }
    let mut img = Image::new(
        Extent3d { width: t.width, height: t.height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        t.levels[0].clone(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.data = Some(data);
    img.texture_descriptor.mip_level_count = t.levels.len() as u32;
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: au,
        address_mode_v: av,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    img
}

/// decoded textures by lowercase name; gpu images per (name, address mode) made on demand
#[derive(Default)]
pub struct TexSet {
    pub by_name: HashMap<String, DecodedTexture>,
    images: HashMap<(String, u32, u32), Handle<Image>>,
    linear: std::collections::HashSet<String>,
}

impl TexSet {
    pub fn add(&mut self, name: &str, t: DecodedTexture) {
        let key = name.trim_end_matches(".vtf").trim_end_matches(".tga").to_ascii_lowercase();
        self.by_name.insert(key, t);
    }

    /// a texture for a program stage. textures loaded through D3DX with non power-of-two sizes are linear on
    /// the xbox (the paint programs scale coordinates by the size for them)
    pub fn program_texture(&mut self, key: &str, au: u32, av: u32, images: &mut Assets<Image>) -> Option<ProgramTexture> {
        let (w, h) = self.by_name.get(key).map(|t| (t.width, t.height))?;
        let linear = self.linear.contains(key);
        let (au, av) = if linear { (3, 3) } else { (au, av) };
        let image = self.image(key, au, av, images)?;
        Some(ProgramTexture { image, size: UVec2::new(w, h), linear })
    }

    /// add a D3DX-loaded texture (tga); linear when not a power of two
    pub fn add_d3dx(&mut self, name: &str, t: DecodedTexture) {
        let key = name.trim_end_matches(".vtf").trim_end_matches(".tga").trim_end_matches(".TGA").to_ascii_lowercase();
        if !(t.width.is_power_of_two() && t.height.is_power_of_two()) {
            self.linear.insert(key.clone());
        }
        self.by_name.insert(key, t);
    }

    pub fn image(&mut self, key: &str, au: u32, av: u32, images: &mut Assets<Image>) -> Option<Handle<Image>> {
        let k = (key.to_string(), au, av);
        if let Some(h) = self.images.get(&k) {
            return Some(h.clone());
        }
        let t = self.by_name.get(key)?;
        let mode = |a: u32| if a == 3 { ImageAddressMode::ClampToEdge } else { ImageAddressMode::Repeat };
        let h = images.add(make_image(t, mode(au), mode(av)));
        self.images.insert(k, h.clone());
        Some(h)
    }
}

/// per-model spawn options (sky layers, colour factors, overridden textures / state)
#[derive(Default, Clone)]
pub struct SpawnOpts {
    pub layer: Option<RenderLayers>,
    pub parent: Option<Entity>,
    pub colour: Vec4,
    pub uv_offset: Vec2,
    pub texture: Option<String>,
    pub state: Option<fn(&mut StageState)>,
    /// fog colour + density (FogRender::render_states)
    pub fog: Vec4,
    /// stage colour from TFACTOR instead of the vertex diffuse
    pub tfactor: bool,
    /// the light set the model is drawn with (render struct light set, Renderer.cpp 0x002A8FD0): stages with flags
    /// bit 0 get d3d lighting (`VsLights::light_fixed_function`; without normals in the material, `set_lit`)
    pub light_set: Option<crate::vs_render::VsLights>,
    /// compartment instances (F3CompartmentManager::AddInstance 0x0008AC20 → 0x0008ABB0): primitives without an
    /// override get "HeadLightBeam" / "HeadLightBeamLit" (needs `Builder::beam`)
    pub beam_lights: bool,
}

impl SpawnOpts {
    pub fn world() -> Self {
        Self { colour: Vec4::ONE, ..default() }
    }

    fn key(&self) -> [u32; 6] {
        let c = self.colour.to_array().map(f32::to_bits);
        let f = self.fog.to_array().map(f32::to_bits);
        [c[0], c[1], c[2] ^ self.tfactor as u32, c[3] ^ f[3], self.uv_offset.x.to_bits() ^ f[0], self.uv_offset.y.to_bits() ^ self.state.is_some() as u32]
    }
}

pub struct Builder<'w> {
    pub meshes: &'w mut Assets<Mesh>,
    pub materials: &'w mut Assets<XboxStageMaterial>,
    pub images: &'w mut Assets<Image>,
    mat_cache: HashMap<(String, Vec<u8>, [u32; 6]), Handle<XboxStageMaterial>>,
    lit_cache: HashMap<(AssetId<XboxStageMaterial>, [u32; 3]), Handle<XboxStageMaterial>>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub tris: usize,
    /// lights for models without a diffuse stream
    pub lights: crate::vs_render::VsLights,
    /// car shader sets (cars only): shaders the car palette replaces with programmable sets
    pub car: Option<CarProgramCtx<'w>>,
    /// when set, every spawned primitive is recorded (car light lookup, GFXHelper.cpp 0x0012B4C0 order)
    pub records: Option<Vec<PrimRecord>>,
    /// when set, the vertex groups CarDamage deforms are collected and the meshes kept in the main world
    pub damage: Option<DamageCollect>,
    /// keep the meshes in the main world (the dynamic envmap's one-off normal bend, DangerZoneRender 0x00121BB0)
    pub keep_mesh_data: bool,
    /// RainSurface's "roaddetail" shader (`rain_surface::RainSurface::material`): the road primitives get a copy
    /// with it on `rain_surface::ROAD_DETAIL_LAYER` (draw list `dword_6D5328`, TrackFX 0x0014D6C0)
    pub road_detail: Option<Handle<XboxStageMaterial>>,
    /// beam light override programs (instances with `SpawnOpts::beam_lights`)
    pub beam: Option<BeamCtx<'w>>,
}

/// a "HeadLightBeam(Lit)" program material and what the per-frame binding needs
#[derive(Clone, Debug)]
pub struct BeamProgram {
    pub material: Handle<ProgramMaterial>,
    pub base: Option<ProgramTexture>,
    pub fog: Vec4,
}

pub struct BeamCtx<'w> {
    pub materials: &'w mut Assets<ProgramMaterial>,
    pub made: Vec<BeamProgram>,
    cache: HashMap<(String, Vec<u8>, bool, [u32; 4]), Handle<ProgramMaterial>>,
}

impl<'w> BeamCtx<'w> {
    pub fn new(materials: &'w mut Assets<ProgramMaterial>) -> Self {
        Self { materials, made: Vec::new(), cache: HashMap::new() }
    }
}

/// GameDataLoading.cpp 0x000A4CF0 — part flags from the part's first draw's vertex inputs (dwords 2/10/14 = the
/// formats of registers 0/2/3): 2 position, 0x20 normal, 4 diffuse; 8 = register 9 with dword 39 == 33 (packed uv,
/// not decoded here: no instance draw is known to have it); 0x80 / 0x400 / 0x800 come from elsewhere (not traced)
pub fn part_vertex_flags(m: &Model, part: &crate::renderer::renderer_types::Part) -> u32 {
    let Some(d) = part.primitives.first().and_then(|p| p.draws.first()) else { return 0 };
    let has = |r: usize| m.read_attr(d, r, 0).ok().flatten().is_some();
    let mut f = 0;
    if has(vreg::POSITION) {
        f |= 2;
    }
    if has(vreg::NORMAL) {
        f |= 0x20;
    }
    if has(vreg::DIFFUSE) {
        f |= 4;
    }
    f
}

/// `fTexCoordScaleFactor` x (GameShaderProgramManager_PC_XBOX 0x0015D250 sets (8, 8, 8, 1))
pub const TEX_COORD_SCALE_FACTOR: f32 = 8.0;

/// GameDataLoading.cpp 0x000A4CF0 flag 8 for one draw: register 9 (uv) present with format 33 (NORMSHORT2)
pub fn packed_uv(m: &Model, d: &crate::renderer::renderer_types::Draw) -> bool {
    m.vertex_descs.get(d.vertex_desc as usize).is_some_and(|v| v.inputs[vreg::TEXCOORD0].format == crate::renderer::renderer_types::vsdt::NORMSHORT2)
}

/// GameDataLoading.cpp 0x0009EF30: does GDL already give this primitive's shader an override (+0)? packed uv
/// (part 8) → DefaultPacked, "trafficbody_0" texture → Traffic, "Overlay_" (0x20 with "_Gloss" → RoadDecals, else
/// 0x200 → Overlays), "windowenv_" on a part with 0x4A0 and 0x804 (0x100 → Overlays)
pub fn gdl_override(shader_name: &str, texture: &str, part_flags: u32) -> bool {
    part_flags & 8 != 0
        || texture.contains("trafficbody_0")
        || shader_name.contains("Overlay_")
        || (shader_name.contains("windowenv_") && part_flags & 0x4A0 != 0 && part_flags & 0x804 != 0)
}

/// CompartmentManager.cpp 0x0008ABB0 — the instance override for a primitive without one: part flags & 0x804 →
/// shader 4 "HeadLightBeam" (Some(false)), else & 0x4A0 → 5 "HeadLightBeamLit" (Some(true))
pub fn beam_override(part_flags: u32) -> Option<bool> {
    if part_flags & 0x804 != 0 {
        Some(false)
    } else if part_flags & 0x4A0 != 0 {
        Some(true)
    } else {
        None
    }
}

/// SaveFileData.cpp 0x001065A0 — "HeadLightBeam" / "HeadLightBeamLit" over the primitive's own stage: created without
/// a reset after "CharacterSpecular" (itself after "Traffic"), so the compiled records are its code 2 (alpha test
/// ALWAYS ref 0, z LESSEQUAL) plus pass 1 clamp (u, v, w) and the programs BeamLights / BeamLights_Lit + BeamLights.pxu;
/// ShaderManager 0x002A4240 lays them over the own records (own texture, blend, z write, bias, stencil stay)
pub fn beam_material(own: &StageState, lit: bool, fog: Vec4) -> ProgramMaterial {
    let mut st = *own;
    if let Some(r) = crate::renderer::shader_manager::compile_alpha_test(2, 0, true, 2) {
        st.apply_commands(&r);
    }
    let v = if lit { crate::game_shader_program_manager::vp::BEAM_LIGHTS_LIT } else { crate::game_shader_program_manager::vp::BEAM_LIGHTS };
    let mut m = ProgramMaterial::new(v, crate::game_shader_program_manager::fp::BEAM_LIGHTS, st);
    m.params.fog = fog;
    m
}

/// GameDataLoading.cpp 0x0009EF30 (GDL_RelocateModel): a shader whose name contains "c_road" or "Specular_Road"
/// (strstr, case-sensitive) gets flags +4 |= 0x10, and DrawList.cpp 0x002C2650 files every primitive of such a
/// shader into the last list whose mask matches its flags — `dword_6D5328` (mask 16, VsRender 0x00058140; the later
/// masks 128/256 are "_reflect" / "windowenv_" flags no road shader carries). No shader in the data has flags set.
pub fn is_road_detail_shader(name: &str) -> bool {
    name.contains("c_road") || name.contains("Specular_Road")
}

/// one VertexIterator group (CarDamage walks each object's first part, every primitive, every draw: vertices
/// 0..=max-min from the draw's stream offset, VertexIterator.cpp 0x002C4DA0)
#[derive(Clone, Debug, Default)]
pub struct DamageGroup {
    /// mesh -> model (the part's matrix, row-vector)
    pub matrix: [f32; 16],
    /// element 0 (position) and element 1 (normal register), mesh space
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

/// a spawned mesh whose vertices come from damage groups: per bevy vertex (group, vertex), `u32::MAX` = none
#[derive(Clone, Debug)]
pub struct DamagePrim {
    pub mesh: Handle<Mesh>,
    pub xf: Mat4,
    pub verts: Vec<(u32, u32)>,
}

#[derive(Clone, Debug, Default)]
pub struct DamageCollect {
    pub groups: Vec<DamageGroup>,
    pub prims: Vec<DamagePrim>,
}

/// one spawned primitive: its entity, mesh and shader
#[derive(Clone, Debug)]
pub struct PrimRecord {
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
    /// shader (Maya material) name
    pub shader_name: String,
    /// primitive +4 (the initial car shader type of GameDataLoading.cpp 0x0009EAF0)
    pub kind: u32,
    /// object index in its model
    pub object: usize,
    /// stage 0 texture (lowercase) and state
    pub texture: String,
    pub state: StageState,
}

/// a program material made for a car shader, with what the per-frame binding needs
#[derive(Clone, Debug)]
pub struct CarProgram {
    pub material: Handle<ProgramMaterial>,
    pub set: CarShaderSet,
    pub base: Option<ProgramTexture>,
}

/// CarPalette_PC_XBOX.cpp 0x00157BA0 context: shader name -> set; ShinyCar and CarDecal become program materials
/// (one per set and base texture), the fixed-function sets keep the stage material for now.
pub struct CarProgramCtx<'w> {
    pub materials: &'w mut Assets<ProgramMaterial>,
    pub frontend: bool,
    pub made: Vec<CarProgram>,
    cache: HashMap<(CarShaderSet, String), Handle<ProgramMaterial>>,
    /// BrakeLight set materials (the car's one "BrakeLight" shader, TFACTOR set per frame)
    pub brake: Vec<Handle<XboxStageMaterial>>,
    brake_cache: HashMap<(String, u32, u32), Handle<XboxStageMaterial>>,
    /// CarShader / ShineyWheel stand-ins: (stage material, CarShader) -> its copy with the car stencil bit
    stencil_cache: HashMap<(AssetId<XboxStageMaterial>, bool), Handle<XboxStageMaterial>>,
}

impl<'w> CarProgramCtx<'w> {
    pub fn new(materials: &'w mut Assets<ProgramMaterial>, frontend: bool) -> Self {
        Self { materials, frontend, made: Vec::new(), cache: HashMap::new(), brake: Vec::new(), brake_cache: HashMap::new(), stencil_cache: HashMap::new() }
    }

    /// the program material of `set` with base texture `key` (one per set and texture, rebound every frame)
    pub fn program(&mut self, set: CarShaderSet, key: &str, base: Option<ProgramTexture>) -> Handle<ProgramMaterial> {
        if let Some(h) = self.cache.get(&(set, key.to_string())) {
            // the model's own stage texture may have been unresolved when this program was made (the decal shaders
            // name `bonnet_off` / `side_off` / `4x1_0_off`, which only DECALST.VCD holds, loaded after the models by
            // CarRender 0x00110220 -> 0x001163A0); the decal pass then asks for the same key with the texture — keep
            // it, else the program is bound without a base (opaque white: the "white bonnet")
            if base.is_some() {
                if let Some(m) = self.made.iter_mut().find(|m| m.material == *h && m.base.is_none()) {
                    m.base = base;
                }
            }
            return h.clone();
        }
        let h = self.materials.add(car_set_material(set));
        self.cache.insert((set, key.to_string()), h.clone());
        self.made.push(CarProgram { material: h.clone(), set, base });
        h
    }
}

impl<'w> Builder<'w> {
    pub fn new(meshes: &'w mut Assets<Mesh>, materials: &'w mut Assets<XboxStageMaterial>, images: &'w mut Assets<Image>) -> Self {
        Self {
            meshes,
            materials,
            images,
            mat_cache: HashMap::new(),
            lit_cache: HashMap::new(),
            bounds_min: Vec3::splat(f32::MAX),
            bounds_max: Vec3::splat(f32::MIN),
            tris: 0,
            lights: crate::vs_render::VsLights::default(),
            car: None,
            records: None,
            damage: None,
            keep_mesh_data: false,
            road_detail: None,
            beam: None,
        }
    }

    /// the beam program material of a primitive (one per own texture, own stage records, lit flag and fog)
    fn beam_program(&mut self, tex: &mut TexSet, shader: &Shader, opts: &SpawnOpts, lit: bool) -> Option<Handle<ProgramMaterial>> {
        let (name, cmds) = match shader.stages.first() {
            Some(s) => (s.texture_name.to_ascii_lowercase(), s.commands.clone()),
            None => (String::new(), vec![0]),
        };
        let f = opts.fog.to_array().map(f32::to_bits);
        let key = (name.clone(), cmds.clone(), lit, f);
        if let Some(h) = self.beam.as_ref()?.cache.get(&key) {
            return Some(h.clone());
        }
        let mut own = StageState::from_commands(&cmds);
        if let Some(st) = opts.state {
            st(&mut own);
        }
        let base = tex.program_texture(&name, own.address_u, own.address_v, self.images);
        let c = self.beam.as_mut()?;
        let h = c.materials.add(beam_material(&own, lit, opts.fog));
        c.cache.insert(key, h.clone());
        c.made.push(BeamProgram { material: h.clone(), base, fog: opts.fog });
        Some(h)
    }

    pub fn material(&mut self, tex: &mut TexSet, shader: &Shader, opts: &SpawnOpts) -> Handle<XboxStageMaterial> {
        let (mut name, cmds) = match shader.stages.first() {
            Some(s) => (s.texture_name.to_ascii_lowercase(), s.commands.clone()),
            None => (String::new(), vec![0]),
        };
        if let Some(t) = &opts.texture {
            name = t.to_ascii_lowercase();
        }
        let ck = (name.clone(), cmds.clone(), opts.key());
        if let Some(m) = self.mat_cache.get(&ck) {
            return m.clone();
        }
        let mut state = StageState::from_commands(&cmds);
        if let Some(f) = opts.state {
            f(&mut state);
        }
        let texture = tex.image(&name, state.address_u, state.address_v, self.images);
        if texture.is_none() && !name.is_empty() && name != "none" {
            log::warn!("texture {name} not found");
        }
        let mut params = XboxStageParams::new(&state);
        params.colour = opts.colour;
        params.uv_offset = opts.uv_offset;
        params.fog = opts.fog;
        if opts.tfactor {
            params.arg.x = 0.0;
        }
        let m = self.materials.add(XboxStageMaterial { params, texture, stencil: None, spec_texture: None, state });
        self.mat_cache.insert(ck, m.clone());
        m
    }

    /// a copy of a stage material drawn with d3d lighting on and no normal stream (`XboxStageMaterial::set_lit`)
    pub fn lit_material(&mut self, mat: Handle<XboxStageMaterial>, lights: &crate::vs_render::VsLights) -> Handle<XboxStageMaterial> {
        let lit = lights.light_fixed_function(None, Vec4::ONE);
        let key = (mat.id(), [lit.x.to_bits(), lit.y.to_bits(), lit.z.to_bits()]);
        if let Some(h) = self.lit_cache.get(&key) {
            return h.clone();
        }
        let Some(mut m) = self.materials.get(&mat).cloned() else { return mat };
        m.set_lit(Some(lights));
        let h = self.materials.add(m);
        self.lit_cache.insert(key, h.clone());
        h
    }

    /// CarPalette_PC_XBOX.cpp 0x001564A0 "CarShader" / 0x00156520 "ShineyWheel" as the override (shader +0) of the
    /// primitive's own stage material (0x00157BA0 → 0x00156440): the set's compiled records over the own stage
    /// (ShaderManager 0x002A4240: alpha test ALWAYS, z LESSEQUAL; own texture, blend, clamp, z write, bias), plus the
    /// stencil: ref 4 REPLACE, write mask all (CarShader: bit 4 set, glow bits cleared) / ~3 (ShineyWheel: glow kept)
    pub fn car_stencil_material(&mut self, mat: Handle<XboxStageMaterial>, set: CarShaderSet) -> Handle<XboxStageMaterial> {
        let car_shader = set == CarShaderSet::CarShader;
        let key = (mat.id(), car_shader);
        if let Some(h) = self.car.as_ref().and_then(|c| c.stencil_cache.get(&key)) {
            return h.clone();
        }
        let Some(mut m) = self.materials.get(&mat).cloned() else { return mat };
        let frontend = self.car.as_ref().is_some_and(|c| c.frontend);
        if let Some(f) = crate::car_palette_pc_xbox::fixed_car_set(set, frontend) {
            m.state.apply_commands(&f.records);
            m.params.alpha_ref = m.state.alpha_ref as f32;
            m.params.alpha_func = if m.state.alpha_test { m.state.alpha_func } else { d3dcmp::ALWAYS };
        }
        m.state.stencil_car = true;
        if car_shader {
            m.state.stencil = false;
        }
        let h = self.materials.add(m);
        if let Some(c) = self.car.as_mut() {
            c.stencil_cache.insert(key, h.clone());
        }
        h
    }

    /// CarPalette_PC_XBOX.cpp 0x00156A60 — the "BrakeLight" set: blend 1, COLOROP MULTIPLYADD (TEXTURE +
    /// TFACTOR * TEXTURE), ALPHAOP MODULATE (TFACTOR * TEXTURE), alpha test ALWAYS; the primitive's own texture.
    pub fn brake_material(&mut self, tex: &mut TexSet, shader: &Shader) -> Handle<XboxStageMaterial> {
        let (name, cmds) = match shader.stages.first() {
            Some(s) => (s.texture_name.to_ascii_lowercase(), s.commands.clone()),
            None => (String::new(), vec![0]),
        };
        let own = StageState::from_commands(&cmds);
        let key = (name.clone(), own.address_u, own.address_v);
        if let Some(h) = self.car.as_ref().and_then(|c| c.brake_cache.get(&key)) {
            return h.clone();
        }
        // stencil ref 7 + REPLACE (0x002A1140(7), 0x002A12C0(0)): a glow mark for bloom and the car bit 4
        let state = StageState { blend_mode: 1, address_u: own.address_u, address_v: own.address_v, stencil: true, stencil_car: true, ..StageState::default() };
        let texture = tex.image(&name, state.address_u, state.address_v, self.images);
        let mut params = XboxStageParams::new(&state);
        params.arg = Vec4::new(0.0, STAGE_MODE_BRAKE_LIGHT, 0.0, 0.0);
        params.colour = Vec4::splat(0.5);
        let h = self.materials.add(XboxStageMaterial { params, texture, stencil: None, spec_texture: None, state });
        if let Some(c) = self.car.as_mut() {
            c.brake_cache.insert(key, h.clone());
            c.brake.push(h.clone());
        }
        h
    }

    /// one bevy mesh per primitive of each object; `xf` places the whole model (game space).
    pub fn add_model(&mut self, commands: &mut Commands, tex: &mut TexSet, m: &Model, xf: Mat4, opts: &SpawnOpts) -> Vec<Handle<XboxStageMaterial>> {
        let mut used = Vec::new();
        for (oi, obj) in m.objects.iter().enumerate() {
            // d3d row-vector matrix: v' = v * M, i.e. column-major M^T
            let om = Mat4::from_cols_array(&obj.matrix);
            let full = xf * om;
            for (pi, part) in obj.parts.iter().enumerate() {
                let beam_on = opts.beam_lights && self.beam.is_some();
                let part_flags = if beam_on { part_vertex_flags(m, part) } else { 0 };
                for prim in &part.primitives {
                    let shader = &m.shaders[prim.shader];
                    // Renderer.cpp 0x002A8FD0: d3d lighting when the stage has flags bit 0 and a light set is bound
                    let ff_lit = opts.light_set.filter(|_| shader.stages.first().is_some_and(|s| s.flags & 1 != 0));
                    // CompartmentManager.cpp 0x0008ABB0 on instances whose shader has no override yet
                    let beam = if beam_on {
                        let t = shader.stages.first().map(|s| s.texture_name.as_str()).unwrap_or("");
                        if gdl_override(&shader.name, t, part_flags) { None } else { beam_override(part_flags) }
                    } else {
                        None
                    };
                    let mut has_normals = false;
                    let (mut pos, mut col, mut uv, mut nrm, mut idx) = (vec![], vec![], vec![], vec![], vec![]);
                    let mut dverts: Vec<(u32, u32)> = Vec::new();
                    for d in &prim.draws {
                        // CarDamage 0x0006DA40: only the first part (`*(obj + 84)`) of each object
                        let group = match self.damage.as_mut() {
                            Some(dc) if pi == 0 => {
                                dc.groups.push(damage_group(m, d, obj.matrix));
                                (dc.groups.len() - 1) as u32
                            }
                            _ => u32::MAX,
                        };
                        let Ok(ind) = m.draw_indices(d) else { continue };
                        // GDL 0x0009EF30: packed uv (part flag 8, 0x000A4CF0) → "DefaultPacked" (vp default_packed:
                        // oT0 = v9 · c4 fTexCoordScaleFactor (8, 8, 8, 1), oD0 = v3)
                        let uv_scale = if packed_uv(m, d) { TEX_COORD_SCALE_FACTOR } else { 1.0 };
                        let mut remap: HashMap<u32, u32> = HashMap::new();
                        for &i in &ind {
                            let n = pos.len() as u32;
                            let v = *remap.entry(i).or_insert_with(|| {
                                let p = m.read_attr(d, vreg::POSITION, i).ok().flatten().unwrap_or([0.0; 4]);
                                let wp = full.transform_point3(Vec3::new(p[0], p[1], p[2]));
                                pos.push(to_bevy(wp.into()).to_array());
                                let t = m.read_attr(d, vreg::TEXCOORD0, i).ok().flatten().unwrap_or([0.0; 4]);
                                uv.push([t[0] * uv_scale, t[1] * uv_scale]);
                                let nn = m.read_attr(d, vreg::NORMAL, i).ok().flatten();
                                let wn = nn
                                    .map(|nn| full.transform_vector3(Vec3::new(nn[0], nn[1], nn[2])).normalize_or_zero())
                                    .unwrap_or(Vec3::Y);
                                nrm.push(to_bevy(wn.into()).to_array());
                                // prelit colours, else lit by the scene's directional lights
                                let mut c = match (m.read_attr(d, vreg::DIFFUSE, i).ok().flatten(), nn) {
                                    (Some(c), _) => c,
                                    (None, Some(_)) => self.lights.light_vertex(wn).to_array(),
                                    (None, None) => [1.0; 4],
                                };
                                // with normals the lit colour is per vertex here; without, the material's `set_lit`
                                // (beam prims keep the raw stream: their vertex program reads v3)
                                if let (Some(l), Some(_), None) = (ff_lit.as_ref(), nn, beam) {
                                    has_normals = true;
                                    c = l.light_fixed_function(Some(wn), Vec4::from_array(c)).to_array();
                                }
                                col.push([c[0], c[1], c[2], c[3]]);
                                dverts.push((group, i));
                                n
                            });
                            idx.push(v);
                        }
                    }
                    if idx.is_empty() {
                        continue;
                    }
                    for p in &pos {
                        let v = Vec3::from_array(*p);
                        self.bounds_min = self.bounds_min.min(v);
                        self.bounds_max = self.bounds_max.max(v);
                    }
                    self.tris += idx.len() / 3;
                    let usage = if self.damage.is_some() || self.keep_mesh_data { RenderAssetUsages::all() } else { RenderAssetUsages::RENDER_WORLD };
                    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, usage);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
                    mesh.insert_indices(Indices::U32(idx));
                    let mesh_h = self.meshes.add(mesh);
                    let road_mesh = mesh_h.clone();
                    if let Some(dc) = self.damage.as_mut() {
                        if dverts.iter().any(|v| v.0 != u32::MAX) {
                            dc.prims.push(DamagePrim { mesh: mesh_h.clone(), xf: full, verts: dverts });
                        }
                    }
                    let base_tex = shader.stages.first().map(|s| s.texture_name.to_ascii_lowercase()).unwrap_or_default();
                    let set = self.car.as_ref().and_then(|c| {
                        let ty = car_shader_type(&shader.name, prim.kind, c.frontend);
                        shader_set_for(ty, &shader.name)
                    });
                    let mut e = if let Some(lit) = beam {
                        // the plain stage material is kept for the A/B harness (with the count-0 instance set the
                        // per-vertex fixed-function result is the ambient alone, as `lit_material` gives)
                        let mut fixed = self.material(tex, shader, opts);
                        if let Some(l) = ff_lit.as_ref() {
                            fixed = self.lit_material(fixed, l);
                        }
                        match self.beam_program(tex, shader, opts, lit) {
                            Some(p) => commands.spawn((
                                Mesh3d(mesh_h.clone()),
                                MeshMaterial3d(p.clone()),
                                crate::projected_light_beam_pc_xbox::BeamPrim { program: p, fixed },
                            )),
                            None => commands.spawn((Mesh3d(mesh_h.clone()), MeshMaterial3d(fixed))),
                        }
                    } else {
                        match set {
                        Some(set @ (CarShaderSet::ShinyCar | CarShaderSet::CarDecal)) => {
                            let state = StageState::from_commands(shader.stages.first().map(|s| s.commands.as_slice()).unwrap_or(&[0]));
                            let base = tex.program_texture(&base_tex, state.address_u, state.address_v, self.images);
                            let h = self.car.as_mut().unwrap().program(set, &base_tex, base);
                            commands.spawn((Mesh3d(mesh_h.clone()), MeshMaterial3d(h)))
                        }
                        Some(CarShaderSet::BrakeLight) => {
                            let mat = self.brake_material(tex, shader);
                            commands.spawn((Mesh3d(mesh_h.clone()), MeshMaterial3d(mat)))
                        }
                        _ => {
                            let mut mat = self.material(tex, shader, opts);
                            if let (Some(l), false) = (ff_lit.as_ref(), has_normals) {
                                mat = self.lit_material(mat, l);
                            }
                            if let Some(set @ (CarShaderSet::CarShader | CarShaderSet::ShineyWheel)) = set {
                                mat = self.car_stencil_material(mat, set);
                            }
                            if !used.contains(&mat) {
                                used.push(mat.clone());
                            }
                            commands.spawn((Mesh3d(mesh_h.clone()), MeshMaterial3d(mat)))
                        }
                        }
                    };
                    if let Some(r) = self.records.as_mut() {
                        let st = shader.stages.first();
                        r.push(PrimRecord {
                            entity: e.id(),
                            mesh: mesh_h,
                            shader_name: shader.name.clone(),
                            kind: prim.kind,
                            object: oi,
                            texture: base_tex.clone(),
                            state: StageState::from_commands(st.map(|s| s.commands.as_slice()).unwrap_or(&[0])),
                        });
                    }
                    if let Some(l) = &opts.layer {
                        e.insert(l.clone());
                    }
                    if let Some(p) = opts.parent {
                        e.insert(ChildOf(p));
                    }
                    // the road detail list's redraw (TrackFX 0x0014D6C0) with the roaddetail shader swapped in
                    if let (Some(rd), None) = (&self.road_detail, &opts.layer) {
                        if is_road_detail_shader(&shader.name) {
                            let layer = RenderLayers::layer(crate::rain_surface::ROAD_DETAIL_LAYER);
                            let mut d = commands.spawn((Mesh3d(road_mesh), MeshMaterial3d(rd.clone()), layer));
                            if let Some(p) = opts.parent {
                                d.insert(ChildOf(p));
                            }
                        }
                    }
                }
            }
        }
        used
    }
}


/// CarDamage 0x0006DA40 / VertexIterator.cpp 0x002C4DA0 — a draw's vertex group: count = max - min + 1 for an
/// indexed draw (else the index count), read from the stream offset; element 1 is the normal register
pub(crate) fn damage_group(m: &Model, d: &crate::renderer::renderer_types::Draw, matrix: [f32; 16]) -> DamageGroup {
    let n = if d.index_kind == 1 { d.max_index as u32 - d.min_index as u32 + 1 } else { d.index_count };
    let mut g = DamageGroup { matrix, ..default() };
    for k in 0..n {
        let p = m.read_attr(d, vreg::POSITION, k).ok().flatten().unwrap_or([0.0; 4]);
        let q = m.read_attr(d, vreg::NORMAL, k).ok().flatten().unwrap_or([0.0; 4]);
        g.positions.push([p[0], p[1], p[2]]);
        g.normals.push([q[0], q[1], q[2]]);
    }
    g
}

/// CarPalette_PC_XBOX.cpp 0x00156C00 (ShinyCar: stage flags 1, blend 1, programs ShinyCarPaint + Default) and
/// 0x00156680 (CarDecal: blend 3, z test without z write, programs Decals + Decals)
pub fn car_set_material(set: CarShaderSet) -> ProgramMaterial {
    let mut st = StageState::default();
    match set {
        CarShaderSet::CarDecal => {
            st.blend_mode = 3;
            st.z_write = false;
            st.z_func = d3dcmp::LESSEQUAL;
            ProgramMaterial::new(vp::DECALS, fp::DECALS, st)
        }
        _ => {
            st.blend_mode = 1;
            // stencil ref 7 + REPLACE: the paint's alpha (envmap alpha) is a bloom source; bit 4 = car
            st.stencil = true;
            st.stencil_car = true;
            ProgramMaterial::new(vp::SHINY_CAR_PAINT, fp::DEFAULT, st)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CompartmentManager 0x0008ABB0 choice and GDL's own overrides
    #[test]
    fn beam_override_choice() {
        assert_eq!(beam_override(0x2 | 0x4 | 0x20), Some(false));
        assert_eq!(beam_override(0x2 | 0x20), Some(true));
        assert_eq!(beam_override(0x2), None);
        assert!(gdl_override("Overlay_x", "", 0) && gdl_override("a", "trafficbody_0", 0) && gdl_override("a", "", 8));
        assert!(gdl_override("windowenv_a", "", 0x24) && !gdl_override("windowenv_a", "", 0x20) && !gdl_override("a", "b", 0x24));
        let own = StageState { blend_mode: 3, alpha_test: true, alpha_func: d3dcmp::GREATER, alpha_ref: 0x28, ..StageState::default() };
        let m = beam_material(&own, true, Vec4::ZERO);
        assert_eq!((m.vertex as usize, m.fragment as usize, m.state.blend_mode), (vp::BEAM_LIGHTS_LIT, fp::BEAM_LIGHTS, 3));
        assert_eq!((m.state.alpha_func, m.state.alpha_ref), (d3dcmp::ALWAYS, 0));
    }

    /// instance primitives of every track by override: writes research/gfx/beam_census.txt
    #[test]
    #[ignore]
    fn beam_census() {
        use crate::compartment_sys::{Compartment, INSTANCE_VISUAL};
        use std::collections::BTreeMap;
        let mut n: BTreeMap<String, usize> = BTreeMap::new();
        for p in crate::test_util::all_vcds() {
            if !p.to_string_lossy().to_ascii_lowercase().ends_with("cpts.vcd") {
                continue;
            }
            let Ok(vcd) = visos::file::Vcd::open_host(&p, &p.to_string_lossy()) else { continue };
            let track = p.parent().and_then(|d| d.file_name()).map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
            for e in vcd.entries() {
                let Some(d) = vcd.data(&e) else { continue };
                let Ok(c) = Compartment::parse(d) else { continue };
                for l in c.instance_lists().unwrap_or_default() {
                    if l.kind != INSTANCE_VISUAL {
                        continue;
                    }
                    for i in &l.instances {
                        let Some(r) = c.resource(i.visual) else { continue };
                        let Ok(m) = Model::parse(r.data) else { continue };
                        for o in &m.objects {
                            for part in &o.parts {
                                let f = part_vertex_flags(&m, part);
                                for prim in &part.primitives {
                                    let sh = &m.shaders[prim.shader];
                                    let st = sh.stages.first();
                                    let t = st.map(|s| s.texture_name.as_str()).unwrap_or("");
                                    let own = StageState::from_commands(st.map(|s| s.commands.as_slice()).unwrap_or(&[0]));
                                    let kind = if gdl_override(&sh.name, t, f) {
                                        "gdl"
                                    } else {
                                        match beam_override(f) {
                                            Some(false) => "beam",
                                            Some(true) => "beamlit",
                                            None => "none",
                                        }
                                    };
                                    let cut = own.alpha_test && own.alpha_func != d3dcmp::ALWAYS;
                                    let lit = st.is_some_and(|s| s.flags & 1 != 0);
                                    *n.entry(format!("{track} {kind} flags {f:#x} stageflag1 {lit} cut {cut} blend {}", own.blend_mode)).or_default() += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        let out: String = n.iter().map(|(k, v)| format!("{v} {k}\n")).collect();
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/beam_census.txt"), out).unwrap();
    }
}
