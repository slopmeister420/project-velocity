//! temporary probe: LADown Race02 raceline / trailblazer lines near x 440..620
use velocity_ai::ai::vs_ai_world::VsAISplineData;

#[test]
#[ignore]
fn race02_lines() {
    let d = visos::file::platform::read(r"\Tracks\LADOWN\CIRCUIT\RACE02\AISpline.spl").expect("spl");
    let s = VsAISplineData::parse(&d).expect("parse");
    let near = |x: f32, z: f32| (430.0..640.0).contains(&x) && (-130.0..-30.0).contains(&z);
    for (i, r) in s.race_sections.iter().enumerate() {
        for (k, p) in r.raceline.points.iter().enumerate() {
            if near(p.x, p.z) {
                println!("race {i} pt {k}: {:.1} {:.1} {:.1}", p.x, p.y, p.z);
            }
        }
    }
    let tl = visos::file::platform::read(r"\Tracks\LADOWN\CIRCUIT\RACE02\Timeline.bun").expect("bun");
    let (b, _) = visos::chunk::Bundle::load_standalone(&tl).expect("bundle");
    let mut out = String::new();
    for t in &b.timelines {
        out += &format!("TIMELINE {:08x} {:?} len {}\n", t.name_crc, t.name, t.length);
        for c in &t.channels {
            out += &format!("  CH {:08x} {:?} type {:08x} {:?} {:?}\n", c.crc, c.name, c.type_crc, c.str_a, c.str_b);
            for tr in &c.tracks {
                for it in &tr.items {
                    let p = match &it.payload {
                        visos::timeline::file_tlf::FilePayload::Typed { crc, type_ref, value, .. } => {
                            format!("{crc:08x} {} {:?}", type_ref.as_ref().map_or("?", |t| t.name.as_str()), value)
                        }
                        o => format!("{o:?}"),
                    };
                    out += &format!("    item k{} t{:.2} l{:.2} {}\n", it.kind, it.start, it.length, &p[..p.len().min(300)]);
                }
            }
        }
    }
    if let Some(inst) = &b.instances {
        for i in &inst.instances {
            let tn = i.type_name().unwrap_or("?");
            if tn.contains("Trigger") || tn.contains("TrailBlazer") {
                let w = i.world;
                out += &format!("INST {:?} type {} pos {:.1?} value {:?}\n", i.name, tn, w[3], i.value);
            }
        }
    }
    std::fs::write(std::env::var("PROBE_OUT").unwrap(), out).unwrap();
    let trg =visos::file::platform::read(r"\Tracks\LADOWN\CIRCUIT\RACE02\Triggers.trg").expect("trg");
    let [track_t, _] = velocity_game::timeline_manager::objects::timeline_trigger::load_level_triggers(r"\Tracks\LADOWN\", r"\Tracks\LADOWN\CIRCUIT\RACE02\");
    for (i, t) in track_t.unwrap_or_default().iter().enumerate() {
        if t.is_trailblazer() || near(t.position[0], t.position[2]) {
            println!("track trigger {i}: pos {:.1?} dim {:.1?} r2 {:.1} type {:08x} target {:08x} params {:08x?} tb {}", &t.position[..3], t.dimensions, t.radius_sq, t.type_crc, t.target_crc, t.params, t.is_trailblazer());
        }
    }
    for (i, t) in velocity_game::timeline_manager::objects::timeline_trigger::parse_triggers(&trg).unwrap().iter().enumerate() {
        println!(
            "trigger {i}: pos {:.1?} dim {:.1?} r2 {:.1} name {:08x} type {:08x} target {:08x} params {:08x?} tb {}",
            &t.position[..3], t.dimensions, t.radius_sq, t.name_crc, t.type_crc, t.target_crc, t.params, t.is_trailblazer()
        );
    }
    for (set, secs) in s.road_sections.iter().enumerate() {
        for (i, sec) in secs.iter().enumerate() {
            for l in &sec.lanes {
                if l.line.get_id() == 0xa777883a {
                    let pts: Vec<_> = l.line.points.iter().take(60).map(|p| format!("({:.0} {:.0})", p.x, p.z)).collect();
                    println!("A7 first: {}", pts.join(" "));
                }
                let pts: Vec<_> = l.line.points.iter().filter(|p| near(p.x, p.z)).map(|p| format!("({:.0} {:.1} {:.0})", p.x, p.y, p.z)).collect();
                if !pts.is_empty() && set != 0 {
                    println!("set {set} sec {i} line {:08x}: {}", l.line.get_id(), pts.join(" "));
                }
            }
        }
    }
}
