//! The frontend's 3D background (FE3DCamera / FE3DVehicles / FE3DCharacters side of the bevy app): the camera
//! element's eight background models (hangout, LA, London, Tokyo, modArea, carSel, hangCE, empty) shown as
//! `Feck2::foyer.backgrounds` says, one `SpawnCarRender` per shown vehicle spot (`Feck2::foyer_vehicles`, parts
//! from the spot's setup), one `SpawnSkinnedModel` per character element (`Feck2::characters`), and the camera.
//!
//! `Feck2::foyer.name` (`SetFoyer`) picks the lights: `Lights.txt` / `PtLights.txt` light the car
//! (`TrackLights` / `TrackPointLights`). Not ported: foyer particles (`particle.pld`).

use std::collections::HashMap;

use bevy::prelude::*;
use visos_render::car_palette::CarColour;
use velocity_physics::vehicle_database::vehicle_database::VehicleSetup;
use visos_render::car_render::{CarSpec, DespawnCarRender, SpawnCarRender, PART_SLOTS};
use visos_render::model_builder::{Builder, SpawnOpts, TexSet};
use visos_render::renderer::renderer_types::Model;
use visos_render::renderer::shader_manager::XboxStageMaterial;
use visos_render::renderer::texture_manager::decode_any_vtf;
use visos_render::specular_chars::SpecCharIndex;
use visos_render::skinned_object_render::{
    CharPack, DespawnSkinnedModel, SkinnedModelAnim, SkinnedModelBlend, SkinnedModelRender, SpawnSkinnedModel,
};

use crate::feck2::screens::common::fe_3d_camera::BACKGROUNDS;
use crate::feck2::screens::common::fe_3d_characters::{FEProp, PROPS_MODEL_VCD, PROPS_TEXTURE_VCD};
use crate::FrontendState;

/// `CarRender` request keys for the foyer cars (+ slot)
const FOYER_CAR_REQUEST: u64 = 0xFE_0001;
/// skinned model request keys for the characters (+ group * 256 + index)
const FOYER_CHAR_REQUEST: u64 = 0xFE_1000;

/// what a spawned foyer car was built from:
/// (vehicle, colour hash, brand, spot, parts, (shadow height, neon) of the setup's underbody); the rotation is not in
/// it: `foyer_car_poses` turns the root (the showroom turntable changes it every frame)
type CarKey = (usize, u32, u32, [u32; 3], [Option<u32>; 7], (Option<u32>, Option<u32>, [Option<String>; 2], Option<String>, Option<String>, visos_render::car_render::CarDecals, [u32; 2]));

#[derive(Resource, Default)]
pub struct FoyerView {
    /// lights loaded for this foyer name
    pub loaded: Option<String>,
    /// background element roots (built the first time they are shown)
    pub backgrounds: [Option<Entity>; 8],
    pub camera: Option<Entity>,
    pub cars: HashMap<usize, CarKey>,
    /// character request -> anim shown
    pub chars: HashMap<u64, Option<String>>,
    /// character requests with a cross-fade on show
    pub blending: std::collections::HashSet<u64>,
    /// character prop elements (FE3DCharacters +61200) by char index: what was built and its root
    pub props: HashMap<usize, (FEProp, Option<Entity>)>,
    /// each built background's prims (for the material texture swaps)
    pub bg_prims: [Vec<BgPrim>; 8],
    /// (background, material) -> (texture, vcd) applied
    pub swapped: HashMap<(usize, String), (String, String)>,
    /// a swapped prim's material as built
    pub originals: HashMap<Entity, Handle<XboxStageMaterial>>,
    /// (vcd, texture, address u, v) -> image (None = not in the vcd)
    pub swap_images: HashMap<(String, String, u32, u32), Option<Handle<Image>>>,
}

/// FE3DCamera.cpp 0x00273990 — FECK2Element3D load of a "CharacterProp" (desc from 0x002609D0): `<model>.xmf` of PROPSM.VCD with the PROPST.VCD
/// textures under a root at the prop's world matrix (0x0025F2A0), on the camera's background list layers
/// (0x00273990). None when the model is missing.
fn build_prop(commands: &mut Commands, b: &mut Builder, p: &FEProp) -> Option<Entity> {
    let mv = visos::file::Vcd::open(PROPS_MODEL_VCD).ok()?;
    let e = mv.find(&format!("{}.xmf", p.model))?;
    let m = match Model::parse(mv.data(&e)?) {
        Ok(m) => m,
        Err(err) => {
            log::warn!("prop {}: {err}", p.model);
            return None;
        }
    };
    let t = Transform::from_translation(Vec3::from_array(p.pos)).with_rotation(Quat::from_rotation_y(p.rotation));
    let root = commands.spawn((t, Visibility::Hidden)).id();
    let mut tex = TexSet::default();
    if let Ok(v) = visos::file::Vcd::open(PROPS_TEXTURE_VCD) {
        for en in v.entries() {
            if let Some(t) = v.data(&en).and_then(|d| decode_any_vtf(d).ok()) {
                tex.add(en.name, t);
            }
        }
    }
    let mut opts = SpawnOpts::world();
    opts.parent = Some(root);
    opts.layer = Some(bevy::camera::visibility::RenderLayers::from_layers(&[0, visos_render::rain_surface::FE_REFL_ENV_LAYER]));
    b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &opts);
    log::info!("prop {}: {} triangles", p.model, b.tris);
    Some(root)
}

#[derive(Component)]
pub struct FoyerCamera;

/// FE3DCamera.cpp 0x00273000 / 0x00273700 — one background element: `<stem>.xmf` with its texture pack, placed at the
/// background's translation (0x00273700, all zero). None when the model is missing.
fn build_background(commands: &mut Commands, b: &mut Builder, index: usize) -> Option<(Entity, Vec<BgPrim>)> {
    let (stem, vcd) = BACKGROUNDS[index];
    let bytes = std::fs::read(visos::file::resolve(&format!("{stem}.xmf"))).ok()?;
    let root = commands.spawn((Transform::default(), Visibility::Hidden)).id();
    let mut tex = TexSet::default();
    if let Ok(v) = visos::file::Vcd::open(vcd) {
        for en in v.entries() {
            if let Some(t) = v.data(&en).and_then(|d| decode_any_vtf(d).ok()) {
                tex.add(en.name, t);
            }
        }
    }
    let mut opts = SpawnOpts::world();
    opts.parent = Some(root);
    // FECK2ElementCamera 0x002999B0 draws its background list (+65, flag +292 = shown, set by SetBackgroundVisible
    // 0x00273620 through vtable +120 = 0x0029A310) into the "FErefl" target when Reflect Environment (`byte_7263CD`)
    // is on; hidden backgrounds drop out of both through the root's Visibility
    opts.layer = Some(bevy::camera::visibility::RenderLayers::from_layers(&[0, visos_render::rain_surface::FE_REFL_ENV_LAYER]));
    // the prims by shader name, for the material texture swaps (FECK2Element3D 0x00298690 FindShader)
    b.records = Some(Vec::new());
    match Model::parse(&bytes) {
        Ok(m) => {
            b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &opts);
        }
        Err(e) => log::warn!("background {stem}: {e}"),
    }
    log::info!("background {index} {stem}: {} triangles", b.tris);
    let prims = b.records.take().unwrap_or_default();
    Some((root, prims.into_iter().map(|r| BgPrim { shader: r.shader_name, entity: r.entity, au: r.state.address_u, av: r.state.address_v }).collect()))
}

/// `texture` (no extension, any case) of `vcd` as an image with the prim's address modes
fn swap_image(vcd: &str, texture: &str, au: u32, av: u32, images: &mut Assets<Image>) -> Option<Handle<Image>> {
    let v = visos::file::Vcd::open(vcd).ok()?;
    let want = texture.to_ascii_lowercase();
    let en = v.entries().find(|e| e.name.to_ascii_lowercase().trim_end_matches(".vtf") == want)?;
    let t = decode_any_vtf(v.data(&en)?).ok()?;
    let mode = |a: u32| if a == 3 { bevy::image::ImageAddressMode::ClampToEdge } else { bevy::image::ImageAddressMode::Repeat };
    Some(images.add(visos_render::model_builder::make_image(&t, mode(au), mode(av))))
}

/// FECK2Element3D.cpp 0x00298690 — `FoyerState.material_textures` onto the built backgrounds: every prim whose
/// shader is the named material draws the new texture (TextureManager 0x0029F650 by name, else loaded from the
/// given vcd; the background's own pack stands in for textures the manager already holds). Swaps no longer
/// wanted (a reinitialised front end) get their built material back.
pub fn foyer_material_textures(
    state: Res<FrontendState>,
    mut view: ResMut<FoyerView>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut prims: Query<&mut MeshMaterial3d<XboxStageMaterial>>,
) {
    let Some(f) = state.frontend.as_ref().filter(|_| state.active) else { return };
    let want: HashMap<(usize, String), (String, String)> = f
        .fe
        .foyer
        .material_textures
        .iter()
        .filter(|m| view.backgrounds.get(m.background).is_some_and(Option::is_some))
        .map(|m| ((m.background, m.material.clone()), (m.texture.clone(), m.vcd.clone())))
        .collect();
    let v = &mut *view;
    // FindShader 0x0012AC80: the first prim (objects, then their prims) whose shader name contains the material name
    // (`strstr`, case sensitive); the swap changes that shader, i.e. every prim drawn with it
    let shader_prims = |prims: &[BgPrim], material: &str| -> Vec<BgPrim> {
        let Some(s) = prims.iter().find(|p| p.shader.contains(material)).map(|p| p.shader.clone()) else { return Vec::new() };
        prims.iter().filter(|p| p.shader == s).cloned().collect()
    };
    // dropped swaps: the built material back
    let gone: Vec<(usize, String)> = v.swapped.keys().filter(|k| !want.contains_key(*k)).cloned().collect();
    for k in gone {
        v.swapped.remove(&k);
        for p in shader_prims(&v.bg_prims[k.0], &k.1) {
            if let (Some(h), Ok(mut m)) = (v.originals.remove(&p.entity), prims.get_mut(p.entity)) {
                m.0 = h;
            }
        }
    }
    for (k, (texture, vcd)) in want {
        if v.swapped.get(&k) == Some(&(texture.clone(), vcd.clone())) {
            continue;
        }
        let matched = shader_prims(&v.bg_prims[k.0], &k.1);
        // spawned this frame: the prims' materials are not queryable until the commands ran
        if matched.iter().any(|p| prims.get(p.entity).is_err()) {
            continue;
        }
        for p in &matched {
            let key = (vcd.clone(), texture.clone(), p.au, p.av);
            let img = match v.swap_images.get(&key) {
                Some(i) => i.clone(),
                None => {
                    let i = swap_image(&vcd, &texture, p.au, p.av, &mut images)
                        .or_else(|| swap_image(BACKGROUNDS[k.0].1, &texture, p.au, p.av, &mut images));
                    if i.is_none() {
                        log::warn!("background {} material {}: no texture {texture} in {vcd}", k.0, k.1);
                    }
                    v.swap_images.insert(key, i.clone());
                    i
                }
            };
            let Ok(mut m) = prims.get_mut(p.entity) else { continue };
            let orig = v.originals.entry(p.entity).or_insert_with(|| m.0.clone()).clone();
            let Some(mut mat) = materials.get(&orig).cloned() else { continue };
            mat.texture = img;
            m.0 = materials.add(mat);
        }
        v.swapped.insert(k, (texture, vcd));
    }
}

/// one spawned primitive of a background element (its shader name, stage 0 address modes)
#[derive(Clone, Debug)]
pub struct BgPrim {
    pub shader: String,
    pub entity: Entity,
    pub au: u32,
    pub av: u32,
}

/// the setup's fitted bodyparts as CarRender part indices: the part's model name (`Bodypart` +0x10, e.g.
/// "f_bumper3") picks the slot and index (FE3DVehicles 0x0025CB20 loads the models by that name)
fn setup_parts(db: &velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase, setup: &VehicleSetup) -> [Option<u32>; 7] {
    let mut parts = CarSpec::new("", Vec3::ONE).parts;
    for p in setup.fitted_bodyparts.iter().flatten() {
        let Some(name) = db.bodypart_sets.get(p.set).and_then(|s| s.parts[p.part_type].get(p.index)).and_then(|b| b.string_10.as_deref()) else {
            continue;
        };
        for (slot, prefix) in PART_SLOTS.iter().enumerate() {
            if let Some(n) = name.strip_prefix(prefix).and_then(|r| r.parse::<u32>().ok()) {
                parts[slot] = Some(n);
            }
        }
    }
    parts
}

/// "\VEHICLES\LA\350Z\" -> "LA/350Z"
fn car_dir(path: &str) -> String {
    let p = path.trim_matches('\\').replace('\\', "/");
    p.strip_prefix("VEHICLES/").or_else(|| p.strip_prefix("Vehicles/")).unwrap_or(&p).to_string()
}

/// FE3DVehicles.cpp 0x0025DB50 — the world matrix of each spawned foyer car: `RotationY(+1248)` then the spot
/// position, every frame (the showroom turntable turns slot 0 without a respawn)
pub fn foyer_car_poses(state: Res<FrontendState>, mut cars: Query<(&visos_render::car_render::CarRender, &mut Transform)>) {
    let Some(f) = state.frontend.as_ref().filter(|_| state.active) else { return };
    for (c, mut t) in &mut cars {
        let Some(slot) = c.request.checked_sub(FOYER_CAR_REQUEST) else { continue };
        if slot >= 0x1000 - 1 {
            continue;
        }
        if let Some(v) = f.fe.foyer_vehicles.slots.get(slot as usize) {
            let want = Transform::from_translation(Vec3::from_array(v.pos)).with_rotation(Quat::from_rotation_y(v.rotation.to_radians()));
            if *t != want {
                *t = want;
            }
        }
    }
}

/// FE3DVehicles.cpp 0x0025BF80 / 0x0025B4D0 / 0x0025BA40 — the door, wheel and brake disc part matrices of each
/// foyer car, every frame. Door: `T(h) * slerp(I, open^-1, t) * T(-h)` about the locator's hinge; front wheels (f_l
/// `-steer`, f_r `+steer`, 0x0025E040) and their discs turn about the pivot, all four sit `bounce` (0x0025E7D0) lower
/// than the root, which the bounce raises.
pub fn foyer_car_parts(
    mut state: ResMut<FrontendState>,
    cars: Query<(&visos_render::car_render::CarRender, &visos_render::car_render::CarRenderParts)>,
    mut tf: Query<&mut Transform>,
) {
    if !state.active {
        return;
    }
    let Some(f) = state.frontend.as_mut() else { return };
    for (c, p) in &cars {
        // CarRender 0x00116BC0 per-kind decal slot counts of the first foyer car (FE3DVehicles +1824 car)
        if c.request == FOYER_CAR_REQUEST {
            let d = Some(p.decal_slots.map(u32::from));
            if f.fe.foyer_vehicles.decal_slots != d {
                f.fe.foyer_vehicles.decal_slots = d;
            }
        }
        let Some(v) = c.request.checked_sub(FOYER_CAR_REQUEST).and_then(|s| f.fe.foyer_vehicles.slots.get(s as usize)) else { continue };
        let mut set = |e: Entity, want: Transform| {
            if let Ok(mut t) = tf.get_mut(e) {
                if *t != want {
                    *t = want;
                }
            }
        };
        for k in 0..8 {
            if let (Some(e), Some((h, open))) = (p.doors[k], p.door_locators[k]) {
                let q = Quat::IDENTITY.slerp(open.inverse(), v.door_t[k]);
                set(e, Transform::from_translation(h - q * h).with_rotation(q));
            }
        }
        let down = Vec3::new(0.0, -v.bounce, 0.0);
        for k in 0..4 {
            let steer = match k {
                0 => -v.steer,
                2 => v.steer,
                _ => 0.0,
            };
            let r = Quat::from_rotation_y(steer.to_radians());
            if p.wheel_pivots[k] != Vec3::ZERO {
                set(p.fe_wheels[k], Transform::from_translation(p.wheel_pivots[k] + down).with_rotation(r * p.wheel_base_rot[k]));
            }
            if let Some(d) = p.discs[k] {
                let mut t = Transform::from_translation(p.disc_pivots[k] + down).with_rotation(r);
                if k >= 2 {
                    t.scale = Vec3::new(-1.0, 1.0, 1.0);
                }
                set(d, t);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_foyer(
    mut commands: Commands,
    state: Res<FrontendState>,
    mut view: ResMut<FoyerView>,
    mut cams: Query<(&mut Transform, &mut Projection), With<FoyerCamera>>,
    mut models: Query<(Entity, &SkinnedModelRender, &mut Visibility, Option<&SpecCharIndex>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut spawn_car: MessageWriter<SpawnCarRender>,
    mut despawn_car: MessageWriter<DespawnCarRender>,
    mut spawn_char: MessageWriter<SpawnSkinnedModel>,
    mut despawn_char: MessageWriter<DespawnSkinnedModel>,
    mut char_anim: MessageWriter<SkinnedModelAnim>,
    mut char_blend: MessageWriter<SkinnedModelBlend>,
    mut particles: MessageWriter<visos_render::stateless_particles::LoadParticles>,
) {
    let fe = state.frontend.as_ref().filter(|_| state.active).map(|f| &f.fe);
    let Some(fe) = fe else {
        // frontend gone: drop the scene, the cars, the characters and the camera
        for r in view.backgrounds.iter_mut().filter_map(Option::take) {
            commands.entity(r).despawn();
        }
        view.bg_prims = Default::default();
        view.swapped.clear();
        view.originals.clear();
        if let Some(c) = view.camera.take() {
            commands.entity(c).despawn();
        }
        for (slot, _) in view.cars.drain() {
            despawn_car.write(DespawnCarRender { request: FOYER_CAR_REQUEST + slot as u64 });
        }
        for (request, _) in view.chars.drain() {
            despawn_char.write(DespawnSkinnedModel { request });
        }
        view.blending.clear();
        for (_, (_, e)) in view.props.drain() {
            if let Some(e) = e {
                commands.entity(e).despawn();
            }
        }
        view.loaded = None;
        return;
    };
    let foyer = &fe.foyer;
    // the camera clears the frame for the frontend even without a scene (legal / movie screens)
    if view.camera.is_none() {
        let c = commands
            .spawn((
                Camera3d::default(),
                Camera { order: 0, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() },
                Projection::Perspective(PerspectiveProjection {
                    fov: crate::feck2::screens::common::fe_3d_camera::FOV_DEFAULT.to_radians(),
                    near: crate::feck2::screens::common::fe_3d_camera::NEAR,
                    far: crate::feck2::screens::common::fe_3d_camera::FAR,
                    aspect_ratio: 4.0 / 3.0,
                    ..default()
                }),
                Transform::default(),
                FoyerCamera,
                // FECK2ElementCamera's view: graphics draws the front-end flares / floor reflection around it
                visos_render::rain_surface::FrontEndCamera,
            ))
            .id();
        view.camera = Some(c);
    }
    // FE3DCamera backgrounds (0x0026F3D0): shown and hidden by `SetBackgroundVisible` (0x00273620); `SetFoyer`
    // (0x00272070) only reloads the lights (FECK2Element3D::LoadFoyerLights 0x00298CD0)
    for i in 0..BACKGROUNDS.len() {
        let show = foyer.backgrounds[i];
        if show && view.backgrounds[i].is_none() {
            let mut b = Builder::new(&mut meshes, &mut materials, &mut images);
            let built = build_background(&mut commands, &mut b, i);
            view.backgrounds[i] = built.as_ref().map(|b| b.0);
            view.bg_prims[i] = built.map(|b| b.1).unwrap_or_default();
            view.swapped.retain(|k, _| k.0 != i);
            if i == 0 {
                // the hangout's placed particles (`\Foyers\hangout\particle.pld`, graphics' StatelessParticles)
                particles.write(visos_render::stateless_particles::LoadParticles { area: "\\Foyers\\hangout\\".into(), sub: None });
            }
        }
        if let Some(r) = view.backgrounds[i] {
            commands.entity(r).insert(if show { Visibility::Inherited } else { Visibility::Hidden });
        }
    }
    if let Some(name) = &foyer.name {
        if view.loaded.as_ref() != Some(name) {
            let text = std::fs::read_to_string(visos::file::resolve(&format!(r"\Foyers\{}\Lights.txt", name))).ok();
            // no lights.txt: VsRender's start-up set 0x000583F0 stays (LoadFoyerLights 0x00298CD0)
            let lights = text
                .as_deref()
                .and_then(visos_render::vs_render::VsLights::parse_lights_txt)
                .unwrap_or_else(visos_render::vs_render::VsLights::vs_render_defaults);
            // LoadFoyerLights 0x00298CD0: the REFLECTION block -> RainSurface +68.. (off when absent)
            commands.insert_resource(visos_render::rain_surface::FeReflection::from_lights_txt(text.as_deref().unwrap_or("")));
            let points = std::fs::read_to_string(visos::file::resolve(&format!(r"\Foyers\{}\PtLights.txt", name)))
                .map(|t| visos_render::vs_render::parse_ptlights_txt(&t))
                .unwrap_or_default();
            commands.insert_resource(visos_render::track_render::TrackLights { lights, fog: Vec4::ZERO });
            commands.insert_resource(visos_render::track_render::TrackPointLights { points });
            view.loaded = Some(name.clone());
        }
    }
    // FE3DVehicles: one car per shown spot of the loaded vehicle file (its foyer need not be the lights' one: the
    // garage reads "garage"'s spots under "modArea" lights)
    let mut want: HashMap<usize, (CarKey, CarSpec, Transform)> = HashMap::new();
    let vs = &fe.foyer_vehicles;
    if let Some(db) = fe.vehicles.as_ref() {
        for (i, v) in vs.slots.iter().enumerate() {
            let Some(setup) = v.setup.as_ref().filter(|_| v.visible) else { continue };
            let Some(vi) = setup.vehicle else { continue };
            let c = &setup.colours[0];
            let parts = setup_parts(db, setup);
            // FE3DVehicles 0x0025C890 -> CarRender 0x001100B0: shadow size, neon and +1816 wheel height from the setup
            let mut spec = CarSpec::new(car_dir(&db.vehicles[vi].path), Vec3::ONE)
                .with_colour(&if c.hash == 0 {
                    // mix-your-own (FE3DVehicles 0x0025E170: hash 0, kind 0): the packed pair of 0x00187BA0
                    CarColour { kind: c.unk_0c, colour_a: c.unk_34, colour_b: c.unk_38 }
                } else {
                    CarColour::from_record(c.unk_0c, c.colour_a, c.colour_b)
                })
                .with_setup_underbody(setup, db);
            // FEAlloyEditor::LoadWheelModel 0x0025A5A0: the setup wheel's model stem (WHEELS.BIN +0x1C, front)
            spec.wheel = setup.wheel.and_then(|w| db.wheels.get(w)).and_then(|w| w.string_1c.clone()).filter(|s| !s.is_empty());
            // FE3DVehicles 0x0025D110: the setup tyre's TyresT.vcd stem (Tyre.name +0x14) on the 4 wheels
            spec.tyre = setup.tyre.and_then(|t| db.tyres.get(t)).and_then(|t| t.name.clone()).filter(|s| !s.is_empty());
            // CarDecals 0x00116BC0 names from the setup's decal slots
            spec.decals = visos_render::car_render::CarDecals::from_setup(setup, db);
            // the mix-your-own colour changes the rgb under one colour hash
            let rgb = if c.hash == 0 { [c.unk_34, c.unk_38] } else { [0; 2] };
            let under = (spec.wheel_height.map(f32::to_bits), spec.neon, spec.brakes.clone(), spec.wheel.clone(), spec.tyre.clone(), spec.decals.clone(), rgb);
            // the spot without the wheel change bounce (it moves the root every frame, `foyer_car_poses`)
            let spot = [v.pos[0], v.ground_y + v.lift, v.pos[2]];
            let k: CarKey = (vi, c.hash, c.brand, spot.map(f32::to_bits), parts, under);
            spec.parts = parts;
            spec.frontend = true;
            let t = Transform::from_translation(Vec3::from_array(v.pos)).with_rotation(Quat::from_rotation_y(v.rotation.to_radians()));
            want.insert(i, (k, spec, t));
        }
    }
    let stale: Vec<usize> = view.cars.iter().filter(|(s, k)| want.get(s).is_none_or(|w| w.0 != **k)).map(|(s, _)| *s).collect();
    for s in stale {
        view.cars.remove(&s);
        despawn_car.write(DespawnCarRender { request: FOYER_CAR_REQUEST + s as u64 });
    }
    for (s, (k, spec, t)) in want {
        if !view.cars.contains_key(&s) {
            spawn_car.write(SpawnCarRender { request: FOYER_CAR_REQUEST + s as u64, spec, transform: t });
            view.cars.insert(s, k);
        }
    }

    // FE3DCharacters: the character elements of the foyer on screen
    let cs = &fe.characters;
    let mut live = std::collections::HashSet::new();
    // character elements belong to the hangout background, whichever lights are loaded
    if cs.created {
        for (g, i, c) in cs.all() {
            if !c.created {
                continue;
            }
            let request = FOYER_CHAR_REQUEST + g as u64 * 256 + i as u64;
            live.insert(request);
            let anim = c.current_anim_name().map(str::to_string);
            match view.chars.get(&request) {
                None => {
                    // element SetPosition (0x00285820): position + rotation about y
                    let t = Transform::from_translation(Vec3::from_array(c.pos)).with_rotation(Quat::from_rotation_y(c.rotation));
                    spawn_char.write(SpawnSkinnedModel { request, pack: CharPack::Chars, model: c.model.clone(), anim: anim.clone(), transform: t });
                    view.chars.insert(request, anim);
                }
                Some(shown) => {
                    let change = if *shown != anim { anim.clone() } else { None };
                    char_anim.write(SkinnedModelAnim { request, anim: change, time: Some(c.time) });
                    view.chars.insert(request, anim);
                }
            }
            // 0x00285350 -> Characters 0x00079EC0: the previous anim under the current one while the 1 s fade runs,
            // then the single layer again (sent once)
            let (prev, weight) = c.blend();
            if prev.is_some() || view.blending.remove(&request) {
                if prev.is_some() {
                    view.blending.insert(request);
                }
                char_blend.write(SkinnedModelBlend { request, prev, weight });
            }
            for (e, m, mut vis, spec) in models.iter_mut() {
                if m.request == request {
                    // 0x00285350 passes element +764 (record +284) to Characters 0x00079EC0 as the specular record
                    let n = SpecCharIndex(c.index as u8);
                    if spec.is_none_or(|s| s.0 != n.0) {
                        commands.entity(e).insert(n);
                    }
                    let v = if c.visible { Visibility::Inherited } else { Visibility::Hidden };
                    if *vis != v {
                        *vis = v;
                    }
                }
            }
        }
    }
    let gone: Vec<u64> = view.chars.keys().filter(|r| !live.contains(r)).copied().collect();
    for request in gone {
        view.chars.remove(&request);
        view.blending.remove(&request);
        despawn_char.write(DespawnSkinnedModel { request });
    }
    // the chars' prop elements: built once per element (a moved / renamed one is rebuilt), shown as the element says
    let want_props: HashMap<usize, &FEProp> = cs.props.iter().enumerate().filter_map(|(i, p)| Some((i, p.as_ref()?))).collect();
    let stale: Vec<usize> = view
        .props
        .iter()
        .filter(|(i, (built, _))| want_props.get(i).is_none_or(|p| p.model != built.model || p.pos != built.pos || p.rotation != built.rotation))
        .map(|(i, _)| *i)
        .collect();
    for i in stale {
        if let Some((_, Some(e))) = view.props.remove(&i) {
            commands.entity(e).despawn();
        }
    }
    for (i, p) in want_props {
        if !view.props.contains_key(&i) {
            let mut b = Builder::new(&mut meshes, &mut materials, &mut images);
            let e = build_prop(&mut commands, &mut b, p);
            view.props.insert(i, (p.clone(), e));
        }
        if let Some((_, Some(e))) = view.props.get(&i) {
            commands.entity(*e).insert(if p.visible { Visibility::Inherited } else { Visibility::Hidden });
        }
    }

    if let Some(c) = view.camera {
        if let Ok((mut t, mut p)) = cams.get_mut(c) {            let (eye, at, up) = foyer.eye();
            *t = Transform::from_translation(Vec3::from_array(eye)).looking_at(Vec3::from_array(at), Vec3::from_array(up));
            if let Projection::Perspective(pp) = &mut *p {
                let fov = foyer.fov.to_radians();
                if pp.fov != fov {
                    pp.fov = fov;
                }
            }
        }
    }
}

/// dev check of the background (`VELOCITY_FOYER_DEBUG=1` logs the background meshes once 90 frames after they
/// appear: count, view-visible, world bounds vs the camera; `=2` also draws them full bright: stage colour from a
/// white TFACTOR instead of the vertex diffuse, i.e. the texture alone)
#[allow(clippy::type_complexity)]
pub fn foyer_debug(
    view: Res<FoyerView>,
    q: Query<(&ChildOf, &MeshMaterial3d<XboxStageMaterial>, &ViewVisibility, Option<&bevy::camera::primitives::Aabb>, &GlobalTransform)>,
    cams: Query<&GlobalTransform, With<FoyerCamera>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut state: Local<(Option<u32>, u32, std::collections::HashSet<AssetId<XboxStageMaterial>>)>,
) {
    let mode = *state.0.get_or_insert_with(|| std::env::var("VELOCITY_FOYER_DEBUG").ok().and_then(|v| v.parse().ok()).unwrap_or(0));
    if mode == 0 {
        return;
    }
    let roots: Vec<Entity> = view.backgrounds.iter().flatten().copied().collect();
    let mine = || q.iter().filter(|(p, ..)| roots.contains(&p.parent()));
    if mode >= 2 {
        for (_, m, ..) in mine() {
            if state.2.insert(m.id()) {
                if let Some(mut mat) = materials.get_mut(&m.0) {
                    mat.params.arg.x = 0.0;
                    mat.params.colour = Vec4::ONE;
                }
            }
        }
    }
    if roots.is_empty() || state.1 > 90 {
        return;
    }
    state.1 += 1;
    if state.1 == 90 {
        let (mut n, mut vis, mut lo, mut hi) = (0, 0, Vec3::MAX, Vec3::MIN);
        for (_, _, v, aabb, gt) in mine() {
            n += 1;
            vis += v.get() as usize;
            if let Some(a) = aabb {
                // world box corners (axis aligned transforms only)
                for s in [-1.0, 1.0] {
                    let c = gt.transform_point(Vec3::from(a.center) + s * Vec3::from(a.half_extents));
                    lo = lo.min(c);
                    hi = hi.max(c);
                }
            }
        }
        let cam = cams.iter().next().map(|g| (g.translation(), g.forward()));
        log::info!("foyer debug: {n} background meshes, {vis} view-visible, world bounds {lo:?}..{hi:?}, camera {cam:?}");
    }
}

/// the foyer camera's MTV overlay calls (FE3DCamera 0x00275E50 / 0x00275F10, the car select screens) onto graphics' `MtvEffect`
pub fn foyer_mtv(mut state: ResMut<FrontendState>, mtv: Option<ResMut<visos_render::mtv_effect::MtvEffect>>) {
    let Some(f) = state.frontend.as_mut() else { return };
    let calls = std::mem::take(&mut f.fe.foyer.mtv);
    let Some(mut mtv) = mtv else { return };
    for c in calls {
        use crate::feck2::screens::common::fe_3d_camera::MtvCall;
        match c {
            MtvCall::Start { seconds, no_glitch } => mtv.start(seconds, no_glitch),
            MtvCall::Stop { immediate } => mtv.stop(immediate),
        }
    }
}
