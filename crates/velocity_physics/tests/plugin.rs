//! The Bevy API: load a track, spawn a car from a `Transform`, drive it with `PhysicsCarInput` through
//! `run_physics_tick`, read `PhysicsCarState`, query the world.
#![cfg(feature = "tr")]

use bevy::prelude::*;
use velocity_physics::plugin::{
    run_physics_tick, LoadPhysicsTrack, PhysicsCar, PhysicsCarInput, PhysicsCarSpawn, PhysicsCarState, PhysicsPlugin,
    PhysicsWorldRes,
};
use velocity_physics::vehicle::vehicle_entity::sim::{RAY_NORMAL, RAY_TRACK};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins(PhysicsPlugin);
    app.world_mut().write_message(LoadPhysicsTrack { track: "ExecTest".into() });
    run_physics_tick(app.world_mut());
    app
}

#[test]
fn spawn_drive_and_query() {
    let mut app = app();
    // a flat straight on ExecTest (see examples/accel.rs): heading +X
    let start = Vec3::new(55.82, -13.17, 133.57);
    let ground = app.world_mut().resource_mut::<PhysicsWorldRes>().ground_height(start.x, start.z, start.y + 3.0).expect("ground");
    assert!((ground - start.y).abs() < 0.05, "ground {ground}");
    let hit = app.world_mut().resource_mut::<PhysicsWorldRes>()
        .ray_cast(start + Vec3::Y * 3.0, start - Vec3::Y * 3.0, RAY_TRACK | RAY_NORMAL, -1).unwrap();
    assert_eq!(hit.kind, 1);
    assert!(hit.normal[1] > 0.99, "{hit:?}");

    let mut inputs = LocalConfigInputs::default_spans("240SX");
    inputs.is_player = true;
    for (k, yaw) in [0.0f32, 0.7].into_iter().enumerate() {
        let rot = Quat::from_rotation_y(yaw);
        let e = app.world_mut().spawn((
            Transform::from_translation(start + Vec3::new(150.0 * k as f32, 0.6, 0.0)).with_rotation(rot),
            PhysicsCarSpawn { inputs: inputs.clone(), pad_slot: k as i32 },
            PhysicsCarInput::default(),
        )).id();
        for _ in 0..48 {
            run_physics_tick(app.world_mut());
        }
        let st = app.world().get::<PhysicsCarState>(e).unwrap().clone();
        let car = *app.world().get::<PhysicsCar>(e).unwrap();
        let tf = *app.world().get::<Transform>(e).unwrap();
        println!("car {} yaw {yaw}: rest {:?} rot {:?}, wheels {:?}", car.id, st.position, st.rotation,
            st.wheels.iter().map(|w| (w.contacts, w.surface)).collect::<Vec<_>>());
        assert!(st.wheels.iter().all(|w| w.contacts > 0), "all wheels on the ground");
        assert!((st.position.y - start.y - 0.23).abs() < 0.05, "ride height");
        // the transform follows the body and keeps the heading it was spawned with
        assert!(tf.rotation.angle_between(rot) < 0.05, "heading {:?} vs {rot:?}", tf.rotation);
        // full throttle: the car drives along its model +X axis
        let p0 = st.position;
        app.world_mut().get_mut::<PhysicsCarInput>(e).unwrap().accelerator = 1.0;
        for _ in 0..72 {
            run_physics_tick(app.world_mut());
        }
        app.world_mut().get_mut::<PhysicsCarInput>(e).unwrap().accelerator = 0.0;
        let st = app.world().get::<PhysicsCarState>(e).unwrap().clone();
        let d = (st.position - p0).normalize();
        println!("  moved {:?} speed {:.1} gear {} engine {:.0}", st.position - p0, st.speed() * 3.6, st.gear, st.engine_speed);
        assert!(st.speed() * 3.6 > 30.0);
        assert!(d.dot(rot * Vec3::X) > 0.98, "drives along +X of its transform: {d:?}");
    }
    // removing the component removes the car
    let e = app.world_mut().query_filtered::<Entity, With<PhysicsCar>>().iter(app.world()).next().unwrap();
    app.world_mut().entity_mut(e).remove::<PhysicsCar>();
    run_physics_tick(app.world_mut());
    assert_eq!(app.world().resource::<PhysicsWorldRes>().lock().as_ref().unwrap().vehicle_ids().len(), 1);
}
