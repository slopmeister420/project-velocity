//! src\CarDamage\CarDamage.cpp (render side) — what the original's CarDamage does to the car model: dented body
//! vertices written back through VertexIterator (0x0006FF80 / 0x00075470), broken head / brake light shader swaps
//! (0x00073700, shaders built in 0x0006C290..), window crack textures (0x000730C0) and the player's light damage bits
//! (CarRender.cpp 0x001157B0). The damage itself runs in `velocity_physics::car_damage`.
//!
//! API: every in-game race car root gets a [`CarDamageSource`] (what physics' `SetupCarDamage` needs, in the order
//! the original walks it). Feed the physics events back as [`CarDamageRenderEvent`]s for the car's render request.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::model_builder::{Builder, DamageCollect, DamagePrim, PrimRecord, TexSet};
use crate::renderer::shader_manager::{StageState, XboxStageMaterial, XboxStageParams};

/// one vertex group of the damage model (physics `DamageMesh`)
#[derive(Clone, Debug, Default)]
pub struct CarDamageMeshSource {
    /// mesh -> model space, row-vector 4x4 (glam: `Mat4::from_cols_array`)
    pub matrix: [f32; 16],
    pub positions: Vec<[f32; 3]>,
    /// vertex element 1 (the normal register)
    pub attr1: Vec<[f32; 3]>,
}

/// what the car's damage model is built from (CarRender.cpp 0x00110220 -> CarDamage 0x0006D150): the `l_head` /
/// `l_brake` locators of the LOD 0 chassis with z negated (car data +768 / +896; zero without locators), and the
/// vertex groups of the LOD 0 chassis (+1092) then each fitted part (+1208 list). Model space is the render root's
/// local space (nose +Z), so `model_to_root` for physics = the model -> body rotation core already applies.
#[derive(Component, Clone, Debug, Default)]
pub struct CarDamageSource {
    pub head: [f32; 3],
    pub brake: [f32; 3],
    pub meshes: Vec<CarDamageMeshSource>,
}

/// a physics CarDamage event for the car spawned with `request`
#[derive(Message, Clone, Debug)]
pub struct CarDamageRenderEvent {
    pub request: u64,
    pub kind: CarDamageRenderKind,
}

#[derive(Clone, Debug)]
pub enum CarDamageRenderKind {
    /// CarDamage 0x00073700(light, 0): 0 left headlight, 1 right headlight, 2 left brake, 3 right brake
    LightBroken(u32),
    LightRepaired(u32),
    /// CarDamage 0x000730C0(window, 0, stage): 0 front, 1 rear, 2 left, 3 right; stage 0..2
    WindowCrack { window: u32, stage: u32 },
    WindowRepaired(u32),
    /// a damage mesh's vertices changed: its whole element 0 / element 1 arrays (mesh space)
    MeshChanged { mesh: usize, positions: Vec<[f32; 3]>, attr1: Vec<[f32; 3]> },
    /// CarDamage 0x00070F40 -> CarDamageFX 0x00078880: the damage car index (impacts only for 0), the world hit
    /// point (+8) and the impulse (+40) already halved
    CollisionFx { vehicle: i32, point: [f32; 3], impulse: f32 },
}

/// `apply_car_damage` runs here; CarDamageFX reads the same events before it (light bits before the break)
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CarDamageRenderSet;

/// CarRender car data +2250 light damage bits of car 0 (CarRender.cpp 0x001157B0: bit `1 << light`); OR-ed into
/// `CarLightInput.damage`
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CarLightDamage(pub u8);

/// per car: the deformable meshes, the light and glass primitives and their swapped materials
#[derive(Component, Clone, Debug)]
pub struct CarDamageRender {
    prims: Vec<DamagePrim>,
    /// group -> indices into `prims`
    group_prims: Vec<Vec<usize>>,
    /// per light: (entity, broken material); the repaired material is the entity's original one
    lights: [Vec<(Entity, Handle<XboxStageMaterial>)>; 4],
    /// per window: every glass primitive of every LOD (stage texture `glass_f/b/l/r`)
    windows: [Vec<Entity>; 4],
    /// per window: crack textures `glass_?_damage1..3`
    cracks: [[Option<Handle<Image>>; 3]; 4],
    originals: HashMap<Entity, Handle<XboxStageMaterial>>,
    cracked: HashMap<(Entity, u32), Handle<XboxStageMaterial>>,
}

pub struct CarDamageRenderPlugin;

impl Plugin for CarDamageRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CarDamageRenderEvent>()
            .add_systems(Update, apply_car_damage.in_set(CarDamageRenderSet))
            .add_plugins(crate::car_damage_fx::CarDamageFxPlugin);
    }
}

/// window -> glass stage texture (0x000730C0)
const GLASS: [&str; 4] = ["glass_f", "glass_b", "glass_l", "glass_r"];

/// CarDamage 0x00073700 — per light the primitive names it swaps, and which broken texture each takes
/// (0x0006C290: `[70]` left head, `[69]` right head, `[68]` left brake, `[67]` right brake `_dam` textures)
const LIGHT_PRIMS: [&[(&str, usize)]; 4] = [
    &[("left_headlight", 0), ("left_head_surround", 0)],
    &[("right_headlight", 1), ("right_head_surround", 1)],
    &[("left_brake", 2), ("left_surround", 2), ("left_reverse", 2)],
    &[("right_brake", 3), ("right_surround", 3), ("right_reverse", 3)],
];

/// CarDamage 0x0006C290 — the broken textures: the stage texture of the first primitive named `left_headlight`
/// (`right_headlight`, `left_brake`, `right_brake`) in the chassis, else the parts, with `_headlight` / `_brake`
/// replaced by `_headlight_dam` / `_brake_dam` (sprintf over the match, so the rest of the name is dropped)
fn broken_texture(records: &[PrimRecord], ranges: &[std::ops::Range<usize>], light: &str, key: &str) -> Option<String> {
    let tex = ranges.iter().find_map(|r| records[r.clone()].iter().find(|p| p.shader_name.contains(light)).map(|p| p.texture.clone()))?;
    let at = tex.find(key)?;
    Some(format!("{}{key}_dam", &tex[..at]))
}

/// CarDamage 0x0006C290 / 0x0006D150 + CarRender.cpp 0x00110220 — build the damage source and the render-side
/// state. `records` = every LOD 0 primitive, `ranges` = its chassis then part models; `all` = the other LODs'.
#[allow(clippy::too_many_arguments)]
pub(crate) fn setup(
    commands: &mut Commands,
    b: &mut Builder,
    tex: &mut TexSet,
    root: Entity,
    collect: DamageCollect,
    head: [f32; 3],
    brake: [f32; 3],
    records: &[PrimRecord],
    ranges: &[std::ops::Range<usize>],
    all: &[PrimRecord],
) {
    let meshes = collect
        .groups
        .iter()
        .map(|g| CarDamageMeshSource { matrix: g.matrix, positions: g.positions.clone(), attr1: g.normals.clone() })
        .collect();
    let mut group_prims = vec![Vec::new(); collect.groups.len()];
    for (i, p) in collect.prims.iter().enumerate() {
        let mut gs: Vec<u32> = p.verts.iter().map(|v| v.0).filter(|g| *g != u32::MAX).collect();
        gs.dedup();
        gs.sort_unstable();
        gs.dedup();
        for g in gs {
            group_prims[g as usize].push(i);
        }
    }
    // broken shaders: stage flags 1, the `_dam` texture, blend 2, no wrap; the head surrounds also z state (1, 0, 0)
    let dam = [
        broken_texture(records, ranges, "left_headlight", "_headlight"),
        broken_texture(records, ranges, "right_headlight", "_headlight"),
        broken_texture(records, ranges, "left_brake", "_brake"),
        broken_texture(records, ranges, "right_brake", "_brake"),
    ];
    let mut broken = |t: usize, surround: bool| -> Handle<XboxStageMaterial> {
        let state = StageState { blend_mode: 2, z_write: !surround, address_u: 3, address_v: 3, ..StageState::default() };
        let texture = dam[t].as_ref().and_then(|n| tex.image(n, 3, 3, b.images));
        b.materials.add(XboxStageMaterial { params: XboxStageParams::new(&state), texture, stencil: None, spec_texture: None, state })
    };
    let mut lights: [Vec<(Entity, Handle<XboxStageMaterial>)>; 4] = Default::default();
    for (l, names) in LIGHT_PRIMS.iter().enumerate() {
        for &(name, t) in names.iter() {
            let m = broken(t, name.ends_with("head_surround"));
            // 0x00073600: per object of each model, the first primitive whose shader name contains the name
            for r in ranges {
                let mut done = Vec::new();
                for p in &records[r.clone()] {
                    if !done.contains(&p.object) && p.shader_name.contains(name) {
                        done.push(p.object);
                        lights[l].push((p.entity, m.clone()));
                    }
                }
            }
        }
    }
    // 0x00072F80: every stage named glass_? (case-insensitive) in every LOD of the chassis and parts
    let mut windows: [Vec<Entity>; 4] = Default::default();
    let mut cracks: [[Option<Handle<Image>>; 3]; 4] = Default::default();
    for (w, g) in GLASS.iter().enumerate() {
        let mut state = None;
        for p in records.iter().chain(all) {
            if p.texture.eq_ignore_ascii_case(g) {
                windows[w].push(p.entity);
                state.get_or_insert(p.state);
            }
        }
        if let Some(s) = state {
            for (k, c) in cracks[w].iter_mut().enumerate() {
                *c = tex.image(&format!("{g}_damage{}", k + 1), s.address_u, s.address_v, b.images);
            }
        }
    }
    commands.entity(root).insert((
        CarDamageSource { head, brake, meshes },
        CarDamageRender {
            prims: collect.prims,
            group_prims,
            lights,
            windows,
            cracks,
            originals: HashMap::new(),
            cracked: HashMap::new(),
        },
        CarLightDamage::default(),
    ));
}

/// the light / window shader swaps and mesh uploads for the events of this frame
fn apply_car_damage(
    mut events: MessageReader<CarDamageRenderEvent>,
    mut cars: Query<(&crate::car_render::CarRender, &mut CarDamageRender, &mut CarLightDamage)>,
    mut mats: Query<&mut MeshMaterial3d<XboxStageMaterial>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for ev in events.read() {
        match &ev.kind {
            CarDamageRenderKind::LightBroken(l) | CarDamageRenderKind::LightRepaired(l) => {
                let broken = matches!(ev.kind, CarDamageRenderKind::LightBroken(_));
                if *l < 4 {
                    // CarRender.cpp 0x001157B0 is always called for car 0 (original quirk: any car's light sets it)
                    for (c, _, mut bits) in &mut cars {
                        if c.request == 0 {
                            bits.0 = if broken { bits.0 | 1 << l } else { bits.0 & !(1 << l) };
                        }
                    }
                }
            }
            _ => {}
        }
        let Some((_, mut dr, _)) = cars.iter_mut().find(|(c, _, _)| c.request == ev.request) else { continue };
        let dr = &mut *dr;
        match &ev.kind {
            CarDamageRenderKind::LightBroken(l) | CarDamageRenderKind::LightRepaired(l) => {
                let broken = matches!(ev.kind, CarDamageRenderKind::LightBroken(_));
                for (e, m) in dr.lights.get(*l as usize).into_iter().flatten() {
                    let Ok(mut cur) = mats.get_mut(*e) else { continue };
                    let orig = dr.originals.entry(*e).or_insert_with(|| cur.0.clone()).clone();
                    cur.0 = if broken { m.clone() } else { orig };
                }
            }
            CarDamageRenderKind::WindowCrack { .. } | CarDamageRenderKind::WindowRepaired(_) => {
                let (w, crack) = match ev.kind {
                    CarDamageRenderKind::WindowCrack { window, stage } => (window as usize, Some(stage.min(2))),
                    CarDamageRenderKind::WindowRepaired(window) => (window as usize, None),
                    _ => unreachable!(),
                };
                if w >= 4 {
                    continue;
                }
                for e in dr.windows[w].clone() {
                    let Ok(mut cur) = mats.get_mut(e) else { continue };
                    let orig = dr.originals.entry(e).or_insert_with(|| cur.0.clone()).clone();
                    cur.0 = match crack {
                        None => orig,
                        Some(s) => match dr.cracked.get(&(e, s)) {
                            Some(h) => h.clone(),
                            None => {
                                let Some(base) = materials.get(&orig).cloned() else { continue };
                                let h = materials.add(XboxStageMaterial { texture: dr.cracks[w][s as usize].clone(), ..base });
                                dr.cracked.insert((e, s), h.clone());
                                h
                            }
                        },
                    };
                }
            }
            CarDamageRenderKind::MeshChanged { mesh, positions, attr1 } => {
                for &pi in dr.group_prims.get(*mesh).into_iter().flatten() {
                    let p = &dr.prims[pi];
                    let Some(mut m) = meshes.get_mut(&p.mesh) else { continue };
                    write_group(&mut m, p, *mesh as u32, positions, attr1);
                }
            }
            // car_damage_fx
            CarDamageRenderKind::CollisionFx { .. } => {}
        }
    }
}

/// VertexIterator.cpp 0x002C5570 element 0 / 1 writes, carried into the bevy mesh built from that group
fn write_group(m: &mut Mesh, p: &DamagePrim, group: u32, positions: &[[f32; 3]], attr1: &[[f32; 3]]) {
    use bevy::mesh::VertexAttributeValues;
    if let Some(VertexAttributeValues::Float32x3(pos)) = m.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        for (v, &(g, k)) in p.verts.iter().enumerate() {
            if g == group {
                if let (Some(dst), Some(src)) = (pos.get_mut(v), positions.get(k as usize)) {
                    *dst = p.xf.transform_point3(Vec3::from_array(*src)).to_array();
                }
            }
        }
    }
    if let Some(VertexAttributeValues::Float32x3(nrm)) = m.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        for (v, &(g, k)) in p.verts.iter().enumerate() {
            if g == group {
                if let (Some(dst), Some(src)) = (nrm.get_mut(v), attr1.get(k as usize)) {
                    *dst = p.xf.transform_vector3(Vec3::from_array(*src)).normalize_or_zero().to_array();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_builder::{damage_group, SpawnOpts};
    use crate::renderer::renderer_types::Model;
    use bevy::ecs::world::CommandQueue;

    fn cars() -> Vec<(String, String)> {
        let root = visos::file::game_dir().join("Vehicles");
        let mut out = Vec::new();
        for city in ["LA", "London", "Tokyo"] {
            let Ok(rd) = std::fs::read_dir(root.join(city)) else { continue };
            for d in rd.flatten() {
                if d.path().is_dir() {
                    out.push((city.to_string(), d.file_name().to_string_lossy().to_string()));
                }
            }
        }
        out
    }

    /// positive control on the model space: every car chassis with locators has its z-negated `l_head` next to
    /// the body's vertices (as the original's bone placement assumes)
    #[test]
    fn every_chassis_head_locator_on_body() {
        let (mut n, mut bad) = (0, Vec::new());
        for (city, car) in cars() {
            let Ok(v) = visos::file::Vcd::open(&format!("PFXVehiclesPFX{city}PFX{car}PFXVEHICLEM.VCD").replace("PFX", "\\")) else { continue };
            let stem = car.to_ascii_lowercase();
            let Some(d) = v.entries().find(|e| e.name.eq_ignore_ascii_case(&format!("{stem}_base.xmf"))).and_then(|e| v.data(&e)) else { continue };
            let m = Model::parse(d).unwrap();
            let Some(h) = m.locator("l_head") else { continue };
            let head = Vec3::new(h[12], h[13], -h[14]);
            let mut best = f32::MAX;
            for o in &m.objects {
                for p in o.parts.first().into_iter().flat_map(|p| &p.primitives) {
                    for dr in &p.draws {
                        let g = damage_group(&m, dr, o.matrix);
                        let mm = Mat4::from_cols_array(&g.matrix);
                        for q in &g.positions {
                            best = best.min(mm.transform_point3(Vec3::from_array(*q)).distance(head));
                        }
                    }
                }
            }
            n += 1;
            if best > 0.35 {
                bad.push(format!("{city}/{car} {best}"));
            }
        }
        assert!(n > 20, "{n}");
        assert!(bad.is_empty(), "{bad:?}");
    }

    /// the group -> mesh map round-trips: writing every group back unchanged reproduces the built meshes
    #[test]
    fn damage_groups_map_onto_meshes() {
        let v = visos::file::Vcd::open(&"PFXVehiclesPFXLAPFX350zPFXVEHICLEM.VCD".replace("PFX", "\\")).unwrap();
        let m = Model::parse(v.get("350z_base.xmf").unwrap()).unwrap();
        let world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let (mut meshes, mut materials, mut images) = (Assets::<Mesh>::default(), Assets::<XboxStageMaterial>::default(), Assets::<Image>::default());
        let mut b = Builder::new(&mut meshes, &mut materials, &mut images);
        b.damage = Some(DamageCollect::default());
        let mut tex = TexSet::default();
        b.add_model(&mut commands, &mut tex, &m, Mat4::IDENTITY, &SpawnOpts::world());
        let dc = b.damage.take().unwrap();
        assert!(dc.groups.len() > 10 && !dc.prims.is_empty());
        let mut checked = 0;
        for p in &dc.prims {
            let before = meshes.get(&p.mesh).unwrap().clone();
            let mut mesh = before.clone();
            for g in 0..dc.groups.len() as u32 {
                write_group(&mut mesh, p, g, &dc.groups[g as usize].positions, &dc.groups[g as usize].normals);
            }
            let (a, b2) = (before.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap(), mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap());
            for (x, y) in a.iter().zip(b2) {
                assert!(Vec3::from_array(*x).distance(Vec3::from_array(*y)) < 1e-5);
                checked += 1;
            }
        }
        assert!(checked > 1000, "{checked}");
    }
}
