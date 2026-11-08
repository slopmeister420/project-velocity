//! drawing the physics objects (not an original module): the render pass PhysicsObjectManager 0x001E0EE0 draws
//! every object with `(flags & 8) == 0 && (flags & 2) != 0` at its matrix (obj +0), which PhysicsObjects
//! 0x001E58C0 first interpolates for awake objects (position lerp, rotation slerp of the body's last two
//! states by the render interp). Here the models are the instances' visual resources from the same compartments
//! physics registered (track `CPTS.VCD:instance.cpt`, race `RACE.VCD:instance.cpt / uinstance.cpt`, lists 1
//! physics and 3 immovable), each built once and shared; the pose is the last two updates' `PhysicsObjects`
//! matrices lerped by `timer.lerp`. Not ported: the draw-distance fade (obj +140, 0x0008DF40) and per-object
//! lighting (0x001E3A60).

use std::collections::HashMap;

use bevy::prelude::*;
use velocity_physics::plugin::PhysicsObjects;
use visos_render::compartment_sys::{Compartment, INSTANCE_IMMOVABLE, INSTANCE_PHYSICS};
use visos_render::model_builder::{Builder, SpawnOpts, TexSet};
use visos_render::renderer::renderer_types::Model;
use visos_render::renderer::shader_manager::XboxStageMaterial;
use visos_render::renderer::texture_manager::decode_any_vtf;
use visos_render::track_render::{TrackLights, TrackRoot};

use crate::game_data_loading::game_data_loading::ELoadStep;
use crate::game_mode::game_manager::{EGameMode, GameManager, LoadStepMessage};
use crate::timeline_manager::objects::timeline_object::TimelineWorld;
use crate::vs_common_main::{GameClock, GameTick, InGameUpdateSet};

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
enum Stage {
    #[default]
    Idle,
    /// templates spawned this frame; their meshes exist next frame
    Templates,
    Live,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Pose {
    pos: Vec3,
    rot: Quat,
}

fn pose(m: &[f32; 16]) -> Pose {
    let (_, rot, pos) = Mat4::from_cols_array(m).to_scale_rotation_translation();
    Pose { pos, rot }
}

#[derive(Resource, Default)]
pub struct PhysicsObjectRender {
    /// track directory under `\Tracks` and the race directory, set by the "Physics Manager" load step
    track: Option<String>,
    race: Option<String>,
    stage: Stage,
    /// object count the entities were matched against (a change rebuilds)
    count: usize,
    /// per `PhysicsObjects` index: (template, drawn root)
    template: Vec<Option<Entity>>,
    root: Vec<Option<Entity>>,
    prev: Vec<Pose>,
    cur: Vec<Pose>,
    shown: Vec<bool>,
    /// type tag 1 (`propld_dummycopcar`, PhysicsObjects 0x001E5F30): drawn as a police car copy, never as itself
    dummy: Vec<bool>,
    applied: Vec<Option<(Pose, bool)>>,
    spawned: Vec<Entity>,
}

/// the "Physics Manager" step: which compartments physics registers
fn load_steps(mut r: MessageReader<LoadStepMessage>, gm: Res<GameManager>, mut st: ResMut<PhysicsObjectRender>) {
    for m in r.read() {
        if m.step != ELoadStep::PhysicsManagerInit {
            continue;
        }
        let Some(race) = &gm.race else { continue };
        let track = race.track_path.trim_matches('\\').rsplit('\\').next().unwrap_or(&race.track).to_string();
        *st = PhysicsObjectRender { track: Some(track), race: Some(crate::game_data_loading::game_data_loading::RacePaths::new(&race.track_path, &race.race_path).race_dir), ..default() };
    }
}

/// per update (after Physics): object positions by name for the timeline objects
fn timeline_objects(objects: Res<PhysicsObjects>, mut tw: ResMut<TimelineWorld>) {
    tw.objects = objects.by_name();
}

/// per update (after Physics): the last two object poses and the visibility flags
fn snapshot(objects: Res<PhysicsObjects>, mut st: ResMut<PhysicsObjectRender>) {
    let st = &mut *st;
    if st.stage != Stage::Idle && objects.0.len() != st.count {
        log::warn!("physics objects: count {} -> {}, rebuilding", st.count, objects.0.len());
        st.stage = Stage::Idle;
    }
    if st.stage == Stage::Idle {
        st.cur = objects.0.iter().map(|o| pose(&o.matrix)).collect();
        st.prev = st.cur.clone();
    } else {
        std::mem::swap(&mut st.prev, &mut st.cur);
        for (c, o) in st.cur.iter_mut().zip(&objects.0) {
            *c = pose(&o.matrix);
        }
    }
    st.shown = objects.0.iter().map(|o| o.flags & 8 == 0 && o.flags & 2 != 0).collect();
    st.dummy = objects.0.iter().map(|o| o.is_dummy_cop_car()).collect();
}

/// the render pose of object `i` (GameManager 0x000B9010: lerp 1 while the bridge cut flag is set)
fn lerped(st: &PhysicsObjectRender, i: usize, t: f32) -> Pose {
    let (a, b) = (st.prev[i], st.cur[i]);
    if a == b { b } else { Pose { pos: a.pos.lerp(b.pos, t), rot: a.rot.slerp(b.rot, t) } }
}

/// PhysicsObjectParticleTracker.cpp 0x001E3B70 (from the object draw 0x001E0710 / 0x001E0EE0): every drawn object
/// with type tag 1 becomes a dummy police car at its matrix; refilled every rendered frame, kept while paused
fn dummy_police_cars(
    clock: Res<GameClock>,
    gm: Res<GameManager>,
    st: Res<PhysicsObjectRender>,
    bridge: Option<Res<crate::game_bridge::game_bridge::GameBridge>>,
    mut out: ResMut<visos_render::car_render::DummyPoliceCars>,
) {
    if gm.paused {
        return;
    }
    let t = clock.timer.lerp.clamp(0.0, 1.0);
    let t = bridge.map_or(t, |b| b.render_lerp(t));
    out.0.clear();
    for i in 0..st.dummy.len().min(st.shown.len()).min(st.cur.len()) {
        if !st.dummy[i] || !st.shown[i] {
            continue;
        }
        let p = lerped(&st, i, t);
        out.0.push(visos_render::car_render::DummyPoliceCar { key: i as u64, matrix: Mat4::from_rotation_translation(p.rot, p.pos) });
    }
}

fn clear_dummy_police_cars(mut out: ResMut<visos_render::car_render::DummyPoliceCars>) {
    out.0.clear();
}

fn norm_tex(n: &str) -> String {
    let l = n.to_ascii_lowercase();
    l.trim_end_matches(".vtf").trim_end_matches(".tga").to_string()
}

fn read_pack(path: &str) -> Vec<(String, Vec<u8>)> {
    let Ok(v) = visos::file::Vcd::open(path) else { return Vec::new() };
    v.entries()
        .filter(|e| e.name.to_ascii_lowercase().ends_with(".cpt"))
        .map(|e| (e.name.to_ascii_lowercase(), v.data(&e).unwrap_or_default().to_vec()))
        .collect()
}

/// which visual model each physics object draws: compartment files (the track's `CPTS.VCD`, then the race's
/// `RACE.VCD`), per object the (file, resource) of its model, and the match statistics
pub struct ObjectPlan {
    pub files: Vec<(String, Vec<u8>)>,
    pub model: Vec<Option<(usize, usize)>>,
    pub matched: usize,
    pub unmatched: usize,
    pub position_error: f32,
    pub rotation_error: f32,
}

/// match every registered physics / immovable instance to its physics object (same name hash and list type,
/// nearest position) and resolve its visual resource: the compartment's own slot, or an empty slot
/// (`uinstance.cpt`) by the slot's hash over all compartments
pub fn plan_objects(track: &str, race: &str, objects: &[velocity_physics::physics_objects::PhysicsObjectInfo]) -> ObjectPlan {
    let mut files = read_pack(&format!(r"\Tracks\{track}\CPTS.VCD"));
    let ntrack = files.len();
    files.extend(read_pack(&format!(r"{}\RACE.VCD", race.trim_end_matches('\\'))));
    let cpts: Vec<Option<Compartment>> = files.iter().map(|(_, b)| Compartment::parse(b).ok()).collect();
    let mut model_by_hash: HashMap<u32, (usize, usize)> = HashMap::new();
    for (k, c) in cpts.iter().enumerate() {
        let Some(c) = c else { continue };
        for (j, r) in c.resources.iter().enumerate() {
            if r.name.is_none() && (r.kind == 0 || r.kind == 8) {
                model_by_hash.entry(r.hash).or_insert((k, j));
            }
        }
    }
    let poses: Vec<Pose> = objects.iter().map(|o| pose(&o.matrix)).collect();
    let mut by_name: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (i, o) in objects.iter().enumerate() {
        by_name.entry((o.name, o.list_type)).or_default().push(i);
    }
    let mut plan = ObjectPlan { files: Vec::new(), model: vec![None; objects.len()], matched: 0, unmatched: 0, position_error: 0.0, rotation_error: 0.0 };
    for (k, c) in cpts.iter().enumerate() {
        let n = files[k].0.as_str();
        let source = if k < ntrack { n == "instance.cpt" } else { n == "instance.cpt" || n == "uinstance.cpt" };
        let Some(c) = c.as_ref().filter(|_| source) else { continue };
        for l in c.instance_lists().unwrap_or_default() {
            if l.kind != INSTANCE_PHYSICS && l.kind != INSTANCE_IMMOVABLE {
                continue;
            }
            for inst in &l.instances {
                let pos = Vec3::from_array(inst.position);
                let found = by_name.get_mut(&(inst.hash, l.kind)).and_then(|v| {
                    let ci = (0..v.len()).min_by(|&a, &b| poses[v[a]].pos.distance_squared(pos).total_cmp(&poses[v[b]].pos.distance_squared(pos)))?;
                    Some(v.swap_remove(ci))
                });
                let Some(oi) = found else {
                    plan.unmatched += 1;
                    continue;
                };
                plan.matched += 1;
                plan.position_error = plan.position_error.max(poses[oi].pos.distance(pos));
                let q = Quat::from_xyzw(inst.rotation[0], inst.rotation[1], inst.rotation[2], inst.rotation[3]).normalize();
                plan.rotation_error = plan.rotation_error.max(q.angle_between(poses[oi].rot));
                plan.model[oi] = match c.resources.iter().position(|r| r.index == inst.visual as usize) {
                    Some(j) => Some((k, j)),
                    None => {
                        let o = c.header.resource_table as usize + 12 * inst.visual as usize + 8;
                        c.bytes.get(o..o + 4).and_then(|h| model_by_hash.get(&u32::from_le_bytes(h.try_into().unwrap()))).copied()
                    }
                };
            }
        }
    }
    drop(cpts);
    plan.files = files;
    plan
}

/// one hidden template per visual model (built once, shared by every object that draws it)
#[allow(clippy::too_many_arguments)]
fn build_templates(
    mut st: ResMut<PhysicsObjectRender>,
    objects: Res<PhysicsObjects>,
    roots: Query<Entity, With<TrackRoot>>,
    (lights, instance_lights): (Option<Res<TrackLights>>, Option<Res<visos_render::track_render::TrackInstanceLights>>),
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if st.stage != Stage::Idle || objects.0.is_empty() {
        return;
    }
    let (Some(track), Some(race)) = (st.track.clone(), st.race.clone()) else { return };
    let Some(track_root) = roots.iter().next() else { return };
    for e in std::mem::take(&mut st.spawned) {
        commands.entity(e).try_despawn();
    }
    let plan = plan_objects(&track, &race, &objects.0);
    let cpts: Vec<Option<Compartment>> = plan.files.iter().map(|(_, b)| Compartment::parse(b).ok()).collect();
    let mut tex_at: HashMap<String, (usize, usize)> = HashMap::new();
    for (k, c) in cpts.iter().enumerate() {
        for (j, r) in c.iter().flat_map(|c| c.resources.iter().enumerate()) {
            if let Some(n) = &r.name {
                tex_at.entry(norm_tex(n)).or_insert((k, j));
            }
        }
    }
    let n = objects.0.len();
    st.root = vec![None; n];
    st.applied = vec![None; n];
    let mut templates: HashMap<(usize, usize), Option<Entity>> = HashMap::new();
    let mut tex = TexSet::default();
    let mut b = Builder::new(&mut meshes, &mut materials, &mut images);
    let mut opts = SpawnOpts::world();
    if let Some(l) = &lights {
        b.lights = l.lights.clone();
        opts.fog = l.fog;
    }
    let mut template = vec![None; n];
    for (i, key) in plan.model.iter().enumerate() {
        let Some(key) = *key else { continue };
        template[i] = *templates.entry(key).or_insert_with(|| {
            let r = &cpts[key.0].as_ref()?.resources[key.1];
            let m = Model::parse(r.data).ok()?;
            for s in m.shaders.iter().flat_map(|s| s.stages.iter()) {
                let name = norm_tex(&s.texture_name);
                if name.is_empty() || tex.by_name.contains_key(&name) {
                    continue;
                }
                let Some(&(tk, tj)) = tex_at.get(&name) else { continue };
                if let Some(Ok(d)) = cpts[tk].as_ref().map(|c| decode_any_vtf(c.resources[tj].data)) {
                    tex.add(&name, d);
                }
            }
            let e = commands.spawn((Transform::default(), Visibility::Hidden, ChildOf(track_root))).id();
            let mut o = opts.clone();
            o.parent = Some(e);
            // CompartmentManager 0x00087D00: physics / immovable instances get the light set of the visual ones
            // (record +120, by the compartment's version); the model's compartment stands for the instance's
            if let (Some(l), Some(c)) = (&lights, cpts[key.0].as_ref()) {
                let sky = instance_lights.as_ref().map_or(Vec4::ZERO, |i| i.sky_ambient);
                o.light_set = Some(l.lights.instance_light_set(c.header.version, sky));
            }
            b.add_model(&mut commands, &mut tex, &m, Mat4::IDENTITY, &o);
            Some(e)
        });
    }
    st.template = template;
    st.spawned = templates.values().flatten().copied().collect();
    log::info!(
        "physics objects: {} of {n} matched ({} instances unmatched), {} models, position error {:.4}, rotation error {:.4}",
        plan.matched,
        plan.unmatched,
        st.spawned.len(),
        plan.position_error,
        plan.rotation_error
    );
    st.count = n;
    st.stage = Stage::Templates;
}

/// one drawn copy per object of its template's meshes
fn instance_objects(
    mut st: ResMut<PhysicsObjectRender>,
    roots: Query<Entity, With<TrackRoot>>,
    children: Query<&Children>,
    parts: Query<(&Mesh3d, &MeshMaterial3d<XboxStageMaterial>)>,
    mut commands: Commands,
) {
    if st.stage != Stage::Templates {
        return;
    }
    let Some(track_root) = roots.iter().next() else { return };
    let st = &mut *st;
    let mut made = 0;
    for i in 0..st.template.len() {
        let Some(t) = st.template[i] else { continue };
        let Ok(ch) = children.get(t) else { continue };
        let p = st.cur[i];
        let vis = if st.shown[i] { Visibility::Inherited } else { Visibility::Hidden };
        let root = commands.spawn((Transform::from_translation(p.pos).with_rotation(p.rot), vis, ChildOf(track_root))).id();
        for c in ch.iter() {
            if let Ok((m, mat)) = parts.get(c) {
                commands.spawn((m.clone(), mat.clone(), ChildOf(root)));
            }
        }
        st.root[i] = Some(root);
        st.applied[i] = Some((p, st.shown[i]));
        st.spawned.push(root);
        made += 1;
    }
    log::info!("physics objects: {made} drawn");
    st.stage = Stage::Live;
}

/// render: each object between the last two updates (only moved objects are rewritten); nothing while paused
fn place_objects(
    clock: Res<GameClock>,
    gm: Res<GameManager>,
    mut st: ResMut<PhysicsObjectRender>,
    mut q: Query<(&mut Transform, &mut Visibility)>,
    bridge: Option<Res<crate::game_bridge::game_bridge::GameBridge>>,
) {
    if st.stage != Stage::Live || gm.paused {
        return;
    }
    // GameManager 0x000B9010: lerp 1 while the bridge cut flag is set
    let t = clock.timer.lerp.clamp(0.0, 1.0);
    let t = bridge.map_or(t, |b| b.render_lerp(t));
    let st = &mut *st;
    for i in 0..st.root.len() {
        let Some(e) = st.root[i] else { continue };
        let p = lerped(st, i, t);
        // 0x001E3B70 draws the police car copy instead of a dummy cop car object
        let want = (p, st.shown[i] && !st.dummy.get(i).copied().unwrap_or(false));
        if st.applied[i] == Some(want) {
            continue;
        }
        let Ok((mut tf, mut vis)) = q.get_mut(e) else { continue };
        tf.translation = p.pos;
        tf.rotation = p.rot;
        *vis = if want.1 { Visibility::Inherited } else { Visibility::Hidden };
        st.applied[i] = Some(want);
    }
}

fn reset(mut st: ResMut<PhysicsObjectRender>) {
    // the entities went with the track root
    *st = PhysicsObjectRender::default();
}

fn not_paused(gm: Res<GameManager>) -> bool {
    !gm.paused
}

pub(crate) fn build(app: &mut App) {
    if !app.world().contains_resource::<PhysicsObjects>() {
        return;
    }
    app.add_systems(
        GameTick,
        timeline_objects.in_set(InGameUpdateSet::World).run_if(in_state(EGameMode::InGame)).run_if(not_paused),
    );
    if !app.world().contains_resource::<Assets<XboxStageMaterial>>() {
        return;
    }
    app.init_resource::<PhysicsObjectRender>()
        .add_systems(PostUpdate, load_steps.run_if(in_state(EGameMode::InGame)))
        .add_systems(OnExit(crate::InRace), reset)
        .add_systems(
            GameTick,
            snapshot.in_set(InGameUpdateSet::World).run_if(in_state(EGameMode::InGame)).run_if(not_paused),
        )
        .add_systems(Update, (build_templates, instance_objects, place_objects).chain().run_if(in_state(EGameMode::InGame)));
    if app.world().contains_resource::<visos_render::car_render::DummyPoliceCars>() {
        app.add_systems(Update, dummy_police_cars.run_if(in_state(EGameMode::InGame)))
            .add_systems(OnExit(crate::InRace), clear_dummy_police_cars);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// London Circuit Race01: every registered physics / immovable instance finds its physics object at its
    /// own position and orientation, and has a model
    #[test]
    fn london_objects_match_their_instances() {
        let Some(gc) = velocity_physics::c_config::load_global_config_with_fallback(0) else { return };
        let mut w = velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld::new(gc, None);
        let race = r"\Tracks\LONDON\CIRCUIT\RACE01\";
        let mut n = 0;
        for (pack, name) in [
            (r"\Tracks\LONDON\CPTS.VCD".to_string(), "instance.cpt"),
            (format!(r"{race}RACE.VCD"), "instance.cpt"),
            (format!(r"{race}RACE.VCD"), "uinstance.cpt"),
        ] {
            if let Some(b) = visos::file::Vcd::open(&pack).ok().and_then(|v| v.get(name).map(<[u8]>::to_vec)) {
                n += w.register_compartment(&b);
            }
        }
        if n == 0 {
            return;
        }
        let objects = w.objects();
        let plan = plan_objects("LONDON", race, &objects);
        let with_model = plan.model.iter().filter(|m| m.is_some()).count();
        eprintln!(
            "{} objects, {} matched, {} unmatched, {with_model} with a model, position error {}, rotation error {}",
            objects.len(),
            plan.matched,
            plan.unmatched,
            plan.position_error,
            plan.rotation_error
        );
        assert_eq!(plan.matched, objects.len());
        assert!(plan.position_error < 1e-3 && plan.rotation_error < 1e-3);
        assert_eq!(with_model, objects.len());
    }
}
