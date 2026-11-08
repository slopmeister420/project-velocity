//! src\TimelineManager\Objects\TimelineAI.cpp (base classes) + RaceTimelineSequencer.cpp object registry — every
//! race timeline channel is bound to a timeline object, one per bundle instance name, whose type comes from the
//! channel's type name (table built at 0x004270B0, registry 0x0016F5B0). Typed events go to the handler of the
//! object type registered for the event's type name (the per-type handler lists at 0x00687770..0x006877E8);
//! events without a handler are ignored.
//!
//! What the handlers do to other subsystems comes out as `TimelineObjectMessage`s (and `AudioCommand`s where the
//! audio crate has the call); timelines, triggers and the cut-scene cameras are the core's and are handled here.

use bevy::math::{Mat4, Vec3};
use bevy::prelude::Message;
use velocity_audio::AudioCommand;
use visos::handy::crt_rand::CrtRand;
use visos::handy::handy_crc::crc32;
use visos::instances::file_instances::FileInstance;
use visos::timeline::file_tlf::{FileItem, FilePayload};
use visos::type_::vs_type::Value;

use super::timeline_camera::TimelineCamera;
use crate::game_mode::update_thread::CWorldModeManager;
use crate::triggers::trigger_update::CTriggers;

/// the object types (`m_objectTypeInfo` index)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ETimelineObjectType {
    Camera = 0,
    CameraShaker = 1,
    ScreenEffectController = 2,
    AiController = 3,
    Physics = 4,
    AudioEmitter = 5,
    AudioController = 6,
    TimelineController = 7,
    ParticleGroupController = 8,
    HeatHaze = 9,
    Trigger = 10,
    Explosion = 11,
    /// registered as "Model" (0x00427E15) but the type table sends "Model" to Physics, so never created
    SkinnedModel = 12,
}

/// channel type name -> object type (`unk_74A6C0`, 22 entries)
pub const OBJECT_TYPES: [(&str, ETimelineObjectType); 22] = {
    use ETimelineObjectType::*;
    [
        ("VSCamera", Camera),
        ("F3CameraShaker", CameraShaker),
        ("F3ScreenEffectController", ScreenEffectController),
        ("F3AIController", AiController),
        ("F3ExplosionObject", Explosion),
        ("F3CollisionObject", Physics),
        ("F3ImmovableObject", Physics),
        ("F3PhysicsObject", Physics),
        ("Model", Physics),
        ("VSAudioEmitter", AudioEmitter),
        ("F3AudioController", AudioController),
        ("F3TimelineController", TimelineController),
        ("F3ParticleGroupController", ParticleGroupController),
        ("F3HeatHaze", HeatHaze),
        ("F3CrashCamStatusTrigger", Trigger),
        ("F3SimpleTrigger", Trigger),
        ("F3SlowDownTimeTrigger", Trigger),
        ("F3TrailBlazerTrigger", Trigger),
        ("F3KillTrigger", Trigger),
        ("F3CameraSwitchTrigger", Trigger),
        ("F3ClampSpeedTrigger", Trigger),
        ("F3SpeedTrigger", Trigger),
    ]
};

/// the event handlers each object type registers (by event type name)
pub fn handlers(t: ETimelineObjectType) -> &'static [&'static str] {
    use ETimelineObjectType::*;
    match t {
        Camera => &["F3EventCutSceneCamera", "F3EventCameraEnvelope"],
        CameraShaker => &["F3EventCameraEnvelope"],
        ScreenEffectController => &[
            "F3EventScreenFlash",
            "F3EventCinematicBars",
            "F3EventPlayerCameraShake",
            "F3EventStoryTFTMovie",
            "F3EventTrailBlazerFirewall",
        ],
        AiController => &["F3EventEnableRaceSection", "F3EventRespawnPlayerOnRaceline", "F3EventPlayerAIControl", "F3EventRespawn"],
        Physics => &[
            "F3EventInstanceSwap",
            "F3EventInstanceVisibility",
            "F3EventInstanceMove",
            "F3EventPhysicsObjectParticleSys",
            "F3EventResetPhysicsObject",
        ],
        AudioEmitter => &["F3EventSound", "F3EventSoundStop", "F3Event2DSound"],
        AudioController => &[
            "F3EventAudioFaderPreset",
            "F3EventAudioToggleEnvironment",
            "F3EventAudioToggleMusic",
            "F3EventAudioToggleReverb",
            "F3EventGlobalAudio",
        ],
        TimelineController => &["F3EventTimeline"],
        ParticleGroupController => &["F3EventParticleSys"],
        HeatHaze => &["F3EventHeatHaze"],
        Trigger => &["F3EventTrigger"],
        Explosion => &["F3EventExplosion"],
        SkinnedModel => &["F3EventInstanceVisibility"],
    }
}

/// RaceTimelineSequencer.cpp 0x00172F20 lookup — CCRC of the type name (case sensitive)
pub fn object_type(type_crc: u32) -> Option<ETimelineObjectType> {
    OBJECT_TYPES.iter().find(|(n, _)| crc32(n.as_bytes(), 0) == type_crc).map(|&(_, t)| t)
}

/// what the race timeline objects ask of other subsystems
#[derive(Message, Clone, Debug, PartialEq)]
pub enum TimelineObjectMessage {
    /// TimelineAI.cpp 0x00176B50 — `VsAIRoadSection +37` of the race section whose raceline id matches
    /// ("Race section %d not found" when none does)
    EnableRaceSection { raceline: u32, enable: bool },
    /// TimelineAI.cpp 0x00176D90 — every race car respawned on the raceline (0x000208D0)
    RespawnRacersOnRaceline { value: f32 },
    /// TimelineAI.cpp 0x00176F00 — single player only: the player car's human control (0x00018920)
    PlayerHumanControl { human: bool },
    /// TimelineAI.cpp 0x00177150 — race (then TFT) cars inside `area` (or human driven) go to the respawn points in
    /// turn: each point probed 10 m up/down for the ground, placed 0.75 up, raced on from the first race section
    /// `inside`: the area trigger's per-vehicle-id bits; `points`: RespawnPoint1..16 (None = no such trigger)
    Respawn { inside: u32, points: Vec<Option<(Vec3, bevy::math::Quat)>>, speed: f32 },
    /// TimelineCamera.cpp: the cut-scene camera took the view (player 0 on the cine camera) / gave it back
    CutSceneCamera { active: bool },
    /// DangerZoneRender 0x00120BF0(focus, blend 0, off 0) (cut scene reset 0x00178740, envelope 0x00179480)
    DepthOfFieldFocus { focus: f32 },
    /// DangerZoneRender 0x00120CF0 (restore after the last cut scene, 0x001783C0)
    DepthOfField { focus: f32, factor: f32 },
    /// 0x00178740 on `dword_718BA8`: 0x00179AE0 force +5 = 1 (never cleared), 0x000BFF70 enabled +1, 0x00179B00
    /// blend from / to +0x38 / +0x3C = focus near / far, 0x00179A30 car focus +3 = 0
    DepthOfFieldSetup { enabled: bool, near: f32, far: f32 },
    /// VehicleEntity 0x001C5CF0 after a cut scene (0.01)
    PlayerCarBlend(f32),
    /// TimelineCameraShaker.cpp 0x0017A490 — DangerZoneRender 0x001270E0 (time in, time out, 0.5)
    ScreenFlash { time_in: f32, time_out: f32, intensity: f32 },
    /// TimelineCameraShaker.cpp 0x0017A540 / 0x0017A580 — crash camera bars on and the HUD off, and back
    CinematicBars(bool),
    /// TimelineCameraShaker.cpp 0x0017A5F0 — player car camera shake (VehicleEntity 0x001D1610)
    PlayerCameraShake { intensity: f32, length: f32 },
    /// TimelineCameraShaker.cpp 0x0017A750 — the trail blazer firewall (TrailBlazer 0x000C63E0)
    TrailBlazerFirewall(bool),
    /// TimelineExplosion.cpp 0x0017ACD0 — PhysicsManager 0x0019FD80, then tyrespray 0x00154B80 (pos, 5, 1): an empty
    /// method in F3_D (stores `this`, `retn 0Ch`), so nothing to wire
    Explosion {
        position: Vec3,
        blast_radius: f32,
        force_objects: f32,
        angular_force_objects: f32,
        force_vehicles: f32,
        angular_force_vehicles: f32,
        sector_angle: f32,
        /// +1 ignore the player car, -1 not
        ignore_player: i32,
    },
    /// StringManager 0x000C92D0 / 0x000C93B0: the heat haze named like the channel on / off
    HeatHaze { name: u32, active: bool },
    /// ParticleManager 0x000FF900 start / 0x000FFA70 stop
    ParticleSystem { system: u32, active: bool },
    /// physics object flags +0x98 bit 1 (visible)
    InstanceVisibility { object: u32, visible: bool },
    /// TimelinePhysics.cpp 0x0017D540 — location with z negated, velocity and angular velocity raw
    InstanceMove { object: u32, location: Vec3, velocity: Vec3, angular_velocity: Vec3 },
    /// PhysicsObjectParticleTracker 0x001E2760: system started and tracked on the object
    PhysicsObjectParticles { object: u32, system: u32, active: bool },
    /// TimelinePhysics.cpp 0x0017DAA0 — `object: None` resets every vehicle (VehicleManager 0x001DE400)
    ResetPhysicsObject { object: Option<u32> },
    /// TimelineParticleGroup.cpp 0x0017CDA0 (physics object vtable +0x14): a path item placed the object
    PhysicsObjectMatrix { object: u32, matrix: Mat4 },
    /// AUDIO_FadeToVolumePreset(preset, item length)
    FadeToVolumePreset { preset: i32, time: f32 },
    /// VsAudio::StartCityAmbience / StopCityAmbience
    CityAmbience(bool),
    /// TimelineSound.cpp — the object's 3D emitter (distances 75 / 150, volume 1) or its 2D one, event by name crc
    Sound { object: u32, event: u32, position: Option<Vec3> },
    /// AUDIO_StopEmitterSamples (the named event, or everything when the name is empty)
    SoundStop { object: u32, event: Option<u32> },
}

/// play a race timeline (handled by the race timeline data after the handlers ran)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineAction {
    Play(u32),
}

/// positions the handlers read from the world (filled by physics / ai)
#[derive(bevy::prelude::Resource, Clone, Debug, Default)]
pub struct TimelineWorld {
    /// player 0's car (PhysicsCarInput 0x001BA270 point)
    pub player: Vec3,
    /// physics objects by name crc
    pub objects: std::collections::HashMap<u32, Vec3>,
    /// game time (WMM sleep)
    pub game_time: f32,
    /// race cars (`VsRaceManager` count)
    pub racers: u32,
    /// the depth of field manager's factor and focus distance (`DangerZoneRender +0x08 / +0x30`)
    pub dof: (f32, f32),
}

/// per-object state
#[derive(Clone, Debug, Default)]
pub enum ObjectState {
    #[default]
    None,
    Camera(Box<TimelineCamera>),
    /// +0x10 the camera object this shaker drives (name crc)
    Shaker { camera: Option<u32> },
    /// +0x18 particle systems this group started
    ParticleGroup { systems: Vec<u32> },
    /// +0x10 the trigger's enabled state at bind
    Trigger { was_enabled: bool },
    /// +0x1C skinned model visible
    SkinnedModel { visible: bool },
}

/// one bound object
#[derive(Clone, Debug)]
pub struct TimelineObject {
    /// +0x04 name crc (the bundle instance it stands for)
    pub name_crc: u32,
    pub name: Option<String>,
    pub kind: ETimelineObjectType,
    /// +0x0C the instance (world matrix and typed data)
    pub world: Mat4,
    pub data: Option<Value>,
    pub state: ObjectState,
}

/// what a handler can reach
pub struct HandlerCtx<'a> {
    pub triggers: &'a mut CTriggers,
    pub wmm: &'a mut CWorldModeManager,
    pub rand: &'a mut CrtRand,
    pub world: &'a TimelineWorld,
    pub out: &'a mut Vec<TimelineObjectMessage>,
    pub audio: &'a mut Vec<AudioCommand>,
    pub actions: &'a mut Vec<TimelineAction>,
    /// trigger effects fired by timelines (traffic / fire engine spawners)
    pub triggered: &'a mut Vec<crate::triggers::trigger_update::TriggerMessage>,
}

/// event phase
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EventPhase {
    Begin { t: f32, length: f32 },
    Update { u: f32 },
    End,
}

/// typed event's type name and data
pub fn typed(item: &FileItem) -> Option<(&str, Option<&Value>)> {
    match &item.payload {
        FilePayload::Typed { type_ref: Some(t), value, .. } => Some((t.name.as_str(), value.as_ref())),
        _ => None,
    }
}

pub fn field_str<'v>(v: Option<&'v Value>, name: &str) -> Option<&'v str> {
    v?.field(name)?.as_str()
}
pub fn field_bool(v: Option<&Value>, name: &str) -> bool {
    v.and_then(|v| v.field(name)).and_then(Value::as_bool).unwrap_or(false)
}
pub fn field_f32(v: Option<&Value>, name: &str) -> f32 {
    v.and_then(|v| v.field(name)).and_then(Value::as_f32).unwrap_or(0.0)
}
pub fn field_i32(v: Option<&Value>, name: &str) -> i32 {
    v.and_then(|v| v.field(name)).and_then(Value::as_i32).unwrap_or(0)
}
pub fn field_vec3(v: Option<&Value>, name: &str) -> Vec3 {
    v.and_then(|v| v.field(name)).and_then(Value::as_vec4).map(|a| Vec3::new(a[0], a[1], a[2])).unwrap_or(Vec3::ZERO)
}
/// CCRC of a string (an empty one is 0: HANDY_CRC32 asserts on a zero length)
pub fn ccrc(s: &str) -> u32 {
    if s.is_empty() { 0 } else { crc32(s.as_bytes(), 0) }
}

/// CCRC of a string field (absent = empty)
pub fn field_crc(v: Option<&Value>, name: &str) -> u32 {
    ccrc(field_str(v, name).unwrap_or(""))
}

/// the objects of the loaded race timelines (`RaceTimelineData+24`, sorted by name crc)
#[derive(Clone, Debug, Default)]
pub struct TimelineObjects {
    pub objects: Vec<TimelineObject>,
}

impl TimelineObjects {
    /// RaceTimelineSequencer.cpp 0x001719E0 — the object of that name
    pub fn find(&self, name_crc: u32) -> Option<usize> {
        self.objects.iter().position(|o| o.name_crc == name_crc)
    }

    /// RaceTimelineSequencer.cpp 0x0016F860 BindToTarget (peer +0x00, per channel of 0x00170B80): the object type
    /// (0x0016F710), the object of that name or a new one (0x0016F7C0) with the type's onBind (vtable +0x08).
    /// Returns the object index, `None` for an unknown type.
    pub fn bind(
        &mut self,
        channel_crc: u32,
        channel_name: Option<&str>,
        type_crc: u32,
        instances: &[FileInstance],
        ctx: &mut HandlerCtx,
    ) -> Option<usize> {
        let Some(kind) = object_type(type_crc) else {
            log::error!("assert: (objectType != TIMELINE_OBJECT_MAX_TYPES) && \"unknown timeline channel target type\"");
            return None;
        };
        if let Some(i) = self.find(channel_crc) {
            return Some(i);
        }
        let inst = instances.iter().find(|i| i.name_crc == channel_crc);
        let world = inst.map(|i| Mat4::from_cols_array_2d(&i.world)).unwrap_or(Mat4::IDENTITY);
        let data = inst.and_then(|i| i.value.clone());
        let mut o = TimelineObject {
            name_crc: channel_crc,
            name: channel_name.map(str::to_owned),
            kind,
            world,
            data,
            state: ObjectState::None,
        };
        // the object constructors and onBind (vtable +0x10 / +0x08)
        match kind {
            ETimelineObjectType::Camera => {
                o.state = ObjectState::Camera(Box::new(TimelineCamera::new(o.data.as_ref(), &o.world)));
            }
            ETimelineObjectType::CameraShaker => o.state = ObjectState::Shaker { camera: None },
            ETimelineObjectType::ParticleGroupController => o.state = ObjectState::ParticleGroup { systems: Vec::new() },
            ETimelineObjectType::Trigger => {
                // TimelineTrigger.cpp 0x0017F380: remember whether the trigger was enabled
                let was = ctx.triggers.find(channel_crc).map(|t| t.file.radius_sq >= 0.0).unwrap_or(true);
                if ctx.triggers.find(channel_crc).is_none() {
                    log::warn!("timeline trigger object not found");
                }
                o.state = ObjectState::Trigger { was_enabled: was };
            }
            _ => {}
        }
        self.objects.push(o);
        // kept sorted by name crc (0x001704F0)
        self.objects.sort_by_key(|o| o.name_crc);
        self.find(channel_crc)
    }

    /// RaceTimelineSequencer.cpp 0x00170570 tail: after every timeline is bound and the objects sorted, vtable
    /// +0x10 of each object in order (only the camera shaker's does anything left to do here: the camera's
    /// 0x00177F20 frame is set by `TimelineCamera::new`)
    pub fn post_bind(&mut self) {
        for i in 0..self.objects.len() {
            if self.objects[i].kind != ETimelineObjectType::CameraShaker {
                continue;
            }
            // TimelineCameraShaker.cpp 0x00179E70: attach to the camera named in its attributes
            let cam = field_crc(self.objects[i].data.as_ref(), "Camera");
            match self.find(cam).filter(|&c| self.objects[c].kind == ETimelineObjectType::Camera) {
                Some(c) => {
                    let shaker = self.objects[i].name_crc;
                    if let ObjectState::Camera(cam) = &mut self.objects[c].state {
                        cam.shaker = Some(shaker);
                    }
                    self.objects[i].state = ObjectState::Shaker { camera: Some(self.objects[c].name_crc) };
                }
                None => log::error!("assert: false && \"timeline camera shaker - target camera not found\""),
            }
        }
    }

    /// vtable +0x14: a path item moved the object (cameras take the position, triggers the whole placement)
    pub fn set_matrix(&mut self, i: usize, m: Mat4, ctx: &mut HandlerCtx) {
        let o = &mut self.objects[i];
        match &mut o.state {
            // TimelineCamera.cpp 0x00178120
            ObjectState::Camera(c) => c.set_position(m.w_axis.truncate()),
            // TimelineParticleGroup.cpp 0x0017CDA0 (the physics object's code sits under that label)
            _ if o.kind == ETimelineObjectType::Physics => {
                ctx.out.push(TimelineObjectMessage::PhysicsObjectMatrix { object: o.name_crc, matrix: m })
            }
            // TimelineTrigger.cpp 0x0017F620
            ObjectState::Trigger { .. } => {
                let name = o.name_crc;
                match ctx.triggers.list.iter_mut().find(|t| t.file.name_crc == name) {
                    Some(t) => {
                        let p = m.w_axis;
                        t.file.position = [p.x, p.y, p.z, p.w];
                        let q = bevy::math::Quat::from_mat4(&m);
                        t.file.rotation = [q.x, q.y, q.z, q.w];
                    }
                    None => log::error!("assert: trigger && \"timeline trigger not found\""),
                }
            }
            _ => {}
        }
        o.world = m;
    }

    /// RaceTimelineSequencer.cpp 0x00171100 (race restart): vtable +0x18 of every object
    pub fn reset(&mut self, ctx: &mut HandlerCtx) {
        for i in 0..self.objects.len() {
            self.reset_object(i, ctx);
        }
    }

    /// vtable +0x18: one object's reset (also RaceTimelineSequencer.cpp 0x00171710, per channel of a timeline)
    pub fn reset_object(&mut self, i: usize, ctx: &mut HandlerCtx) {
        {
            let o = &mut self.objects[i];
            match &mut o.state {
                // TimelineCamera.cpp 0x001780E0
                ObjectState::Camera(c) => {
                    if c.active {
                        c.end(ctx);
                    }
                }
                // TimelineParticleGroup.cpp 0x0017C260
                ObjectState::ParticleGroup { systems } => {
                    for s in systems.drain(..) {
                        ctx.out.push(TimelineObjectMessage::ParticleSystem { system: s, active: false });
                    }
                }
                // TimelineTrigger.cpp 0x0017F430
                ObjectState::Trigger { was_enabled } => set_trigger_enabled(ctx.triggers, o.name_crc, *was_enabled),
                // TimelineSound.cpp 0x0017E770
                _ if o.kind == ETimelineObjectType::AudioEmitter => {
                    ctx.out.push(TimelineObjectMessage::SoundStop { object: o.name_crc, event: None })
                }
                // TimelineParticleGroup.cpp 0x0017CD20 (physics object)
                _ if o.kind == ETimelineObjectType::Physics => {
                    ctx.out.push(TimelineObjectMessage::ResetPhysicsObject { object: Some(o.name_crc) })
                }
                _ => {}
            }
        }
    }

    /// dispatch one typed event to the handler of its object's type
    pub fn event(&mut self, i: usize, item: &FileItem, phase: EventPhase, ctx: &mut HandlerCtx) {
        let Some((name, value)) = typed(item) else { return };
        let kind = self.objects[i].kind;
        if !handlers(kind).contains(&name) {
            return;
        }
        use super::*;
        match kind {
            ETimelineObjectType::Camera | ETimelineObjectType::CameraShaker => {
                timeline_camera::event(self, i, name, value, phase, ctx)
            }
            ETimelineObjectType::ScreenEffectController => timeline_camera_shaker::screen_effect(name, value, phase, ctx),
            ETimelineObjectType::AiController => timeline_ai::event(name, value, phase, ctx),
            ETimelineObjectType::Physics => timeline_physics::event(&self.objects[i], name, value, phase, ctx),
            ETimelineObjectType::SkinnedModel => timeline_skinned_model::event(&mut self.objects[i], value, phase),
            ETimelineObjectType::AudioEmitter => timeline_sound::event(&self.objects[i], name, value, phase, ctx),
            ETimelineObjectType::AudioController => timeline_global_audio::event(name, value, phase, ctx),
            ETimelineObjectType::TimelineController => timeline_general::event(value, phase, ctx),
            ETimelineObjectType::ParticleGroupController => {
                timeline_particle_group::event(&mut self.objects[i], value, phase, ctx)
            }
            ETimelineObjectType::HeatHaze => timeline_heat_haze::event(&self.objects[i], value, phase, ctx),
            ETimelineObjectType::Trigger => timeline_trigger::event(&self.objects[i], value, phase, ctx),
            ETimelineObjectType::Explosion => timeline_explosion::event(&self.objects[i], phase, ctx),
        }
    }

    /// per update: the cut-scene cameras (VsDynamicCamera update, vtable 0x00640FD0 +0x10)
    pub fn update_cameras(&mut self, dt: f32, ctx: &mut HandlerCtx) -> Option<crate::game_bridge::camera_bridge::CameraBridgeData> {
        let mut view = None;
        for i in 0..self.objects.len() {
            let shaker = match &self.objects[i].state {
                ObjectState::Camera(c) if c.active => c.shaker.and_then(|s| self.find(s)).and_then(|s| self.objects[s].data.clone()),
                _ => continue,
            };
            if let ObjectState::Camera(c) = &mut self.objects[i].state {
                view = Some(c.update(dt, shaker.as_ref(), ctx));
            }
        }
        view
    }
}

/// TriggerLoading.cpp 0x0017FF50 — a trigger is switched off by a negative bounding radius
pub fn set_trigger_enabled(triggers: &mut CTriggers, name_crc: u32, on: bool) {
    if let Some(t) = triggers.list.iter_mut().find(|t| t.file.name_crc == name_crc) {
        let [w, h, d] = t.file.dimensions;
        t.file.radius_sq = if on { (w * w + h * h + d * d) / 4.0 } else { -1.0 };
    }
}
