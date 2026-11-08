//! src\TimelineManager\RaceTimelineSequencer.cpp — the race's `timeline.bun`: one `TimelineSequencer` per timeline
//! (bound at load, 0x00170B80), played by TIMELINE triggers (`PlayTimeline` 0x001711E0) and stepped every update.
//!
//! Each channel is bound to a timeline object (`objects::timeline_object`) whose handlers turn typed events into
//! `TimelineObjectMessage`s; item activity is also published raw as `RaceTimelineEvent`s (bone animations and the
//! path matrices are for the owners of the animated models).

use bevy::prelude::*;
use velocity_audio::AudioCommand;
use visos::handy::crt_rand::CrtRand;
use visos::file::VsFileDirectoryDetails;
use visos::handy::handy_crc::crc32;
use visos::instances::file_instances::{FileInstance, FileInstanceLink};
use visos::timeline::file_tlf::{FileItem, FileTimeline};
use visos::timeline::timeline_sequencer::{ESequencerState, TimelinePeer, TimelineSequencer};

use super::objects::timeline_object::{
    EventPhase, HandlerCtx, TimelineAction, TimelineObjectMessage, TimelineObjects, TimelineWorld,
};
use super::timeline_sequencer::model_file_name;
use crate::game_bridge::camera_bridge::CameraBridgeData;
use crate::game_mode::trail_blazer::{TrailBlazerMessage, TrailEvent};
use crate::game_mode::update_thread::CWorldModeManager;
use crate::triggers::trigger_update::{CTriggers, TriggerMessage};
use crate::vs_common_main::GameClock;

/// what the sequencer peer was asked to do (`timeline` = index into `RaceTimelines::timelines`).
#[derive(Message, Clone, Debug)]
pub enum RaceTimelineEvent {
    Begin { timeline: usize, channel: usize, item: FileItem, t: f32, length: f32 },
    Update { timeline: usize, channel: usize, item: FileItem, u: f32 },
    End { timeline: usize, channel: usize, item: FileItem },
    /// path item result (glam convention: the D3D rows are the columns, translation in `w_axis`)
    Matrix { timeline: usize, channel: usize, matrix: Mat4 },
    BoneAnim { timeline: usize, channel: usize, item: FileItem, t: f32 },
}

#[derive(Resource, Default)]
pub struct RaceTimelines {
    pub timelines: Vec<FileTimeline>,
    pub sequencers: Vec<TimelineSequencer>,
    /// last matrix per [timeline][channel] (stands in for the bound object's matrix)
    pub matrices: Vec<Vec<Mat4>>,
    /// the bound objects (+0x18) and each channel's object name crc
    pub objects: TimelineObjects,
    pub channel_objects: Vec<Vec<Option<u32>>>,
    /// +0x7C `m_restorePointSet` (the trailblazer's restore point)
    pub restore_point_set: bool,
    /// +0x54 timelines played since the restore point (every PlayTimeline call, repeats included)
    pub restore_timelines: Vec<usize>,
    /// +0x68 physics objects (name crc) reported by messages 2 / 4 since the restore point (0x00172570)
    pub restore_objects: Vec<u32>,
    /// +0x2C `m_animations` (sorted by hash, 0x00171B90)
    pub animations: Vec<RaceTimelineResource>,
    /// +0x40 the "Model" instance links' geometry (0x00171F20)
    pub models: Vec<RaceTimelineResource>,
}

/// an animation (+0x2C, 8 bytes) or model (+0x40) record: `VsStringAnsi::Hash` of the name and what loaded
/// (`file` = the pack entry found, `None` = the load failed)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceTimelineResource {
    pub hash: u32,
    pub file: Option<String>,
}

/// VsStringAnsi.cpp 0x002B46B0 `VsStringAnsi::Hash`: case-sensitive HANDY_CRC32, 0 for an empty name
pub fn vs_string_hash(s: &str) -> u32 {
    if s.is_empty() { 0 } else { crc32(s.as_bytes(), 0) }
}

/// the cut-scene camera's view this update (player 0), `None` while no timeline camera holds it
#[derive(Resource, Default, Clone, Debug)]
pub struct TimelineCameraView(pub Option<CameraBridgeData>);

struct Peer<'a> {
    timeline: usize,
    matrices: &'a mut Vec<Mat4>,
    out: &'a mut Vec<RaceTimelineEvent>,
}

impl TimelinePeer for Peer<'_> {
    fn target_matrix(&mut self, c: usize) -> Mat4 {
        self.matrices[c]
    }
    fn set_target_matrix(&mut self, c: usize, _item: &FileItem, m: Mat4) {
        self.matrices[c] = m;
        self.out.push(RaceTimelineEvent::Matrix { timeline: self.timeline, channel: c, matrix: m });
    }
    fn event_begin(&mut self, c: usize, item: &FileItem, t: f32, length: f32) {
        self.out.push(RaceTimelineEvent::Begin { timeline: self.timeline, channel: c, item: item.clone(), t, length });
    }
    fn event_update(&mut self, c: usize, item: &FileItem, u: f32) {
        self.out.push(RaceTimelineEvent::Update { timeline: self.timeline, channel: c, item: item.clone(), u });
    }
    fn event_end(&mut self, c: usize, item: &FileItem) {
        self.out.push(RaceTimelineEvent::End { timeline: self.timeline, channel: c, item: item.clone() });
    }
    fn bone_anim(&mut self, c: usize, item: &FileItem, t: f32) {
        self.out.push(RaceTimelineEvent::BoneAnim { timeline: self.timeline, channel: c, item: item.clone(), t });
    }
}

impl RaceTimelines {
    /// RaceTimelineSequencer.cpp 0x00170570 (after the file load): bind a sequencer per timeline (0x00170B80).
    pub fn new(timelines: Vec<FileTimeline>, out: &mut Vec<RaceTimelineEvent>) -> Self {
        let mut r = RaceTimelines {
            sequencers: vec![TimelineSequencer::default(); timelines.len()],
            matrices: timelines.iter().map(|t| vec![Mat4::IDENTITY; t.channels.len()]).collect(),
            channel_objects: timelines.iter().map(|t| vec![None; t.channels.len()]).collect(),
            objects: TimelineObjects::default(),
            timelines,
            restore_point_set: false,
            restore_timelines: Vec::new(),
            restore_objects: Vec::new(),
            animations: Vec::new(),
            models: Vec::new(),
        };
        for i in 0..r.timelines.len() {
            let mut peer = Peer { timeline: i, matrices: &mut r.matrices[i], out };
            if !r.sequencers[i].bind(&r.timelines[i], &mut peer) {
                log::warn!("Error binding race timeline");
            }
        }
        r
    }

    /// RaceTimelineSequencer.cpp 0x001718F0 — timeline index by name crc.
    pub fn find(&self, name_crc: u32) -> Option<usize> {
        self.timelines.iter().position(|t| t.name_crc == name_crc)
    }

    /// RaceTimelineSequencer.cpp 0x001711E0 PlayTimeline: a timeline not already playing restarts from 0.
    pub fn play(&mut self, name_crc: u32, out: &mut Vec<RaceTimelineEvent>) {
        let Some(i) = self.find(name_crc) else {
            log::warn!("RaceTimelineData::PlayTimeline: Could not find timeline");
            return;
        };
        if self.restore_point_set {
            self.restore_timelines.push(i);
        }
        let mut peer = Peer { timeline: i, matrices: &mut self.matrices[i], out };
        let s = &mut self.sequencers[i];
        if s.state != ESequencerState::Playing {
            s.set_time(0.0, &self.timelines[i], &mut peer);
            s.play(&self.timelines[i], &mut peer);
        }
    }

    /// RaceTimelineSequencer.cpp 0x001723D0 — per update: step every sequencer.
    pub fn update(&mut self, dt: f32, out: &mut Vec<RaceTimelineEvent>) {
        for i in 0..self.timelines.len() {
            let mut peer = Peer { timeline: i, matrices: &mut self.matrices[i], out };
            self.sequencers[i].update(dt, &self.timelines[i], &mut peer);
        }
    }

    /// RaceTimelineSequencer.cpp 0x00170B80 — every channel's object, made from the bundle instance of the same
    /// name (channels of an unknown type stay unbound)
    pub fn bind_objects(&mut self, instances: &[FileInstance], ctx: &mut HandlerCtx) {
        for t in 0..self.timelines.len() {
            for c in 0..self.timelines[t].channels.len() {
                let ch = &self.timelines[t].channels[c];
                let o = self.objects.bind(ch.crc, ch.name.as_deref(), ch.type_crc, instances, ctx);
                self.channel_objects[t][c] = o.map(|i| self.objects.objects[i].name_crc);
            }
        }
        self.objects.post_bind();
    }

    /// RaceTimelineSequencer.cpp 0x00171B90 LoadAnimations — every channel's bone animation (+0x0C) from `\ASP\ASP.vcd`
    /// (GDL_LoadAnim 0x000A0080: name + ".gaf"), kept sorted by name hash (qsort 0x00171B30); "undefined" names are
    /// cleared, vertex animations (+0x10) are not supported (logged and cleared). Duplicates load again, as the original.
    pub fn load_animations(&mut self, dir: Option<&VsFileDirectoryDetails>) {
        if !self.animations.is_empty() {
            log::error!("assert: m_animations.GetCount() == 0");
        }
        const UNDEFINED: &str = "undefined";
        for t in &mut self.timelines {
            for ch in &mut t.channels {
                if ch.str_a.as_deref().is_some_and(|a| a.eq_ignore_ascii_case(UNDEFINED)) {
                    ch.str_a = None;
                }
                if let Some(v) = ch.str_b.take() {
                    if !v.eq_ignore_ascii_case(UNDEFINED) {
                        log::warn!("RaceTimelineData::LoadAnimations - vertex animation not supported, found one called '{v}'");
                    }
                }
            }
        }
        for t in &self.timelines {
            for ch in &t.channels {
                let Some(name) = ch.str_a.as_deref() else { continue };
                let f = format!("{name}.gaf");
                let file = dir.and_then(|d| d.load_file(&f).ok()).map(|_| f);
                if file.is_none() {
                    log::warn!("Failed to load anim \"{name}.gaf\"");
                    log::error!("assert: anim.m_pAnimation");
                }
                self.animations.push(RaceTimelineResource { hash: vs_string_hash(name), file });
            }
        }
        self.animations.sort_by_key(|a| a.hash);
    }

    /// RaceTimelineSequencer.cpp 0x00171F20 — every instance link of type "Model" loads its geometry from `\ASP\ASP.vcd`
    /// (LoadModelGeometry 0x0009FE10, the same pack pushed as the texture directory) into the link's +0x0C; the
    /// geometry itself is the render crate's, this keeps the lookup (hash + file) and its log
    pub fn load_models(&mut self, links: &[FileInstanceLink], dir: Option<&VsFileDirectoryDetails>) {
        let model = crc32(b"Model", 0);
        for l in links.iter().filter(|l| l.crc == model) {
            let name = l.a.as_deref().unwrap_or("");
            let f = model_file_name(name);
            let file = dir.and_then(|d| d.load_file(&f).ok()).map(|_| f.clone());
            if file.is_none() {
                log::warn!("Failed to load model \"{f}\"");
                log::error!("assert: modelResource.m_pModel");
            }
            self.models.push(RaceTimelineResource { hash: vs_string_hash(name), file });
        }
    }

    /// RaceTimelineSequencer.cpp 0x00172260 — the animation of that name crc (sorted search); a miss returns the
    /// first animation (original), `None` only when there are none
    pub fn find_animation(&self, name_crc: u32) -> Option<&RaceTimelineResource> {
        for a in &self.animations {
            if a.hash == name_crc {
                return Some(a);
            }
            if a.hash > name_crc {
                break;
            }
        }
        self.animations.first()
    }

    /// RaceTimelineSequencer.cpp 0x00172320 — the model of that name crc (linear search)
    pub fn find_model(&self, name_crc: u32) -> Option<&RaceTimelineResource> {
        self.models.iter().find(|m| m.hash == name_crc)
    }

    /// RaceTimelineSequencer.cpp 0x00170E50 (GameManager 0x000BAC30 on the race's shutdown): objects, sequencers,
    /// animations, models and the restore lists freed
    pub fn unload(&mut self) {
        *self = RaceTimelines::default();
    }

    /// RaceTimelineSequencer.cpp 0x00171100 — race restart: every object's reset, every sequencer stopped and
    /// rewound
    pub fn reset(&mut self, ctx: &mut HandlerCtx) {
        self.objects.reset(ctx);
        let mut sink = Vec::new();
        for i in 0..self.timelines.len() {
            let mut peer = Peer { timeline: i, matrices: &mut self.matrices[i], out: &mut sink };
            self.sequencers[i].stop(&self.timelines[i], &mut peer);
            self.sequencers[i].set_time(0.0, &self.timelines[i], &mut peer);
            self.sequencers[i].played = false;
        }
        self.keep_restore_point();
    }

    /// RaceTimelineSequencer.cpp 0x00171490 — start recording what a restore has to undo (the listeners for
    /// messages 2 and 4 are the `restore_objects` feed, `record_physics_object`)
    pub fn set_restore_point(&mut self) {
        if self.restore_point_set {
            log::error!("assert: !m_restorePointSet");
        }
        self.restore_point_set = true;
    }

    /// RaceTimelineSequencer.cpp 0x00172570 — a physics object hit (message 2, 0x0019FD80) or woken (message 4,
    /// 0x001E4710) while the restore point is set, recorded once
    pub fn record_physics_object(&mut self, object: u32) {
        if self.restore_point_set && !self.restore_objects.contains(&object) {
            self.restore_objects.push(object);
        }
    }

    /// RaceTimelineSequencer.cpp 0x001713D0 — stops the emitter samples of every TimelineSkinnedModel object
    /// (0x0017EB80); no such object is ever made ("Model" binds to physics), so nothing to do
    pub fn restore(&mut self) {}

    /// RaceTimelineSequencer.cpp 0x00171620 — stop and rewind every timeline played since the restore point
    /// (0x002B4E30), reset the objects of its channels (0x00171710), reset the recorded physics objects
    /// (0x001E37D0), then drop the restore point (0x00171580)
    pub fn restore_and_clear(&mut self, ctx: &mut HandlerCtx) {
        let mut sink = Vec::new();
        for i in std::mem::take(&mut self.restore_timelines) {
            let mut peer = Peer { timeline: i, matrices: &mut self.matrices[i], out: &mut sink };
            self.sequencers[i].stop(&self.timelines[i], &mut peer);
            self.sequencers[i].set_time(0.0, &self.timelines[i], &mut peer);
            self.sequencers[i].played = false;
            for c in 0..self.timelines[i].channels.len() {
                let bound = self.channel_objects[i][c];
                match bound.and_then(|n| self.objects.find(n)) {
                    Some(o) => self.objects.reset_object(o, ctx),
                    None => log::error!("assert: target->objectRef.pUserData"),
                }
            }
        }
        for o in std::mem::take(&mut self.restore_objects) {
            ctx.out.push(TimelineObjectMessage::ResetPhysicsObject { object: Some(o) });
        }
        self.keep_restore_point();
    }

    /// RaceTimelineSequencer.cpp 0x00171580 — forget the restore point (lists cleared, listeners removed)
    pub fn keep_restore_point(&mut self) {
        self.restore_timelines.clear();
        self.restore_objects.clear();
        self.restore_point_set = false;
    }

    /// the objects' side of the peer: typed events to their handlers, path matrices to the objects; timelines
    /// played by handlers start (and dispatch) in the same pass
    pub fn dispatch(&mut self, events: &mut Vec<RaceTimelineEvent>, ctx: &mut HandlerCtx) {
        let mut done = 0;
        while done < events.len() {
            let e = events[done].clone();
            done += 1;
            let (t, c) = match &e {
                RaceTimelineEvent::Begin { timeline, channel, .. }
                | RaceTimelineEvent::Update { timeline, channel, .. }
                | RaceTimelineEvent::End { timeline, channel, .. }
                | RaceTimelineEvent::Matrix { timeline, channel, .. } => (*timeline, *channel),
                RaceTimelineEvent::BoneAnim { .. } => continue,
            };
            let bound = self.channel_objects.get(t).and_then(|v| v.get(c)).copied().flatten();
            let Some(i) = bound.and_then(|n| self.objects.find(n)) else { continue };
            match &e {
                RaceTimelineEvent::Begin { item, t, length, .. } => {
                    self.objects.event(i, item, EventPhase::Begin { t: *t, length: *length }, ctx)
                }
                RaceTimelineEvent::Update { item, u, .. } => self.objects.event(i, item, EventPhase::Update { u: *u }, ctx),
                RaceTimelineEvent::End { item, .. } => self.objects.event(i, item, EventPhase::End, ctx),
                RaceTimelineEvent::Matrix { matrix, .. } => self.objects.set_matrix(i, *matrix, ctx),
                RaceTimelineEvent::BoneAnim { .. } => {}
            }
            for a in std::mem::take(ctx.actions) {
                let TimelineAction::Play(name) = a;
                self.play(name, events);
            }
        }
    }
}

/// what the objects read and write besides the timelines themselves
#[derive(bevy::ecs::system::SystemParam)]
pub struct TimelineObjectParams<'w, 's> {
    pub triggers: ResMut<'w, CTriggers>,
    pub wmm: ResMut<'w, CWorldModeManager>,
    pub world: Res<'w, TimelineWorld>,
    pub rand: Local<'s, CrtRand>,
}

/// one read position over `TrailBlazerMessage` shared by both `trail_blazer_timeline_calls` instances, and the
/// timeline events their PlayTimeline calls made (dispatched with the update's own by `update_race_timelines`)
#[derive(Resource, Default)]
pub struct TrailBlazerTimelineCalls {
    cursor: bevy::ecs::message::MessageCursor<TrailBlazerMessage>,
    events: Vec<RaceTimelineEvent>,
}

/// the trailblazer's calls into `TimelineBridge__98D10(&unk_6F5BC8)`, made inside the ai update. Runs before the
/// physics frame (Ai-set messages, in race_cars' Physics chain) and again after the ai's trigger pass (Timeline
/// set), like `apply_trail_blazer_calls`, so a restore point is set / cleared before that frame's object messages
/// 2 / 4 arrive and RestoreAndClear's object resets reach the same physics frame, as in the original.
pub fn trail_blazer_timeline_calls(
    msgs: Res<Messages<TrailBlazerMessage>>,
    mut calls: ResMut<TrailBlazerTimelineCalls>,
    tl: Option<ResMut<RaceTimelines>>,
    mut p: TimelineObjectParams,
    mut ow: MessageWriter<TimelineObjectMessage>,
    mut aw: MessageWriter<AudioCommand>,
    mut tw: MessageWriter<TriggerMessage>,
) {
    let TrailBlazerTimelineCalls { cursor, events } = &mut *calls;
    let Some(mut tl) = tl else {
        cursor.clear(&msgs);
        return;
    };
    let (mut out, mut audio, mut actions, mut triggered) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut ctx = HandlerCtx {
        triggers: &mut p.triggers,
        wmm: &mut p.wmm,
        rand: &mut p.rand,
        world: &p.world,
        out: &mut out,
        audio: &mut audio,
        actions: &mut actions,
        triggered: &mut triggered,
    };
    for m in cursor.read(&msgs) {
        match m.event {
            TrailEvent::TimelineSetRestorePoint => tl.set_restore_point(),
            TrailEvent::TimelineRestore => tl.restore(),
            TrailEvent::TimelineRestoreAndClear => tl.restore_and_clear(&mut ctx),
            TrailEvent::TimelineKeep => tl.keep_restore_point(),
            TrailEvent::PlayTimeline(n) => tl.play(n, events),
            _ => {}
        }
        for a in std::mem::take(ctx.actions) {
            let TimelineAction::Play(name) = a;
            tl.play(name, events);
        }
    }
    ow.write_batch(out);
    aw.write_batch(audio);
    tw.write_batch(triggered);
}

/// the restore point's listeners on MessageManager channels 2 (car hit an object, PhysicsManager 0x0019FD80) and
/// 4 (object woken, 0x001E4710) → 0x00172570, fed with the physics frame's notices right after it ran
pub fn record_object_notices(
    tl: Option<ResMut<RaceTimelines>>,
    mut r: MessageReader<velocity_physics::plugin::PhysicsObjectNoticeMessage>,
) {
    use velocity_physics::physics_objects::PhysicsObjectNotice;
    let Some(mut tl) = tl else {
        r.clear();
        return;
    };
    for m in r.read() {
        let (PhysicsObjectNotice::Hit { object } | PhysicsObjectNotice::Woken { object }) = m.0;
        tl.record_physics_object(object);
    }
}

/// `TimelineObjectMessage::ResetPhysicsObject { object: Some(name) }` (TimelinePhysics 0x0017DAA0, RestoreAndClear
/// 0x00171620, the physics timeline object's reset) waiting for the next physics frame: messages written after it
/// would expire before the next GameTick at high frame rates, so they are collected here
#[derive(Resource, Default)]
pub struct PhysicsObjectResets {
    cursor: bevy::ecs::message::MessageCursor<TimelineObjectMessage>,
    pending: Vec<u32>,
    /// `{ object: None }` = 0x001DE400 on the physics object manager (PhysicsManager+24): every placed object
    reset_all: u32,
    /// F3EventInstanceMove (TimelinePhysics 0x0017D540)
    moves: Vec<velocity_physics::plugin::PhysicsInstanceMove>,
    /// F3EventExplosion (TimelineExplosion 0x0017ACD0 -> PhysicsManager 0x0019FD80)
    explosions: Vec<velocity_physics::plugin::PhysicsExplosion>,
    /// `GameModeMessage::RaceRestart` seen since the last physics frame
    restart_cursor: bevy::ecs::message::MessageCursor<crate::game_mode::game_manager::GameModeMessage>,
    restart: u32,
}

impl PhysicsObjectResets {
    fn collect(&mut self, msgs: &Messages<TimelineObjectMessage>) {
        for m in self.cursor.read(msgs) {
            match *m {
                TimelineObjectMessage::ResetPhysicsObject { object: Some(o) } => self.pending.push(o),
                TimelineObjectMessage::ResetPhysicsObject { object: None } => self.reset_all += 1,
                TimelineObjectMessage::InstanceMove { object, location, velocity, angular_velocity } => {
                    self.moves.push(velocity_physics::plugin::PhysicsInstanceMove { object, location, velocity, angular_velocity })
                }
                TimelineObjectMessage::Explosion {
                    position,
                    blast_radius,
                    force_objects,
                    angular_force_objects,
                    force_vehicles,
                    angular_force_vehicles,
                    sector_angle,
                    ignore_player,
                } => self.explosions.push(velocity_physics::plugin::PhysicsExplosion(
                    velocity_physics::vehicle::vehicle_entity::sim::Explosion {
                        position: position.to_array(),
                        blast_radius,
                        force_objects,
                        angular_force_objects,
                        force_vehicles,
                        angular_force_vehicles,
                        sector_angle,
                        ignore_vehicle: ignore_player,
                    },
                )),
                _ => {}
            }
        }
    }
}

/// collect the resets written since the physics frame (after the Timeline set)
pub fn collect_object_resets(
    msgs: Res<Messages<TimelineObjectMessage>>,
    modes: Option<Res<Messages<crate::game_mode::game_manager::GameModeMessage>>>,
    mut r: ResMut<PhysicsObjectResets>,
) {
    r.collect(&msgs);
    r.collect_restarts(modes.as_deref());
}

impl PhysicsObjectResets {
    fn collect_restarts(&mut self, modes: Option<&Messages<crate::game_mode::game_manager::GameModeMessage>>) {
        if let Some(modes) = modes {
            let n = self.restart_cursor.read(modes).filter(|m| **m == crate::game_mode::game_manager::GameModeMessage::RaceRestart).count();
            self.restart += n as u32;
        }
    }
}

/// 0x001E1C00 + 0x001E37D0 for every collected reset, applied by the physics frame that follows (race_cars'
/// Physics chain, before `physics_tick`)
pub fn send_object_resets(
    msgs: Res<Messages<TimelineObjectMessage>>,
    mut r: ResMut<PhysicsObjectResets>,
    mut w: MessageWriter<velocity_physics::plugin::ResetPhysicsObject>,
    mut all: MessageWriter<velocity_physics::plugin::ResetAllPhysicsObjects>,
    mut mv: MessageWriter<velocity_physics::plugin::PhysicsInstanceMove>,
    mut ex: MessageWriter<velocity_physics::plugin::PhysicsExplosion>,
    mut restart: MessageWriter<velocity_physics::plugin::RestartPhysicsWorld>,
    modes: Option<Res<Messages<crate::game_mode::game_manager::GameModeMessage>>>,
) {
    r.collect(&msgs);
    r.collect_restarts(modes.as_deref());
    // GameManager::RestartRace 0x000BE0F0 -> MaterialProperties 0x0019C390 (applied first in the physics Setup)
    for _ in 0..std::mem::take(&mut r.restart) {
        restart.write(velocity_physics::plugin::RestartPhysicsWorld);
    }
    for e in std::mem::take(&mut r.explosions) {
        ex.write(e);
    }
    for o in std::mem::take(&mut r.pending) {
        w.write(velocity_physics::plugin::ResetPhysicsObject(o));
    }
    for _ in 0..std::mem::take(&mut r.reset_all) {
        all.write(velocity_physics::plugin::ResetAllPhysicsObjects);
    }
    for m in std::mem::take(&mut r.moves) {
        mv.write(m);
    }
}

/// TIMELINE triggers, then the per-update step (InGameUpdateSet::Timeline), the objects' handlers and the
/// cut-scene cameras.
#[allow(clippy::too_many_arguments)]
pub fn update_race_timelines(
    mut tl: ResMut<RaceTimelines>,
    clock: Res<GameClock>,
    mut trig: ParamSet<(MessageReader<TriggerMessage>, MessageWriter<TriggerMessage>)>,
    mut p: TimelineObjectParams,
    mut view: ResMut<TimelineCameraView>,
    mut w: MessageWriter<RaceTimelineEvent>,
    mut ow: MessageWriter<TimelineObjectMessage>,
    mut aw: MessageWriter<AudioCommand>,
    mut calls: ResMut<TrailBlazerTimelineCalls>,
) {
    // the trailblazer's PlayTimeline calls of this update (`trail_blazer_timeline_calls`) come first
    let mut out = std::mem::take(&mut calls.events);
    let plays: Vec<u32> = trig
        .p0()
        .read()
        .filter_map(|m| if let TriggerMessage::Timeline { name_crc } = m { Some(*name_crc) } else { None })
        .collect();
    let (mut msgs, mut audio, mut actions, mut triggered) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for n in plays {
        tl.play(n, &mut out);
    }
    let dt = clock.timer.scaled_dt();
    tl.update(dt, &mut out);
    let mut ctx = HandlerCtx {
        triggers: &mut p.triggers,
        wmm: &mut p.wmm,
        rand: &mut p.rand,
        world: &p.world,
        out: &mut msgs,
        audio: &mut audio,
        actions: &mut actions,
        triggered: &mut triggered,
    };
    tl.dispatch(&mut out, &mut ctx);
    view.0 = tl.objects.update_cameras(dt, &mut ctx);
    for e in out {
        w.write(e);
    }
    for m in msgs {
        ow.write(m);
    }
    for a in audio {
        aw.write(a);
    }
    let mut tw = trig.p1();
    for t in triggered {
        tw.write(t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_a_shipped_race_timeline() {
        // every race dir with a timeline.bun; play each timeline to the end without panicking
        let root = visos::file::platform::resolve("\\Tracks");
        let Ok(rd) = std::fs::read_dir(&root) else { return };
        let mut played = 0;
        for track in rd.flatten() {
            for f in walk(&track.path()) {
                let Ok(b) = std::fs::read(&f) else { continue };
                let mut env = visos::type_::vs_type::CEnvironment::with_builtins();
                let Ok(bun) = visos::chunk::Bundle::load(&b, &mut env) else { continue };
                let mut out = Vec::new();
                let mut r = RaceTimelines::new(bun.timelines.clone(), &mut out);
                for t in &bun.timelines {
                    r.play(t.name_crc, &mut out);
                }
                for _ in 0..24 * 60 {
                    r.update(1.0 / 24.0, &mut out);
                }
                played += bun.timelines.len();
                for e in &out {
                    if let RaceTimelineEvent::Matrix { matrix, .. } = e {
                        assert!(matrix.is_finite());
                    }
                }
            }
        }
        eprintln!("played {played} race timelines");
    }

    /// every shipped race timeline bound to its objects and played through the handlers: every channel of a
    /// registered type finds its object, and the handlers produce messages
    #[test]
    fn objects_bind_and_handle_shipped_timelines() {
        let root = visos::file::platform::resolve("\\Tracks");
        let Ok(rd) = std::fs::read_dir(&root) else { return };
        let (mut channels, mut typed_channels, mut bound, mut msgs, mut cams) = (0, 0, 0, 0, 0);
        let mut kinds = std::collections::BTreeMap::new();
        for track in rd.flatten() {
            for f in walk(&track.path()) {
                let Ok(b) = std::fs::read(&f) else { continue };
                let mut env = visos::type_::vs_type::CEnvironment::with_builtins();
                let Ok(bun) = visos::chunk::Bundle::load(&b, &mut env) else { continue };
                let mut out = Vec::new();
                let mut r = RaceTimelines::new(bun.timelines.clone(), &mut out);
                let mut triggers = CTriggers::default();
                let mut wmm = CWorldModeManager::default();
                let mut rand = CrtRand::default();
                let world = TimelineWorld { racers: 1, ..Default::default() };
                let (mut m, mut a, mut ac, mut tr) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
                let mut ctx = HandlerCtx {
                    triggers: &mut triggers,
                    wmm: &mut wmm,
                    rand: &mut rand,
                    world: &world,
                    out: &mut m,
                    audio: &mut a,
                    actions: &mut ac,
                    triggered: &mut tr,
                };
                let inst = bun.instances.as_ref().map(|i| i.instances.clone()).unwrap_or_default();
                r.bind_objects(&inst, &mut ctx);
                for t in &r.timelines {
                    for c in &t.channels {
                        channels += 1;
                        if super::super::objects::timeline_object::object_type(c.type_crc).is_some() {
                            typed_channels += 1;
                        }
                    }
                }
                bound += r.channel_objects.iter().flatten().filter(|o| o.is_some()).count();
                for o in &r.objects.objects {
                    *kinds.entry(format!("{:?}", o.kind)).or_insert(0) += 1;
                }
                for t in r.timelines.iter().map(|t| t.name_crc).collect::<Vec<_>>() {
                    r.play(t, &mut out);
                }
                for _ in 0..24 * 60 {
                    r.update(1.0 / 24.0, &mut out);
                    r.dispatch(&mut out, &mut ctx);
                    if r.objects.update_cameras(1.0 / 24.0, &mut ctx).is_some() {
                        cams += 1;
                    }
                    out.clear();
                }
                msgs += ctx.out.len() + ctx.audio.len();
            }
        }
        eprintln!("channels {channels}, of a known type {typed_channels}, bound {bound}, messages {msgs}, camera frames {cams}, {kinds:?}");
        assert_eq!(bound, typed_channels);
    }

    /// the trailblazer's restore point: timelines played after it are stopped and rewound by RestoreAndClear,
    /// timelines played before it keep running; Keep forgets it
    #[test]
    fn restore_point_stops_timelines_played_since() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        let b = std::fs::read(visos::file::platform::resolve(r"\Tracks\LADOWN\CIRCUIT\RACE02\Timeline.bun")).unwrap();
        let mut env = visos::type_::vs_type::CEnvironment::with_builtins();
        let bun = visos::chunk::Bundle::load(&b, &mut env).unwrap();
        assert!(bun.timelines.len() >= 2, "{} timelines", bun.timelines.len());
        let mut out = Vec::new();
        let mut r = RaceTimelines::new(bun.timelines.clone(), &mut out);
        let (a, c) = (r.timelines[0].name_crc, r.timelines[1].name_crc);
        r.play(a, &mut out);
        r.set_restore_point();
        r.play(c, &mut out);
        assert_eq!(r.restore_timelines, vec![1]);
        r.update(0.1, &mut out);
        let mut triggers = CTriggers::default();
        let mut wmm = CWorldModeManager::default();
        let mut rand = CrtRand::default();
        let world = TimelineWorld::default();
        let (mut m, mut au, mut ac, mut tr) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut ctx = HandlerCtx {
            triggers: &mut triggers,
            wmm: &mut wmm,
            rand: &mut rand,
            world: &world,
            out: &mut m,
            audio: &mut au,
            actions: &mut ac,
            triggered: &mut tr,
        };
        r.restore_and_clear(&mut ctx);
        assert!(!r.restore_point_set && r.restore_timelines.is_empty());
        assert_ne!(r.sequencers[1].state, ESequencerState::Playing);
        r.play(a, &mut out);
        assert!(r.restore_timelines.is_empty());
    }

    /// 0x00171B90 / 0x00171F20 on every shipped race timeline: which animations and models the original's lookup
    /// finds in `\ASP\ASP.vcd`, and the sorted hash search
    #[test]
    fn race_timeline_anim_and_model_loads() {
        let Ok(asp) = VsFileDirectoryDetails::open("\\ASP\\ASP.vcd") else { return };
        let root = visos::file::platform::resolve("\\Tracks");
        let Ok(rd) = std::fs::read_dir(&root) else { return };
        let (mut anims, mut anims_ok, mut models, mut models_ok) = (0, 0, 0, 0);
        let mut missing = Vec::new();
        for track in rd.flatten() {
            for f in walk(&track.path()) {
                let Ok(b) = std::fs::read(&f) else { continue };
                let mut env = visos::type_::vs_type::CEnvironment::with_builtins();
                let Ok(bun) = visos::chunk::Bundle::load(&b, &mut env) else { continue };
                let mut out = Vec::new();
                let mut r = RaceTimelines::new(bun.timelines.clone(), &mut out);
                r.load_animations(Some(&asp));
                let links = bun.instances.as_ref().map(|i| i.links.clone()).unwrap_or_default();
                r.load_models(&links, Some(&asp));
                assert!(r.animations.windows(2).all(|w| w[0].hash <= w[1].hash));
                for a in &r.animations {
                    assert_eq!(r.find_animation(a.hash).map(|x| x.hash), Some(a.hash));
                }
                for m in &r.models {
                    assert_eq!(r.find_model(m.hash).map(|x| x.hash), Some(m.hash));
                }
                assert!(r.timelines.iter().flat_map(|t| &t.channels).all(|c| c.str_b.is_none()));
                for t in &r.timelines {
                    for c in &t.channels {
                        if let Some(a) = c.str_a.as_deref() {
                            if r.find_animation(vs_string_hash(a)).and_then(|x| x.file.as_ref()).is_none() {
                                missing.push(a.to_string());
                            }
                        }
                    }
                }
                anims += r.animations.len();
                anims_ok += r.animations.iter().filter(|a| a.file.is_some()).count();
                models += r.models.len();
                models_ok += r.models.iter().filter(|m| m.file.is_some()).count();
            }
        }
        eprintln!("race timeline anims {anims_ok}/{anims}, models {models_ok}/{models}");
        // LADown circuit Race04's "BB_Fall" is shipped in no pack: the original logs "Failed to load anim" too
        assert_eq!(missing, vec!["BB_Fall".to_string()]);
        assert_eq!(anims_ok + 1, anims);
        assert_eq!(models_ok, models);
    }

    fn walk(p: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut v = Vec::new();
        if let Ok(rd) = std::fs::read_dir(p) {
            for e in rd.flatten() {
                let q = e.path();
                if q.is_dir() {
                    v.extend(walk(&q));
                } else if q.file_name().is_some_and(|n| n.eq_ignore_ascii_case("timeline.bun")) {
                    v.push(q);
                }
            }
        }
        v
    }
}
