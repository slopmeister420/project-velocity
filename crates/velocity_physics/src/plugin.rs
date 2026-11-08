//! Bevy side of the physics: one [`PhysicsWorld`] (the original `CPhysicsManager` with its vehicles)
//! driven by the game's fixed update. Core runs [`run_physics_tick`] inside `GameTick`'s
//! `InGameUpdateSet::Physics` and sets [`PhysicsTimestep`] from the game clock.

use bevy::ecs::schedule::ScheduleLabel;
use std::sync::{Arc, Mutex, MutexGuard};

use bevy::prelude::*;

use crate::c_config::load_global_config_with_fallback;
use crate::car_damage::{CarDamageEvent, CarDamageManager, DamageMessage, DamageModel};
use crate::physics_objects::{PhysicsObjectEvent, PhysicsObjectInfo};
use crate::vehicle::vehicle_entity::cx::TrackWorld;
use crate::vehicle::vehicle_entity::sim::{PadInput, PhysicsWorld, RayCastHit, SpawnParams, WheelState};
use crate::vehicle::vehicle_entity::sim_ai::CarCollision;
use crate::vehicle::vehicle_entity::LocalConfigInputs;

/// The physics frame (`PhysicsManager__19F920`): every system that touches the world runs here, once per
/// game update.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PhysicsTick;

/// Order inside [`PhysicsTick`].
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PhysicsSet {
    /// track load / unload, car spawn / removal
    Setup,
    /// `ComputeForcesAndTorques` for every car, `ResolveCollisions` (collision passes + `Evolve`)
    Step,
    /// car state components and `Transform`s
    Output,
}

/// The physics world, shared so the AI's view of the cars (built on [`SharedPhysicsWorld`] by whoever links
/// `velocity_ai`) reaches the same cars; `None` until a track is loaded.
pub type SharedPhysicsWorld = Arc<Mutex<Option<PhysicsWorld>>>;

#[derive(Resource, Default, Clone)]
pub struct PhysicsWorldRes(pub SharedPhysicsWorld);

impl PhysicsWorldRes {
    /// the world (`None` before a track is loaded)
    pub fn lock(&self) -> MutexGuard<'_, Option<PhysicsWorld>> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
    /// `PhysicsManager::RayCast` (see [`PhysicsWorld::ray_cast`]); `None` without a world or hit
    pub fn ray_cast(&mut self, from: Vec3, to: Vec3, flags: i32, ignore_car: i32) -> Option<RayCastHit> {
        self.lock().as_mut()?.ray_cast(from.to_array(), to.to_array(), flags, ignore_car)
    }
    /// height of the track collision under (x, z), searching down from `top`
    pub fn ground_height(&mut self, x: f32, z: f32, top: f32) -> Option<f32> {
        use crate::vehicle::vehicle_entity::sim::{RAY_NORMAL, RAY_TRACK};
        self.ray_cast(Vec3::new(x, top, z), Vec3::new(x, top - 1000.0, z), RAY_TRACK | RAY_NORMAL, -1).map(|h| h.point[1])
    }
}

/// A collision the last physics frame reported (`MessageManager` message 0); the AI's
/// `AiCollisionMessage { a, b, impact }` (VsAIDriver.cpp 0x000160B0) is built from it.
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsCollision(pub CarCollision);

/// Seconds of game time per physics frame (`GameClock.timer.scaled_dt()`; the original runs at 1/24 and
/// asserts <= 0.041841). Physics advances `dt * 24 * LC->timestep` per frame (0.03 s for 1/24).
#[derive(Resource, Clone, Copy, Debug)]
pub struct PhysicsTimestep(pub f32);

impl Default for PhysicsTimestep {
    fn default() -> Self {
        Self(1.0 / 24.0)
    }
}

/// Load a track's collision (`\Tracks\<track>\gfx\arena.msh` + `arena.qdt`) and create a fresh world
/// (`::GC` from `Main.cfg`). Any previous world and its cars are dropped.
#[derive(Message, Clone, Debug)]
pub struct LoadPhysicsTrack {
    /// directory under `\Tracks` (e.g. `LACoast`)
    pub track: String,
}

/// Register a race's placed objects (`RACE.VCD` `instance.cpt` + `uinstance.cpt`, CCompartment::RegisterInstances
/// 0x002E7B50) into the current world, after its [`LoadPhysicsTrack`]. `race` is the race directory, e.g.
/// `\Tracks\London\Circuit\Race01`. The track's own `CPTS.VCD` `instance.cpt` is registered by the track load.
#[derive(Message, Clone, Debug)]
pub struct LoadPhysicsRaceObjects {
    pub race: String,
}

/// What the physics objects raised during the last frame (particles, trails, impact sounds, timelines). Core
/// forwards `CollisionParticles` as `visos_render::stateless_particles::ObjectCollisionParticles` and `Trail`
/// as `ParticleTrail` (graphics' ParticleManager 0x000FFC30 / 0x001000E0).
#[derive(Message, Clone, Debug)]
pub struct PhysicsObjectMessage(pub PhysicsObjectEvent);

/// MessageManager channels 2 (car hit object) and 4 (object woken) from the last frame, for
/// `RaceTimelines::record_physics_object` (RaceTimelineSequencer's restore point listens on both).
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsObjectNoticeMessage(pub crate::physics_objects::PhysicsObjectNotice);

/// `TimelineObjectMessage::ResetPhysicsObject { object: Some(name) }`: reset the placed object with that instance
/// name hash (PhysicsObjectManager 0x001E1C00 lookup + 0x001E37D0). Applied before the next physics frame.
#[derive(Message, Clone, Copy, Debug)]
pub struct ResetPhysicsObject(pub u32);

/// `TimelineObjectMessage::ResetPhysicsObject { object: None }` (F3EventResetPhysicsObject with `all`, 0x0017DAA0
/// -> 0x001DE400): reset every placed object. Applied before the next physics frame, after `ResetPhysicsObject`.
#[derive(Message, Clone, Copy, Debug)]
pub struct ResetAllPhysicsObjects;

/// `TimelineObjectMessage::InstanceMove` (F3EventInstanceMove, TimelinePhysics 0x0017D540): `location` with z
/// already negated, velocity / angular velocity as in the event. Applied before the next physics frame.
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsInstanceMove {
    pub object: u32,
    pub location: Vec3,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
}

/// `TimelineObjectMessage::Explosion` (TimelineExplosion 0x0017ACD0 -> PhysicsManager 0x0019FD80): applied in
/// `PhysicsSet::Setup` of the next physics tick, after the resets and moves; draws from the shared rand stream
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsExplosion(pub crate::vehicle::vehicle_entity::sim::Explosion);

/// PhysicsObjectManager 0x001E18E0 called from outside the physics frame (the AI's VsAIRaceVehicle 0x00019DD0);
/// applied in `PhysicsSet::Setup` of the next physics tick
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsFrictionSpot(pub crate::vehicle::vehicle_entity::sim::FrictionSpot);

/// The physics part of `GameManager::RestartRace` (MaterialProperties 0x0019C390: every object reset 0x001DE400,
/// +0x10 / +0x5F08 = 1.0, every friction spot removed 0x001B3850, every tracker body's owner cleared). Applied
/// first in `PhysicsSet::Setup` of the next physics tick.
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct RestartPhysicsWorld;

/// A puddle record for graphics' PuddleRender (0x001E1A20 -> LightStreak__133750 on `unk_720840`), from the last
/// frame (or the Setup of this tick); core forwards it to `puddle_render`
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsPuddleMessage(pub crate::physics_objects::PhysicsPuddle);

/// `SlipstreamRender::SetStrength(vehicle, strength)` (0x0013FB30) from the last frame; core forwards it 1:1 as
/// `visos_render::race_effects::SlipstreamStrength`. `vehicle` is the original's index (see `take_slipstream`).
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsSlipstream {
    pub vehicle: i32,
    pub strength: f32,
}

/// Every placed physics object after the last frame (movables move once hit). `TimelineWorld.objects` is
/// `by_name()`.
#[derive(Resource, Clone, Debug, Default)]
pub struct PhysicsObjects(pub Vec<PhysicsObjectInfo>);

impl PhysicsObjects {
    /// position by instance name hash (the timeline's object crc)
    pub fn by_name(&self) -> std::collections::HashMap<u32, Vec3> {
        self.0.iter().map(|o| (o.name, Vec3::new(o.matrix[12], o.matrix[13], o.matrix[14]))).collect()
    }
}

/// Drop the world (mode shutdown).
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct UnloadPhysicsTrack;

/// Put on an entity to create a car there (`VehicleManager::Init` + `VehicleEntity::Create`). Replaced by
/// [`PhysicsCar`] + [`PhysicsCarState`] once the car exists. The entity's `Transform` (game coordinates)
/// gives position and heading.
#[derive(Component, Clone, Debug)]
pub struct PhysicsCarSpawn {
    /// config name, spans and performance modifiers (`LocalConfigInputs::from_setup` for a database
    /// vehicle + setup, `default_spans(name)` otherwise)
    pub inputs: LocalConfigInputs,
    /// `VsTrafficManager` pad record this car reads ([`PhysicsCarInput`]); players are 0, 1
    pub pad_slot: i32,
}

/// A car in the physics world.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicsCar {
    /// vehicle id (= index in the vehicle array while no car has been removed before it)
    pub id: usize,
    pub pad_slot: i32,
}

/// The pad record the car reads each frame (`VsTrafficManager` +24 + 36 * slot, read by
/// `VehicleEntity__1CB530`). Players only; AI-driven cars use the AI's controls.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct PhysicsCarInput {
    /// analogue accelerate (+) / brake (-) axis
    pub axis: f32,
    /// accelerator trigger 0..1
    pub accelerator: f32,
    /// brake trigger 0..1
    pub brake: f32,
    /// steering -1..1
    pub steer: f32,
    pub handbrake: bool,
    pub gear_up: bool,
    pub gear_down: bool,
    /// the two boost buttons (0x40, 0x80)
    pub boost: [bool; 2],
}

impl PhysicsCarInput {
    pub fn pad(&self) -> PadInput {
        let mut b = 0u32;
        for (on, bit) in [(self.handbrake, 2), (self.gear_up, 4), (self.gear_down, 8), (self.boost[0], 0x40), (self.boost[1], 0x80)] {
            if on {
                b |= bit;
            }
        }
        PadInput { axis: self.axis, accelerator: self.accelerator, brake: self.brake, steer: self.steer, buttons: b }
    }
}

/// The car after the last physics frame (game coordinates).
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct PhysicsCarState {
    /// body matrix as `Evolve` leaves it at VE+4788 (row-vector 4x4; `Mat4::from_cols_array`)
    pub body_matrix: [f32; 16],
    pub position: Vec3,
    pub rotation: Quat,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    /// FastCar gear (car+2976: 1.. forward, 0 neutral, < 0 reverse)
    pub gear: i32,
    /// engine speed from the driven wheels (FastCar 0x0035D090)
    pub engine_speed: f32,
    /// FL, FR? in FastCar wheel order 0..3 (0/3 front, 1/2 back)
    pub wheels: [WheelState; 4],
}

impl PhysicsCarState {
    pub fn speed(&self) -> f32 {
        Vec3::new(self.velocity.x, 0.0, self.velocity.z).length()
    }
    /// wheel `i`'s render matrix in game space
    pub fn wheel_matrix(&self, i: usize) -> Mat4 {
        Mat4::from_cols_array(&self.wheels[i].matrix)
    }
}

/// A wall mesh for the barrier pass, built at race load after the track (`Armco::BuildMesh` 0x00012440 for
/// professional races, `BarrierRender__108030` otherwise): vertices and `[v0, v1, v2, material]` faces as
/// the builder lays them out (material 21). An empty mesh clears the set.
#[derive(Message, Clone, Debug)]
pub struct SetPhysicsWalls {
    pub armco: bool,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u16; 4]>,
}

/// Build a car's damage model (CarDamage 0x0006D150; CarRender loads it with the car model, 0x00110220 -> 0x0006AE70).
/// `vehicle` = physics id; `model_to_root` = the damage model space (x lateral, y up, z front) -> the car entity's
/// local space (+X forward), row-vector 4x4; `deform` = body dents on (the player's car).
#[derive(Message, Clone, Debug)]
pub struct SetupCarDamage {
    pub vehicle: usize,
    pub model: DamageModel,
    pub model_to_root: [f32; 16],
    pub deform: bool,
}

/// Repair damage: `None` = every car (`GameManager::RestartRace` -> 0x0006B090), else one car (0x00075780 full).
#[derive(Message, Clone, Copy, Debug)]
pub struct RepairCarDamage(pub Option<usize>);

/// What the renderer has to show: broken lights, window cracks, collision particles, changed meshes (read the
/// vertices from [`CarDamageManager`]: `get(vehicle).model.meshes[mesh]`).
#[derive(Message, Clone, Copy, Debug)]
pub struct PhysicsCarDamage(pub CarDamageEvent);

/// per car: damage model space -> the car entity's local space
#[derive(Resource, Default)]
pub struct CarDamagePoses(pub std::collections::HashMap<i32, [f32; 16]>);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsWorldRes>()
            .init_resource::<PhysicsTimestep>()
            .add_message::<LoadPhysicsTrack>()
            .add_message::<UnloadPhysicsTrack>()
            .add_message::<PhysicsCollision>()
            .add_message::<SetPhysicsWalls>()
            .add_message::<LoadPhysicsRaceObjects>()
            .add_message::<PhysicsObjectMessage>()
            .add_message::<ResetPhysicsObject>()
            .add_message::<ResetAllPhysicsObjects>()
            .add_message::<PhysicsInstanceMove>()
            .add_message::<PhysicsObjectNoticeMessage>()
            .add_message::<PhysicsSlipstream>()
            .add_message::<PhysicsExplosion>()
            .add_message::<PhysicsFrictionSpot>()
            .add_message::<RestartPhysicsWorld>()
            .add_message::<PhysicsPuddleMessage>()
            .add_message::<SetupCarDamage>()
            .add_message::<RepairCarDamage>()
            .add_message::<PhysicsCarDamage>()
            .init_resource::<CarDamageManager>()
            .init_resource::<CarDamagePoses>()
            .init_resource::<PhysicsObjects>()
            .init_resource::<crate::physics_tweaks::PhysicsTweaks>()
            .add_systems(First, crate::physics_tweaks::sync_physics_tweaks)
            .init_schedule(PhysicsTick)
            .configure_sets(PhysicsTick, (PhysicsSet::Setup, PhysicsSet::Step, PhysicsSet::Output).chain())
            .add_systems(PhysicsTick, (load_track, load_race_objects, reset_objects, set_walls, spawn_cars, remove_cars).chain().in_set(PhysicsSet::Setup))
            .add_systems(PhysicsTick, step.in_set(PhysicsSet::Step))
            .add_systems(PhysicsTick, (write_state, car_damage).chain().in_set(PhysicsSet::Output));
    }
}

/// Exclusive system for core: run one physics frame (`app.add_systems(GameTick,
/// run_physics_tick.in_set(InGameUpdateSet::Physics))`).
pub fn run_physics_tick(world: &mut World) {
    world.run_schedule(PhysicsTick);
}

fn load_track(world: Res<PhysicsWorldRes>, mut load: MessageReader<LoadPhysicsTrack>, mut unload: MessageReader<UnloadPhysicsTrack>,
    cars: Query<Entity, With<PhysicsCar>>, mut commands: Commands) {
    let mut changed = false;
    let mut world = world.lock();
    for _ in unload.read() {
        *world = None;
        changed = true;
    }
    for m in load.read() {
        let Some(gc) = load_global_config_with_fallback(0) else {
            log::error!("physics: Main.cfg / Mainsafe.cfg failed to load");
            continue;
        };
        let track = match TrackWorld::load(&m.track) {
            Ok(t) => Some(t),
            Err(e) => {
                log::error!("physics: collision for {}: {e}", m.track);
                None
            }
        };
        let mut w = PhysicsWorld::new(gc, track);
        // the track's resident compartment: placed props (CompartmentManager -> RegisterInstances)
        let n = register_cpt(&mut w, &format!(r"\Tracks\{}\CPTS.VCD", m.track), "instance.cpt");
        log::info!("physics: {} objects in {}", n, m.track);
        *world = Some(w);
        changed = true;
    }
    if changed {
        // cars belonged to the old world
        for e in &cars {
            commands.entity(e).remove::<(PhysicsCar, PhysicsCarState)>();
        }
    }
}

/// register `name` of the VCD `pack` (missing files register nothing)
fn register_cpt(w: &mut PhysicsWorld, pack: &str, name: &str) -> usize {
    match visos::file::Vcd::open(pack) {
        Ok(v) => v.get(name).map(|b| w.register_compartment(b)).unwrap_or(0),
        Err(_) => 0,
    }
}

fn load_race_objects(world: Res<PhysicsWorldRes>, mut msgs: MessageReader<LoadPhysicsRaceObjects>) {
    let mut world = world.lock();
    for m in msgs.read() {
        let Some(w) = world.as_mut() else { continue };
        // diagnostic: run a race without its placed objects
        if std::env::var_os("VELOCITY_PHYS_NO_RACE_OBJECTS").is_some() {
            continue;
        }
        let pack = format!(r"{}\RACE.VCD", m.race.trim_end_matches('\\'));
        let n = register_cpt(w, &pack, "instance.cpt") + register_cpt(w, &pack, "uinstance.cpt");
        log::info!("physics: {} race objects in {}", n, m.race);
    }
}

#[allow(clippy::too_many_arguments)]
fn reset_objects(world: Res<PhysicsWorldRes>, mut msgs: MessageReader<ResetPhysicsObject>,
    mut all: MessageReader<ResetAllPhysicsObjects>, mut moves: MessageReader<PhysicsInstanceMove>,
    mut restart: MessageReader<RestartPhysicsWorld>, mut explosions: MessageReader<PhysicsExplosion>,
    mut spots: MessageReader<PhysicsFrictionSpot>) {
    let mut world = world.lock();
    let Some(w) = world.as_mut() else {
        msgs.clear();
        all.clear();
        moves.clear();
        restart.clear();
        explosions.clear();
        spots.clear();
        return;
    };
    if restart.read().count() > 0 {
        w.restart_race();
    }
    for m in msgs.read() {
        w.reset_object(m.0);
    }
    if all.read().count() > 0 {
        w.reset_all_objects();
    }
    for m in moves.read() {
        if !w.instance_move(m.object, m.location.to_array(), m.velocity.to_array(), m.angular_velocity.to_array()) {
            log::warn!("Timeline F3EventInstanceMove - target object {:#x} not found", m.object);
        }
    }
    for e in explosions.read() {
        w.explosion(&e.0);
    }
    for s in spots.read() {
        w.friction_spot(&s.0);
    }
}

fn set_walls(world: Res<PhysicsWorldRes>, mut msgs: MessageReader<SetPhysicsWalls>) {
    let mut world = world.lock();
    for m in msgs.read() {
        let Some(w) = world.as_mut() else { continue };
        let t = TrackWorld::from_walls(&m.vertices, &m.faces);
        if m.armco {
            w.set_armco_walls(t);
        } else {
            w.set_barrier_walls(t);
        }
    }
}

fn spawn_cars(world: Res<PhysicsWorldRes>, q: Query<(Entity, &PhysicsCarSpawn, Option<&Transform>)>, mut commands: Commands) {
    let mut world = world.lock();
    let Some(w) = world.as_mut() else { return };
    for (e, s, tf) in &q {
        let tf = tf.copied().unwrap_or_default();
        // `Create` stores the conjugate of the VS quaternion it is given (the car faces rotate(q, +X))
        let q = tf.rotation.conjugate();
        let id = w.spawn(&SpawnParams { inputs: s.inputs.clone(), position: tf.translation.to_array(), orientation: q.to_array(), slot: s.pad_slot });
        commands.entity(e).remove::<PhysicsCarSpawn>().insert((PhysicsCar { id, pad_slot: s.pad_slot }, PhysicsCarState::default()));
    }
}

fn remove_cars(world: Res<PhysicsWorldRes>, mut removed: RemovedComponents<PhysicsCar>, cars: Query<&PhysicsCar>) {
    let gone: Vec<Entity> = removed.read().collect();
    if gone.is_empty() {
        return;
    }
    let mut world = world.lock();
    let Some(w) = world.as_mut() else { return };
    // ids still owned by an entity stay
    let live: Vec<usize> = cars.iter().map(|c| c.id).collect();
    for id in w.vehicle_ids() {
        if !live.contains(&id) {
            w.remove(id);
        }
    }
}

fn step(world: Res<PhysicsWorldRes>, dt: Res<PhysicsTimestep>, cars: Query<(&PhysicsCar, Option<&PhysicsCarInput>)>) {
    let mut world = world.lock();
    let Some(w) = world.as_mut() else { return };
    if w.vehicle_ids().is_empty() {
        return;
    }
    let mut pads = vec![PadInput::default(); 4];
    for (c, inp) in &cars {
        if let (Some(i), Ok(slot)) = (inp, usize::try_from(c.pad_slot)) {
            if pads.len() <= slot {
                pads.resize(slot + 1, PadInput::default());
            }
            pads[slot] = i.pad();
        }
    }
    w.step(dt.0.min(0.041841004), &pads);
}

fn write_state(world: Res<PhysicsWorldRes>, mut cars: Query<(&PhysicsCar, &mut PhysicsCarState, Option<&mut Transform>)>,
    mut collisions: MessageWriter<PhysicsCollision>, mut events: MessageWriter<PhysicsObjectMessage>,
    mut slip: MessageWriter<PhysicsSlipstream>, mut objects: ResMut<PhysicsObjects>,
    mut notices: MessageWriter<PhysicsObjectNoticeMessage>, mut puddles: MessageWriter<PhysicsPuddleMessage>) {
    let mut world = world.lock();
    let Some(w) = world.as_mut() else {
        objects.0.clear();
        return;
    };
    for c in w.take_collisions() {
        collisions.write(PhysicsCollision(c));
    }
    for e in w.take_object_events() {
        events.write(PhysicsObjectMessage(e));
    }
    let trace = std::env::var_os("VELOCITY_PHYS_TRACE").is_some();
    for n in w.take_object_notices() {
        if trace {
            log::info!("physics trace: {n:?}");
        }
        notices.write(PhysicsObjectNoticeMessage(n));
    }
    for (vehicle, strength) in w.take_slipstream() {
        slip.write(PhysicsSlipstream { vehicle, strength });
    }
    for p in w.take_puddles() {
        puddles.write(PhysicsPuddleMessage(p));
    }
    objects.0 = w.objects();
    let gear_trace = std::env::var_os("VELOCITY_PHYS_GEAR_TRACE").is_some();
    for (c, mut st, tf) in &mut cars {
        if !w.vehicle_ids().contains(&c.id) {
            continue;
        }
        let m = w.body_matrix(c.id);
        let mat = Mat4::from_cols_array(&m);
        let (_, rot, pos) = mat.to_scale_rotation_translation();
        *st = PhysicsCarState {
            body_matrix: m,
            position: pos,
            rotation: rot,
            velocity: Vec3::from_array(w.velocity(c.id)),
            angular_velocity: Vec3::from_array(w.angular_velocity(c.id)),
            gear: w.car_gear(c.id),
            engine_speed: w.engine_speed(c.id),
            wheels: std::array::from_fn(|i| w.wheel(c.id, i)),
        };
        if gear_trace && w.slot(c.id).human {
            let s = w.slot(c.id);
            let v = st.velocity.length();
            // car+2976 gear, +3284 last auto choice, +3296 manual hold (VE+3536), VE+48 hold timer,
            // traffic pad accelerator
            log::info!("phys gear: car {} speed {:.1} gear {} rpm {:.0} auto {} hold {} t48 {:.2} acc {:.2}",
                c.id, v, st.gear, st.engine_speed * 9.549_297, s.car.m.i(3284), s.car.m.i(3296),
                s.ve.m.f(48), w.glob.traffic.f((24 + 36 * s.slot.max(0)) as usize + 4));
        }
        if let Some(mut tf) = tf {
            tf.translation = pos;
            tf.rotation = rot;
        }
    }
}

/// CarDamage per physics frame: setup / repair requests, message 0 to each car's listener (0x0006B820), then the
/// manager update (0x0006ADC0). Cars that left the world are dropped (0x0006BB20).
fn car_damage(world: Res<PhysicsWorldRes>, dt: Res<PhysicsTimestep>, mut mgr: ResMut<CarDamageManager>,
    mut poses: ResMut<CarDamagePoses>, mut setup: MessageReader<SetupCarDamage>, mut repair: MessageReader<RepairCarDamage>,
    mut collisions: MessageReader<PhysicsCollision>, mut out: MessageWriter<PhysicsCarDamage>) {
    let world = world.lock();
    let Some(w) = world.as_ref() else {
        mgr.cars.clear();
        poses.0.clear();
        return;
    };
    for s in setup.read() {
        mgr.add(s.vehicle as i32, s.model.clone(), s.deform);
        poses.0.insert(s.vehicle as i32, s.model_to_root);
    }
    let ids = w.vehicle_ids();
    mgr.cars.retain(|c| ids.contains(&(c.car_id as usize)));
    for r in repair.read() {
        match r.0 {
            None => mgr.repair_all(),
            Some(v) => {
                if let Some(c) = mgr.get_mut(v as i32) {
                    c.repair(true);
                }
            }
        }
    }
    let matrix = |id: i32| -> Option<[f32; 16]> {
        if id < 0 || !ids.contains(&(id as usize)) {
            return None;
        }
        let root = Mat4::from_cols_array(&w.body_matrix(id as usize));
        let local = Mat4::from_cols_array(poses.0.get(&id)?);
        Some((root * local).to_cols_array())
    };
    for c in collisions.read() {
        mgr.on_message(&DamageMessage::from(&c.0), &matrix);
    }
    mgr.update(dt.0);
    for e in mgr.take_events() {
        out.write(PhysicsCarDamage(e));
    }
}
