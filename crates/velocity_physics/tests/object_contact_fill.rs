//! Car vs movable objects (PhysicsManager 0x001AAED0 -> compound contacts 0x00378750): every contact record that
//! reaches a message must be written. Regression for the sweep's 0xCCCCCCCC contact point (London2 Circuit Race03
//! wet: the stack fill of 0x001B32E0's record array reached the Impact / CollisionParticles events and the
//! car-damage intake).
#![cfg(feature = "tr")]

use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::physics_objects::PhysicsObjectEvent;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{PhysicsWorld, SpawnParams, RAY_NORMAL, RAY_TRACK};
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;
use visos::file::Vcd;

const FILL: f32 = -107374176.0; // 0xCCCCCCCC

fn bad(p: &[f32; 3]) -> bool {
    p.iter().any(|x| x.to_bits() == 0xCCCC_CCCC || !x.is_finite() || x.abs() > 1.0e6)
}

fn run(track: &str, near: Option<([f32; 2], f32)>, max: usize) -> usize {
    let gc = load_global_config_with_fallback(0).unwrap();
    let mut w = PhysicsWorld::new(gc, Some(TrackWorld::load(track).expect("collision")));
    let cpts = Vcd::open(&format!(r"\Tracks\{track}\CPTS.VCD")).unwrap();
    assert!(w.register_compartment(cpts.get("instance.cpt").unwrap()) > 0);
    let objs: Vec<_> = w
        .objects()
        .into_iter()
        .filter(|o| o.list_type == 1 && o.collides)
        .filter(|o| near.map_or(true, |(c, r)| (o.matrix[12] - c[0]).hypot(o.matrix[14] - c[1]) < r))
        .take(max)
        .collect();
    let mut bad_n = 0;
    let c = VehicleControlsRaw { throttle: 1.0, ..Default::default() };
    for o in &objs {
        let (x, y, z) = (o.matrix[12], o.matrix[13], o.matrix[14]);
        let Some(g) = w.ray_cast([x, y + 3.0, z + 8.0], [x, y - 3.0, z + 8.0], RAY_TRACK | RAY_NORMAL, -1) else { continue };
        let a = 0.0f32;
        let mut inputs = LocalConfigInputs::default_spans("350Z");
        inputs.is_player = true;
        let id = w.spawn(&SpawnParams { inputs, position: [x, g.point[1] + 0.6, z + 8.0], orientation: [0.0, a.sin(), 0.0, a.cos()], slot: -1 });
        w.set_human_control(id, false);
        w.set_control_mode(id, 0);
        for _ in 0..48 {
            w.set_controls(id, &c);
            w.step(1.0 / 24.0, &[]);
            for m in w.take_collisions() {
                if bad(&m.point) || bad(&m.normal) {
                    eprintln!("object {:#x} kind {} at {:?}: collision {:?}", o.name, o.kind, [x, y, z], m.point);
                    bad_n += 1;
                }
            }
            for e in w.take_object_events() {
                let p = match &e {
                    PhysicsObjectEvent::Impact { point, .. } | PhysicsObjectEvent::CollisionParticles { point, .. } => *point,
                    _ => continue,
                };
                if bad(&p) {
                    eprintln!("object {:#x} kind {} at {:?}: {:?}", o.name, o.kind, [x, y, z], e);
                    bad_n += 1;
                }
            }
        }
        w.remove(id);
        w.reset_all_objects();
    }
    eprintln!("{track}: {} objects driven into, {bad_n} unwritten contact points", objs.len());
    let _ = FILL;
    bad_n
}

/// the object of the sweep report (London2 Circuit Race03 wet, object at (415.7, 6.8, -202.2))
#[test]
fn london2_object_contact_points_written() {
    assert_eq!(run("London2", Some(([415.7, -202.2], 15.0)), 40), 0);
}

/// a wider sample: the first movables of London2 and London
#[test]
fn movable_object_contact_points_written() {
    assert_eq!(run("London2", None, 150), 0);
    assert_eq!(run("London", None, 150), 0);
}
