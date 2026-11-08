//! src/Physics/PhysicsObjects: placed physics objects (movable compound objects, immovable meshes)
pub mod physics_object_manager;
pub mod physics_object_particle_tracker;
pub mod physics_objects;

/// What the object code hands to other subsystems, queued in `PhysGlobals::obj_events` and drained by the
/// owner after each physics frame.
#[derive(Clone, Debug, PartialEq)]
pub enum PhysicsObjectEvent {
    /// ParticleManager 0x000FFC30 from the collision handler 0x001E1F80: debris of a hit object
    CollisionParticles {
        /// instance type (`*(obj+136)+52`, ParticleManager 0x00101300)
        kind: u32,
        /// the object's matrix (obj +0, row-vector 4x4, translation in row 3)
        object: [f32; 16],
        /// message +8
        point: [f32; 3],
        /// message +40
        impulse: f32,
    },
    /// ParticleManager 0x001000E0 (FireOffParticleTrail) from the active-object update 0x001E4C30:
    /// event 24 propane trail, 26 wheel fire; `from`/`to` = the object's position before / after the step
    Trail { from: [f32; 3], to: [f32; 3], event: u32 },
    /// VsAudio::PlayPhysicsImpact from 0x001E1F80. `sound` is the instance's old-style id (`*(obj+136)+48`,
    /// to go through `VsAudioEvents::MapOldId`) unless `mapped`; 2031 and "none" are already mapped there
    /// (1146417784 / 207699420). `again` = the object had been hit before (flags 0x1000).
    Impact { sound: u32, mapped: bool, point: [f32; 3], impulse: f32, again: bool },
    /// RaceTimelineSequencer::PlayTimeline from the immovable pass 0x001A89F0 (instance +36)
    PlayTimeline { timeline: u32 },
    /// kind 23 (fire hydrant) hit, 0x001E1F80: `VsAudio::AddEnvSound(-33188540, object position, 10, 100, 50)`
    /// (after the friction spot 0x001E18E0, whether or not its ground ray hit)
    Explosion { point: [f32; 3] },
}

/// LightStreak__133750 on `unk_720840` (graphics' PuddleRender list) from the friction spot 0x001E1A20: the
/// 48-byte record, appended then re-sorted by race distance (qsort 0x001337C0 / 0x00133800) and the list marked
/// dirty (0x00133A20). Separate from [`PhysicsObjectEvent`] (core matches that exhaustively).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsPuddle {
    /// +0 the ground point under the spot
    pub position: [f32; 3],
    /// +12 (w) 15.0
    pub radius: f32,
    /// +16 / +20 random unit direction in xz (2 x VsRandRange(-1, 1), normalised)
    pub direction: [f32; 2],
    /// +24 race distance (sort key; the hydrant's comes from [`RaceDistanceHook`](crate::vehicle::vehicle_entity::cx::RaceDistanceHook))
    pub race_distance: f32,
    /// +28 = 2
    pub kind: i32,
    /// +32 / +36 = 30.0 x 30.0
    pub size: [f32; 2],
    /// +40 = 480
    pub life: i32,
    /// +44 = the uninitialised word | 1 (0xCCCCCCCD in the debug build)
    pub flags: u32,
}

/// MessageManager channels 2 and 4 as heard by RaceTimelineSequencer's restore point. `object` = instance name
/// hash (record +0x4C, `RaceTimelineSequencer__172C70`), as `PhysicsObjects::by_name` uses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsObjectNotice {
    /// channel 2 (PhysicsManager 0x0019FD80 / 0x001A89F0: a car hit the object, msg +4)
    Hit { object: u32 },
    /// channel 4 (0x001E4710 gave the object a rigid body, msg +0)
    Woken { object: u32 },
}

/// One placed object, for `TimelineWorld` and debugging.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsObjectInfo {
    /// instance name hash (record +0x4C)
    pub name: u32,
    /// instance type (record +0x34)
    pub kind: u32,
    /// 1 movable, 3 immovable
    pub list_type: u32,
    /// the object's matrix (obj +0)
    pub matrix: [f32; 16],
    /// a movable object currently simulated (has a rigid body)
    pub awake: bool,
    /// obj +152 (1 collides, 2 visible, 4 hit, 0x20 broken, 0x1000 hit before)
    pub flags: u32,
    /// a movable with a compound object / an immovable with collision
    pub collides: bool,
    /// instance record +56 u16 (0x001E5E80 type): 0 streetlamp, 1 `propld_dummycopcar`, 2 hydrant, 0xFFFF none
    pub object_type: u16,
    /// instance record +58 u16 (0x001E5E80 index: hydrant number, else 0 / 0xFFFF)
    pub object_index: u16,
}

impl PhysicsObjectInfo {
    /// PhysicsObjects.cpp 0x001E5F30 (`type == 1`), the test 0x001E3B70 draws the dummy police car with
    pub fn is_dummy_cop_car(&self) -> bool {
        self.object_type == 1
    }
}
