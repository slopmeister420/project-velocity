//! Render\CarRender.cpp / Render\CarPalette.cpp — player and opponent cars: chassis model plus the selected body
//! parts from `\Vehicles\<city>\<car>\VEHICLEM.VCD` (high LODs in `VehicleS.vcd`), textures from `VEHICLET.VCD`,
//! `\Vehicles\COMMONT.VCD` and `\vehicles\wheels\wheels_t.vcd`, wheels from `\vehicles\wheels\wheels_m.vcd`.
//! Painted shaders use the car paint program (`CarShaderManager`), the rest the fixed-function stage.
//!
//! API: write `SpawnCarRender`; the spawned root carries `CarRender { request }` and `CarRenderParts`. Drive it by
//! setting the root `Transform` (chassis pose, game space) and each wheel entity's `Transform` (relative to the
//! root) every frame — e.g. from the physics wheel render matrices. `DespawnCarRender` or despawning the root
//! removes it.

use bevy::prelude::*;

use crate::car_palette_pc_xbox::{
    car_decal_inputs, envmap_matrix, paint_colours, paint_params, pearlescent_matrix, shiny_car_inputs, CarPaintState, CarSetTextures,
    CarShaderSet,
};
use crate::global_mat_input_resolver::{gmi, GlobalMatInputResolver};
use crate::model_builder::{Builder, CarProgram, CarProgramCtx, PrimRecord, SpawnOpts, TexSet, STAGE_MODE_REVLIGHT};
use crate::renderer::renderer_types::Model;
use crate::renderer::shader_program_manager::{ObjectInputs, ProgramMaterial, ProgramMaterialPlugin};
use crate::renderer::shader_manager::{StageState, XboxStageMaterial, XboxStageParams};
use crate::renderer::texture_manager::decode_any_vtf;

/// body part slots in load order; index 0 of each is the stock part (`scoop` has no 0: none fitted).
pub const PART_SLOTS: [&str; 7] = ["bonnet", "f_bumper", "r_bumper", "exhaust", "spoiler", "trim", "scoop"];

/// what to build.
#[derive(Clone, Debug)]
pub struct CarSpec {
    /// directory under `\Vehicles`, e.g. "LA/350z"
    pub dir: String,
    /// part index per `PART_SLOTS` entry (None = not fitted)
    pub parts: [Option<u32>; 7],
    /// paint colour and pearlescent colour (CarPalette c10 / c11), gamma space 0..1
    pub colour: Vec3,
    pub pearl: Vec3,
    /// wheel model stem in wheels_m.vcd (default `<car>_wheel`)
    pub wheel: Option<String>,
    /// paint type (car data +1780): 0 solid, 1/2 pearlescent mixes, 3 pearl colour as given
    pub paint_type: u32,
    /// built for the front end (CarShaderManager +12) / shown in the showroom (CarRender +49)
    pub frontend: bool,
    /// a traffic car: `dir` is the TRAFFIC.BIN model name, everything comes from `\Vehicles\TrafficM.vcd` /
    /// `TrafficT.vcd` (VehicleEntity.cpp 0x001C0D60, no-setup branch)
    pub traffic: bool,
    /// the setup's decals (CarRender.cpp 0x00116510); empty slots get "bonnet_off"
    pub decals: CarDecals,
    /// draw the licence plate (LicencePlateRender.cpp 0x0012E1E0: car 0's rear bumper); request 0 always does
    pub licence_plate: bool,
    /// car data +1808 / +1812: shadow width / length (vehicle data +0x254 / +0x258, see `with_setup_underbody`)
    pub shadow: [f32; 2],
    /// car data +2232 neon colour (D3DCOLOR) when +2328 bit 1 (neon fitted) is set
    pub neon: Option<u32>,
    /// car data +1816: the front-end shadow height (quad at y − h/2), the setup wheel's WHEELS.BIN +0x0C
    /// (`with_setup_underbody`; FE3DVehicles 0x0025C890 → CarRender 0x001100B0)
    pub wheel_height: Option<f32>,
    /// car data +2328 bit 0x4000: a police car (CarRender.cpp 0x00110220: car index past the race + traffic cars);
    /// the first one is what `DummyPoliceCars` draw (0x00119250)
    pub police: bool,
    /// the setup's brake disc / pad part model names (BrakePart +0x10, wheels_m.vcd), loaded by the front end
    /// (FEAlloyEditor 0x0025A7F0 -> FE3DVehicles +2484 / +2488)
    pub brakes: [Option<String>; 2],
    /// the setup's tyre texture (Tyre.name +0x14, `\Vehicles\TyresT.vcd`), applied to the "_tyre" shader of all 4
    /// wheels (FE3DVehicles 0x0025D110 -> CarRender 0x00118910 -> 0x00118850)
    pub tyre: Option<String>,
}

impl CarSpec {
    pub fn new(dir: impl Into<String>, colour: Vec3) -> Self {
        Self {
            dir: dir.into(),
            parts: [Some(0), Some(0), Some(0), Some(0), Some(0), Some(0), None],
            colour,
            pearl: colour,
            wheel: None,
            paint_type: 0,
            frontend: false,
            traffic: false,
            decals: CarDecals::default(),
            licence_plate: false,
            shadow: SHADOW_SIZE_DEFAULT,
            neon: None,
            wheel_height: None,
            police: false,
            brakes: [None, None],
            tyre: None,
        }
    }

    /// CarRender.cpp 0x00110220 (+1808 / +1812 = the vehicle data's +0x254 / +0x258 when the car has a database
    /// entry, else 2.9 / 4.8) and 0x00115920 (neon = the setup's colour slot 1 at +0xC8: fitted when its brand
    /// +0x04 is set and its id +0x00 is not crc("OFF"); +2232 = the slot's +0x38 colour)
    pub fn with_setup_underbody(
        mut self,
        setup: &velocity_physics::vehicle_database::VehicleSetup,
        db: &velocity_physics::vehicle_database::CVehicleDatabase,
    ) -> Self {
        self.shadow = setup.vehicle.and_then(|v| db.vehicles.get(v)).map_or(SHADOW_SIZE_DEFAULT, |v| v.data.unk_254);
        let c = &setup.colours[1];
        self.neon = (c.brand != 0 && c.hash != NEON_OFF_CRC)
            .then(|| crate::car_palette::CarColour::from_record(c.unk_0c, c.colour_a, c.colour_b).colour_b);
        self.wheel_height = setup.wheel.and_then(|w| db.wheels.get(w)).map(|w| w.unk_0c);
        self.brakes = setup.brake_parts.map(|b| b.and_then(|b| db.brake_parts.get(b)).and_then(|p| p.name.clone()));
        self
    }

    /// a traffic car (`name` = TRAFFIC.BIN model, e.g. "car01"): body `<name>`, all four wheels `<name>_wheel`
    pub fn traffic(name: impl Into<String>, colour: Vec3) -> Self {
        let mut s = Self::new(name, colour);
        s.traffic = true;
        s.parts = [None; 7];
        s
    }

    /// the setup's body colour record (`SVehicleSetup` +0x88): paint, pearl and paint type as the original sets them
    pub fn with_colour(mut self, c: &crate::car_palette::CarColour) -> Self {
        let (paint, pearl, ty) = c.apply();
        self.colour = paint;
        self.pearl = pearl;
        self.paint_type = ty;
        self
    }

    pub fn car_name(&self) -> &str {
        self.dir.rsplit(['/', '\\']).next().unwrap_or(&self.dir)
    }
}

#[derive(Message, Clone, Debug)]
pub struct SpawnCarRender {
    /// caller key, copied to `CarRender::request`
    pub request: u64,
    pub spec: CarSpec,
    pub transform: Transform,
}

#[derive(Message, Clone, Debug)]
pub struct DespawnCarRender {
    pub request: u64,
}

#[derive(Component, Clone, Debug)]
pub struct CarRender {
    pub request: u64,
    pub spec: CarSpec,
}

/// entities of a spawned car. wheel order: front left, front right, rear left, rear right (initial placement
/// estimated from the body bounds; physics overwrites it).
#[derive(Component, Clone, Debug)]
pub struct CarRenderParts {
    pub body: Entity,
    pub wheels: [Entity; 4],
    /// body bounds in car space (min, max)
    pub bounds: (Vec3, Vec3),
    pub wheel_radius: f32,
    /// program materials of the ShinyCar / CarDecal sets (inputs rebound every frame)
    pub programs: Vec<CarProgram>,
    /// point light slots kept between frames (VsRender.cpp 0x00059140)
    pub light_slots: [i16; 2],
    /// front-end door / boot part entities (children of the root, vertices in car space), FE3DVehicles 0x0025BF80
    /// order `FE_DOOR_NAMES`; `None` when the car has no such model (in-game cars have none: doors are in the body)
    pub doors: [Option<Entity>; 8],
    /// each part's locator in car space: hinge (position, z negated like 0x0010F330's l_head) and the open pose
    pub door_locators: [Option<(Vec3, Quat)>; 8],
    /// the four wheel entities in FE3DVehicles order: 0 f_l, 1 r_l, 2 f_r, 3 r_r (`wheels` is f_l, f_r, r_l, r_r)
    pub fe_wheels: [Entity; 4],
    /// car-space wheel pivot per `fe_wheels` slot (locator `f_l_wheel` / `r_l_wheel`, z negated; right wheels x
    /// mirrored; no locator: the estimated placement). Frontend cars have the wheel `Transform` set to
    /// `T(pivot) * wheel_base_rot[k]`; steer = `T(pivot) * RotY(∓steer) * wheel_base_rot[k]`
    pub wheel_pivots: [Vec3; 4],
    pub wheel_base_rot: [Quat; 4],
    /// brake disc + pad entities per `fe_wheels` slot (FE3DVehicles 0x0025BA40; frontend cars with `CarSpec::brakes`),
    /// children of the root at `T(disc_pivots[k])` (right side also scale -1 in x); steer = `T(pivot) * RotY(∓steer)
    /// * base scale`, like the wheels
    pub discs: [Option<Entity>; 4],
    pub disc_pivots: [Vec3; 4],
    /// CarRender 0x00116BC0 per decal kind 0 chassis, 1 f_bumper, 2 r_bumper, 3 trim, 4 front screen, 5 rear screen:
    /// how many logo slots the car's models have (0 for traffic cars)
    pub decal_slots: [u8; 6],
}

/// FE3DVehicles 0x0025BF80 part order (GameDataLoading locator names / `<car>_<name>` models)
pub const FE_DOOR_NAMES: [&str; 8] = ["f_l_door", "f_r_door", "r_l_door", "r_r_door", "m_boot", "b_boot", "bonnet", "t_boot"];

/// a locator matrix in car space: position with z negated, rotation = the 3x3 conjugated by the z flip
fn door_locator(m: &[f32; 16]) -> (Vec3, Quat) {
    let c = |r: usize| Vec3::new(m[4 * r], m[4 * r + 1], m[4 * r + 2]);
    let (x, y, z) = (c(0).normalize_or_zero(), c(1).normalize_or_zero(), c(2).normalize_or_zero());
    let s = Mat3::from_diagonal(Vec3::new(1.0, 1.0, -1.0));
    let q = Quat::from_mat3(&(s * Mat3::from_cols(x, y, z) * s));
    (Vec3::new(m[12], m[13], -m[14]), q)
}

/// pivots for `CarRenderParts::fe_wheels` (see there); in the front end also places the wheels on them
fn fe_wheel_pivots(commands: &mut Commands, wheels: &[Entity; 4], locs: [Option<[f32; 16]>; 2], frontend: bool) -> ([Entity; 4], [Vec3; 4], [Quat; 4]) {
    // slot k -> index into `wheels` (f_l, f_r, r_l, r_r)
    let idx = [0usize, 2, 1, 3];
    // FE3DVehicles 0x0025B4D0: the f_l/r_l locators sit at +x (the car's left); left wheels use the model as is,
    // right wheels (a4 2, 3) a -1 x mirror matrix (applied to the wheel's LOD groups in `spawn_car`)
    let left_rot = Quat::IDENTITY;
    let mut pivots = [Vec3::ZERO; 4];
    let mut rots = [Quat::IDENTITY; 4];
    for k in 0..4 {
        rots[k] = if k < 2 { left_rot } else { Quat::IDENTITY };
        if let Some(l) = locs[k % 2] {
            let p = Vec3::new(l[12], l[13], -l[14]);
            pivots[k] = if k < 2 { p } else { Vec3::new(-p.x, p.y, p.z) };
            if frontend {
                commands.entity(wheels[idx[k]]).insert(Transform::from_translation(pivots[k]).with_rotation(rots[k]));
            }
        }
    }
    (idx.map(|i| wheels[i]), pivots, rots)
}

/// FE3DVehicles 0x0025BA40 + 0x0025C890: the brake disc and pad models of the setup (`spec.brakes`, `<name>_high.xmf`
/// of wheels_m.vcd) drawn at each wheel with one matrix (CarRender 0x00110140): position = the car's `f_l_wheel` /
/// `r_l_wheel` locator + the front wheel model's "disk" locator, z negated; the right side mirrored (x negated and
/// scale -1 in x). Front end only. Returns the disc entities (FE slot order) and their pivots.
fn spawn_brake_discs(
    commands: &mut Commands,
    b: &mut Builder,
    tex: &mut TexSet,
    wheels_m: Option<&visos::file::Vcd>,
    spec: &CarSpec,
    root: Entity,
    locs: [Option<[f32; 16]>; 2],
    wheel: Option<&Model>,
) -> ([Option<Entity>; 4], [Vec3; 4]) {
    let mut out = ([None; 4], [Vec3::ZERO; 4]);
    let Some(w) = wheels_m.filter(|_| spec.frontend) else { return out };
    let load = |n: &Option<String>| -> Option<Model> {
        let n = n.as_ref()?.to_ascii_lowercase();
        let d = w.get(&format!("{n}_high.xmf")).or_else(|| w.get(&format!("{n}.xmf")))?;
        Model::parse(d).ok()
    };
    let models: Vec<Model> = spec.brakes.iter().filter_map(load).collect();
    if models.is_empty() {
        return out;
    }
    let disk = wheel.and_then(|m| m.locator("disk")).map_or(Vec3::ZERO, |d| Vec3::new(d[12], d[13], d[14]));
    for k in 0..4 {
        let Some(l) = locs[k % 2] else { continue };
        let p = Vec3::new(l[12], l[13], l[14]) + disk;
        let p = Vec3::new(if k < 2 { p.x } else { -p.x }, p.y, -p.z);
        let mut t = Transform::from_translation(p);
        if k >= 2 {
            t.scale = Vec3::new(-1.0, 1.0, 1.0);
        }
        let e = commands.spawn((t, Visibility::default(), ChildOf(root))).id();
        for m in &models {
            b.add_model(commands, tex, m, Mat4::IDENTITY, &SpawnOpts { parent: Some(e), ..SpawnOpts::world() });
        }
        out.0[k] = Some(e);
        out.1[k] = p;
    }
    out
}

/// LOD groups (CarRender data +2249 = `lod`): `lods[l]` holds the chassis + parts of LOD l, `wheel_lods[l]` the
/// four wheel models; one LOD in the front end.
#[derive(Component, Clone, Debug)]
pub struct CarLod {
    pub lods: Vec<Entity>,
    pub wheel_lods: Vec<[Entity; 4]>,
    pub lod: u8,
    /// +2251 set for every view by the camera sitting in this car (CarRender 0x001127F0): nothing of it is drawn
    pub in_car: bool,
}

/// what the car lights need from the game each frame (absent = no vehicle: brake lights held at 0.5, as
/// CarRender.cpp 0x00112A50 does without a vehicle entity). Put it on the `CarRender` root.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CarLightInput {
    /// VehicleEntity +168 > 0 (brake input)
    pub braking: bool,
    /// the displayed gear (VehicleEntity 0x001B74A0) is negative
    pub reversing: bool,
    /// CarRender data +2250 light damage: 1/2 left/right headlight, 4/8 left/right brake light broken
    pub damage: u8,
}

/// the light primitives and their state (CarRender data +1828 brake, +1832 reverse)
#[derive(Component, Clone, Debug)]
pub struct CarLights {
    pub group: Entity,
    pub brake: [Option<Entity>; 2],
    pub head: [Option<Entity>; 2],
    pub reverse: [Option<(Entity, Handle<XboxStageMaterial>)>; 2],
    /// the car's BrakeLight set materials (every `_brake` shader)
    pub brake_materials: Vec<Handle<XboxStageMaterial>>,
    /// `_l2_1` brake materials (flag 0x200)
    pub l2: bool,
    pub brake_level: f32,
    pub reverse_level: f32,
}

/// every car's light set (CarRender data +1028) by `CarRender.request` as left after the cars are drawn: 0x00119EF0's,
/// overwritten by the light pass' 0x001123E0 (`VsLights::car_point_light_set`) unless the view sits in the car; the 3d
/// hud reads the player car's (OSD3dInterface.cpp 0x000EA2B0, `VsLights::hud_light_set`).
#[derive(Resource, Clone, Debug, Default)]
pub struct CarLightSets(pub std::collections::HashMap<u64, crate::vs_render::VsLights>);

pub struct CarRenderPlugin;

impl Plugin for CarRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ProgramMaterialPlugin>() {
            app.add_plugins(ProgramMaterialPlugin);
        }
        if !app.is_plugin_added::<crate::car_damage_render::CarDamageRenderPlugin>() {
            app.add_plugins(crate::car_damage_render::CarDamageRenderPlugin);
        }
        if !app.is_plugin_added::<crate::licence_plate_render::LicencePlatePlugin>() {
            app.add_plugins(crate::licence_plate_render::LicencePlatePlugin);
        }
        app.add_message::<SpawnCarRender>()
            .add_message::<DespawnCarRender>()
            .init_resource::<DriverSway>()
            .init_resource::<WheelBlurTexture>()
            .init_resource::<CarLightSets>()
            .init_resource::<DummyPoliceCars>()
            .init_resource::<DummyPoliceClones>()
            .init_resource::<PendingCarPrograms>()
            .add_systems(PostUpdate, flush_car_programs.after(bevy::camera::visibility::VisibilitySystems::CheckVisibility))
            .add_systems(PostUpdate, draw_frontend_shadows.in_set(crate::effect_draw::EffectOrder::Shadows))
            .add_systems(
                Update,
                (process_car_messages, fe_closest_lights, update_car_lights, latch_wheel_blur, update_wheel_blur, update_car_driver, bind_car_programs, render_dummy_police_cars).chain(),
            );
    }
}

/// `byte_6865E4` (FeLightsAddTweaks tweak, .data 1): CarRender 0x00118D70 runs ClosestLights
pub const FE_CLOSEST_LIGHTS: bool = true;

/// what `fe_closest_lights` remembers between frames
#[derive(Default)]
pub(crate) struct FeClosestState {
    /// live front-end cars in add order (`order[0]` = car 0, CarRender +304)
    order: Vec<Entity>,
    /// car 0's last position (its record outlives the car)
    car0: Vec3,
    /// the set last written, to tell a foyer lights reload from our own write
    written: Option<crate::vs_render::VsLights>,
}

fn same_lights(a: &crate::vs_render::VsLights, b: &crate::vs_render::VsLights) -> bool {
    a.dir == b.dir && a.colour == b.colour && a.ambient == b.ambient
}

/// CarRender.cpp 0x00118D70 — `VsRender::ClosestLights` (0x0005A710) on car 0's position (row 3 of the matrix at
/// CarRender +304) when gated by `byte_6865E4`. Front end only: called by AddCar 0x0010F330 when the first car is added
/// (m_nCars becomes 1; AddCar's only caller is FE3DVehicles 0x0025B310) and by FECK2Element3D::LoadFoyerLights
/// 0x00298CD0 right after the foyer's lights.txt / PtLights.txt (here: `TrackLights` replaced while a
/// `FrontEndCamera` exists). The FE removes cars one by one, the original only on a full clear: the first-car call
/// fires again whenever the front-end car count goes from 0 to more.
pub(crate) fn fe_closest_lights(
    fe_cam: Query<(), With<crate::rain_surface::FrontEndCamera>>,
    cars: Query<(Entity, &CarRender, &Transform)>,
    lights: Option<ResMut<crate::track_render::TrackLights>>,
    points: Option<Res<crate::track_render::TrackPointLights>>,
    mut st: Local<FeClosestState>,
) {
    if fe_cam.is_empty() {
        st.order.clear();
        st.written = None;
        return;
    }
    st.order.retain(|e| cars.get(*e).is_ok_and(|(_, c, _)| c.spec.frontend));
    let was_empty = st.order.is_empty();
    let mut new: Vec<(u64, Entity)> =
        cars.iter().filter(|(e, c, _)| c.spec.frontend && !st.order.contains(e)).map(|(e, c, _)| (c.request, e)).collect();
    new.sort_by_key(|n| n.0);
    st.order.extend(new.into_iter().map(|n| n.1));
    if let Some((_, _, t)) = st.order.first().and_then(|e| cars.get(*e).ok()) {
        st.car0 = t.translation;
    }
    let Some(mut lights) = lights else { return };
    let reloaded = lights.is_changed() && st.written.as_ref().is_none_or(|w| !same_lights(w, &lights.lights));
    let first_car = was_empty && !st.order.is_empty();
    if FE_CLOSEST_LIGHTS && (reloaded || first_car) {
        let pts = points.as_ref().map_or(&[][..], |p| &p.points[..]);
        let pos = st.car0;
        lights.lights.closest_lights(pts, pos);
        st.written = Some(lights.lights);
    }
}

/// CarRender.cpp 0x00118910 / 0x00118850 — FindShader("_tyre") on each wheel, texture swapped for the tyre's
/// `TyresT.vcd` one: the shader's stage 0 texture name is rebound to it (the set is per car)
fn apply_tyre_texture(tex: &mut TexSet, tyre: &str, wms: &[Option<Model>]) {
    let Ok(v) = visos::file::Vcd::open("\\Vehicles\\TyresT.vcd") else { return };
    let want = tyre.to_ascii_lowercase();
    let want = want.trim_end_matches(".vtf");
    let Some(t) = v
        .entries()
        .find(|e| e.name.to_ascii_lowercase().trim_end_matches(".vtf") == want)
        .and_then(|e| v.data(&e).and_then(|d| decode_any_vtf(d).ok()))
    else {
        return;
    };
    for m in wms.iter().flatten() {
        for s in m.shaders.iter().filter(|s| s.name.contains("_tyre")) {
            if let Some(st) = s.stages.first() {
                tex.add(&st.texture_name, t.clone());
            }
        }
    }
}

pub(crate) fn add_vcd_textures(tex: &mut TexSet, path: &str) {
    if let Ok(v) = visos::file::Vcd::open(path) {
        for e in v.entries() {
            if !e.name.to_ascii_lowercase().ends_with(".vtf") {
                continue;
            }
            if let Some(t) = v.data(&e).and_then(|d| decode_any_vtf(d).ok()) {
                tex.add(e.name, t);
            }
        }
    }
}

/// VehicleEntity.cpp 0x001C08B0 / 0x001C0D60 — model LODs `%s`, `%s_med`, `%s_low` (chassis `%s_base…`), wheels
/// `%s_wheel`, `_Med`, `_Low` (CarRender data +1092.. / +1104 + 16·lod)
const LOD_SUFFIX: [&str; 3] = ["", "_med", "_low"];
const WHEEL_LOD_SUFFIX: [&str; 3] = ["", "_Med", "_Low"];

/// CarRender.cpp 0x0010F330 — `AddCar` merged into the build: one car under a new root entity (locators
/// `l_head` / `l_brake` z negated, "missing headlight" warning, light lookups 0x00115110 / 0x001152D0). In game the three LODs of VEHICLEM.VCD are built and
/// switched per frame (`update_car_lights`); the front end (`spec.frontend`) uses the `_high` models of VehicleS.vcd.
pub fn spawn_car(commands: &mut Commands, b: &mut Builder, spec: &CarSpec, request: u64, transform: Transform) -> Entity {
    if spec.traffic {
        return spawn_traffic_car(commands, b, spec, request, transform);
    }
    let base = format!("\\Vehicles\\{}", spec.dir.replace('/', "\\"));
    let name = spec.car_name().to_string();
    let mut tex = TexSet::default();
    add_vcd_textures(&mut tex, "\\Vehicles\\COMMONT.VCD");
    add_vcd_textures(&mut tex, "\\vehicles\\wheels\\wheels_t.vcd");
    // FEAlloyEditor_Basic 0x002590A0: the front end loads the interior pack first while +15694 is set (1 from
    // 0x00258770 / 0x00259580; only the customisation view cycle 0x002527A0 clears it, not ported)
    if spec.frontend {
        add_vcd_textures(&mut tex, &format!("{base}\\VEHICLEI.VCD"));
    }
    add_vcd_textures(&mut tex, &format!("{base}\\VEHICLET.VCD"));
    let models = visos::file::Vcd::open(&format!("{base}\\VEHICLEM.VCD")).ok();
    let high = if spec.frontend { visos::file::Vcd::open(&format!("{base}\\VehicleS.vcd")).ok() } else { None };
    let nlods = if spec.frontend { 1 } else { 3 };
    // the front end's high LOD if present, else the model of that LOD, else the next better LOD
    let get = |stem: &str, lod: usize| -> Option<Vec<u8>> {
        if let Some(d) = high.as_ref().and_then(|h| h.get(&format!("{stem}_high.xmf")).or_else(|| h.get(&format!("{stem}.xmf")))) {
            return Some(d.to_vec());
        }
        (0..=lod).rev().find_map(|l| models.as_ref().and_then(|m| m.get(&format!("{stem}{}.xmf", LOD_SUFFIX[l]))).map(<[u8]>::to_vec))
    };
    let root = commands.spawn((transform, Visibility::default())).id();
    let body = commands.spawn((Transform::default(), Visibility::default(), ChildOf(root))).id();
    let mut stems = vec![format!("{name}_base")];
    for (slot, idx) in PART_SLOTS.iter().zip(spec.parts) {
        if let Some(i) = idx {
            stems.push(format!("{slot}{i}"));
        }
    }
    // the high LOD bodies are open at the doors; their doors (and boots) are separate models in the same pack, each
    // its own child entity of the root (FE3DVehicles 0x0025BF80 moves them; see `CarRenderParts::doors`)
    let open_body = high.as_ref().is_some_and(|h| h.get(&format!("{name}_base_high.xmf")).is_some());
    let mut doors: [Option<Entity>; 8] = [None; 8];
    let mut door_locs: [Option<[f32; 16]>; 8] = [None; 8];
    let mut wheel_locs: [Option<[f32; 16]>; 2] = [None; 2];
    let mut door_ranges: [std::ops::Range<usize>; 8] = Default::default();
    let mut lods = Vec::new();
    let (mut lo, mut hi) = (Vec3::ZERO, Vec3::ZERO);
    let mut records = Vec::new();
    let mut chassis_records = 0;
    // CarDamage works on the in-game LOD 0 chassis + parts (CarRender.cpp 0x00110220 -> 0x0006AE70)
    let damage = !spec.frontend;
    let (mut ranges, mut other_records, mut collect) = (Vec::new(), Vec::new(), None);
    let (mut head, mut brake) = ([0f32; 3], [0f32; 3]);
    let mut driver_pos = None;
    let mut beams = None;
    let mut flare_locs = crate::flare_render::CarFlareLocators::default();
    // per LOD: every record and each stem's range (the decal and plate lookups work per model)
    let (mut lod_recs, mut stem_ranges) = (Vec::new(), Vec::new());
    for lod in 0..nlods {
        let mut sr = Vec::new();
        let group = commands
            .spawn((Transform::default(), if lod == 0 { Visibility::Inherited } else { Visibility::Hidden }, ChildOf(body)))
            .id();
        lods.push(group);
        let opts = SpawnOpts { parent: Some(group), ..SpawnOpts::world() };
        b.bounds_min = Vec3::splat(f32::MAX);
        b.bounds_max = Vec3::splat(f32::MIN);
        b.records = Some(Vec::new());
        if lod == 0 && damage {
            b.damage = Some(Default::default());
        }
        b.keep_mesh_data = !spec.frontend;
        for (k, stem) in stems.iter().enumerate() {
            let start = b.records.as_ref().map_or(0, Vec::len);
            match get(stem, lod).as_deref().map(Model::parse) {
                Some(Ok(m)) => {
                    b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &opts);
                    // car data +768 / +896: the chassis' l_head / l_brake translations, z negated
                    if lod == 0 && k == 0 {
                        driver_pos = driver_locator(&m);
                        let row3 = |n: &str| m.locator(n).map(|x| [x[12], x[13], -x[14]]);
                        if let (Some(h), Some(r)) = (row3("l_head"), row3("l_brake")) {
                            (head, brake) = (h, r);
                            beams = Some(crate::track_fx::CarHeadBeams::new(Vec3::from_array(h), Vec4::from_array(m.sphere)));
                        } else {
                            log::warn!("WARNING: Vehicle missing headlight information ({})", spec.dir);
                        }
                        flare_locs = crate::flare_render::CarFlareLocators::from_chassis(&m);
                        for (l, n) in door_locs.iter_mut().zip(FE_DOOR_NAMES) {
                            *l = m.locator(n);
                        }
                        wheel_locs = [m.locator("f_l_wheel"), m.locator("r_l_wheel")];
                    }
                    // 0x00110220: a reverse locator missing on the chassis comes from part 0's model (+1208 +44)
                    if lod == 0 && k == 1 {
                        flare_locs.reverse_from_part(&m);
                    }
                    if lod == 0 && k > 0 && stem.starts_with("exhaust") {
                        flare_locs.nos_from_exhaust(&m);
                    }
                }
                Some(Err(e)) => log::warn!("car {}: {stem}: {e}", spec.dir),
                None => log::debug!("car {}: no {stem}", spec.dir),
            }
            if lod == 0 && k == 0 {
                chassis_records = b.records.as_ref().map_or(0, Vec::len);
            }
            let end = b.records.as_ref().map_or(0, Vec::len);
            if lod == 0 {
                ranges.push(start..end);
            }
            sr.push(start..end);
        }
        if spec.frontend && open_body {
            for (i, n) in FE_DOOR_NAMES.iter().enumerate() {
                let Some(data) = get(&format!("{name}_{n}"), lod) else { continue };
                let Ok(m) = Model::parse(&data) else { continue };
                let start = b.records.as_ref().map_or(0, Vec::len);
                let e = commands.spawn((Transform::default(), Visibility::Inherited, ChildOf(root))).id();
                b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &SpawnOpts { parent: Some(e), ..SpawnOpts::world() });
                doors[i] = Some(e);
                if lod == 0 {
                    door_ranges[i] = start..b.records.as_ref().map_or(0, Vec::len);
                }
            }
        }
        stem_ranges.push(sr);
        lod_recs.push(b.records.clone().unwrap_or_default());
        if lod == 0 {
            (lo, hi) = (b.bounds_min, b.bounds_max);
            records = b.records.take().unwrap_or_default();
            collect = b.damage.take();
        } else {
            other_records.extend(b.records.take().unwrap_or_default());
        }
        b.keep_mesh_data = false;
    }
    // chassis + fitted parts of every LOD (CarRender +1092 / +1208), for the dynamic envmap's normal bend
    let body_meshes = CarBodyMeshes(lod_recs.iter().flatten().map(|r| r.mesh.clone()).collect());
    // wheels
    let wheel_stem = spec.wheel.clone().unwrap_or_else(|| format!("{name}_wheel"));
    let wheels_m = visos::file::Vcd::open("\\vehicles\\wheels\\wheels_m.vcd").ok();
    let wheel_model = |lod: usize| -> Option<Vec<u8>> {
        let w = wheels_m.as_ref()?;
        if spec.frontend {
            return w.get(&format!("{wheel_stem}_high.xmf")).or_else(|| w.get(&format!("{wheel_stem}.xmf"))).map(<[u8]>::to_vec);
        }
        (0..=lod).rev().find_map(|l| w.get(&format!("{wheel_stem}{}.xmf", WHEEL_LOD_SUFFIX[l]))).map(<[u8]>::to_vec)
    };
    let wdata: Vec<Option<Vec<u8>>> = (0..nlods).map(wheel_model).collect();
    let wms: Vec<Option<Model>> = wdata.iter().map(|d| d.as_deref().and_then(|d| Model::parse(d).ok())).collect();
    let radius = wms[0].as_ref().map(|m| m.sphere[3] * std::f32::consts::FRAC_1_SQRT_2).unwrap_or(0.33);
    if let Some(t) = spec.tyre.as_deref() {
        apply_tyre_texture(&mut tex, t, &wms);
    }
    let wrefs: Vec<Option<&Model>> = wms.iter().map(Option::as_ref).collect();
    let (wheels, wheel_lods, blur_prims) = place_wheels(commands, b, &mut tex, &wrefs, root, lo, hi, radius);
    let fe = fe_wheel_pivots(commands, &wheels, wheel_locs, spec.frontend);
    if spec.frontend {
        for lod in &wheel_lods {
            for k in [1, 3] {
                commands.entity(lod[k]).insert(Transform::from_scale(Vec3::new(-1.0, 1.0, 1.0)));
            }
        }
    }
    let discs = spawn_brake_discs(commands, b, &mut tex, wheels_m.as_ref(), spec, root, wheel_locs, wrefs[0]);
    let door_locators = door_locs.map(|l| l.map(|m| door_locator(&m)));
    // 0x001134A0 gate: race cars (type 0) only, never in the front end (renderer +49), blur texture loaded
    log::debug!(
        "wheel blur {}: spokes texture '{}', blur prims {}",
        spec.dir,
        blur_prims[0].spoke_texture,
        blur_prims.iter().filter(|w| w.blur.is_some()).count()
    );
    if !spec.frontend && !spec.traffic && blur_prims.iter().any(|w| w.blur.is_some()) {
        commands.entity(root).insert(CarWheelBlur { wheels: blur_prims, spokes: 1.0, blur: 0.0 });
    }
    // the driver is never drawn in the front end (renderer +49)
    if let (Some(pos), false) = (driver_pos, spec.frontend) {
        if let Some(d) = spawn_driver(commands, b, &mut tex, wheels_m.as_ref(), root, Vec3::from_array(pos)) {
            commands.entity(root).insert(d);
        }
    }
    if let Some(hb) = beams {
        commands.entity(root).insert(hb);
    }
    commands.entity(root).insert((flare_locs, crate::flare_render::CarFlareState::default()));
    // +1816 (front-end shadow height, CarRender 0x001100B0 from FE3DVehicles 0x0025C890) = the setup's WHEELS.BIN
    // record +0x0C; without one, the wheel diameter stands in
    commands.entity(root).insert(CarShadow::new(spec.shadow, spec.wheel_height.unwrap_or(2.0 * radius), spec.neon));
    let prim_lists = CarPrimLists::build(&records, chassis_records, ranges.get(1..).unwrap_or_default(), &door_ranges);
    let lights = car_lights_setup(commands, b, &mut tex, body, &records, chassis_records);
    let models = CarModels { stems: &stems, recs: &lod_recs, ranges: &stem_ranges };
    add_vcd_textures(&mut tex, "\\Vehicles\\DECALST.VCD");
    apply_car_decals(commands, b, &mut tex, spec, &models);
    if spec.licence_plate || request == 0 {
        if let Some(p) = crate::licence_plate_render::find_plate_prims(&models) {
            commands.entity(root).insert(p);
        }
    }
    if let Some(c) = collect {
        crate::car_damage_render::setup(commands, b, &mut tex, root, c, head, brake, &records, &ranges, &other_records);
    }
    commands.entity(root).insert((
        CarRender { request, spec: spec.clone() },
        CarRenderParts {
            body,
            wheels,
            bounds: (lo, hi),
            wheel_radius: radius,
            programs: b.car.as_ref().map(|c| c.made.clone()).unwrap_or_default(),
            light_slots: [-1, -1],
            doors,
            door_locators,
            fe_wheels: fe.0,
            wheel_pivots: fe.1,
            wheel_base_rot: fe.2,
            discs: discs.0,
            disc_pivots: discs.1,
            decal_slots: std::array::from_fn(|k| decal_slot_count(&models, k as u32, spec.frontend, spec.traffic)),
        },
        CarLod { lods, wheel_lods, lod: 0, in_car: false },
        lights,
        prim_lists,
        CarColourTexture(b.images.add(car_colour_image(spec))),
    ));
    if !spec.frontend {
        commands.entity(root).insert(body_meshes);
    }
    root
}

/// the car's "CarColour" texture (CarPalette_PC_XBOX.cpp 0x001577F0, refilled by 0x00157F80 from +2236 / +2240);
/// built as the original does, though nothing samples it (see `car_colour_gradient`)
#[derive(Component, Clone, Debug)]
pub struct CarColourTexture(pub Handle<Image>);

/// 0x00157F80 → 0x001561C0 for a spec: paint / pearl D3DCOLORs (`CarSpec` keeps them as byte · 0.003921569, so the
/// bytes are recovered by rounding), A8R8G8B8 pixels stored as rgba8 rows
pub fn car_colour_image(spec: &CarSpec) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let d = |c: Vec3| {
        let b = |x: f32| ((x * 255.0).round().clamp(0.0, 255.0)) as u32;
        0xFF00_0000 | (b(c.x) << 16) | (b(c.y) << 8) | b(c.z)
    };
    let (a, p) = crate::car_palette_pc_xbox::car_colour_ends(d(spec.colour), d(spec.pearl));
    let px = crate::car_palette_pc_xbox::car_colour_gradient(a, p);
    let data: Vec<u8> = px.iter().flat_map(|&c| [(c >> 16) as u8, (c >> 8) as u8, c as u8, (c >> 24) as u8]).collect();
    let n = crate::car_palette_pc_xbox::CAR_COLOUR_SIZE;
    Image::new(
        Extent3d { width: n, height: n, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// the meshes of a car's chassis and fitted parts at every LOD (kept in the main world for in-game cars)
#[derive(Component, Clone, Debug, Default)]
pub struct CarBodyMeshes(pub Vec<Handle<Mesh>>);

/// CarRender.cpp 0x00115110 / 0x001152D0 / 0x00117AA0 (GFXHelper.cpp 0x0012B4C0 lookups) — the light primitives:
/// `left_brake_l2_1` else `left_brake` (and right; an `_l2_1` material sets flag 0x200), `left_headlight` /
/// `right_headlight` in the chassis; `left_reverse` / `right_reverse` in the chassis, then the parts. The brake and
/// headlight primitives become type 9 (left out of the car's own draw, mask 0x200) and are drawn by 0x00112A50 while
/// the LOD is not 2; the reverse lights get a second "revlight" draw.
fn car_lights_setup(
    commands: &mut Commands,
    b: &mut Builder,
    tex: &mut TexSet,
    body: Entity,
    records: &[PrimRecord],
    chassis: usize,
) -> CarLights {
    let find = |rs: &[PrimRecord], sub: &str| rs.iter().position(|r| r.shader_name.contains(sub));
    let group = commands.spawn((Transform::default(), Visibility::Inherited, ChildOf(body))).id();
    let ch = &records[..chassis.min(records.len())];
    let mut l2 = false;
    let mut brake = [None; 2];
    for (k, side) in ["left", "right"].iter().enumerate() {
        brake[k] = match find(ch, &format!("{side}_brake_l2_1")) {
            Some(i) => {
                l2 = true;
                Some(i)
            }
            None => find(ch, &format!("{side}_brake")),
        };
        if brake[k].is_none() {
            log::debug!("FAILED to find car {side}_brake  material");
        }
    }
    let head = [find(ch, "left_headlight"), find(ch, "right_headlight")];
    let mut take = |i: Option<usize>| {
        i.map(|i| {
            commands.entity(records[i].entity).insert(ChildOf(group));
            records[i].entity
        })
    };
    let brake = [take(brake[0]), take(brake[1])];
    let head = [take(head[0]), take(head[1])];
    // "revlight" (CarPalette.cpp 0x0010D7E0): blend 2, z test without z write, TFACTOR alpha over the texture
    let reverse = ["left_reverse", "right_reverse"].map(|n| {
        let i = find(ch, n).or_else(|| find(records, n))?;
        let r = &records[i];
        let state = StageState { blend_mode: 2, z_write: false, address_u: r.state.address_u, address_v: r.state.address_v, ..StageState::default() };
        let texture = tex.image(&r.texture, state.address_u, state.address_v, b.images);
        let mut params = XboxStageParams::new(&state);
        params.arg = Vec4::new(0.0, STAGE_MODE_REVLIGHT, 0.0, 0.0);
        params.colour = Vec4::ZERO;
        let m = b.materials.add(XboxStageMaterial { params, texture, stencil: None, spec_texture: None, state });
        let e = commands.spawn((Mesh3d(r.mesh.clone()), MeshMaterial3d(m.clone()), Visibility::Hidden, ChildOf(group))).id();
        Some((e, m))
    });
    CarLights {
        group,
        brake,
        head,
        reverse,
        brake_materials: b.car.as_ref().map(|c| c.brake.clone()).unwrap_or_default(),
        l2,
        brake_level: 0.0,
        reverse_level: 0.0,
    }
}

/// VehicleEntity.cpp 0x001C0D60 (traffic branch): `<name>` and four `<name>_wheel` from TrafficM.vcd with the
/// TrafficT.vcd textures
fn spawn_traffic_car(commands: &mut Commands, b: &mut Builder, spec: &CarSpec, request: u64, transform: Transform) -> Entity {
    let name = spec.dir.as_str();
    let mut tex = TexSet::default();
    // `\Vehicles\CommonT.vcd` is loaded for every model by GameDataLoading::LoadGameData 0x0009D170 (the texture
    // manager resolves names over all loaded packs): the traffic windows / undersides / tyres (win01_envmap,
    // bottom0N, traff_tyre01) live there, not in TrafficT — without it they drew untextured white
    add_vcd_textures(&mut tex, "\\Vehicles\\COMMONT.VCD");
    add_vcd_textures(&mut tex, "\\Vehicles\\TrafficT.vcd");
    let models = visos::file::Vcd::open("\\Vehicles\\TrafficM.vcd").ok();
    let get = |stem: &str| models.as_ref().and_then(|m| m.get(&format!("{stem}.xmf"))).and_then(|d| Model::parse(d).ok());
    let root = commands.spawn((transform, Visibility::default())).id();
    let body = commands.spawn((Transform::default(), Visibility::default(), ChildOf(root))).id();
    b.bounds_min = Vec3::splat(f32::MAX);
    b.bounds_max = Vec3::splat(f32::MIN);
    match get(name) {
        Some(m) => {
            b.add_model(commands, &mut tex, &m, Mat4::IDENTITY, &SpawnOpts { parent: Some(body), ..SpawnOpts::world() });
        }
        None => log::warn!("traffic car: no model {name}"),
    }
    let (lo, hi) = (b.bounds_min, b.bounds_max);
    let wm = get(&format!("{name}_wheel"));
    let radius = wm.as_ref().map(|m| m.sphere[3] * std::f32::consts::FRAC_1_SQRT_2).unwrap_or(0.33);
    let (wheels, _, _) = place_wheels(commands, b, &mut tex, &[wm.as_ref()], root, lo, hi, radius);
    commands.entity(root).insert((
        CarRender { request, spec: spec.clone() },
        CarRenderParts {
            body,
            wheels,
            bounds: (lo, hi),
            wheel_radius: radius,
            programs: b.car.as_ref().map(|c| c.made.clone()).unwrap_or_default(),
            light_slots: [-1, -1],
            doors: [None; 8],
            door_locators: [None; 8],
            fe_wheels: [wheels[0], wheels[2], wheels[1], wheels[3]],
            wheel_pivots: [Vec3::ZERO; 4],
            wheel_base_rot: [Quat::IDENTITY; 4],
            discs: [None; 4],
            disc_pivots: [Vec3::ZERO; 4],
            decal_slots: [0; 6],
        },
        // +1816 stand-in for the no-vehicle branch: the wheel diameter
        CarShadow::new(spec.shadow, 2.0 * radius, spec.neon),
    ));
    root
}

/// wheel entities at positions estimated from the body bounds (physics overwrites them), each with one child per
/// LOD model (`wm[lod]`)
#[allow(clippy::too_many_arguments)]
fn place_wheels(
    commands: &mut Commands,
    b: &mut Builder,
    tex: &mut TexSet,
    wm: &[Option<&Model>],
    root: Entity,
    lo: Vec3,
    hi: Vec3,
    radius: f32,
) -> ([Entity; 4], Vec<[Entity; 4]>, [WheelBlurPrims; 4]) {
    let half = (hi - lo) * 0.5;
    let centre = (hi + lo) * 0.5;
    let mut wheels = [Entity::PLACEHOLDER; 4];
    let mut lods = vec![[Entity::PLACEHOLDER; 4]; wm.len()];
    let mut blur: [WheelBlurPrims; 4] = Default::default();
    for (k, w) in wheels.iter_mut().enumerate() {
        let side = if k % 2 == 0 { -1.0 } else { 1.0 };
        let front = if k < 2 { 1.0 } else { -1.0 };
        let pos = Vec3::new(centre.x + side * (half.x - radius * 0.5), lo.y + radius, centre.z + front * half.z * 0.62);
        // the wheel model faces +x; left-hand wheels are turned round
        let rot = if side < 0.0 { Quat::from_rotation_y(std::f32::consts::PI) } else { Quat::IDENTITY };
        *w = commands.spawn((Transform::from_translation(pos).with_rotation(rot), Visibility::default(), ChildOf(root))).id();
        for (lod, m) in wm.iter().enumerate() {
            let vis = if lod == 0 { Visibility::Inherited } else { Visibility::Hidden };
            let g = commands.spawn((Transform::default(), vis, ChildOf(*w))).id();
            lods[lod][k] = g;
            if let Some(m) = m {
                let o = SpawnOpts { parent: Some(g), ..SpawnOpts::world() };
                // LOD 0: the GFXHelper 0x0012B4C0 lookups of 0x00117980 need the primitives
                let saved = if lod == 0 { b.records.replace(Vec::new()) } else { None };
                b.add_model(commands, tex, m, Mat4::IDENTITY, &o);
                if lod == 0 {
                    let recs = std::mem::replace(&mut b.records, saved).unwrap_or_default();
                    blur[k] = wheel_blur_prims(commands, b, tex, &recs, g);
                }
            }
        }
    }
    (wheels, lods, blur)
}

/// CarRender.cpp 0x00117980 — the LOD 0 wheel's first "wheel_spokes" primitive (prim type 10; its texture +2268)
/// and first "blur_0" primitive (type 11, never in the normal draw: hidden here); 0x00116F60 redraws them with the
/// spoke-fade shader (renderer +36, CarPalette 0x0010D7E0) and "wblur" (+32, CarRender 0x00110220, texture
/// `<spoke texture>_blur` from wheels_t.vcd), both blend 3, z test without z write, stage 0 MODULATE(DIFFUSE,
/// TEXTURE) / alpha TEXTURE, stage 1 colour CURRENT, alpha CURRENT · TFACTOR. The fade copies share the mesh.
fn wheel_blur_prims(commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, recs: &[PrimRecord], g: Entity) -> WheelBlurPrims {
    let find = |sub: &str| recs.iter().find(|r| r.shader_name.contains(sub));
    let mut out = WheelBlurPrims::default();
    let fade = |commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, r: &PrimRecord, texture: &str, own_address: bool| {
        let (au, av) = if own_address { (r.state.address_u, r.state.address_v) } else { (0, 0) };
        let state = StageState { blend_mode: 3, z_write: false, address_u: au, address_v: av, ..StageState::default() };
        // the blur copy is made even without its own texture: "wblur" takes car 0's (`latch_wheel_blur`)
        let texture = tex.image(texture, state.address_u, state.address_v, b.images);
        if texture.is_none() && own_address {
            return None;
        }
        let params = XboxStageParams::new(&state);
        let m = b.materials.add(XboxStageMaterial { params, texture, stencil: None, spec_texture: None, state });
        let e = commands.spawn((Mesh3d(r.mesh.clone()), MeshMaterial3d(m.clone()), Visibility::Hidden, ChildOf(g))).id();
        Some((e, m))
    };
    if let Some(s) = find("wheel_spokes") {
        out.spoke_texture = s.texture.clone();
        out.spokes = fade(commands, b, tex, s, &s.texture.clone(), true).map(|(e, m)| (s.entity, e, m));
    }
    if let Some(bl) = find("blur_0") {
        commands.entity(bl.entity).insert(Visibility::Hidden);
        // byte_68661D: the blur texture exists
        let name = format!("{}_blur", out.spoke_texture);
        out.blur = fade(commands, b, tex, bl, &name, false);
    }
    out
}

/// CarRender.cpp 0x00110220: `LoadModelGeometry("driver01", wheels_m.vcd)` -> renderer +52 `m_pDriverModel`
pub const DRIVER_MODEL: &str = "driver01";
/// `flt_686834` / `flt_686830`: head pitch per m/s of velocity change along the car, braking / accelerating
const SWAY_PITCH_BRAKE: f32 = 0.5;
const SWAY_PITCH_ACCEL: f32 = f32::from_bits(0x3DCC_CCCD);
/// `flt_686838`: head roll per m/s of sideways velocity change
const SWAY_ROLL: f32 = f32::from_bits(0x3E4C_CCCD);
/// `dword_686828` / `dword_68682C`: lerp rate towards a larger / smaller target
const SWAY_ATTACK: f32 = 0.5;
const SWAY_RELAX: f32 = f32::from_bits(0x3E4C_CCCD);
/// `flt_686820` / `flt_686824`: pitch range; `flt_68681C`: roll limit
const SWAY_PITCH_MIN: f32 = -0.25;
const SWAY_PITCH_MAX: f32 = 0.75;
const SWAY_ROLL_MAX: f32 = f32::from_bits(0x3EB3_3333);

/// car data +912: the LOD 0 chassis' "driver" locator translation, z negated (CarRender.cpp 0x00110220, which then
/// sets +2328 bit 0x800)
pub fn driver_locator(m: &Model) -> Option<[f32; 3]> {
    m.locator("driver").map(|x| [x[12], x[13], -x[14]])
}

/// the car's driver (renderer +52 drawn at car data +912 by CarRender.cpp 0x00119900)
#[derive(Component, Clone, Copy, Debug)]
pub struct CarDriver {
    pub entity: Entity,
    /// car data +912
    pub pos: Vec3,
}

/// what the driver draw needs from the game each frame (core fills it; absent = plain racing)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct CarDriverInput {
    /// `byte_7065A9` (CarPalette.cpp 0x0010D580): front end, or single player (`byte_6F7157 == 0`) in camera mode 3,
    /// or `dword_749110 == 2` — draws the drivers of non-race cars too and sways car 0's driver
    pub cine: bool,
    /// car 0's linear velocity (VehicleEntity 0x001C6480 -> VsAIRaceVehicle 0x00024200), game space
    pub velocity: Vec3,
}

/// the head sway state of car 0's driver (renderer +116 previous velocity, +132 pitch, +136 roll)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct DriverSway {
    pub prev: Vec3,
    pub pitch: f32,
    pub roll: f32,
    /// real time not yet run as 60 Hz frames
    acc: f32,
}

impl DriverSway {
    /// CarRender.cpp 0x00118DD0 — one rendered frame: lean the head against the change of velocity since the last
    /// frame (`z` = car matrix row 2, the nose)
    pub fn step(&mut self, velocity: Vec3, z: Vec3) {
        let delta = velocity - self.prev;
        self.prev = velocity;
        let c = z.cross(delta);
        let mut side = c.length();
        if c.y < 0.0 {
            side = 0.0 - side;
        }
        let along = z.dot(delta);
        let pitch = if along <= 0.0 { (0.0 - SWAY_PITCH_BRAKE) * along } else { (0.0 - SWAY_PITCH_ACCEL) * along };
        let roll = (0.0 - SWAY_ROLL) * side;
        let rp = if pitch.abs() <= self.pitch.abs() { SWAY_RELAX } else { SWAY_ATTACK };
        let rr = if roll.abs() <= self.roll.abs() { SWAY_RELAX } else { SWAY_ATTACK };
        // CRim 0x000452C0 = (b - a) * t + a
        self.pitch = (pitch - self.pitch) * rp + self.pitch;
        self.roll = (roll - self.roll) * rr + self.roll;
        self.pitch = self.pitch.clamp(SWAY_PITCH_MIN, SWAY_PITCH_MAX);
        self.roll = self.roll.clamp(0.0 - SWAY_ROLL_MAX, SWAY_ROLL_MAX);
    }

    /// 0x00118DD0 tail: RotX(pitch)·RotY(0)·RotZ(roll) (Renderer 0x002AB7E0) before the car matrix
    pub fn rotation(&self) -> Mat4 {
        crate::car_damage_fx::rotation_xyz(self.pitch, 0.0, self.roll)
    }
}

/// the driver model as a hidden child of the car root at the locator (shown by `update_car_driver`)
fn spawn_driver(
    commands: &mut Commands,
    b: &mut Builder,
    tex: &mut TexSet,
    wheels_m: Option<&visos::file::Vcd>,
    root: Entity,
    pos: Vec3,
) -> Option<CarDriver> {
    let Some(m) = wheels_m.and_then(|w| w.get(&format!("{DRIVER_MODEL}.xmf"))).and_then(|d| Model::parse(d).ok()) else {
        log::warn!("car render: no {DRIVER_MODEL}.xmf in wheels_m.vcd");
        return None;
    };
    let e = commands.spawn((Transform::from_translation(pos), Visibility::Hidden, ChildOf(root))).id();
    b.add_model(commands, tex, &m, Mat4::IDENTITY, &SpawnOpts { parent: Some(e), ..SpawnOpts::world() });
    Some(CarDriver { entity: e, pos })
}

/// CarRender.cpp 0x00119900 driver part, every rendered frame: drawn (FuryInstance 0x0012A7D0, no object lights)
/// when the car has the locator (+2328 & 0x800), +2304 is clear, not in the front end, the car is a race car (+1784
/// == 0) or `byte_7065A9`, and LOD 0 (+2249). Matrix = the car's with the translation at car·+912; car 0 under
/// `byte_7065A9` gets the head sway of 0x00118DD0 first (run as 60 Hz frames of the real delta).
fn update_car_driver(
    time: Res<Time>,
    input: Option<Res<CarDriverInput>>,
    mut sway: ResMut<DriverSway>,
    cars: Query<(&CarRender, &CarLod, &CarDriver, &GlobalTransform)>,
    mut parts: Query<(&mut Transform, &mut Visibility)>,
) {
    let input = input.map(|i| *i).unwrap_or_default();
    for (car, lod, d, g) in &cars {
        let show = !car.spec.frontend && (!car.spec.traffic || input.cine) && lod.lod == 0;
        let mut local = Mat4::from_translation(d.pos);
        if input.cine && car.request == 0 {
            sway.acc = (sway.acc + time.delta_secs()).min(0.25);
            let z = g.to_matrix().z_axis.truncate();
            while sway.acc >= 1.0 / XBOX_FRAME_RATE {
                sway.acc -= 1.0 / XBOX_FRAME_RATE;
                sway.step(input.velocity, z);
            }
            local *= sway.rotation();
        }
        if let Ok((mut t, mut v)) = parts.get_mut(d.entity) {
            let nt = Transform::from_matrix(local);
            if *t != nt {
                *t = nt;
            }
            let nv = if show { Visibility::Inherited } else { Visibility::Hidden };
            if *v != nv {
                *v = nv;
            }
        }
    }
}

/// CarPalette.cpp 0x0010D7E0 LOD thresholds (+55756/+55760 LOD 1, +55764/+55768 LOD 2) for cars beside / ahead of
/// or behind the camera, compared against the fov-scaled squared distance
const LOD_NEAR: [f32; 2] = [400.0, 400.0];
const LOD_FAR: [f32; 2] = [1600.0, 1600.0];
/// `dword_6867A0`: brake / reverse level change per rendered frame
const LIGHT_STEP: f32 = 0.25;
/// rendered frames per second the per-frame steps were made for: the TV refresh, 60 Hz NTSC / PAL60 (core.md
/// "Original render rate"; 50 Hz PAL50 not modelled)
pub const XBOX_FRAME_RATE: f32 = 60.0;

/// 0x00110220: +1808 / +1812 without a vehicle database entry (0x4039999A, 0x4099999A)
pub const SHADOW_SIZE_DEFAULT: [f32; 2] = [f32::from_bits(0x4039_999A), f32::from_bits(0x4099_999A)];
/// 0x00115920: `CCRCChecked("OFF", 1113245403)`, the neon slot's "off" colour id (COLOURS.BIN record 168)
pub const NEON_OFF_CRC: u32 = 0x425A_C6DB;
/// `dword_6865F4` / `dword_6865F8`: length / width scale of the quad for shaders other than "carshadow"
pub const SHADOW_INNER_SCALE: [f32; 2] = [f32::from_bits(0x3F4C_CCCD), f32::from_bits(0x3F66_6666)];
/// `dword_686780`: the FastCar wheel under each quad corner
pub const SHADOW_CORNER_WHEEL: [usize; 4] = [3, 0, 2, 1];
/// `flt_68677C`: ground ray length below a wheel off the ground
pub const SHADOW_RAY: f32 = f32::from_bits(0x41C8_0000);
/// the ray starts this far above the wheel's ground point (`0.1f`)
pub const SHADOW_RAY_LIFT: f32 = 0.1;
/// `dword_686778`: the quad's height above the ground
pub const SHADOW_LIFT: f32 = f32::from_bits(0x3C23_D70A);
/// `dword_6865E8`: headlight level (+1820) change per call per rendered frame
pub const HEADLIGHT_STEP: f32 = f32::from_bits(0x3E4C_CCCD);
/// `dword_7184B0` (bss, never written): added to the no-vehicle quad height
pub const SHADOW_FE_OFFSET: f32 = 0.0;
/// `dword_706604` = `dword_71823C` VISOS(0x90, 0, 0, 0) in game / `dword_709D58` VISOS(0x60, 0, 0, 0) in the front end
/// (CarPalette 0x0010D7E0; static inits 0x00424A90 / 0x00424AC0)
pub const SHADOW_COLOUR: [[u8; 4]; 2] = [[0x90, 0, 0, 0], [0x60, 0, 0, 0]];
/// `byte_6F5B7C` "Graphics.Shadow.Silhouette" (GameManager 0x000B4170 tweak, bss 0): car 0 skips its shadow
pub const SHADOW_SILHOUETTE: bool = false;

/// CarRender data of the shadow / neon quad 0x00111B20 (inserted on every in-game car root)
#[derive(Component, Clone, Copy, Debug)]
pub struct CarShadow {
    /// +1808 width, +1812 length
    pub size: [f32; 2],
    /// +1816 (front end: FE3DVehicles 0x0025C890's wheel value; only the no-vehicle branch reads it)
    pub height: f32,
    /// +2232 when +2328 bit 1 (neon fitted)
    pub neon: Option<u32>,
    /// +1820 headlight level (1 from the car init 0x00110220), read by TrackFX's beams
    pub level: f32,
    /// +932: normalised c0 + c1 − c2 − c3 of the last quad; +1804: mean height of corners 0 and 3 (no reader ported)
    pub normal: Vec3,
    pub mid_y: f32,
}

impl CarShadow {
    pub fn new(size: [f32; 2], height: f32, neon: Option<u32>) -> Self {
        Self { size, height, neon, level: 1.0, normal: Vec3::ZERO, mid_y: 0.0 }
    }
}

/// what 0x00111B20 reads from a car's VehicleEntity (+1788) and FastCar (VE+240)
#[derive(Clone, Copy, Debug)]
pub struct ShadowVehicle {
    /// 0x001C68F0 per FastCar wheel: centre (0x00361390) − body up (0x001BB200(1)) · radius (0x0035FB40)
    pub ground: [Vec3; 4],
    /// 0x00364BB0: FastCar +1088 + 4i > 0
    pub touches: [bool; 4],
    /// car data +1784 == 0 (race car; only those test their wheels)
    pub race_car: bool,
    /// VE +4388 != 0
    pub flag_4388: bool,
}

/// CarRender.cpp 0x00111B20 — the flat quad under a car (`full` = the "carshadow" shader at full size, else the
/// length / width are scaled by 0.8 / 0.9): corners from the car matrix +0 (x · −width, z · length), each lowered
/// onto the ground under its wheel (wheel ground point when the wheel touches or the car is not a race car, else a
/// 25 m ray from 0.1 above it; a miss draws nothing and leaves the level) plus 0.01; without a vehicle at y −
/// +1816/2. With a vehicle the headlight level +1820 moves by `step`: down while VE+4388 is set or a wheel is off the
/// ground, else up (≤ 1). Returns the strip c0, c1, c2, c3 (uv corners, one colour, world space).
pub fn shadow_quad(
    matrix: Mat4,
    sh: &mut CarShadow,
    vehicle: Option<&ShadowVehicle>,
    full: bool,
    colour: u32,
    step: f32,
    ray: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>,
) -> Option<[crate::effect_draw::EffectVertex; 4]> {
    // (`dword_7065A4` == 0 || ≥ 6: the garage customisation mode, 0 in game)
    let pos = matrix.w_axis.truncate();
    let (mut z, mut x) = (matrix.z_axis.truncate(), matrix.x_axis.truncate());
    if full {
        z *= sh.size[1];
        x *= 0.0 - sh.size[0];
    } else {
        z *= sh.size[1] * SHADOW_INNER_SCALE[0];
        x *= (0.0 - sh.size[0]) * SHADOW_INNER_SCALE[1];
    }
    let half_x = x * 0.5;
    let mut c = [Vec3::ZERO; 4];
    c[0] = (pos + z * 0.5) - half_x;
    c[1] = c[0] + x;
    c[2] = c[0] - z;
    c[3] = c[2] + x;
    let uv = [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE];
    let mut v = [Vec3::ZERO; 4];
    let mut all_touch = true;
    if let Some(ve) = vehicle {
        for i in 0..4 {
            let w = SHADOW_CORNER_WHEEL[i];
            let mut g = ve.ground[w];
            if ve.race_car && !ve.touches[w] {
                all_touch = false;
                let mut to = g;
                g.y += SHADOW_RAY_LIFT;
                to.y -= SHADOW_RAY;
                let t = ray(g, to)?;
                g.y -= t * SHADOW_RAY;
            }
            let y = g.y + SHADOW_LIFT;
            c[i].y = y;
            v[i] = Vec3::new(c[i].x, y, c[i].z);
        }
    } else {
        let base = pos.y - sh.height / 2.0;
        for i in 0..4 {
            c[i].y = base + SHADOW_FE_OFFSET;
            v[i] = c[i];
        }
    }
    if let Some(ve) = vehicle {
        if ve.flag_4388 {
            sh.level = (sh.level - step).max(0.0);
        } else {
            sh.normal = (c[0] + c[1] - c[2] - c[3]).normalize();
            sh.mid_y = (c[3].y + c[0].y) * 0.5;
            sh.level = if all_touch { (sh.level + step).min(1.0) } else { (sh.level - step).max(0.0) };
        }
    }
    Some(std::array::from_fn(|i| crate::effect_draw::EffectVertex::new(v[i], colour, uv[i])))
}

/// CarRender.cpp 0x0011A6E0 (from the car draw 0x00119900, after TrackFX) — per car: `culled` (+2251, also set
/// when the car's `dword_6865F0` zone bit is clear) cars only move the level when the camera sits in them (+1804 =
/// y − 0.5; all wheels down and VE+4388 clear → +932 = z, level up, else down); visible cars without a neon get the
/// "carshadow" quad in `colour` (`dword_706604`), car 0 not with the silhouette tweak (whose draw 0x001638F0 is
/// empty). Neon cars get theirs from TrackFX 0x0014B030 pass 4.
#[allow(clippy::too_many_arguments)]
pub fn shadow_pass(
    index: usize,
    matrix: Mat4,
    sh: &mut CarShadow,
    vehicle: Option<&ShadowVehicle>,
    culled: bool,
    inside: bool,
    zone_mask: i32,
    colour: u32,
    step: f32,
    ray: &mut dyn FnMut(Vec3, Vec3) -> Option<f32>,
) -> Option<[crate::effect_draw::EffectVertex; 4]> {
    let culled = culled || (index < 32 && zone_mask & (1 << index) == 0);
    if culled {
        if inside {
            sh.mid_y = matrix.w_axis.y - 0.5;
            if let Some(ve) = vehicle {
                if ve.touches.iter().all(|&t| t) && !ve.flag_4388 {
                    sh.normal = matrix.z_axis.truncate();
                    sh.level = (sh.level + step).min(1.0);
                } else {
                    sh.level = (sh.level - step).max(0.0);
                }
            }
        }
        return None;
    }
    if sh.neon.is_some() || (index == 0 && SHADOW_SILHOUETTE) {
        return None;
    }
    shadow_quad(matrix, sh, vehicle, true, colour, step, ray)
}

/// FECK2ElementCamera.cpp 0x002999B0 → CarRender 0x00119900(…, 0, 1) → 0x0011A6E0 for the front-end cars: no
/// vehicle, so the full "carshadow" quad sits at y − +1816/2 in `dword_706604` = VISOS(0x60) (`dword_709D58`, set by
/// CarPalette 0x0010D7E0 in the front end), depth bias 82 (showroom); `dword_6865F0` is -1 there (0x0010E1A1). The
/// culled flag (+2251, 0x001127F0) is taken from the root's visibility.
fn draw_frontend_shadows(
    mut cars: Query<(&CarRender, &GlobalTransform, &mut CarShadow, &InheritedVisibility)>,
    mut draw: ResMut<crate::effect_draw::EffectDraw>,
) {
    let mut rows: Vec<_> = cars.iter_mut().filter(|r| r.0.spec.frontend).collect();
    if rows.is_empty() {
        return;
    }
    rows.sort_by_key(|r| r.0.request);
    let sh = crate::car_palette::carshadow_shader(true);
    let colour = crate::light_streak::visos_colour(SHADOW_COLOUR[1]);
    for (k, (_, t, s, vis)) in rows.iter_mut().enumerate() {
        if let Some(q) = shadow_pass(k, t.to_matrix(), s, None, !vis.get(), false, -1, colour, 0.0, &mut |_, _| None) {
            draw.strip(&sh, &q);
        }
    }
}

/// the wheel spin the blur reads (0x00116E30: `AUDIO_Resample__361500(VE+240, 0)` = FastCar +1432, `WHEEL_OMEGA[0]`,
/// rad/s of wheel 0); core inserts it on in-game car roots. Without it the wheels show as stopped.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CarWheelSpin {
    pub omega: f32,
}

/// one wheel's 0x00117980 records: spokes (normal entity, fade copy, fade material), blur (copy, "wblur" material)
#[derive(Clone, Debug, Default)]
pub struct WheelBlurPrims {
    pub spoke_texture: String,
    pub spokes: Option<(Entity, Entity, Handle<XboxStageMaterial>)>,
    pub blur: Option<(Entity, Handle<XboxStageMaterial>)>,
}

/// CarRender data +2260 spokes level, +2264 blur level, per car with a blur texture
#[derive(Component, Clone, Debug)]
pub struct CarWheelBlur {
    pub wheels: [WheelBlurPrims; 4],
    pub spokes: f32,
    pub blur: f32,
}

/// 0x00116E30 constants: spokes fade from `dword_6867E4` half-turns/s at `dword_6867DC` per unit, blur from
/// `dword_6867E8` at `dword_6867E0`
const SPOKES_FROM: f32 = 5.0;
const SPOKES_RATE: f32 = f32::from_bits(0x3DCC_CCCD);
const BLUR_FROM: f32 = 2.0;
const BLUR_RATE: f32 = f32::from_bits(0x3DCC_CCCD);

/// CarRender.cpp 0x00116E30 — v = |ω|/π; spokes = 1 up to 5, then 1 − min((v − 5)·0.1, 1) and the normal spokes
/// (prim type 10, mask 1024) are left out; blur = 0 up to 2, then min((v − 2)·0.1, 1). Returns (spokes, blur, hide).
pub fn wheel_blur_levels(omega: f32) -> (f32, f32, bool) {
    let v = omega.abs() * 0.318_309_87_f32;
    let (spokes, hide) = if v <= SPOKES_FROM { (1.0, false) } else { (1.0 - ((v - SPOKES_FROM) * SPOKES_RATE).min(1.0), true) };
    let blur = if v <= BLUR_FROM { 0.0 } else { ((v - BLUR_FROM) * BLUR_RATE).min(1.0) };
    (spokes, blur, hide)
}

/// "wblur" (renderer +32) texture: CarRender.cpp 0x00110220 loads `<dword_7065AC>_blur` once, and 0x00117980 latches
/// `dword_7065AC` from car 0's spoke texture only — every car's blur discs use car 0's texture (original quirk,
/// kept). `byte_68661D` = that texture exists; without it no car runs the blur (0x001134A0). Car 0 = the lowest
/// request among the cars with blur records (race car 0 in game; the viewers start at 1 / 20).
#[derive(Resource, Clone, Debug, Default)]
pub struct WheelBlurTexture {
    pub car0: Option<u64>,
    pub image: Option<Handle<Image>>,
}

/// CarRender.cpp 0x00110220 / 0x00117980 — share car 0's blur texture with every car's "wblur" copies
fn latch_wheel_blur(
    cars: Query<(&CarRender, &CarWheelBlur)>,
    mut shared: ResMut<WheelBlurTexture>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
) {
    let car0 = cars.iter().min_by_key(|(r, _)| r.request);
    let image = car0.and_then(|(_, wb)| {
        let (_, m) = wb.wheels.iter().find_map(|w| w.blur.as_ref())?;
        materials.get(m)?.texture.clone()
    });
    shared.car0 = car0.map(|(r, _)| r.request);
    shared.image = image.clone();
    let Some(image) = image else { return };
    for (_, wb) in &cars {
        for (_, m) in wb.wheels.iter().filter_map(|w| w.blur.as_ref()) {
            if materials.get(m).is_some_and(|x| x.texture.as_ref().map(Handle::id) != Some(image.id())) {
                materials.get_mut(m).unwrap().texture = Some(image.clone());
            }
        }
    }
}

/// per rendered frame: 0x001134A0 → 0x00116E30 levels, 0x00112A50 → 0x00116F60 when spokes < 1 or blur > 0: the blur
/// discs at TFACTOR alpha VISOS(blur·128) (= min(2·_ftol(x·128), 255)), the spokes faded at spokes·128 while 0 < spokes
/// < 1. The original draws the LOD 0 records at every LOD; here the copies sit in the LOD 0 wheel group (deviation:
/// none beyond LOD 0). The debug override `byte_7184E0` is off.
fn update_wheel_blur(
    mut cars: Query<(&mut CarWheelBlur, Option<&CarWheelSpin>)>,
    mut vis: Query<&mut Visibility>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    shared: Res<WheelBlurTexture>,
) {
    let alpha = |x: f32| ((2 * ((x * 128.0) as i32 as u8) as u32).min(255)) as f32 / 255.0;
    // byte_68661D: car 0's blur texture was found
    if shared.image.is_none() {
        return;
    }
    for (mut wb, spin) in &mut cars {
        let (spokes, blur, hide) = wheel_blur_levels(spin.map_or(0.0, |s| s.omega));
        wb.spokes = spokes;
        wb.blur = blur;
        let draw = spokes < 1.0 || blur > 0.0;
        for w in wb.wheels.iter() {
            if let Some((normal, copy, m)) = &w.spokes {
                // the type 10 mask (only the LOD 0 model's prim carries the type)
                show(&mut vis, *normal, !hide);
                let fade = draw && w.blur.is_some() && spokes > 0.0 && spokes < 1.0;
                show(&mut vis, *copy, fade);
                if fade && materials.get(m).is_some_and(|x| x.params.colour.w != alpha(spokes)) {
                    materials.get_mut(m).unwrap().params.colour.w = alpha(spokes);
                }
            }
            if let (Some((copy, m)), true) = (&w.blur, w.spokes.is_some()) {
                let on = draw && blur > 0.0;
                show(&mut vis, *copy, on);
                if on && materials.get(m).is_some_and(|x| x.params.colour.w != alpha(blur)) {
                    materials.get_mut(m).unwrap().params.colour.w = alpha(blur);
                }
            }
        }
    }
}
/// `dword_68679C`: floor of `_l2_1` brake lights
const L2_FLOOR: f32 = 0.3;

/// the CarRender tweak variables the port reads ("Graphics.LOD" / "Graphics.Lights", CarPalette 0x0010E3F0)
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct CarRenderTweaks {
    /// `byte_7065BB` "Force Car LOD", `dword_7065BC` "Force LOD Level" (bss: off, 0)
    pub force_lod: bool,
    pub force_lod_level: i32,
    /// `flt_6866A0` "Point Light Intensity"
    pub point_light_intensity: f32,
}

impl Default for CarRenderTweaks {
    fn default() -> Self {
        Self { force_lod: false, force_lod_level: 0, point_light_intensity: crate::vs_render::CAR_POINT_LIGHT_SCALE }
    }
}

/// CarRender.cpp 0x00111920 — LOD 0/1/2 from the squared camera distance scaled by (fov / 90°)²
/// (`vfov` = VsBaseCamera +0x88, 90° without a camera)
pub fn select_lod(car: Vec3, cam: Vec3, cam_fwd: Vec3, vfov: Option<f32>) -> u8 {
    let d = cam - car;
    let fov = vfov.map_or(90.0, |f| f * 57.295776);
    let d2 = (fov * fov) * d.length_squared() / 8100.0;
    let dot = d.normalize_or_zero().dot(cam_fwd);
    let k = if (-0.5..=0.5).contains(&dot) { 0 } else { 1 };
    if d2 >= LOD_FAR[k] {
        2
    } else if d2 >= LOD_NEAR[k] {
        1
    } else {
        0
    }
}

/// 0x00112A50 brake/reverse level ramp, clamped to 0..1
fn ramp(v: f32, on: bool, step: f32) -> f32 {
    if on {
        (v + step).min(1.0)
    } else {
        (v - step).max(0.0)
    }
}

fn show(vis: &mut Query<&mut Visibility>, e: Entity, on: bool) {
    if let Ok(mut v) = vis.get_mut(e) {
        let want = if on { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}

/// per rendered frame: CarRender.cpp 0x00119EF0 (LOD select, 0x00111920) and the light pass of 0x00112A50 — brake
/// level towards the brake input (0.5 without a vehicle; `_l2_1` cars lifted to 0.3 + 0.7·v), written as the TFACTOR
/// (v,v,v,v) of the car's BrakeLight shader (CarShaderManager 0x001589F0 → 0x00158000) for each brake light drawn
/// (0 for a broken one; the shader is shared, so the last one set — the right — wins at draw time, as in the
/// original), brake/headlight primitives drawn while the LOD is not 2, and the reverse lights redrawn with
/// "revlight" at TFACTOR alpha = level·128 doubled (VISOS_ColourXBOX 0x002B11C0). The original latches the first
/// car's reverse texture into the shared revlight shader; here each car keeps its own.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_car_lights(
    mut commands: Commands,
    time: Res<Time>,
    camera: Query<(&GlobalTransform, Option<&Projection>), With<crate::track_render::TrackCamera>>,
    views: Query<(&GlobalTransform, Option<&Projection>, &crate::view_render::TrackView), With<crate::view_render::SecondaryTrackCamera>>,
    split: Option<Res<crate::view_render::SplitScreen>>,
    mirror: Query<(&GlobalTransform, Option<&Projection>, &crate::view_render::MirrorView), Without<crate::track_render::TrackCamera>>,
    head: Option<Res<crate::track_fx::HeadlightView>>,
    head_views: Option<Res<crate::track_fx::HeadlightViews>>,
    mut cars: Query<(
        Entity,
        &CarRender,
        &GlobalTransform,
        &mut CarLod,
        &mut CarLights,
        Option<&CarLightInput>,
        Option<&crate::car_damage_render::CarLightDamage>,
        Option<&mut crate::view_render::CarViewLod>,
    )>,
    mut vis: Query<&mut Visibility>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    children: Query<&Children>,
    mesh_q: Query<(), With<Mesh3d>>,
    tweaks: Option<Res<CarRenderTweaks>>,
) {
    // 0x00111920: "Force Car LOD" outside the front end (renderer +49) returns "Force LOD Level" as is
    let force = tweaks.as_ref().filter(|t| t.force_lod).map(|t| t.force_lod_level);
    let select_lod = |car: Vec3, cam: Vec3, fwd: Vec3, fov: Option<f32>, frontend: bool| match force {
        Some(l) if !frontend => l.clamp(0, 2) as u8,
        _ => select_lod(car, cam, fwd, fov),
    };
    let fov_of = |p: Option<&Projection>| match p {
        Some(Projection::Perspective(p)) => Some(p.fov),
        _ => None,
    };
    let cam = camera.iter().next().map(|(t, p)| (t.translation(), t.forward().as_vec3(), fov_of(p)));
    // split screen: CarRender 0x00119900 runs inside each view with that view's camera, so each view picks its own
    // LOD; here the groups are shown to the views that picked them through `VIEW_CAR_LAYERS`
    let mut view_cams: Vec<(usize, (Vec3, Vec3, Option<f32>))> = Vec::new();
    if split.as_ref().is_some_and(|s| s.on()) {
        if let Some(c) = cam {
            view_cams.push((0, c));
        }
        for (t, p, v) in &views {
            view_cams.push((v.player.min(1), (t.translation(), t.forward().as_vec3(), fov_of(p))));
        }
    } else if split.as_ref().is_some_and(|s| s.mirror.is_some()) {
        // the mirror view runs CarRender 0x00119900 with the mirror camera: the second view slot
        if let (Some(c), Some((t, p, _))) = (cam, mirror.iter().find(|m| m.2.on)) {
            view_cams.push((0, c));
            view_cams.push((1, (t.translation(), t.forward().as_vec3(), fov_of(p))));
        }
    }
    let step = LIGHT_STEP * time.delta_secs() * XBOX_FRAME_RATE;
    // CarRender 0x001127F0 per view: the car the view's camera sits in is culled (+2251) — no body, wheels or lights
    // in that view (view slot 0 = player 0's, 1 = player 1's or the mirror's)
    let head = head.as_deref().copied().unwrap_or_default();
    let hv = head_views.as_deref().copied().unwrap_or_default();
    let inside1 = if split.as_ref().is_some_and(|s| s.on()) { hv.player1_view().inside } else { hv.mirror_view(&head).inside };
    let inside_of = |p: usize| if p == 0 { head.inside } else { inside1 };
    for (e, r, t, mut lod, mut lights, input, light_damage, view_lod) in &mut cars {
        let l = match cam {
            Some((p, f, fov)) if lod.lods.len() > 1 => select_lod(t.translation(), p, f, fov, r.spec.frontend),
            _ => 0,
        };
        let req = Some(r.request);
        let in_car = if view_cams.len() > 1 { view_cams.iter().all(|(p, _)| inside_of(*p) == req) } else { head.inside == req };
        let mut on = l != 2 && !in_car;
        if view_cams.len() > 1 && lod.lods.len() > 1 {
            let sel: Vec<(usize, u8)> = view_cams
                .iter()
                .filter(|(p, _)| inside_of(*p) != req)
                .map(|(p, (cp, cf, fov))| (*p, select_lod(t.translation(), *cp, *cf, *fov, r.spec.frontend)))
                .collect();
            let masks = crate::view_render::CarViewLod::from_selection(&sel, lod.lods.len());
            on = masks.lights != 0;
            let changed = view_lod.as_deref() != Some(&masks);
            if changed {
                for (k, g) in lod.lods.iter().enumerate() {
                    show(&mut vis, *g, masks.lods[k] != 0);
                    crate::view_render::set_view_layers(&mut commands, &children, &mesh_q, *g, masks.lods[k]);
                }
                for (k, ws) in lod.wheel_lods.iter().enumerate() {
                    let m = masks.lods.get(k).copied().unwrap_or(0);
                    for w in ws {
                        show(&mut vis, *w, m != 0);
                        crate::view_render::set_view_layers(&mut commands, &children, &mesh_q, *w, m);
                    }
                }
                crate::view_render::set_view_layers(&mut commands, &children, &mesh_q, lights.group, masks.lights);
                match view_lod {
                    Some(mut v) => *v = masks,
                    None => {
                        commands.entity(e).insert(masks);
                    }
                }
            }
            lod.lod = l;
            lod.in_car = in_car;
        } else if l != lod.lod || lod.is_added() || view_lod.is_some() || in_car != lod.in_car {
            if view_lod.is_some() {
                // back to one view: the groups lose their view layers (`layer_cars` puts `CAR_LAYER` back)
                let groups = lod.lods.iter().chain(lod.wheel_lods.iter().flatten()).chain(std::iter::once(&lights.group));
                for g in groups {
                    crate::view_render::clear_view_layers(&mut commands, &children, &mesh_q, *g);
                }
                commands.entity(e).remove::<crate::view_render::CarViewLod>();
            }
            for (k, g) in lod.lods.iter().enumerate() {
                show(&mut vis, *g, k == l as usize && !in_car);
            }
            for (k, ws) in lod.wheel_lods.iter().enumerate() {
                for w in ws {
                    show(&mut vis, *w, k == l as usize && !in_car);
                }
            }
            lod.lod = l;
            lod.in_car = in_car;
        }
        show(&mut vis, lights.group, on);
        if !on {
            continue;
        }
        let damage = input.map_or(0, |i| i.damage) | light_damage.map_or(0, |d| d.0);
        let mut v = 0.5;
        if let Some(i) = input {
            lights.brake_level = ramp(lights.brake_level, i.braking, step);
            lights.reverse_level = ramp(lights.reverse_level, i.reversing, step);
            v = lights.brake_level;
        }
        if lights.l2 {
            v = (1.0 - L2_FLOOR) * v + L2_FLOOR;
        }
        let mut tfactor = None;
        for k in 0..2 {
            let broken = damage & (4 << k) != 0;
            let draw = lights.brake[k].is_some() && !(lights.l2 && broken);
            if let Some(e) = lights.brake[k] {
                show(&mut vis, e, draw);
            }
            if draw {
                tfactor = Some(if broken { 0.0 } else { v });
            }
        }
        if let Some(tf) = tfactor {
            let c = (crate::car_palette::visos_colour(tf, tf, tf, tf) & 0xFF) as f32 / 255.0;
            for h in &lights.brake_materials {
                if let Some(m) = materials.get(h) {
                    if m.params.colour != Vec4::splat(c) {
                        materials.get_mut(h).unwrap().params.colour = Vec4::splat(c);
                    }
                }
            }
        }
        let r = lights.reverse_level;
        let a = ((2 * ((r * 128.0) as i32 as u8) as u32).min(255)) as f32 / 255.0;
        for k in 0..2 {
            if let Some((e, m)) = &lights.reverse[k] {
                let draw = r != 0.0 && damage & (4 << k) == 0;
                show(&mut vis, *e, draw);
                if draw && materials.get(m).is_some_and(|x| x.params.colour.w != a) {
                    materials.get_mut(m).unwrap().params.colour.w = a;
                }
            }
        }
    }
}

/// CarShaderManager_PC_XBOX.cpp 0x00158280 — the shared car textures: `EnvMapTest\envmap1` (refmap, a 132x132
/// tga, so linear) and `Effect_glitter_002_128` (noise).
#[derive(Resource, Clone, Default)]
pub struct CarShaderTextures(pub CarSetTextures);

fn load_car_textures(images: &mut Assets<Image>) -> CarSetTextures {
    let mut tex = TexSet::default();
    if let Some(t) = visos::file::platform::read("\\EnvMapTest\\ENVMAP1.TGA").ok().and_then(|d| crate::renderer::d3dx_tga::decode_tga(&d)) {
        tex.add_d3dx("envmap1", t);
    }
    add_vcd_textures(&mut tex, "\\VFX\\misc.vcd");
    CarSetTextures {
        refmap: tex.program_texture("envmap1", 1, 1, images),
        noise: tex.program_texture("effect_glitter_002_128", 1, 1, images),
        dynamic_envmap: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn process_car_messages(
    mut commands: Commands,
    mut spawn: MessageReader<SpawnCarRender>,
    mut despawn: MessageReader<DespawnCarRender>,
    cars: Query<(Entity, &CarRender)>,
    lights: Option<Res<crate::track_render::TrackLights>>,
    car_tex: Option<Res<CarShaderTextures>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut programs: ResMut<Assets<ProgramMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for d in despawn.read() {
        for (e, c) in &cars {
            if c.request == d.request {
                commands.entity(e).despawn();
            }
        }
    }
    let reqs: Vec<SpawnCarRender> = spawn.read().cloned().collect();
    if reqs.is_empty() {
        return;
    }
    if car_tex.is_none() {
        commands.insert_resource(CarShaderTextures(load_car_textures(&mut images)));
    }
    let l = lights.map(|l| l.lights).unwrap_or_default();
    for r in reqs {
        let mut b = Builder::new(&mut meshes, &mut materials, &mut images);
        b.lights = l;
        b.car = Some(CarProgramCtx::new(&mut programs, r.spec.frontend));
        spawn_car(&mut commands, &mut b, &r.spec, r.request, r.transform);
    }
}

/// per-frame binding of the car shader sets: CarRender.cpp lighting (0x00119900 -> VsRender.cpp 0x00059140, the
/// two nearest point lights swapped in), CarRender.cpp 0x00119EF0 (neon scale), CarPalette_PC_XBOX.cpp 0x001570F0
/// / 0x001573E0 (paint, pearl, pearlescent matrix), CarPalette.cpp 0x0010CE90 (envmap matrix), then the program
/// inputs (ShaderProgramManager.cpp 0x002DF7F0 / 0x002DFBB0).
#[allow(clippy::too_many_arguments)]
fn bind_car_programs(
    lights: Option<Res<crate::track_render::TrackLights>>,
    points: Option<Res<crate::track_render::TrackPointLights>>,
    car_tex: Option<Res<CarShaderTextures>>,
    envmap: Option<Res<crate::car_shader_manager_pc_xbox::DynamicEnvmap>>,
    mut globals: ResMut<GlobalMatInputResolver>,
    camera: Query<&GlobalTransform, With<crate::track_render::TrackCamera>>,
    mut cars: Query<(&GlobalTransform, &CarRender, &mut CarRenderParts, Option<&CarLod>)>,
    mut programs: ResMut<Assets<ProgramMaterial>>,
    mut sets: ResMut<CarLightSets>,
    tweaks: Option<Res<CarRenderTweaks>>,
    mut pending: ResMut<PendingCarPrograms>,
) {
    sets.0.clear();
    pending.0.clear();
    let defer = !crate::golden::perf_off("carvis");
    let Some(car_tex) = car_tex else { return };
    let point_scale = tweaks.as_ref().map_or(crate::vs_render::CAR_POINT_LIGHT_SCALE, |t| t.point_light_intensity);
    let l0 = lights.as_ref().map(|l| l.lights).unwrap_or_default();
    let fog = lights.as_ref().map(|l| l.fog).unwrap_or(Vec4::ZERO);
    let cam = camera.iter().next().copied().unwrap_or_default();
    let (cam_pos, cam_dir, cam_up) = (cam.translation(), cam.forward().as_vec3(), cam.up().as_vec3());
    // DangerZoneRender 0x00122AA0: the "Envmap" target the sets read while the dynamic envmap is on (+57)
    let dynamic = envmap.as_ref().and_then(|e| e.program_texture());
    let mut set_tex = car_tex.0.clone();
    set_tex.dynamic_envmap = dynamic.clone();
    for (t, car, mut parts, lod) in &mut cars {
        let spec = &car.spec;
        let pts = points.as_ref().map_or(&[][..], |p| &p.points[..]);
        let mut slots = parts.light_slots;
        let search = lod.is_none_or(|lod| lod.lod != 2);
        let l = l0.car_light_set(pts, t.translation(), &mut slots, search, spec.frontend, point_scale);
        parts.light_slots = slots;
        // CarRender.cpp 0x00112A50 -> 0x001123E0 after the car is drawn: what the 3d hud (and beams) copy later
        let after = if lod.is_some_and(|lod| lod.in_car) {
            l
        } else {
            l.car_point_light_set(pts, t.translation(), crate::vs_render::CAR_BRIGHTNESS).0
        };
        sets.0.insert(car.request, after);
        globals.set_vector(gmi::NEON_UNDER_GLOW_SCALE, Vec4::splat(if spec.frontend { 1.0 } else { 0.0 }));
        let params = paint_params(spec.frontend, false, spec.paint_type);
        let (paint, pearl) = paint_colours(spec.colour.extend(1.0), spec.pearl.extend(1.0), spec.paint_type, params);
        let state = CarPaintState {
            frontend: spec.frontend,
            paint_type: spec.paint_type,
            paint,
            pearl,
            // CarPalette.cpp 0x0010CE90: DangerZoneRender 0x001218A0 while the dynamic envmap is on, else 0x00114D80
            envmap: if dynamic.is_some() {
                crate::danger_zone_render::dynamic_envmap::envmap_matrix(t.to_matrix(), cam_pos, cam_up)
            } else {
                envmap_matrix(t.to_matrix(), cam_pos)
            },
            pearlescent: pearlescent_matrix(cam_dir, params),
        };
        for p in &parts.programs {
            let inputs = match p.set {
                CarShaderSet::CarDecal => car_decal_inputs(&state, &set_tex),
                _ => shiny_car_inputs(&state, &set_tex),
            };
            if defer {
                let Some(cur) = programs.get(&p.material) else { continue };
                let mut n = cur.clone();
                n.params.fog = fog;
                n.bind_inputs(&inputs, &ObjectInputs { lights: l, base: p.base.clone() }, &globals);
                pending.0.push((p.material.clone(), n.binding()));
                continue;
            }
            ProgramMaterial::rebind(&mut programs, &p.material, |m| {
                m.params.fog = fog;
                m.bind_inputs(&inputs, &ObjectInputs { lights: l, base: p.base.clone() }, &globals);
            });
        }
    }
}

/// perf (`VELOCITY_PERF_OFF=carvis` = write in `bind_car_programs`): this frame's car program bindings, written after
/// visibility only for materials some visible entity draws. Bevy re-prepares a 3.4 KB uniform + bind group per write and
/// moving cars change every frame; an unseen material keeps stale constants until it is seen again (written that frame).
#[derive(Resource, Default)]
pub struct PendingCarPrograms(pub Vec<(Handle<ProgramMaterial>, crate::renderer::shader_program_manager::ProgramBinding)>);

fn flush_car_programs(
    mut pending: ResMut<PendingCarPrograms>,
    seen: Query<(&MeshMaterial3d<ProgramMaterial>, &ViewVisibility)>,
    mut programs: ResMut<Assets<ProgramMaterial>>,
    mut events: MessageWriter<AssetEvent<ProgramMaterial>>,
) {
    if pending.0.is_empty() {
        return;
    }
    let visible: std::collections::HashSet<AssetId<ProgramMaterial>> = seen.iter().filter(|(_, v)| v.get()).map(|(m, _)| m.0.id()).collect();
    for (h, b) in pending.0.drain(..) {
        if visible.contains(&h.id()) && programs.get(&h).is_some_and(|m| !m.same_binding(&b)) {
            // runs after `AssetEventSystems` (visibility needs mesh bounds, which need asset events): `get_mut` would
            // only announce the change next frame. The event goes straight to this frame's extract; no `AssetChanges`
            // entry, so the meshes are not re-specialised (the pipeline key, `ProgramKey`, is not part of the binding)
            if let Some(m) = programs.get_mut_untracked(&h) {
                m.apply_binding(b);
                events.write(AssetEvent::Modified { id: h.id() });
            }
        }
    }
}

/// one decal slot: DECALS.BIN record name (+0x18) and the setup's colour flags (1 `_white`, 2 `_black`)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecalRef {
    pub name: String,
    pub flags: u32,
}

/// the decal slots of a setup (SVehicleSetup +0x194..+0x228, CarRender.cpp 0x00116510 reads them as `a3[101..138]`)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CarDecals {
    /// +0x194 bonnet decal ("bonnet_decal" shaders of the bonnet part, all LODs)
    pub bonnet: Option<String>,
    /// +0x198 side decal ("side_decal" shaders of the chassis, all LODs)
    pub side: Option<String>,
    /// +0x19C / flags +0x1C4: chassis `logo01`..`logo10`
    pub chassis: [Option<DecalRef>; 10],
    /// +0x1EC / +0x1F4: front bumper `logo01`/`logo02`
    pub f_bumper: [Option<DecalRef>; 2],
    /// +0x1FC / +0x204: rear bumper
    pub r_bumper: [Option<DecalRef>; 2],
    /// +0x20C / +0x214: skirt (`Trim` models)
    pub skirt: [Option<DecalRef>; 2],
    /// +0x21C / +0x220: chassis `logofw` (front screen)
    pub front_screen: Option<DecalRef>,
    /// +0x224 / +0x228: chassis `logorw` (rear screen), else the spoiler's
    pub rear_screen: Option<DecalRef>,
}

impl CarDecals {
    /// the setup's decal slots, by DECALS.BIN record name
    pub fn from_setup(setup: &velocity_physics::vehicle_database::VehicleSetup, db: &velocity_physics::vehicle_database::CVehicleDatabase) -> Self {
        let name = |d: Option<usize>| d.and_then(|i| db.decals.get(i)).and_then(|d| d.name.clone());
        let r = |d: Option<usize>, flags: u32| name(d).map(|name| DecalRef { name, flags });
        let mut c = Self { bonnet: name(setup.decal_0), side: name(setup.decal_1), ..default() };
        for i in 0..10 {
            c.chassis[i] = r(setup.decals_3[i], setup.decals_3_flags[i]);
        }
        for i in 0..2 {
            c.f_bumper[i] = r(setup.decals_3_1[i], setup.decals_3_1_flags[i]);
            c.r_bumper[i] = r(setup.decals_3_2[i], setup.decals_3_2_flags[i]);
            c.skirt[i] = r(setup.decals_3_3[i], setup.decals_3_3_flags[i]);
        }
        c.front_screen = r(setup.decal_2_4, setup.decal_2_4_flags);
        c.rear_screen = r(setup.decal_2_5, setup.decal_2_5_flags);
        c
    }
}

/// the models of one spawned car: `stems[k]` (0 = chassis, then the fitted parts `<slot><n>`, then the front end's
/// doors) and per LOD the primitive records with each stem's range
pub struct CarModels<'a> {
    pub stems: &'a [String],
    pub recs: &'a [Vec<PrimRecord>],
    pub ranges: &'a [Vec<std::ops::Range<usize>>],
}

impl CarModels<'_> {
    /// the records of stem `k` at `lod` (empty when the model or LOD is absent)
    pub fn model(&self, k: usize, lod: usize) -> &[PrimRecord] {
        match (self.recs.get(lod), self.ranges.get(lod).and_then(|r| r.get(k))) {
            (Some(recs), Some(r)) => &recs[r.clone()],
            _ => &[],
        }
    }

    /// the stem index of a fitted part (`PART_SLOTS` name, case-insensitive)
    pub fn part(&self, slot: &str) -> Option<usize> {
        self.stems.iter().position(|s| {
            s.len() > slot.len() && s[..slot.len()].eq_ignore_ascii_case(slot) && s[slot.len()..].bytes().all(|c| c.is_ascii_digit())
        })
    }

    /// CarRender data +1184: the front end's extra models (FE3DVehicles 0x0025D600 sets them; zero in game)
    fn extras(&self) -> Vec<usize> {
        (0..self.stems.len()).filter(|&k| self.stems[k].ends_with("_door")).collect()
    }
}

/// CarRender.cpp 0x00116280 — a decal's texture: record name, `_white` for flag 1, `_black` for flag 2 (loaded from
/// `\Vehicles\DECALST.VCD`)
pub fn decal_texture(name: &str, flags: u32) -> String {
    let s = match flags {
        1 => "_white",
        2 => "_black",
        _ => "",
    };
    format!("{name}{s}").to_ascii_lowercase()
}

/// a car primitive's shader type (GameDataLoading.cpp 0x0009EAF0; the primitive record's +4)
fn prim_type(r: &PrimRecord, frontend: bool) -> u32 {
    crate::car_palette_pc_xbox::car_shader_type(&r.shader_name, r.kind, frontend)
}

/// set the stage texture of shader `rec.shader_name` (every primitive of the model using it): the decal shaders
/// (types 4/5) are CarDecal programs, so a program material with the new base texture
fn set_shader_texture(commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, model: &[PrimRecord], shader: &str, key: &str, frontend: bool) {
    for r in model.iter().filter(|r| r.shader_name == shader) {
        let ty = prim_type(r, frontend);
        match crate::car_palette_pc_xbox::shader_set_for(ty, &r.shader_name) {
            Some(set @ (CarShaderSet::CarDecal | CarShaderSet::ShinyCar)) => {
                let base = tex.program_texture(key, r.state.address_u, r.state.address_v, b.images);
                if base.is_none() {
                    log::warn!("decal texture {key} not found");
                }
                if let Some(c) = b.car.as_mut() {
                    let h = c.program(set, key, base);
                    commands.entity(r.entity).insert(MeshMaterial3d(h));
                }
            }
            _ => log::warn!("decal shader {} (type {ty}) is not a CarDecal", r.shader_name),
        }
    }
}

/// CarRender.cpp 0x00115CC0 — every type 4 shader whose name contains `name` gets texture `key`
fn decal_model(commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, model: &[PrimRecord], name: &str, key: &str, frontend: bool) -> bool {
    let shaders: Vec<String> = model
        .iter()
        .filter(|r| prim_type(r, frontend) == 4 && r.shader_name.contains(name))
        .map(|r| r.shader_name.clone())
        .collect();
    for s in &shaders {
        set_shader_texture(commands, b, tex, model, s, key, frontend);
    }
    !shaders.is_empty()
}

/// FuryInstance.cpp 0x0012AC80 — the first primitive of type `ty` whose shader name contains `name`
fn find_shader<'a>(model: &'a [PrimRecord], name: &str, ty: u32, frontend: bool) -> Option<&'a str> {
    model.iter().find(|r| prim_type(r, frontend) == ty && r.shader_name.contains(name)).map(|r| r.shader_name.as_str())
}

/// CarRender.cpp 0x00115BB0 — `bonnet_decal` (kind 0: the BONNET part, LODs 0-2) or `side_decal` (kind 1: the
/// chassis, LODs 0-2), then the front end's extra models
fn bonnet_side_decal(commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, m: &CarModels, kind: usize, key: &str, frontend: bool) {
    // off_6867D4
    let name = ["bonnet_decal", "side_decal"][kind];
    let stem = if kind != 0 { Some(0) } else { m.part("bonnet") };
    if let Some(k) = stem {
        for lod in 0..3 {
            decal_model(commands, b, tex, m.model(k, lod), name, key, frontend);
        }
    }
    for k in m.extras() {
        decal_model(commands, b, tex, m.model(k, 0), name, key, frontend);
    }
}

/// CarRender.cpp 0x00116AD0 — the LOD 0 model of decal kind 0 chassis, 1 front bumper, 2 rear bumper, 3 skirt,
/// 4/5 chassis (the name's digits become `fw` / `rw`)
fn logo_model(m: &CarModels, kind: u32, name: &mut String) -> Option<usize> {
    match kind {
        1 => m.part("f_bumper"),
        2 => m.part("r_bumper"),
        3 => m.part("trim"),
        4 => {
            name.replace_range(4..6, "fw");
            Some(0)
        }
        5 => {
            name.replace_range(4..6, "rw");
            Some(0)
        }
        _ => Some(0),
    }
}

/// CarRender.cpp 0x00116040 — logo `index` of decal kind `kind`: the first type 5 shader named `logo%02d` (index + 1)
/// in the kind's model gets texture `key`; kind 5 falls back to the spoiler, kind 0 to the front end's extra models.
/// (`logofw`/`logorw` materials classify as type 2, never 5, so the screen strips stay as modelled — kept.)
fn logo_decal(commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, m: &CarModels, kind: u32, index: u32, key: &str, frontend: bool) {
    let mut name = format!("logo{:02}", index + 1);
    let Some(k) = logo_model(m, kind, &mut name) else { return };
    let mut hit = find_shader(m.model(k, 0), &name, 5, frontend).map(|s| (k, s.to_string()));
    if hit.is_none() && kind == 5 {
        if let Some(sp) = m.part("spoiler") {
            hit = find_shader(m.model(sp, 0), &name, 5, frontend).map(|s| (sp, s.to_string()));
        }
    }
    match hit {
        Some((k, s)) => set_shader_texture(commands, b, tex, m.model(k, 0), &s, key, frontend),
        None if kind == 0 => {
            for k in m.extras() {
                if let Some(s) = find_shader(m.model(k, 0), &name, 5, frontend).map(str::to_string) {
                    set_shader_texture(commands, b, tex, m.model(k, 0), &s, key, frontend);
                }
            }
        }
        None => {}
    }
}

/// CarRender.cpp 0x00116BC0 — how many logo slots decal kind `kind` (0..5) has on this car: the `logo%02d` (1..10)
/// type 5 shaders found in the kind's model (kind 0 also in the front end's extra models); kinds 4/5 count the
/// single `logofw`/`logorw` shader (kind 5 plus the spoiler's). 0 for traffic cars.
pub fn decal_slot_count(m: &CarModels, kind: u32, frontend: bool, traffic: bool) -> u8 {
    if traffic {
        return 0;
    }
    let mut name = "logo00".to_string();
    let Some(k) = logo_model(m, kind, &mut name) else { return 0 };
    let mut n = 0u8;
    if kind == 4 || kind == 5 {
        n += find_shader(m.model(k, 0), &name, 5, frontend).is_some() as u8;
        if kind == 5 {
            if let Some(sp) = m.part("spoiler") {
                n += find_shader(m.model(sp, 0), &name, 5, frontend).is_some() as u8;
            }
        }
        return n;
    }
    for i in 1..=10u32 {
        let name = format!("logo{i:02}");
        let mut hit = find_shader(m.model(k, 0), &name, 5, frontend).is_some();
        if !hit && kind == 0 {
            hit = m.extras().iter().any(|&e| find_shader(m.model(e, 0), &name, 5, frontend).is_some());
        }
        n += hit as u8;
    }
    n
}

/// CarRender.cpp 0x001163A0 (`\Vehicles\DECALST.VCD`) -> 0x00116510 — the setup's decals onto one car; an empty
/// slot gets "bonnet_off". Traffic cars (vehicle type 6/7) are skipped (they never get here).
fn apply_car_decals(commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, spec: &CarSpec, m: &CarModels) {
    let off = "bonnet_off";
    let d = &spec.decals;
    let fe = spec.frontend;
    let key = |r: &Option<DecalRef>| r.as_ref().map(|r| decal_texture(&r.name, r.flags)).unwrap_or_else(|| off.to_string());
    bonnet_side_decal(commands, b, tex, m, 0, &d.bonnet.as_deref().map(|n| decal_texture(n, 0)).unwrap_or_else(|| off.into()), fe);
    bonnet_side_decal(commands, b, tex, m, 1, &d.side.as_deref().map(|n| decal_texture(n, 0)).unwrap_or_else(|| off.into()), fe);
    for (i, r) in d.chassis.iter().enumerate() {
        logo_decal(commands, b, tex, m, 0, i as u32, &key(r), fe);
    }
    for (kind, slots) in [(1, &d.f_bumper), (2, &d.r_bumper), (3, &d.skirt)] {
        for (i, r) in slots.iter().enumerate() {
            logo_decal(commands, b, tex, m, kind, i as u32, &key(r), fe);
        }
    }
    logo_decal(commands, b, tex, m, 4, 0, &key(&d.front_screen), fe);
    logo_decal(commands, b, tex, m, 5, 0, &key(&d.rear_screen), fe);
}

/// `dword_686730` / `flt_686734` / `dword_686738`: the dummy's wheel offsets (x, y, z in the car frame)
pub const DUMMY_WHEEL_OFFSET: Vec3 = Vec3::new(f32::from_bits(0x3F4C_CCCD), f32::from_bits(0xBDCC_CCCD), f32::from_bits(0x3FA6_6666));
/// `dword_68673C..686750`: the dummy's two top lights (+816 / +832), whatever the police car's locators are
pub const DUMMY_TOP_LIGHTS: [Vec3; 2] = [
    Vec3::new(f32::from_bits(0xBECC_CCCD), f32::from_bits(0x3FB3_3333), f32::from_bits(0xBE99_999A)),
    Vec3::new(f32::from_bits(0x3ECC_CCCD), f32::from_bits(0x3FB3_3333), f32::from_bits(0xBE99_999A)),
];
/// +2344 bytes FF 70 00 80 / +2348 bytes 00 00 FF 80: blue and red (D3DCOLOR)
pub const DUMMY_TOP_COLOURS: [u32; 2] = [0x8000_70FF, 0x80FF_0000];
/// 0x001192F0: the top lights only within 200 m (squared 40000)
pub const DUMMY_LIGHT_RANGE_SQ: f32 = 40000.0;

/// 0x001192F0 wheel i: x mirrored for wheels 1 and 2, z for 2 and 3 (car-frame offset; the dummy's wheels keep the
/// car's rotation — no left/right turn, original)
pub fn dummy_wheel_offset(i: usize) -> Vec3 {
    let mut o = DUMMY_WHEEL_OFFSET;
    if i >= 2 {
        o.z = 0.0 - o.z;
    }
    if i == 1 || i == 2 {
        o.x = 0.0 - o.x;
    }
    o
}

/// one "propld_dummycopcar" physics object to draw this frame (PhysicsObjectParticleTracker 0x001E3B70: type tag 1,
/// visible); `key` stands in for the object address the original derives the light phase from (`addr >> 6`)
#[derive(Clone, Copy, Debug)]
pub struct DummyPoliceCar {
    pub key: u64,
    pub matrix: Mat4,
}

/// the dummy police cars of this frame (core fills it every frame; the objects themselves are not drawn)
#[derive(Resource, Clone, Debug, Default)]
pub struct DummyPoliceCars(pub Vec<DummyPoliceCar>);

/// spawned copies: key -> root, and the police car they were copied from
#[derive(Resource, Default)]
pub struct DummyPoliceClones {
    pub roots: std::collections::HashMap<u64, Entity>,
    pub source: Option<Entity>,
    warned: bool,
}

/// CarRender.cpp 0x00119250: the first car (render order) with +2328 bit 0x4000
pub fn police_car<'a, T>(cars: impl Iterator<Item = (&'a CarRender, T)>) -> Option<(&'a CarRender, T)> {
    let mut v: Vec<_> = cars.filter(|(r, _)| !r.spec.frontend).collect();
    v.sort_by_key(|(r, _)| r.request);
    v.into_iter().find(|(r, _)| r.spec.police)
}

/// CarRender.cpp 0x001192F0 "RenderDummyPoliceCar" — a copy of the police car's data (0x0011BA70) at the object
/// matrix: LOD 0 chassis + parts at it, the four LOD 0 wheels at `dummy_wheel_offset` (DrawList 0x002C23C0, no
/// lights, shadow or LOD switch). No police car: "couldn't find an index to a police car" and nothing is drawn. The
/// top lights are `flare_render::dummy_top_lights`.
fn render_dummy_police_cars(
    mut commands: Commands,
    dummies: Res<DummyPoliceCars>,
    mut st: ResMut<DummyPoliceClones>,
    cars: Query<(Entity, &CarRender, &CarLod)>,
    mut roots: Query<&mut Transform>,
) {
    let source = police_car(cars.iter().map(|(e, r, l)| (r, (e, l)))).map(|(_, x)| x);
    if st.source != source.map(|s| s.0) {
        for (_, e) in st.roots.drain() {
            commands.entity(e).try_despawn();
        }
        st.source = source.map(|s| s.0);
    }
    if source.is_none() && !dummies.0.is_empty() && !st.warned {
        log::warn!("ERROR: CGenericCarRender::RenderDummyPoliceCar - couldn't find an index to a police car!!");
    }
    st.warned = source.is_none() && !dummies.0.is_empty();
    st.roots.retain(|k, e| {
        let keep = dummies.0.iter().any(|d| d.key == *k);
        if !keep {
            commands.entity(*e).try_despawn();
        }
        keep
    });
    let Some((_, lod)) = source else { return };
    for d in &dummies.0 {
        let t = Transform::from_matrix(d.matrix);
        if let Some(&e) = st.roots.get(&d.key) {
            if let Ok(mut rt) = roots.get_mut(e) {
                if *rt != t {
                    *rt = t;
                }
            }
            continue;
        }
        let root = commands.spawn((t, Visibility::Visible)).id();
        if let Some(&g) = lod.lods.first() {
            commands.entity(g).clone_and_spawn_with_opt_out(|b| {
                b.linked_cloning(true);
            }).insert((ChildOf(root), Visibility::Inherited));
        }
        if let Some(w) = lod.wheel_lods.first() {
            for (i, &g) in w.iter().enumerate() {
                let pivot = commands.spawn((Transform::from_translation(dummy_wheel_offset(i)), Visibility::Inherited, ChildOf(root))).id();
                commands.entity(g).clone_and_spawn_with_opt_out(|b| {
                    b.linked_cloning(true);
                }).insert((ChildOf(pivot), Transform::default(), Visibility::Inherited));
            }
        }
        st.roots.insert(d.key, root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 0x001192F0: wheels 0 (+x, +z), 1 (−x, +z), 2 (−x, −z), 3 (+x, −z), all 0.1 down
    #[test]
    fn dummy_wheel_offsets() {
        let o: Vec<Vec3> = (0..4).map(dummy_wheel_offset).collect();
        assert_eq!(o[0], Vec3::new(0.8, -0.1, 1.3));
        assert_eq!(o[1], Vec3::new(-0.8, -0.1, 1.3));
        assert_eq!(o[2], Vec3::new(-0.8, -0.1, -1.3));
        assert_eq!(o[3], Vec3::new(0.8, -0.1, -1.3));
    }

    fn shadow_car(touch: [bool; 4]) -> (Mat4, CarShadow, ShadowVehicle) {
        let m = Mat4::from_translation(Vec3::new(10.0, 1.0, 20.0));
        let ground = [Vec3::new(1.0, 0.2, 1.5), Vec3::new(-1.0, 0.3, 1.5), Vec3::new(-1.0, 0.4, -1.5), Vec3::new(1.0, 0.5, -1.5)];
        let ve = ShadowVehicle { ground: ground.map(|g| g + Vec3::new(10.0, 0.0, 20.0)), touches: touch, race_car: true, flag_4388: false };
        (m, CarShadow::new([2.0, 4.0], 0.6, None), ve)
    }

    #[test]
    fn shadow_quad_corners() {
        let (m, mut sh, ve) = shadow_car([true; 4]);
        sh.level = 0.5;
        let mut ray = |_: Vec3, _: Vec3| -> Option<f32> { panic!("no ray with all wheels down") };
        let q = shadow_quad(m, &mut sh, Some(&ve), true, 0xFF00_0000, 0.2, &mut ray).unwrap();
        // c0 = pos + z·L/2 + x·W/2 (x scaled by −W), then +x, −z, −z+x; y from wheels 3, 0, 2, 1 + 0.01
        let p: Vec<Vec3> = q.iter().map(|v| v.pos).collect();
        assert_eq!(p[0], Vec3::new(11.0, 0.5 + SHADOW_LIFT, 22.0));
        assert_eq!(p[1], Vec3::new(9.0, 0.2 + SHADOW_LIFT, 22.0));
        assert_eq!(p[2], Vec3::new(11.0, 0.4 + SHADOW_LIFT, 18.0));
        assert_eq!(p[3], Vec3::new(9.0, 0.3 + SHADOW_LIFT, 18.0));
        assert_eq!(q.map(|v| v.uv), [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE]);
        assert!((sh.level - 0.7).abs() < 1e-6);
        // inner quad: 0.8 length, 0.9 width
        let q = shadow_quad(m, &mut sh, Some(&ve), false, 0, 0.2, &mut ray).unwrap();
        assert!((q[0].pos.x - (10.0 + 0.9)).abs() < 1e-5 && (q[0].pos.z - (20.0 + 1.6)).abs() < 1e-5);
        assert!((sh.level - 0.9).abs() < 1e-6);
    }

    #[test]
    fn shadow_quad_rays() {
        let (m, mut sh, ve) = shadow_car([true, false, true, true]);
        // wheel 1 (corner 3) is in the air: ray from 0.1 above its ground point, 25 down
        let mut seen = Vec::new();
        let mut ray = |a: Vec3, b: Vec3| {
            seen.push((a, b));
            Some(0.1)
        };
        let q = shadow_quad(m, &mut sh, Some(&ve), true, 0, 0.2, &mut ray).unwrap();
        assert_eq!(seen.len(), 1);
        assert!((seen[0].0.y - 0.4).abs() < 1e-6 && (seen[0].1.y - (0.3 - 25.0)).abs() < 1e-5);
        assert!((q[3].pos.y - (0.4 - 2.5 + SHADOW_LIFT)).abs() < 1e-5);
        assert!((sh.level - 0.8).abs() < 1e-6);
        // a miss draws nothing and keeps the level
        let mut miss = |_: Vec3, _: Vec3| -> Option<f32> { None };
        assert!(shadow_quad(m, &mut sh, Some(&ve), true, 0, 0.2, &mut miss).is_none());
        assert!((sh.level - 0.8).abs() < 1e-6);
        // no vehicle: y − height/2, level untouched
        let q = shadow_quad(m, &mut sh, None, true, 0, 0.2, &mut miss).unwrap();
        assert!(q.iter().all(|v| (v.pos.y - 0.7).abs() < 1e-6));
    }

    /// positive control: the "OFF" crc is a neon-slot (kind 3) record of COLOURS.BIN
    #[test]
    fn neon_off_record() {
        let Ok(b) = visos::file::platform::read("\\Vehicles\\COLOURS.BIN") else { return };
        let n = i16::from_le_bytes([b[4], b[5]]).max(0) as usize;
        let u = |r: &[u8], o: usize| u32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        let off: Vec<usize> = (0..n).filter(|&i| u(&b[6 + i * 64..], 0) == NEON_OFF_CRC).collect();
        // one "OFF" record per optional slot kind
        assert!(off.iter().any(|&i| u(&b[6 + i * 64..], 0x0C) == 3), "{off:?}");
    }

    #[test]
    fn shadow_pass_gates() {
        let (m, mut sh, ve) = shadow_car([true; 4]);
        let mut miss = |_: Vec3, _: Vec3| -> Option<f32> { None };
        assert!(shadow_pass(0, m, &mut sh, Some(&ve), false, false, -1, 0, 0.2, &mut miss).is_some());
        // zone bit clear = culled; in-car moves the level only
        sh.level = 0.0;
        assert!(shadow_pass(1, m, &mut sh, Some(&ve), false, true, !2, 0, 0.2, &mut miss).is_none());
        assert!((sh.level - 0.2).abs() < 1e-6 && (sh.mid_y - 0.5).abs() < 1e-6);
        // a neon car draws its shadow in TrackFX instead
        sh.neon = Some(0xFFFF_0000);
        assert!(shadow_pass(0, m, &mut sh, Some(&ve), false, false, -1, 0, 0.2, &mut miss).is_none());
    }

    #[test]
    fn wheel_blur_thresholds() {
        let pi = std::f32::consts::PI;
        assert_eq!(wheel_blur_levels(0.0), (1.0, 0.0, false));
        assert_eq!(wheel_blur_levels(2.0 * pi), (1.0, 0.0, false));
        // 4 half-turns/s: blur 0.2, spokes still whole
        let (s, b, h) = wheel_blur_levels(-4.0 * pi);
        assert!(s == 1.0 && !h && (b - 0.2).abs() < 1e-5);
        // 10 half-turns/s: spokes 0.5 and the normal ones left out, blur 0.8
        let (s, b, h) = wheel_blur_levels(10.0 * pi);
        assert!(h && (s - 0.5).abs() < 1e-5 && (b - 0.8).abs() < 1e-5);
        assert_eq!(wheel_blur_levels(100.0 * pi), (0.0, 1.0, true));
    }

    #[test]
    fn lod_thresholds() {
        let f = Vec3::NEG_Z;
        assert_eq!(select_lod(Vec3::new(0.0, 0.0, -19.9), Vec3::ZERO, f, None), 0);
        assert_eq!(select_lod(Vec3::new(0.0, 0.0, -20.0), Vec3::ZERO, f, None), 1);
        assert_eq!(select_lod(Vec3::new(40.0, 0.0, 0.0), Vec3::ZERO, f, None), 2);
        // a 45 degree fov halves the scaled distance
        assert_eq!(select_lod(Vec3::new(0.0, 0.0, -30.0), Vec3::ZERO, f, Some(45f32.to_radians())), 0);
    }

    /// positive control: every DECALS.BIN decal has its texture (plain, `_white` or `_black`) in DECALST.VCD, and
    /// "bonnet_off" exists
    #[test]
    fn every_decal_texture_in_decalst() {
        let db = velocity_physics::vehicle_database::CVehicleDatabase::load().unwrap();
        let v = visos::file::Vcd::open("\\Vehicles\\DECALST.VCD").unwrap();
        let names: std::collections::HashSet<String> =
            v.entries().map(|e| e.name.to_ascii_lowercase().trim_end_matches(".vtf").to_string()).collect();
        assert!(names.contains("bonnet_off"));
        let mut found = Vec::new();
        for d in &db.decals {
            let Some(n) = &d.name else { continue };
            if (0..3).any(|f| names.contains(&decal_texture(n, f))) {
                found.push((d.kind, n.clone()));
            }
        }
        // the shipped DECALST.VCD predates DECALS.BIN: the vinyl_* records have no texture (the original's load
        // fails the same way); only these resolve
        assert!(db.decals.len() > 200, "{} decals", db.decals.len());
        assert_eq!(found.len(), 5, "{found:?}");
    }

    /// positive control: every car's three chassis LODs parse, and every chassis with brake materials has both
    #[test]
    fn every_car_chassis_lod_and_lights() {
        let root = visos::file::game_dir().join("Vehicles");
        let (mut cars, mut lit) = (0, 0);
        for city in ["LA", "London", "Tokyo"] {
            let Ok(rd) = std::fs::read_dir(root.join(city)) else { continue };
            for d in rd.flatten() {
                let car = d.file_name().to_string_lossy().to_string();
                let Ok(v) = visos::file::Vcd::open(&format!("\\Vehicles\\{city}\\{car}\\VEHICLEM.VCD")) else { continue };
                let stem = format!("{car}_base");
                if v.get(&format!("{stem}.xmf")).is_none() {
                    continue;
                }
                cars += 1;
                for sfx in LOD_SUFFIX {
                    let d = v.get(&format!("{stem}{sfx}.xmf")).unwrap_or_else(|| panic!("{city}/{car}: no {stem}{sfx}"));
                    Model::parse(d).unwrap_or_else(|e| panic!("{city}/{car} {stem}{sfx}: {e}"));
                }
                let m = Model::parse(v.get(&format!("{stem}.xmf")).unwrap()).unwrap();
                let has = |s: &str| m.shaders.iter().any(|x| x.name.contains(s));
                if has("_brake") {
                    lit += 1;
                }
            }
        }
        assert!(cars > 20, "{cars} cars");
        assert!(lit * 2 > cars, "{lit} of {cars} chassis have brake lights");
    }

    /// positive control for FEAlloyEditor_Basic 0x002590A0: the front-end `_high` models' stage textures resolve in
    /// COMMONT + wheels_t + VEHICLEI + VEHICLET (the interior pack holds the seats, dash, door jambs); lists the rest
    #[test]
    fn frontend_textures_resolve() {
        let root = visos::file::game_dir().join("Vehicles");
        let names = |p: &str| -> Vec<String> {
            visos::file::Vcd::open(p)
                .map(|v| v.entries().map(|e| e.name.trim_end_matches(".vtf").to_ascii_lowercase()).collect())
                .unwrap_or_default()
        };
        let common: Vec<String> = [names("\\Vehicles\\COMMONT.VCD"), names("\\vehicles\\wheels\\wheels_t.vcd")].concat();
        let (mut cars, mut with, mut without, mut lines) = (0, 0, 0, Vec::new());
        for city in ["LA", "London", "Tokyo"] {
            let Ok(rd) = std::fs::read_dir(root.join(city)) else { continue };
            for d in rd.flatten() {
                let base = format!("\\Vehicles\\{city}\\{}", d.file_name().to_string_lossy());
                let Ok(v) = visos::file::Vcd::open(&format!("{base}\\VehicleS.vcd")) else { continue };
                cars += 1;
                let (int, own) = (names(&format!("{base}\\VEHICLEI.VCD")), names(&format!("{base}\\VEHICLET.VCD")));
                let mut used = std::collections::BTreeSet::new();
                for e in v.entries().filter(|e| e.name.to_ascii_lowercase().ends_with(".xmf")) {
                    let Some(m) = v.get(e.name).and_then(|x| Model::parse(x).ok()) else { continue };
                    for s in m.shaders.iter().filter_map(|s| s.stages.first()) {
                        let t = s.texture_name.to_ascii_lowercase();
                        if !t.is_empty() && t != "none" {
                            used.insert(t);
                        }
                    }
                }
                for t in used {
                    let base_hit = common.contains(&t) || own.contains(&t);
                    without += !base_hit as u32;
                    if !base_hit && !int.contains(&t) {
                        with += 1;
                        lines.push(format!("{base} {t}"));
                    }
                }
            }
        }
        lines.insert(0, format!("{cars} front-end cars: {without} missing textures without VEHICLEI, {with} with it"));
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/gfx/fe_texture_census.txt");
        let _ = std::fs::write(out, lines.join("\n"));
        assert!(cars > 10 && with < without, "{}", lines[0]);
    }

    /// in-game census (white car surfaces): every VEHICLEM / TrafficM model's stage textures against the packs the
    /// in-game spawn loads (COMMONT + wheels_t + VEHICLET, traffic TrafficT); lists the misses with the shader names
    /// and whether VEHICLEI / another pack would have them (`research/gfx/game_texture_census.txt`)
    #[test]
    #[ignore]
    fn ingame_textures_resolve() {
        let root = visos::file::game_dir().join("Vehicles");
        let names = |p: &str| -> Vec<String> {
            visos::file::Vcd::open(p)
                .map(|v| v.entries().map(|e| e.name.trim_end_matches(".vtf").to_ascii_lowercase()).collect())
                .unwrap_or_default()
        };
        let common: Vec<String> = [names("\\Vehicles\\COMMONT.VCD"), names("\\vehicles\\wheels\\wheels_t.vcd")].concat();
        let mut lines = Vec::new();
        let scan = |model_pack: &str, packs: &[Vec<String>], extra: &[(String, Vec<String>)], lines: &mut Vec<String>| -> (u32, u32) {
            let Ok(v) = visos::file::Vcd::open(model_pack) else { return (0, 0) };
            let (mut used, mut miss) = (0, 0);
            for e in v.entries().filter(|e| e.name.to_ascii_lowercase().ends_with(".xmf")) {
                let Some(m) = v.get(e.name).and_then(|x| Model::parse(x).ok()) else { continue };
                for s in &m.shaders {
                    for st in &s.stages {
                        let t = st.texture_name.to_ascii_lowercase();
                        if t.is_empty() || t == "none" {
                            continue;
                        }
                        used += 1;
                        if common.contains(&t) || packs.iter().any(|p| p.contains(&t)) {
                            continue;
                        }
                        miss += 1;
                        let found: Vec<&str> = extra.iter().filter(|(_, n)| n.contains(&t)).map(|(k, _)| k.as_str()).collect();
                        lines.push(format!("{model_pack} {} shader '{}' tex '{t}' in {found:?}", e.name, s.name));
                    }
                }
            }
            (used, miss)
        };
        let (mut tu, mut tm) = (0, 0);
        for city in ["LA", "London", "Tokyo"] {
            let Ok(rd) = std::fs::read_dir(root.join(city)) else { continue };
            for d in rd.flatten() {
                let base = format!("\\Vehicles\\{city}\\{}", d.file_name().to_string_lossy());
                let own = names(&format!("{base}\\VEHICLET.VCD"));
                let extra = vec![
                    ("VEHICLEI".to_string(), names(&format!("{base}\\VEHICLEI.VCD"))),
                    ("DECALST".to_string(), names("\\Vehicles\\DECALST.VCD")),
                    ("misc".to_string(), names("\\VFX\\misc.vcd")),
                ];
                let (u, m) = scan(&format!("{base}\\VEHICLEM.VCD"), &[own], &extra, &mut lines);
                tu += u;
                tm += m;
            }
        }
        let extra = vec![("COMMONT".to_string(), names("\\Vehicles\\COMMONT.VCD"))];
        let (u, m) = scan("\\Vehicles\\TrafficM.vcd", &[names("\\Vehicles\\TrafficT.vcd")], &extra, &mut lines);
        lines.insert(0, format!("race cars: {tm} of {tu} stage textures missing; traffic: {m} of {u}"));
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/gfx/game_texture_census.txt");
        let _ = std::fs::write(out, lines.join("\n"));
    }

    /// census of the fixed-function sets over every car model (VEHICLEM + wheels_m): how many CarShader /
    /// ShineyWheel shaders carry their own alpha test that the set's ALWAYS overrides
    #[test]
    fn fixed_set_alpha_census() {
        use crate::car_palette_pc_xbox::{car_shader_type, shader_set_for};
        use crate::renderer::shader_manager::{d3dcmp, StageState};
        let root = visos::file::game_dir().join("Vehicles");
        let mut paths = vec!["\\vehicles\\wheels\\wheels_m.vcd".to_string()];
        for city in ["LA", "London", "Tokyo"] {
            let Ok(rd) = std::fs::read_dir(root.join(city)) else { continue };
            for d in rd.flatten() {
                paths.push(format!("\\Vehicles\\{city}\\{}\\VEHICLEM.VCD", d.file_name().to_string_lossy()));
            }
        }
        let (mut fixed, mut tested) = (0, std::collections::BTreeMap::<String, u32>::new());
        for p in &paths {
            let Ok(v) = visos::file::Vcd::open(p) else { continue };
            for e in v.entries().filter(|e| e.name.to_ascii_lowercase().ends_with(".xmf")) {
                let Some(m) = v.get(e.name).and_then(|d| Model::parse(d).ok()) else { continue };
                let prims = m.objects.iter().flat_map(|o| o.parts.iter()).flat_map(|p| p.primitives.iter());
                for pr in prims {
                    let s = &m.shaders[pr.shader];
                    let set = shader_set_for(car_shader_type(&s.name, pr.kind, false), &s.name);
                    if !matches!(set, Some(CarShaderSet::CarShader | CarShaderSet::ShineyWheel)) {
                        continue;
                    }
                    fixed += 1;
                    let st = StageState::from_commands(s.stages.first().map(|x| x.commands.as_slice()).unwrap_or(&[0]));
                    if st.alpha_test && st.alpha_func != d3dcmp::ALWAYS {
                        let t = s.stages.first().map(|x| x.texture_name.to_ascii_lowercase()).unwrap_or_default();
                        // GREATEREQUAL 0 passes everything: only the tests that cut texels count
                        if !(st.alpha_func == d3dcmp::GREATEREQUAL && st.alpha_ref == 0) {
                            *tested.entry(format!("func {} ref {} blend {} {}", st.alpha_func, st.alpha_ref, st.blend_mode, t)).or_default() += 1;
                        }
                    }
                }
            }
        }
        assert!(fixed > 100, "{fixed}");
        let n: u32 = tested.values().sum();
        let mut lines: Vec<String> = tested.iter().map(|(k, c)| format!("{c}\t{k}")).collect();
        lines.insert(0, format!("{fixed} fixed-set primitives, {n} with an alpha test that cuts texels (overridden to ALWAYS)"));
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../research/gfx/fixed_set_census.txt");
        let _ = std::fs::write(out, lines.join("\n"));
    }

    /// positive control: `driver01` (wheels_m.vcd, CarRender.cpp 0x00110220) parses and the LOD 0 chassis carry a
    /// "driver" locator inside the body bounds
    #[test]
    fn driver_model_and_locators() {
        let w = visos::file::Vcd::open("\\vehicles\\wheels\\wheels_m.vcd").unwrap();
        let d = Model::parse(w.get(&format!("{DRIVER_MODEL}.xmf")).expect("driver01.xmf")).unwrap();
        assert!(!d.objects.is_empty());
        let root = visos::file::game_dir().join("Vehicles");
        let (mut cars, mut with) = (0, 0);
        for city in ["LA", "London", "Tokyo"] {
            let Ok(rd) = std::fs::read_dir(root.join(city)) else { continue };
            for e in rd.flatten() {
                let car = e.file_name().to_string_lossy().to_string();
                let Ok(v) = visos::file::Vcd::open(&format!("\\Vehicles\\{city}\\{car}\\VEHICLEM.VCD")) else { continue };
                let Some(b) = v.get(&format!("{car}_base.xmf")) else { continue };
                cars += 1;
                let m = Model::parse(b).unwrap();
                if let Some(p) = driver_locator(&m) {
                    with += 1;
                    let (c, r) = (Vec3::new(m.sphere[0], m.sphere[1], m.sphere[2]), m.sphere[3]);
                    assert!(Vec3::from_array(p).distance(c) < r, "{city}/{car}: driver {p:?} outside the sphere");
                }
            }
        }
        println!("{with} of {cars} chassis have a driver locator");
        assert!(with * 2 > cars, "{with} of {cars}");
    }
}

/// CarRender.cpp 0x00117BB0 `glass_*_alpha_0` / `logo*w_alpha_0` names (`off_6867FC`)
const GLASS_PRIMS: [&str; 6] = ["glass_f_alpha_0", "glass_b_alpha_0", "glass_l_alpha_0", "glass_r_alpha_0", "logofw_alpha_0", "logorw_alpha_0"];

/// prim lookups cached in the car data: carbon prims (+1916, 0x001174F0), the six windscreen / logo strips (+2024)
/// and the door glass (+2108, from the door models +1184); all of them are tagged prim type 7 (drawn in the alpha
/// pass). Our materials already carry the blend, so nothing reads these yet except as lookups.
#[derive(Component, Clone, Debug, Default)]
pub struct CarPrimLists {
    /// up to 3 `_carbon` prims (chassis first, then each part's first)
    pub carbon: Vec<Entity>,
    pub glass: [Option<Entity>; 6],
    /// 0x00117BB0 cleared flag +2328 & 0x8000 and slot 0: one of the first four was found nowhere
    pub glass_complete: bool,
    /// glass of the f_l / f_r / r_l / r_r door models (`glass_l_alpha_0`, else `glass_r_alpha_0`)
    pub door_glass: [Option<Entity>; 4],
}

impl CarPrimLists {
    /// 0x001174F0 + 0x00117BB0 over the LOD 0 records: chassis = `..chassis`, `parts` = one record range per fitted
    /// part model, `doors` = per front-end door model
    pub fn build(records: &[PrimRecord], chassis: usize, parts: &[std::ops::Range<usize>], doors: &[std::ops::Range<usize>; 8]) -> Self {
        let first = |r: std::ops::Range<usize>, sub: &str| records.get(r).and_then(|rs| rs.iter().find(|p| p.shader_name.contains(sub))).map(|p| p.entity);
        let ch = 0..chassis.min(records.len());
        let mut out = CarPrimLists { glass_complete: true, ..Default::default() };
        for r in std::iter::once(&ch).chain(parts) {
            if let Some(e) = first(r.clone(), "_carbon") {
                if out.carbon.len() < 3 {
                    out.carbon.push(e);
                }
            }
        }
        for (i, n) in GLASS_PRIMS.iter().enumerate() {
            out.glass[i] = std::iter::once(&ch).chain(parts).find_map(|r| first(r.clone(), n));
            if out.glass[i].is_none() && i < 4 {
                out.glass_complete = false;
                break;
            }
        }
        for k in 0..4 {
            out.door_glass[k] = first(doors[k].clone(), "glass_l_alpha_0").or_else(|| first(doors[k].clone(), "glass_r_alpha_0"));
        }
        out
    }
}
