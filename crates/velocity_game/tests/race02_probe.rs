//! temporary probe: LADown Race02 trailblazer end (deck / street / race objects)
use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, RAY_NORMAL, RAY_OBJECTS, RAY_TRACK};

#[test]
#[ignore]
fn race02_triggers() {
    use velocity_game::timeline_manager::objects::timeline_trigger::load_level_triggers;
    let big = visos::handy::handy_crc::crc32(b"TriggerBigParticles", 0);
    for (k, l) in load_level_triggers(r"\Tracks\LADOWN\", r"\Tracks\LADOWN\CIRCUIT\RACE02\").iter().enumerate() {
        for (i, t) in l.iter().flatten().enumerate() {
            let p = t.position;
            if (p[0] > 500.0 && p[2] < -40.0 && p[2] > -120.0) || t.target_crc == big || t.name_crc == big {
                println!("list {k} #{i} pos {:.1?} rot {:.2?} dim {:.1?} r2 {} name {:#x} type {:#x} target {:#x} big {}", p, t.rotation, t.dimensions, t.radius_sq, t.name_crc, t.type_crc, t.target_crc, t.target_crc == big);
            }
        }
    }
}

#[test]
#[ignore]
fn race02_trigger_timelines() {
    use visos::handy::handy_crc::{crc32, crc32_no_case};
    let b = std::fs::read(visos::file::platform::resolve(r"\Tracks\LADOWN\CIRCUIT\RACE02\Timeline.bun")).unwrap();
    let mut env = visos::type_::vs_type::CEnvironment::with_builtins();
    let bun = visos::chunk::Bundle::load(&b, &mut env).unwrap();
    let want = [0xc63160bbu32, 0x94a20ae4];
    for t in &bun.timelines {
        for c in &t.channels {
            let names = [c.name.as_deref(), c.str_a.as_deref(), c.str_b.as_deref()];
            let hit = want.contains(&c.crc)
                || names.iter().flatten().any(|n| want.contains(&crc32(n.as_bytes(), 0)) || want.contains(&crc32_no_case(n.as_bytes(), 0)) || n.contains("BigPart") || n.contains("Trigger"));
            for tr in &c.tracks {
                for it in &tr.items {
                    let s = format!("{:?}", it.payload);
                    for q in s.split('"').skip(1).step_by(2).filter(|q| !q.is_empty()) {
                        if want.contains(&crc32(q.as_bytes(), 0)) || want.contains(&crc32_no_case(q.as_bytes(), 0)) {
                            println!("ITEM timeline {:#x} {:?} channel {:?} start {} names {q}: {:.300}", t.name_crc, t.name, c.name, it.start, s);
                        }
                    }
                }
            }
            if hit && false {
                println!("timeline {:#x} {:?} channel {:#x} {:?}", t.name_crc, t.name, c.crc, names);
                for tr in &c.tracks {
                    for it in &tr.items {
                        println!("  track {:?} item start {} len {} {:.200}", tr.name, it.start, it.length, format!("{:?}", it.payload));
                    }
                }
            }
        }
    }
}

#[test]
#[ignore]
fn race02_probe() {
    let gc = load_global_config_with_fallback(0).unwrap();
    let track = TrackWorld::load("LADown").expect("LADown collision");
    let mut w = PhysicsWorld::new(gc, Some(track));
    let v = visos::file::Vcd::open(r"\Tracks\LADOWN\CIRCUIT\RACE02\RACE.VCD").unwrap();
    let mut n = 0;
    for name in ["instance.cpt", "uinstance.cpt"] {
        if let Some(b) = v.get(name) {
            n += w.register_compartment(b);
        }
    }
    println!("objects {n}");
    for o in w.objects() {
        let p = [o.matrix[12], o.matrix[13], o.matrix[14]];
        if o.name == 0x00aca5b4 || o.name == 0xf8e7a95e || ((p[0] - 489.5).abs() < 25.0 && p[2] > -60.0 && p[2] < 0.0) {
            println!("MOVE obj {:08x} kind {} list {} pos {:.2?} flags {:#x} collides {} m {:.2?}", o.name, o.kind, o.list_type, p, o.flags, o.collides, &o.matrix[..12]);
        }
        if (p[0] - 560.0).abs() < 60.0 && (p[2] + 60.0).abs() < 40.0 {
            println!("obj {:08x} kind {} list {} pos {:.1?} flags {:#x} collides {}", o.name, o.kind, o.list_type, p, o.flags, o.collides);
        }
    }
    // layers on a grid
    for xi in 0..=24 {
        let x = 345.0 + 5.0 * xi as f32;
        let mut row = String::new();
        for &z in &[-45.0f32, -50.0, -55.0, -60.0, -65.0, -70.0, -75.0, -80.0, -85.0] {
            let mut top = 20.0f32;
            let mut l = Vec::new();
            while top > -40.0 && l.len() < 4 {
                let Some(h) = w.ray_cast([x, top, z], [x, -40.0, z], RAY_TRACK | RAY_NORMAL, -1) else { break };
                l.push(format!("{:.1}{}", h.point[1], if h.normal[1] < 0.0 { "v" } else { "" }));
                top = h.point[1] - 0.05;
            }
            row += &format!(" z{z}:[{}]", l.join(","));
        }
        println!("x{x}:{row}");
    }
    // the wall at the deck end: height / z extent
    for &y in &[-9.4f32, -9.0, -8.0, -7.0, -6.0, -4.0, -2.0, 0.0] {
        let mut row = String::new();
        for zi in 0..=12 {
            let z = -44.0 - 2.0 * zi as f32;
            let h = w.ray_cast([565.0, y, z], [600.0, y, z], RAY_TRACK | RAY_NORMAL, -1);
            row += &match h {
                Some(h) => format!(" {:.1}", h.point[0]),
                None => " -".into(),
            };
        }
        println!("wall y{y}: z-44..-68 step -2:{row}");
    }
    // horizontal casts from the stuck point toward the hole
    for &(f, t) in &[([570.0f32, -9.0, -55.2], [600.0f32, -9.0, -55.2]), ([570.0, -8.5, -55.2], [600.0, -8.5, -58.0]), ([570.0, -9.0, -53.0], [600.0, -9.0, -60.0])] {
        for (lbl, fl) in [("track", RAY_TRACK | RAY_NORMAL), ("objects", RAY_OBJECTS | RAY_NORMAL)] {
            let h = w.ray_cast(f, t, fl, -1);
            println!("{lbl} {f:?}->{t:?}: {:?}", h.map(|h| (h.point, h.normal)));
        }
    }
}
