//! glue (not an original file): the flag girl intro's models on the render crate's skinned model API — one
//! skinned model per `FlagGirlModel` of the bound intro (TimelineBridge 0x00098C60 places them), its world matrix and
//! visibility from the intro's `FlagGirlEvent`s, its anim from the intro's bone anim items (TimelineSequencer.cpp
//! 0x002B5810).

use bevy::prelude::*;
use visos_render::skinned_object_render::{
    flag_girl_model, BoneAnimCache, CharPack, DespawnSkinnedModel, SkinnedModelAnim, SkinnedModelRender, SpawnSkinnedModel,
};

use velocity_physics::plugin::PhysicsWorldRes;
use velocity_physics::vehicle::vehicle_entity::sim::{RAY_NORMAL, RAY_TRACK};

use crate::game_mode::game_manager::{EGameMode, GameManager};
use crate::game_mode::race_session::{FlagGirl, RaceSessionMessage};
use crate::timeline_manager::timeline_sequencer::{bone_anim_frame, FlagGirlEvent};

/// skinned model requests of the flag girls (request = BASE + model index)
const BASE: u64 = 0x0F1A_6000;

#[derive(Clone, Debug, Default)]
struct ModelState {
    matrix: Option<Mat4>,
    /// the matrix of the update before, and the update `matrix` was written in (TimelineBridge render 0x00098A80)
    prev_matrix: Mat4,
    matrix_tick: i32,
    visible: bool,
    anim: Option<String>,
    /// unwrapped frame of the last update, the one before, the update it came from and the anim's loop flag
    frame: Option<f32>,
    prev_frame: f32,
    frame_tick: i32,
    looping: bool,
    /// the anim the renderer was last given
    sent: Option<String>,
}

/// the spawned flag girls and what the intro last said about each
#[derive(Resource, Default)]
pub struct FlagGirlModels {
    names: Vec<String>,
    models: Vec<ModelState>,
}

/// a new intro: drop the old girls, spawn one skinned model per intro model (hidden until placed)
fn spawn_models(
    fg: Res<FlagGirl>,
    mut st: ResMut<FlagGirlModels>,
    mut spawn: MessageWriter<SpawnSkinnedModel>,
    mut despawn: MessageWriter<DespawnSkinnedModel>,
) {
    if !fg.is_changed() {
        return;
    }
    let names: Vec<String> = fg.0.as_ref().map(|f| f.peer.models.iter().map(|m| m.name.clone()).collect()).unwrap_or_default();
    if names == st.names {
        return;
    }
    for k in 0..st.models.len() {
        despawn.write(DespawnSkinnedModel { request: BASE + k as u64 });
    }
    st.models.clear();
    st.names = names.clone();
    for (k, n) in names.iter().enumerate() {
        let model = flag_girl_model(n);
        log::info!("flag girl model {k}: {n} -> {model}");
        spawn.write(SpawnSkinnedModel { request: BASE + k as u64, pack: CharPack::FlagGirl, model, anim: None, transform: Transform::default() });
        st.models.push(ModelState { visible: true, ..default() });
    }
}

/// GameManager 0x000B9010 draws flag girl k with Characters 0x0007A460(a11 = k): its SpecularChars record / light set
fn spec_char_index(
    q: Query<(Entity, &SkinnedModelRender), Without<visos_render::specular_chars::SpecCharIndex>>,
    st: Res<FlagGirlModels>,
    mut commands: Commands,
) {
    for (e, r) in &q {
        let Some(k) = r.request.checked_sub(BASE).map(|k| k as usize).filter(|k| *k < st.models.len()) else { continue };
        commands.entity(e).insert(visos_render::specular_chars::SpecCharIndex(k as u8));
    }
}

/// the intro's model events (matrix, show / hide, anim), applied to the skinned roots
fn apply_events(
    mut r: MessageReader<RaceSessionMessage>,
    mut st: ResMut<FlagGirlModels>,
    mut cache: Option<ResMut<BoneAnimCache>>,
    mut anim: MessageWriter<SkinnedModelAnim>,
    mut q: Query<(&SkinnedModelRender, &mut Transform, &mut Visibility)>,
    gm: Res<GameManager>,
    clock: Res<crate::vs_common_main::GameClock>,
    mut physics: Option<ResMut<PhysicsWorldRes>>,
    bridge: Option<Res<crate::game_bridge::game_bridge::GameBridge>>,
) {
    // GameManager::Render 0x000BBB00: the timeline bridge lerp is 1 while the bridge cut flag is set
    let lerp = clock.timer.lerp.clamp(0.0, 1.0);
    let lerp = bridge.map_or(lerp, |b| b.render_lerp(lerp));
    // events of this frame's updates arrive in update order: each new one pushes the last into `prev`
    let tick = clock.timer.total_updates;
    for m in r.read() {
        let RaceSessionMessage::FlagGirl(e) = m else { continue };
        match e {
            FlagGirlEvent::ModelMatrix { model, matrix } => {
                if let Some(s) = st.models.get_mut(*model) {
                    s.prev_matrix = s.matrix.unwrap_or(*matrix);
                    s.matrix = Some(*matrix);
                    s.matrix_tick = tick;
                }
            }
            FlagGirlEvent::ModelVisible { model, visible } => {
                if let Some(s) = st.models.get_mut(*model) {
                    s.visible = *visible;
                }
            }
            FlagGirlEvent::BoneAnim { model, animation, frame, .. } => {
                let Some(s) = st.models.get_mut(*model) else { continue };
                let looping = cache.as_deref_mut().and_then(|c| c.get(CharPack::FlagGirl, animation)).is_some_and(|a| a.looping != 0);
                if s.anim.as_deref() != Some(animation.as_str()) {
                    log::debug!("flag girl {model}: anim {animation}");
                    s.anim = Some(animation.clone());
                    s.frame = None;
                }
                s.prev_frame = s.frame.unwrap_or(*frame);
                s.frame = Some(*frame);
                s.frame_tick = tick;
                s.looping = looping;
            }
            _ => {}
        }
    }
    for (r, mut tf, mut vis) in &mut q {
        let Some(k) = r.request.checked_sub(BASE).map(|k| k as usize) else { continue };
        let Some(s) = st.models.get_mut(k) else { continue };
        // the render side samples between the last two updates (only what the latest update wrote): the bone
        // anims at the lerped sequencer time (TimelineSequencer render 0x002B4C50, frame linear in time within an
        // item), the model matrix with its translation lerped (0x00098A80)
        let t = lerp;
        if let Some(mut m) = s.matrix {
            if s.matrix_tick == tick && !gm.paused {
                let a = s.prev_matrix.w_axis;
                m.w_axis = (m.w_axis - a) * t + a;
            }
            *tf = Transform::from_matrix(m);
            let grid_y = gm.event_def.as_ref().and_then(|d| d.start_positions.get(k)).map(|p| p[1]);
            if let (Some(gy), Some(ph)) = (grid_y, physics.as_deref_mut()) {
                let y = drop_to_ground(ph, tf.translation, gy);
                log::trace!("flag girl {k}: y {} -> {y} (grid {gy})", tf.translation.y);
                tf.translation.y = y;
            }
        }
        *vis = if s.visible && s.matrix.is_some() { Visibility::Inherited } else { Visibility::Hidden };
        if let Some(f) = s.frame {
            let f = if s.frame_tick == tick && !gm.paused { (f - s.prev_frame) * t + s.prev_frame } else { f };
            let t = bone_anim_frame(f, s.looping);
            let changed = s.sent != s.anim;
            s.sent = s.anim.clone();
            anim.write(SkinnedModelAnim { request: r.request, anim: if changed { s.anim.clone() } else { None }, time: Some(t) });
        }
    }
}

/// GameManager.cpp 0x000B9010 (draw ALPHA): the model drops onto the ground under it, ray from grid y + 5 to
/// grid y - 5 (`VsAITools::28380` -> 0x0019D350, flags 0 = track | normal, no ignored car); grid slot = model index
fn drop_to_ground(ph: &mut PhysicsWorldRes, pos: Vec3, grid_y: f32) -> f32 {
    let from = Vec3::new(pos.x, grid_y + 5.0, pos.z);
    let to = Vec3::new(pos.x, grid_y - 5.0, pos.z);
    ph.ray_cast(from, to, RAY_TRACK | RAY_NORMAL, -1).map_or(pos.y, |h| h.point[1])
}

/// mode shutdown: the girls go with the race
fn despawn_models(mut st: ResMut<FlagGirlModels>, mut despawn: MessageWriter<DespawnSkinnedModel>) {
    for k in 0..st.models.len() {
        despawn.write(DespawnSkinnedModel { request: BASE + k as u64 });
    }
    st.models.clear();
    st.names.clear();
}

pub(crate) fn build(app: &mut App) {
    if !app.world().contains_resource::<Messages<SpawnSkinnedModel>>() {
        app.add_message::<SpawnSkinnedModel>().add_message::<DespawnSkinnedModel>().add_message::<SkinnedModelAnim>();
    }
    app.init_resource::<FlagGirlModels>()
        .add_systems(Update, (spawn_models, apply_events, spec_char_index).chain().run_if(in_state(EGameMode::InGame)))
        .add_systems(OnExit(crate::InRace), despawn_models);
}
