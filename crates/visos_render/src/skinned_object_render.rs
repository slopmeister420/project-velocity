//! Render\SkinnedObjectRender.cpp (+ the skinned branch of Renderer.cpp's draw) — skinned characters: a skinned
//! XMF2 model from a character pack (`\Chars\CHARS_{M,T,A}.VCD`, `\FlagGirl\FG_{M,T,A}.vcd`) posed every frame by a
//! `.gaf` bone anim through the blend tree. Vertices carry palette slots (`BoneBlend4.vxu` `v15`), resolved here
//! to skeleton bones; the bone matrices drive bevy joints (joint = model-space bone world, inverse bind = +0xB0).
//!
//! API: write `SpawnSkinnedModel` (pack, model file stem, optional anim); the root gets `SkinnedModelRender`.
//! Move it by its `Transform` (game space), hide it with `Visibility`, change or drive the anim with
//! `SkinnedModelAnim` (anim = `.gaf` stem in the pack; `time` = normalised time to hold, else it advances and
//! wraps as SkinnedObjectRender does). `DespawnSkinnedModel` removes it.
//! BoneBlend4's per-vertex lighting of the posed normals runs on the cpu every frame (`relight_skinned`).
//! The Characters.pxu specular pass (`specular_chars`) rides on the same per-frame pass: uv 1, c10 and the spec
//! texture of every primitive's own material.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;

use crate::bone_anim::BoneAnim;
use crate::bone_anim_blend_tree::{update_leaf, world_matrices, Layer};
use crate::bone_skeleton_model::BoneSkeletonModel;
use crate::model_builder::{Builder, SpawnOpts, TexSet};
use crate::renderer::renderer_types::{vreg, Model};
use crate::renderer::shader_manager::XboxStageMaterial;
use crate::specular_chars::{SpecCharIndex, SpecularChars};

/// character resource packs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CharPack {
    /// `\Chars\CHARS_*.VCD` (front end characters, crowd, dancers)
    Chars,
    /// `\FlagGirl\FG_*.vcd` (race start flag girls)
    FlagGirl,
}

impl CharPack {
    fn vcd(self, kind: char) -> String {
        match self {
            CharPack::Chars => format!("\\Chars\\CHARS_{kind}.VCD"),
            CharPack::FlagGirl => format!("\\FlagGirl\\FG_{kind}.vcd"),
        }
    }
}

/// flag girl timeline instance link name (e.g. "LON_FLAGGIRL_A0") -> model stem in FG_M ("lon_flaggirl_a")
pub fn flag_girl_model(instance: &str) -> String {
    instance.trim_end_matches(|c: char| c.is_ascii_digit()).to_ascii_lowercase()
}

#[derive(Message, Clone, Debug)]
pub struct SpawnSkinnedModel {
    pub request: u64,
    pub pack: CharPack,
    /// model file stem in the pack's M vcd (case insensitive)
    pub model: String,
    pub anim: Option<String>,
    pub transform: Transform,
}

#[derive(Message, Clone, Debug)]
pub struct DespawnSkinnedModel {
    pub request: u64,
}

#[derive(Message, Clone, Debug)]
pub struct SkinnedModelAnim {
    pub request: u64,
    /// new anim (`.gaf` stem); `None` keeps the current one
    pub anim: Option<String>,
    /// normalised time to show (0..1); `None` = advance by the frame time
    pub time: Option<f32>,
}

/// Characters.cpp 0x00079EC0's cross-fade (the front-end character element 0x00285350, while the new anim's weight
/// a13 < 1 and a previous anim a3 exists): the root gets the previous anim as a layer of weight 1 at time a15 first,
/// then the current one at weight a13 (BoneAnimBlendTree 0x002DB300 twice, blended by 0x002DB610 / 0x002DBB40).
/// `prev: None` (or weight ≥ 1) = the single layer again. The caller owns the previous anim's clock.
#[derive(Message, Clone, Debug)]
pub struct SkinnedModelBlend {
    pub request: u64,
    /// previous anim (`.gaf` stem) and its normalised time
    pub prev: Option<(String, f32)>,
    /// the current anim's layer weight
    pub weight: f32,
}

#[derive(Component, Clone)]
pub struct SkinnedModelRender {
    pub request: u64,
    pub pack: CharPack,
    pub model: String,
    pub skeleton: Arc<BoneSkeletonModel>,
    pub anim: Option<Arc<BoneAnim>>,
    /// normalised anim time (instance +32)
    pub time: f32,
    /// held by `SkinnedModelAnim::time`
    pub held: bool,
    /// `SkinnedModelBlend`: the previous anim layer (anim, time) and the current layer's weight
    pub blend: Option<(Arc<BoneAnim>, f32)>,
    pub weight: f32,
    /// joint entities, one per skeleton bone
    pub joints: Vec<Entity>,
    /// per primitive mesh: bind-pose normals and the major bone (palette slot 0) of each vertex, relit every frame
    pub lit: Vec<SkinnedPrimLighting>,
    /// model bounding sphere radius (+0x1C): SpecularChars' position = root bone + radius up
    pub radius: f32,
}

/// BoneBlend4.vxu lighting input of one skinned primitive
#[derive(Clone)]
pub struct SkinnedPrimLighting {
    pub mesh: Handle<Mesh>,
    pub normals: Vec<Vec3>,
    pub major: Vec<u16>,
    /// the primitive's own material (Characters.pxu mode, rebound every frame by `specular_chars`)
    pub material: Handle<XboxStageMaterial>,
    /// SpecularChars 0x00143540 kind (8.. = spec material)
    pub kind: u32,
    /// stage +0x0C bit 0x10 (0x00143740's gloss test)
    pub gloss: bool,
    /// the stage has a blend record (code 1) for 0x002A2BE0 to rewrite
    pub blend_record: bool,
}

/// parsed anims per pack (by lower-case stem)
#[derive(Resource, Default)]
pub struct BoneAnimCache(HashMap<(CharPack, String), Option<Arc<BoneAnim>>>);

impl BoneAnimCache {
    pub fn get(&mut self, pack: CharPack, name: &str) -> Option<Arc<BoneAnim>> {
        let key = (pack, name.to_ascii_lowercase());
        self.0
            .entry(key)
            .or_insert_with(|| {
                let v = visos::file::Vcd::open(&pack.vcd('A')).ok()?;
                let d = v.get(&format!("{name}.gaf"))?;
                match BoneAnim::parse_file(d) {
                    Ok(mut a) if !a.is_empty() => Some(Arc::new(a.swap_remove(0))),
                    Ok(_) => None,
                    Err(e) => {
                        log::warn!("bone anim {name}: {e}");
                        None
                    }
                }
            })
            .clone()
    }
}

pub struct SkinnedObjectRenderPlugin;

impl Plugin for SkinnedObjectRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoneAnimCache>()
            .add_message::<SpawnSkinnedModel>()
            .add_message::<DespawnSkinnedModel>()
            .add_message::<SkinnedModelAnim>()
            .add_message::<SkinnedModelBlend>()
            .add_systems(Update, (process_skinned_messages, update_skinned_models).chain());
    }
}

/// one bevy mesh per primitive of a skinned model: bind-pose vertices, joints = skeleton bones through the
/// primitive's palette (`BoneBlend4.vxu`: index * 4 into the palette's matrices).
pub fn add_skinned_model(commands: &mut Commands, b: &mut Builder, tex: &mut TexSet, m: &Model, opts: &SpawnOpts) -> Vec<(Entity, SkinnedPrimLighting)> {
    let mut out = Vec::new();
    for obj in &m.objects {
        for part in &obj.parts {
            for prim in &part.primitives {
                let shader = &m.shaders[prim.shader];
                let (mut pos, mut nrm, mut uv, mut col, mut ji, mut jw, mut idx) = (vec![], vec![], vec![], vec![], vec![], vec![], vec![]);
                for d in &prim.draws {
                    let Ok(ind) = m.draw_indices(d) else { continue };
                    let mut remap: HashMap<u32, u32> = HashMap::new();
                    for &i in &ind {
                        let n = pos.len() as u32;
                        let v = *remap.entry(i).or_insert_with(|| {
                            let p = m.read_attr(d, vreg::POSITION, i).ok().flatten().unwrap_or([0.0; 4]);
                            pos.push([p[0], p[1], p[2]]);
                            let nn = m.read_attr(d, vreg::NORMAL, i).ok().flatten().map(|n| Vec3::new(n[0], n[1], n[2]).normalize_or_zero());
                            let wn = nn.unwrap_or(Vec3::Y);
                            nrm.push(wn.to_array());
                            let t = m.read_attr(d, vreg::TEXCOORD0, i).ok().flatten().unwrap_or([0.0; 4]);
                            uv.push([t[0], t[1]]);
                            col.push(b.lights.light_vertex(wn).to_array());
                            let s = m.read_attr(d, vreg::BLENDINDICES, i).ok().flatten().unwrap_or([0.0; 4]);
                            let w = m.read_attr(d, vreg::BLENDWEIGHT, i).ok().flatten().unwrap_or([1.0, 0.0, 0.0, 0.0]);
                            let bone = |k: usize| prim.palette.get(s[k] as usize).copied().unwrap_or(0) as u16;
                            ji.push([bone(0), bone(1), bone(2), bone(3)]);
                            let sum = (w[0] + w[1] + w[2] + w[3]).max(1e-6);
                            jw.push([w[0] / sum, w[1] / sum, w[2] / sum, w[3] / sum]);
                            n
                        });
                        idx.push(v);
                    }
                }
                if idx.is_empty() {
                    continue;
                }
                b.tris += idx.len() / 3;
                let normals: Vec<Vec3> = nrm.iter().map(|n: &[f32; 3]| Vec3::from_array(*n)).collect();
                let major: Vec<u16> = ji.iter().map(|j: &[u16; 4]| j[0]).collect();
                // colours are rewritten every frame (`relight_skinned_models`): keep the main-world copy
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
                mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_INDEX, VertexAttributeValues::Uint16x4(ji));
                mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, jw);
                mesh.insert_indices(Indices::U32(idx));
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.0f32; 2]; normals.len()]);
                let mesh_h = b.meshes.add(mesh);
                let shared = b.material(tex, shader, opts);
                // one material per primitive: the Characters.pxu inputs differ per character and spec material
                let mut own = b.materials.get(&shared).cloned().expect("material just built");
                own.params.arg.w = 1.0;
                let mat = b.materials.add(own);
                let mut e = commands.spawn((Mesh3d(mesh_h.clone()), MeshMaterial3d::<XboxStageMaterial>(mat.clone()), NoFrustumCulling));
                if let Some(p) = opts.parent {
                    e.insert(ChildOf(p));
                }
                let st = shader.stages.first();
                let blend_record = st.is_some_and(|s| crate::renderer::shader_manager::has_command(&s.commands, 1));
                out.push((
                    e.id(),
                    SkinnedPrimLighting {
                        mesh: mesh_h,
                        normals,
                        major,
                        material: mat,
                        kind: crate::specular_chars::spec_kind(&shader.name, prim.kind),
                        gloss: st.is_some_and(|s| s.unk0c & 0x10 != 0),
                        blend_record,
                    },
                ));
            }
        }
    }
    out
}

/// skinned primitives draw through GameDataLoading.cpp 0x0009EF30's "CharacterSpecular" override (0x001065A0,
/// +0 = SaveFileData 0x00106A40(3)): its compiled records land on the own stage's (0x002A4240). It emits only
/// code 2 from 0x002A0F40(2) = alpha test ALWAYS ref 0, z LESSEQUAL (so the own cut-outs draw uncut, as the car
/// sets), plus pass 1 / the Characters fragment program (`specular_chars`). The own blend stays; SpecularChars
/// rewrites the `spec_*` primitives' blend 2 to 1 / 0 every draw (`specular_chars::spec_list`).
fn character_state(s: &mut crate::renderer::shader_manager::StageState) {
    if let Some(r) = crate::renderer::shader_manager::compile_alpha_test(2, 0, true, 2) {
        s.apply_commands(&r);
    }
}

/// spawn one skinned model under a new root
pub fn spawn_skinned_model(
    commands: &mut Commands,
    b: &mut Builder,
    bind: &mut Assets<SkinnedMeshInverseBindposes>,
    anims: &mut BoneAnimCache,
    r: &SpawnSkinnedModel,
) -> Option<Entity> {
    let mv = visos::file::Vcd::open(&r.pack.vcd('M')).ok()?;
    let data = mv.get(&format!("{}.xmf", r.model))?.to_vec();
    let m = match Model::parse(&data) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("skinned model {}: {e}", r.model);
            return None;
        }
    };
    let skel = Arc::new(BoneSkeletonModel::parse(&data).ok()??);
    let mut tex = TexSet::default();
    crate::car_render::add_vcd_textures(&mut tex, &r.pack.vcd('T'));
    let root = commands.spawn((r.transform, Visibility::default())).id();
    let joints: Vec<Entity> = skel
        .bones
        .iter()
        .map(|bn| commands.spawn((Transform::from_matrix(bn.world), ChildOf(root))).id())
        .collect();
    let inv = bind.add(SkinnedMeshInverseBindposes::from(skel.bones.iter().map(|bn| bn.inv_world).collect::<Vec<_>>()));
    let opts = SpawnOpts { parent: Some(root), state: Some(character_state), ..SpawnOpts::world() };
    let mut lit = Vec::new();
    for (e, l) in add_skinned_model(commands, b, &mut tex, &m, &opts) {
        commands.entity(e).insert(SkinnedMesh { inverse_bindposes: inv.clone(), joints: joints.clone() });
        lit.push(l);
    }
    let anim = r.anim.as_deref().and_then(|a| anims.get(r.pack, a));
    commands.entity(root).insert(SkinnedModelRender {
        request: r.request,
        pack: r.pack,
        model: r.model.clone(),
        skeleton: skel,
        anim,
        time: 0.0,
        held: false,
        blend: None,
        weight: 1.0,
        joints,
        lit,
        radius: m.sphere[3],
    });
    Some(root)
}

#[allow(clippy::too_many_arguments)]
fn process_skinned_messages(
    mut commands: Commands,
    mut spawn: MessageReader<SpawnSkinnedModel>,
    mut despawn: MessageReader<DespawnSkinnedModel>,
    mut anim_msg: MessageReader<SkinnedModelAnim>,
    mut blend_msg: MessageReader<SkinnedModelBlend>,
    mut models: Query<(Entity, &mut SkinnedModelRender)>,
    mut cache: ResMut<BoneAnimCache>,
    lights: Option<Res<crate::track_render::TrackLights>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut bind: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    spec: Option<Res<SpecularChars>>,
) {
    for d in despawn.read() {
        for (e, m) in &models {
            if m.request == d.request {
                commands.entity(e).despawn();
            }
        }
    }
    for a in anim_msg.read() {
        for (_, mut m) in &mut models {
            if m.request != a.request {
                continue;
            }
            if let Some(name) = &a.anim {
                let next = cache.get(m.pack, name);
                if next.is_none() {
                    log::warn!("skinned model {}: no anim {name}", m.model);
                }
                m.anim = next;
                m.time = 0.0;
            }
            match a.time {
                Some(t) => {
                    m.time = t.clamp(0.0, 1.0);
                    m.held = true;
                }
                None => m.held = false,
            }
        }
    }
    for b in blend_msg.read() {
        for (_, mut m) in &mut models {
            if m.request != b.request {
                continue;
            }
            m.weight = b.weight;
            m.blend = match &b.prev {
                Some((name, t)) if b.weight < 1.0 => cache.get(m.pack, name).map(|a| (a, *t)),
                _ => None,
            };
        }
    }
    let reqs: Vec<SpawnSkinnedModel> = spawn.read().cloned().collect();
    if reqs.is_empty() {
        return;
    }
    let l = lights.map(|l| l.lights).unwrap_or_default();
    if spec.is_none() {
        let front_end = reqs[0].pack == CharPack::Chars;
        let t = reqs[0].pack.vcd('T');
        commands.insert_resource(SpecularChars::init(front_end, &["\\VFX\\misc.vcd", &t], &mut images));
    }
    for r in reqs {
        let mut b = Builder::new(&mut meshes, &mut materials, &mut images);
        b.lights = l;
        if spawn_skinned_model(&mut commands, &mut b, &mut bind, &mut cache, &r).is_none() {
            log::warn!("skinned model {:?} {} not found", r.pack, r.model);
        }
    }
}

/// SkinnedObjectRender.cpp 0x0013AF80 — advance the anim (frame increment, wrapped past 1), run the blend tree and
/// pose the joints
#[allow(clippy::too_many_arguments)]
fn update_skinned_models(
    time: Res<Time>,
    mut models: Query<(&mut SkinnedModelRender, &Transform, Option<&SpecCharIndex>)>,
    mut joints: Query<&mut Transform, Without<SkinnedModelRender>>,
    lights: Option<Res<crate::track_render::TrackLights>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    spec: Option<Res<SpecularChars>>,
    cameras: Query<(&Camera, &GlobalTransform, Has<crate::track_render::TrackCamera>), With<Camera3d>>,
) {
    let dt = time.delta_secs();
    let l = lights.map(|l| l.lights).unwrap_or_default();
    // VsRender 0x0005ACD0: the render camera (the game's world camera, else the lowest-order active 3d camera)
    let camera = cameras
        .iter()
        .filter(|(c, _, _)| c.is_active)
        .min_by_key(|(c, _, track)| (!*track, c.order))
        .map(|(_, g, _)| g.translation())
        .unwrap_or(Vec3::ZERO);
    for (mut m, root, index) in &mut models {
        let world: Vec<Mat4> = match m.anim.clone() {
            Some(anim) => {
                if !m.held {
                    m.time += anim.frame_inc(dt);
                    if m.time > 1.0 {
                        m.time -= 1.0;
                    }
                }
                // Characters 0x00079EC0: the previous anim's layer (weight 1) first, then the current one
                let qts = match m.blend.clone() {
                    Some((prev, t)) => update_leaf(&m.skeleton, &[Layer { anim: &prev, weight: 1.0, time: t }, Layer { anim: &anim, weight: m.weight, time: m.time }]),
                    None => update_leaf(&m.skeleton, &[Layer { anim: &anim, weight: 1.0, time: m.time }]),
                };
                let world = world_matrices(&m.skeleton, &qts);
                for (j, w) in m.joints.iter().zip(&world) {
                    if let Ok(mut t) = joints.get_mut(*j) {
                        *t = Transform::from_matrix(*w);
                    }
                }
                world
            }
            None => m.skeleton.bones.iter().map(|bn| bn.world).collect(),
        };
        let skins: Vec<Mat4> = world.iter().zip(&m.skeleton.bones).map(|(w, bn)| *w * bn.inv_world).collect();
        let root_m = root.to_matrix();
        let Some((sc, settings)) = spec.as_deref().and_then(|s| s.settings.as_ref().map(|set| (s, set))) else {
            relight_skinned(&m.lit, &skins, root_m, &l, None, &mut meshes);
            continue;
        };
        let idx = index.map(|i| i.0 as usize).unwrap_or(0);
        // Characters.cpp 0x0007A460: in game (0x00144BC0) the character is lit by the record's diffuse set
        let lights = match settings.diffuse.get(idx) {
            Some(d) if !sc.front_end => crate::specular_chars::diffuse_light_set(d),
            _ => l,
        };
        // Characters.cpp 0x00079EC0 / 0x0007A460: position = root bone in the world, + the model radius up
        let mut pos = root_m.transform_point3(world.first().map(|w| w.w_axis.truncate()).unwrap_or(Vec3::ZERO));
        pos.y += m.radius;
        let dist = camera.distance(pos);
        let rec = settings.records.get(idx).copied().unwrap_or_default();
        let (light_dir, _) = crate::specular_chars::spec_light(&rec, dist);
        let (rows, hv) = crate::specular_chars::reflection(light_dir, camera, pos);
        relight_skinned(&m.lit, &skins, root_m, &lights, Some(&rows), &mut meshes);
        let prims: Vec<(u32, bool)> = m.lit.iter().map(|p| (p.kind, p.gloss)).collect();
        let draws = crate::specular_chars::character_draws(&rec, &prims, dist, hv, sc.front_end, sc.enabled);
        for (p, d) in m.lit.iter().zip(draws) {
            let tex = sc.texture(d.texture);
            let blend = d.blend.filter(|_| p.blend_record).map(|b| b as u32);
            let Some(cur) = materials.get(&p.material) else { continue };
            let same = cur.params.spec == d.colour && cur.spec_texture == tex && blend.is_none_or(|b| cur.state.blend_mode == b);
            if !same {
                if let Some(mut mat) = materials.get_mut(&p.material) {
                    mat.params.spec = d.colour;
                    mat.spec_texture = tex;
                    if let Some(b) = blend {
                        mat.state.blend_mode = b;
                    }
                }
            }
        }
    }
}

/// BoneBlend4.vxu lighting, per rendered frame: the normal through the major bone's skin matrix (palette slot 0, not
/// renormalised) and the world matrix, then oD0 = ambient + sum((n . L_i > 0) * (n . L_i) * colour_i) with the
/// object's lights (Characters.cpp 0x00079EC0 builds them from the scene's lights 0-2 + ambient), saturated.
/// With `refl` (the SpecularChars reflection rows) the same world normal also gives oT1 (`specular_chars::spec_uv`).
pub fn relight_skinned(
    prims: &[SkinnedPrimLighting],
    skins: &[Mat4],
    root: Mat4,
    lights: &crate::vs_render::VsLights,
    refl: Option<&[Vec3; 2]>,
    meshes: &mut Assets<Mesh>,
) {
    for p in prims {
        let Some(mut mesh) = meshes.get_mut(&p.mesh) else { continue };
        let mut col: Vec<[f32; 4]> = Vec::with_capacity(p.normals.len());
        let mut uv1: Vec<[f32; 2]> = Vec::with_capacity(p.normals.len());
        for (n, &k) in p.normals.iter().zip(&p.major) {
            let s = skins.get(k as usize).copied().unwrap_or(Mat4::IDENTITY);
            let w = root.transform_vector3(s.transform_vector3(*n));
            col.push(lights.light_vertex(w).to_array());
            if let Some(r) = refl {
                uv1.push(crate::specular_chars::spec_uv(w, r));
            }
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
        if refl.is_some() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv1);
        }
    }
}

#[cfg(test)]
mod tests {
    /// every shipped `Tracks\*\Circuit\RaceNN\skininst.bdl` ("SkinInst.bdl", string 0x0063CB54) is a 105-byte bundle
    /// whose Instances chunk holds no instance: there are no skinned track objects to place
    #[test]
    fn skininst_bundles_are_empty() {
        let tracks = visos::file::game_dir().join("Tracks");
        let mut n = 0;
        for t in std::fs::read_dir(&tracks).into_iter().flatten().flatten() {
            for kind in ["Circuit", "Sprint", "Drag", "Drift"] {
                for r in std::fs::read_dir(t.path().join(kind)).into_iter().flatten().flatten() {
                    let p = r.path().join("skininst.bdl");
                    let Ok(d) = std::fs::read(&p) else { continue };
                    let (b, _) = visos::chunk::Bundle::load_standalone(&d).unwrap_or_else(|e| panic!("{}: {e:?}", p.display()));
                    assert!(b.instances.as_ref().is_none_or(|i| i.instances.is_empty()), "{}", p.display());
                    n += 1;
                }
            }
        }
        assert!(n >= 24, "{n} skininst bundles");
    }
}
