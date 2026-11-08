//! Physics objects (PhysicsObjectManager / PhysicsObjectParticleTracker / PhysicsObjects.cpp): every shipped
//! compartment's Physics and Immovable instances register (positive control over all tracks and races), and
//! a car driven into a movable London object hits it (message 2 -> 0x001E1F80), wakes it and pushes it.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::physics_objects::{PhysicsObjectEvent, PhysicsObjectNotice};
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams, RAY_NORMAL, RAY_TRACK};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;
use visos::file::Vcd;

/// `\Tracks\<t>` compartments in load order: the track's `instance.cpt`, then each race's
fn compartments(track: &str) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    if let Ok(v) = Vcd::open(&format!(r"\Tracks\{track}\CPTS.VCD")) {
        if let Some(b) = v.get("instance.cpt") {
            out.push(("CPTS:instance".into(), b.to_vec()));
        }
    }
    let root = visos::file::resolve(&format!(r"\Tracks\{track}"));
    let mut races = Vec::new();
    for mode in std::fs::read_dir(&root).into_iter().flatten().flatten() {
        for race in std::fs::read_dir(mode.path()).into_iter().flatten().flatten() {
            let p = race.path();
            for f in std::fs::read_dir(&p).into_iter().flatten().flatten() {
                if f.file_name().to_string_lossy().eq_ignore_ascii_case("race.vcd") {
                    races.push(f.path());
                }
            }
        }
    }
    races.sort();
    for r in races {
        let rel = r.strip_prefix(visos::file::game_dir()).unwrap().to_string_lossy().replace('/', "\\");
        let Ok(v) = Vcd::open(&format!(r"\{rel}")) else { continue };
        for n in ["instance.cpt", "uinstance.cpt"] {
            if let Some(b) = v.get(n) {
                out.push((format!("{rel}:{n}"), b.to_vec()));
            }
        }
    }
    out
}

fn tracks() -> Vec<String> {
    let root = visos::file::resolve(r"\Tracks");
    let mut t: Vec<String> = std::fs::read_dir(root).unwrap().flatten().filter(|e| e.path().is_dir()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    t.sort();
    t
}

#[test]
fn register_all_compartments() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut total = 0;
    let mut total_cops = 0;
    for t in tracks() {
        let cpts = compartments(&t);
        if cpts.is_empty() {
            continue;
        }
        let mut w = PhysicsWorld::new(gc.clone(), Some(TrackWorld::flat(3000.0, 14)));
        let mut n = 0;
        // the manager holds 128 lists (two per compartment at most)
        for (_, b) in cpts.iter().take(60) {
            n += w.register_compartment(b);
        }
        let objs = w.objects();
        assert_eq!(objs.len(), n, "{t}: objects made vs listed");
        for o in &objs {
            assert!(o.matrix.iter().all(|x| x.is_finite()), "{t}: object {:#x} matrix {:?}", o.name, o.matrix);
        }
        let mov = objs.iter().filter(|o| o.list_type == 1).count();
        let tags: Vec<usize> = (0..3).map(|k| objs.iter().filter(|o| o.object_type == k).count()).collect();
        for o in &objs {
            assert!(o.object_type < 3 || o.object_type == 0xFFFF, "{t}: object type {}", o.object_type);
        }
        let cops = objs.iter().filter(|o| o.is_dummy_cop_car()).count();
        eprintln!("{t}: {} compartments, {n} objects ({mov} movable), lamps/cops/hydrants {tags:?}", cpts.len());
        total += n;
        total_cops += cops;
    }
    assert!(total > 1000, "only {total} objects");
    assert!(total_cops > 0, "no propld_dummycopcar object");
}

#[test]
fn car_hits_movable_object() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load("London").expect("London collision");
    let mut w = PhysicsWorld::new(gc, Some(track));
    let cpts = compartments("London");
    assert!(w.register_compartment(&cpts[0].1) > 0);
    let objs = w.objects();
    // a movable object standing on open ground: the track is hit straight below it and 10 m south of it
    let mut pick = None;
    for o in objs.iter().filter(|o| o.list_type == 1) {
        let (x, y, z) = (o.matrix[12], o.matrix[13], o.matrix[14]);
        let down = |w: &mut PhysicsWorld, x: f32, z: f32| w.ray_cast([x, y + 3.0, z], [x, y - 3.0, z], RAY_TRACK | RAY_NORMAL, -1);
        let (Some(g0), Some(g1)) = (down(&mut w, x, z), down(&mut w, x, z + 10.0)) else { continue };
        if (g0.point[1] - g1.point[1]).abs() < 0.3 && g1.normal[1] > 0.95 {
            pick = Some((*o, g1.point[1]));
            break;
        }
    }
    let (o, gy) = pick.expect("a movable object on flat ground");
    eprintln!("object at {:?}, ground {gy}, flags {:#x}", &o.matrix[12..15], o.flags);
    let a = -std::f32::consts::FRAC_PI_4;
    let mut inputs = LocalConfigInputs::default_spans("350Z");
    inputs.is_player = true;
    // facing -z, 10 m south of the object
    let id = w.spawn(&SpawnParams { inputs, position: [o.matrix[12], gy + 0.6, o.matrix[14] + 10.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: -1 });
    w.set_human_control(id, false);
    w.set_control_mode(id, 0);
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    let mut impacts = 0;
    let mut woke = false;
    let (mut hit_msgs, mut woken_msgs) = (0, 0);
    for _ in 0..24 * 6 {
        w.set_controls(id, &c);
        w.step(1.0 / 24.0, &[]);
        let _ = w.take_collisions();
        for n in w.take_object_notices() {
            match n {
                PhysicsObjectNotice::Hit { object } if object == o.name => hit_msgs += 1,
                PhysicsObjectNotice::Woken { object } if object == o.name => woken_msgs += 1,
                _ => {}
            }
        }
        impacts += w.take_object_events().iter().filter(|e| matches!(e, PhysicsObjectEvent::Impact { .. })).count();
        woke |= w.objects().iter().any(|p| p.name == o.name && p.awake);
        if std::env::var("POTRACE").is_ok() {
            let p = w.objects().into_iter().find(|p| p.name == o.name).unwrap();
            eprintln!("car {:?} obj {:?} awake {}", w.position(id), &p.matrix[12..15], p.awake);
        }
    }
    let after = w.objects().into_iter().find(|p| p.name == o.name).unwrap();
    let moved = ((after.matrix[12] - o.matrix[12]).powi(2) + (after.matrix[14] - o.matrix[14]).powi(2)).sqrt();
    eprintln!("object {:#x} kind {}: {impacts} impacts, woke {woke}, moved {moved:.2} m, car at {:?}", o.name, o.kind, w.position(id));
    assert!(w.position(id).iter().all(|x| x.is_finite()));
    assert!(after.matrix.iter().all(|x| x.is_finite()));
    assert!(impacts > 0, "no hit reported");
    assert!(woke && moved > 0.5, "object not pushed");
    // channels 2 / 4 for the restore point
    eprintln!("messages: {hit_msgs} hit, {woken_msgs} woken");
    assert!(hit_msgs > 0 && woken_msgs > 0, "messages 2 / 4 not raised");
    // reset (0x001E1C00 + 0x001E37D0): back to its placement, asleep, hit flag cleared; unknown names do nothing
    let unknown = (1u32..).find(|n| !objs.iter().any(|p| p.name == *n)).unwrap();
    assert!(!w.reset_object(unknown));
    assert!(w.reset_object(o.name));
    let r = w.objects().into_iter().find(|p| p.name == o.name).unwrap();
    let back = ((r.matrix[12] - o.matrix[12]).powi(2) + (r.matrix[13] - o.matrix[13]).powi(2) + (r.matrix[14] - o.matrix[14]).powi(2)).sqrt();
    eprintln!("after reset: {back:.4} m from the placement, awake {}, flags {:#x}", r.awake, r.flags);
    assert!(back < 0.01 && !r.awake && r.flags & 0x1004 == 0, "object not reset");
    // the body went back to the free list: the object can be woken again
    for _ in 0..24 {
        w.set_controls(id, &c);
        w.step(1.0 / 24.0, &[]);
    }
    assert!(w.objects().iter().all(|p| p.matrix.iter().all(|x| x.is_finite())));
}

/// More wakes than the 30 rigid bodies (0x001E4710 reuses the oldest; sleeping objects return theirs, 0x001E4AE0):
/// waking every movable of a London list one after another never runs out of bodies.
#[test]
fn many_wakes_reuse_bodies() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load("London").expect("London collision");
    let mut w = PhysicsWorld::new(gc, Some(track));
    let cpts = compartments("London");
    assert!(w.register_compartment(&cpts[0].1) > 0);
    let names: Vec<u32> = w.objects().iter().filter(|o| o.list_type == 1 && o.collides).map(|o| o.name).take(80).collect();
    assert!(names.len() > 40);
    let mut woken = 0;
    for (k, n) in names.iter().enumerate() {
        assert!(w.wake_object(*n));
        woken += 1;
        w.step(1.0 / 24.0, &[]);
        let awake = w.objects().iter().filter(|o| o.awake).count();
        assert!(awake <= 30, "{awake} awake after {k} wakes");
    }
    eprintln!("{woken} wakes, {} awake", w.objects().iter().filter(|o| o.awake).count());
}

/// F3EventInstanceMove (0x0017D540): a movable is placed at the location, woken and launched with the velocity;
/// the reset of every object (0x001DE400) puts it back to sleep at its placement.
#[test]
fn instance_move_and_reset_all() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load("London").expect("London collision");
    let mut w = PhysicsWorld::new(gc, Some(track));
    let cpts = compartments("London");
    assert!(w.register_compartment(&cpts[0].1) > 0);
    let o = *w.objects().iter().find(|o| o.list_type == 1 && o.collides).expect("a movable");
    let to = [o.matrix[12] + 2.0, o.matrix[13] + 5.0, o.matrix[14]];
    // `step` runs the physics frame only with a car in the world
    let _car = w.spawn(&SpawnParams { inputs: LocalConfigInputs::default_spans("350Z"), position: [o.matrix[12], o.matrix[13] + 1.0, o.matrix[14] + 40.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: -1 });
    let unknown = (1u32..).find(|n| !w.objects().iter().any(|p| p.name == *n)).unwrap();
    assert!(!w.instance_move(unknown, to, [0.0; 3], [0.0; 3]));
    assert!(w.instance_move(o.name, to, [10.0, 0.0, 0.0], [0.0, 1.0, 0.0]));
    let p = w.objects().into_iter().find(|p| p.name == o.name).unwrap();
    assert!(p.awake && (p.matrix[12] - to[0]).abs() < 1e-3 && (p.matrix[13] - to[1]).abs() < 1e-3, "not moved: {:?}", &p.matrix[12..15]);
    for _ in 0..6 {
        w.step(1.0 / 24.0, &[]);
    }
    let p = w.objects().into_iter().find(|p| p.name == o.name).unwrap();
    eprintln!("moved object from {to:?} to {:?} in 6 frames, awake {}", &p.matrix[12..15], p.awake);
    assert!(p.matrix[12] > to[0] + 0.5 && p.matrix.iter().all(|x| x.is_finite()), "velocity not applied");
    w.reset_all_objects();
    let r = w.objects().into_iter().find(|p| p.name == o.name).unwrap();
    let back = ((r.matrix[12] - o.matrix[12]).powi(2) + (r.matrix[13] - o.matrix[13]).powi(2) + (r.matrix[14] - o.matrix[14]).powi(2)).sqrt();
    assert!(back < 0.01 && !r.awake, "not reset: {back} m, awake {}", r.awake);
    assert!(w.objects().iter().all(|p| !p.awake));
    w.step(1.0 / 24.0, &[]);
}
