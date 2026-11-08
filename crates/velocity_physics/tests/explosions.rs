//! Explosions and friction spots: PhysicsManager 0x0019FD80 (TimelineExplosion) pushes movable objects and cars and
//! draws from the shared rand stream; PhysicsObjectManager 0x001E18E0 -> 0x001E1A20 -> 0x001B3500 registers a
//! friction spot on the ground and queues a puddle; a hydrant (kind 23) hit through the explosion's message 2
//! (0x001E1F80) drops one; the race restart 0x0019C390 removes them.
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::physics_objects::PhysicsObjectEvent;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{Explosion, FrictionSpot, PhysicsWorld, SpawnParams, RAY_NORMAL, RAY_TRACK};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;
use visos::file::Vcd;

fn world(track: &str) -> PhysicsWorld {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::load(track).expect("collision")));
    let cpt = Vcd::open(&format!(r"\Tracks\{track}\CPTS.VCD")).unwrap().get("instance.cpt").unwrap().to_vec();
    w.register_compartment(&cpt);
    w
}

/// the first race of `track` whose RACE.VCD has instances of type `kind`
fn race_with_kind(w: &mut PhysicsWorld, track: &str, kind: u32) -> bool {
    let root = visos::file::resolve(&format!(r"\Tracks\{track}"));
    let mut races = Vec::new();
    for mode in std::fs::read_dir(&root).into_iter().flatten().flatten() {
        for race in std::fs::read_dir(mode.path()).into_iter().flatten().flatten() {
            for f in std::fs::read_dir(race.path()).into_iter().flatten().flatten() {
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
            let Some(b) = v.get(n) else { continue };
            let mut t = world(track);
            t.register_compartment(b);
            let tags: Vec<(u32, u32)> = t.objects().iter().filter(|o| o.object_type == 2).map(|o| (o.kind, o.list_type)).collect();
            if !tags.is_empty() {
                eprintln!("{rel}:{n} hydrant tags (kind, list) {tags:?}");
            }
            if t.objects().iter().any(|o| o.kind == kind && o.list_type == 1) {
                w.register_compartment(b);
                eprintln!("race {rel}:{n}");
                return true;
            }
        }
    }
    false
}

fn ground(w: &mut PhysicsWorld, x: f32, y: f32, z: f32) -> Option<f32> {
    w.ray_cast([x, y + 3.0, z], [x, y - 3.0, z], RAY_TRACK | RAY_NORMAL, -1).map(|h| h.point[1])
}

/// a movable object (not awake) with flat ground under it and 6 m south of it: (object, ground y there)
fn movable_on_ground(w: &mut PhysicsWorld) -> (velocity_physics::physics_objects::PhysicsObjectInfo, f32) {
    let objs = w.objects();
    for o in objs.iter().filter(|o| o.list_type == 1 && o.collides && !o.awake) {
        let (x, y, z) = (o.matrix[12], o.matrix[13], o.matrix[14]);
        let (Some(g0), Some(g1)) = (ground(w, x, y, z), ground(w, x, y, z + 6.0)) else { continue };
        if (g0 - g1).abs() < 0.3 {
            return (*o, g1);
        }
    }
    panic!("no movable object on flat ground")
}

fn crt_steps(mut s: u32, n: usize) -> u32 {
    for _ in 0..n {
        s = s.wrapping_mul(214013).wrapping_add(2531011);
    }
    s
}

#[test]
fn explosion_pushes_objects_and_cars() {
    let mut w = world("London");
    let (o, gy) = movable_on_ground(&mut w);
    let c = [o.matrix[12], o.matrix[13], o.matrix[14]];
    let id = w.spawn(&SpawnParams { inputs: LocalConfigInputs::default_spans("350Z"), position: [c[0], gy + 0.6, c[2] + 6.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: -1 });
    for _ in 0..24 {
        w.step(1.0 / 24.0, &[]);
    }
    let _ = w.take_object_events();
    let p = w.position(id);
    let v0 = w.velocity(id);
    let d = ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt();
    // ignoring index 0 (our car): only objects move, 3 draws per object within the radius
    w.set_rand_state(12345);
    let e = Explosion { position: c, blast_radius: 15.0, force_objects: 8.0, angular_force_objects: 1.0, force_vehicles: 6.0, angular_force_vehicles: 0.5, sector_angle: 0.0, ignore_vehicle: 0 };
    let n0 = w.explosion(&e);
    assert!(w.velocity(id) == v0, "ignored car pushed");
    let after = w.objects().into_iter().find(|p| p.name == o.name).unwrap();
    assert!(after.awake && n0 >= 1, "object not woken ({n0})");
    let draws = (0..300).find(|k| crt_steps(12345, *k) == w.rand_state()).expect("rand draws not from the handed-in stream");
    assert!(draws % 3 == 0 && draws >= 3, "{draws} draws");
    // the car at index 0: velocity += (car - centre) / d * (r - d) / r * force, 3 more draws
    let s1 = w.rand_state();
    let n1 = w.explosion(&Explosion { ignore_vehicle: -1, ..e });
    let v1 = w.velocity(id);
    let dv = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let want = (15.0 - d) / 15.0 * 6.0;
    let got = (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt();
    eprintln!("explosion: {n0} objects, then {n1}; car {d:.2} m away gets {got:.3} m/s (want {want:.3}), {draws} draws");
    assert!((got - want).abs() < 1e-3 * want.max(1.0), "car push {got} != {want}");
    assert!(dv[2] * (p[2] - c[2]) > 0.0, "car pushed towards the centre");
    assert!(n1 > n0 - 1 && crt_steps(s1, 3) != w.rand_state());
    for _ in 0..12 {
        w.step(1.0 / 24.0, &[]);
    }
    assert!(w.objects().iter().all(|p| p.matrix.iter().all(|x| x.is_finite())));
    assert!(w.position(id).iter().all(|x| x.is_finite()));
}

#[test]
fn friction_spot_and_restart() {
    let mut w = world("London");
    let (o, gy) = movable_on_ground(&mut w);
    let (x, z) = (o.matrix[12], o.matrix[14] + 6.0);
    // mid-air: the ray (+-1 m, `unk_7805B0`) misses, nothing registered
    let miss = FrictionSpot { position: [x, gy + 5.0, z], friction_high: 0.5, radius: 3.0, race_distance: 7.0, ignore_car: -1, expiry: f32::MAX };
    assert!(!w.friction_spot(&miss));
    assert!(w.friction_spots().is_empty() && w.take_puddles().is_empty());
    w.set_rand_state(99);
    assert!(w.friction_spot(&FrictionSpot { position: [x, gy + 0.5, z], ..miss }));
    assert_eq!(w.rand_state(), crt_steps(99, 2), "two VsRandRange draws");
    let spots = w.friction_spots();
    eprintln!("spots {spots:?}");
    assert_eq!(spots.len(), 1);
    assert!((spots[0].1[1] - gy).abs() < 1e-3 && spots[0].2 == 3.0);
    let p = w.take_puddles();
    assert_eq!(p.len(), 1);
    let p = p[0];
    eprintln!("puddle {p:?}");
    assert!((p.direction[0].hypot(p.direction[1]) - 1.0).abs() < 1e-5);
    assert!(p.radius == 15.0 && p.kind == 2 && p.size == [30.0, 30.0] && p.life == 480 && p.race_distance == 7.0);
    assert_eq!(p.flags, 0xCCCC_CCCD, "the original ORs 1 into an uninitialised word");
    // a car parked on the spot keeps running
    let id = w.spawn(&SpawnParams { inputs: LocalConfigInputs::default_spans("350Z"), position: [x, gy + 0.6, z], orientation: [0.0, 0.0, 0.0, 1.0], slot: -1 });
    for _ in 0..24 {
        w.step(1.0 / 24.0, &[]);
    }
    assert!(w.position(id).iter().all(|v| v.is_finite()));
    w.restart_race();
    assert!(w.friction_spots().is_empty(), "restart keeps spots");
    w.step(1.0 / 24.0, &[]);
}

/// LADown's hydrants (instance type 23): the explosion's message 2 reaches the hydrant case of 0x001E1F80, which drops
/// a friction spot under it (race distance from the hook), sets flag 0x40 and plays the hydrant sound once.
#[test]
fn hydrant_drops_friction_spot() {
    let mut w = world("LADown");
    assert!(race_with_kind(&mut w, "LADown", 23), "no LADown race with a hydrant");
    let objs = w.objects();
    let hydrants: Vec<_> = objs.iter().filter(|o| o.kind == 23).copied().collect();
    eprintln!("{} hydrants (kind 23), {} movable", hydrants.len(), hydrants.iter().filter(|o| o.list_type == 1).count());
    let Some(h) = hydrants.iter().find(|o| o.list_type == 1 && o.collides).copied() else {
        panic!("no movable hydrant");
    };
    let c = [h.matrix[12], h.matrix[13], h.matrix[14]];
    let gy = ground(&mut w, c[0], c[1] + 1.0, c[2] + 8.0).unwrap_or(c[1]);
    let _car = w.spawn(&SpawnParams { inputs: LocalConfigInputs::default_spans("350Z"), position: [c[0], gy + 0.6, c[2] + 40.0], orientation: [0.0, 0.0, 0.0, 1.0], slot: -1 });
    w.set_race_distance_hook(Some(std::sync::Arc::new(std::sync::Mutex::new(|_: [f32; 3]| 42.0f32))));
    let e = Explosion { position: [c[0] + 0.5, c[1], c[2]], blast_radius: 3.0, force_objects: 1.0, angular_force_objects: 0.0, force_vehicles: 0.0, angular_force_vehicles: 0.0, sector_angle: 0.0, ignore_vehicle: -1 };
    w.explosion(&e);
    let ev = w.take_object_events();
    let sounds = ev.iter().filter(|e| matches!(e, PhysicsObjectEvent::Explosion { .. })).count();
    let puddles = w.take_puddles();
    let flags = w.objects().into_iter().find(|p| p.name == h.name).unwrap().flags;
    eprintln!("hydrant {:#x}: {sounds} sounds, {} puddles {:?}, spots {:?}, flags {flags:#x}", h.name, puddles.len(), puddles.first(), w.friction_spots());
    assert_eq!(sounds, 1);
    if puddles.is_empty() {
        // no ground within 1 m under the hydrant's origin: 0x40 stays clear (original)
        assert!(flags & 0x40 == 0);
    } else {
        assert!(flags & 0x40 != 0 && puddles[0].race_distance == 42.0 && w.friction_spots().len() == 1);
        // a second hit does nothing more (0x40 set)
        w.explosion(&e);
        assert!(w.take_puddles().is_empty());
    }
}
