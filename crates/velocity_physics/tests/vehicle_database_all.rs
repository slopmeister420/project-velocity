//! every vehicle database bin in the install: VEHDB.VCD (vehicle/setup/upgrades), XMLBIN.VCD and the
//! loose copies of WHEELS/TRAFFIC/COLOURS/GLOBALS/DECALS, then the full database build.

use velocity_physics::vehicle_database::vehicle_database::{BinReader, NUM_BODYPART_TYPES};
use velocity_physics::vehicle_database::{CVehicleDatabase, VehicleSetup, string_hash};
use visos::file::{Vcd, VsFileDirectoryDetails, platform};

#[test]
fn string_hash_matches_binary() {
    assert_eq!(string_hash("LON_BUS01"), 527812301);
    assert_eq!(string_hash("240sx"), 1961211601);
    assert_eq!(string_hash("SIZE1"), 609530544);
}

#[test]
fn every_vehdb_bin() {
    let vcd = Vcd::open(r"\Vehicles\VEHDB.VCD").expect("vehdb");
    let mut db = CVehicleDatabase::default();
    let (mut nv, mut ns, mut nu) = (0, 0, 0);
    for e in vcd.entries() {
        let name = e.name.to_ascii_lowercase();
        if let Some(car) = name.strip_suffix("upgrades.bin") {
            assert!(db.load_upgrade_bin(&vcd, &name, car).unwrap(), "{name}");
            nu += 1;
        } else if let Some(car) = name.strip_suffix("setup.bin") {
            db.load_setup_bin(&vcd, &name, car).unwrap();
            ns += 1;
        } else if let Some(car) = name.strip_suffix("vehicle.bin") {
            db.load_vehicle_bin(&vcd, &name, car).unwrap();
            nv += 1;
        } else {
            panic!("unexpected {name}");
        }
    }
    assert_eq!((nv, ns, nu), (33, 33, 33));
    assert_eq!(db.vehicles.len(), 33);
    assert_eq!(db.num_vehicles as usize, db.vehicles.len());
    assert_eq!(db.num_setups as usize, db.setups.len());
    assert_eq!(db.num_bodypart_sets as usize, db.bodypart_sets.len());
    for v in &db.vehicles {
        assert_eq!(v.hash, string_hash(v.name.as_deref().unwrap()), "vehicle hash is its name");
        assert!(matches!(v.data.tier, 1..=8 | 100), "{:?} tier {}", v.name, v.data.tier);
        assert_eq!(v.data.colours.len(), v.data.num_colours as usize);
        assert!(v.setup_hash != 0);
    }
    for s in &db.bodypart_sets {
        assert!(db.find_vehicle(s.vehicle_hash).is_some());
        for t in 0..NUM_BODYPART_TYPES {
            assert_eq!(s.parts[t].len(), s.part_counts[t] as usize);
            for p in &s.parts[t] {
                assert!(p.level <= 12, "level {}", p.level);
                assert_eq!(p.names.len(), s.num_names as usize);
            }
        }
    }
    for s in &db.setups {
        assert!(db.find_vehicle(s.vehicle_hash).is_some());
        assert!(db.find_bodypart_set(s.bodypart_set_hash).is_some());
    }
}

fn xml_and_loose(xml_name: &str, loose: &str) -> Vec<(String, Vec<u8>)> {
    let xml = VsFileDirectoryDetails::open(r"\VEHICLES\XMLBIN.VCD").unwrap();
    vec![
        (format!("xmlbin:{xml_name}"), xml.load_file(xml_name).unwrap()),
        (loose.to_string(), platform::read(loose).unwrap()),
    ]
}

#[test]
fn every_global_bin() {
    for (n, b) in xml_and_loose("wheels.bin", r"\Vehicles\Wheels\WHEELS.BIN") {
        let mut db = CVehicleDatabase::default();
        let mut r = BinReader::new(&n, &b);
        db.load_wheels(&mut r, r"\Vehicles\Wheels\").unwrap();
        assert!(r.at_end(), "{n}");
        assert_eq!((db.wheels.len(), db.brake_parts.len(), db.tyres.len()), (208, 12, 5));
        assert!(db.tyres.iter().all(|t| t.grip <= 4));
        assert!(db.brake_parts.iter().all(|b| b.strings.len() == b.num_strings as usize));
        assert!(db.find_brake_part(string_hash("BRAKE_DISC_DEFAULT")).is_some());
        assert!(db.find_brake_part(string_hash("BRAKE_PAD_DEFAULT")).is_some());
        assert!(db.find_tyre(string_hash("TYRE_VS_MEDIUM")).is_some());
    }
    for (n, b) in xml_and_loose("Traffic.BIN", r"\Vehicles\TRAFFIC.BIN") {
        let mut db = CVehicleDatabase::default();
        let mut r = BinReader::new(&n, &b);
        db.load_traffic(&mut r).unwrap();
        assert!(r.at_end(), "{n}");
        assert_eq!(db.traffic.len(), 18);
        for t in &db.traffic {
            assert_eq!(t.hash, string_hash(t.name.as_deref().unwrap()), "{:?}", t.name);
            assert!(t.half_wheel_span_front_back > 0.5 && t.half_wheel_span_front_back < 5.0);
            assert!(t.half_track > 0.3 && t.half_track < 2.0);
            // LON_4X4 and LON_VAN01 ship a zero wheel radius
            assert!((0.0..1.0).contains(&t.wheel_radius), "{:?}", t.name);
        }
        assert!(db.find_traffic_by_name("LON_BUS01").is_some());
    }
    for (n, b) in xml_and_loose("COLOURS.BIN", r"\Vehicles\COLOURS.BIN") {
        let mut db = CVehicleDatabase::default();
        let mut r = BinReader::new(&n, &b);
        db.load_colours(&mut r).unwrap();
        assert!(r.at_end(), "{n}");
        assert_eq!(db.colours.len(), 203);
        for c in &db.colours {
            assert!(c.colour_a.iter().chain(&c.colour_b).all(|v| (0.0..=255.0).contains(v)));
        }
    }
    for (n, b) in xml_and_loose("GLOBALS.BIN", r"\Vehicles\GLOBALS.BIN") {
        let mut db = CVehicleDatabase::default();
        let mut r = BinReader::new(&n, &b);
        db.load_globals(&mut r).unwrap();
        assert!(r.at_end(), "{n}");
        assert!(db.globals.unwrap().values.iter().all(|v| v.is_finite()));
    }
    for (n, b) in xml_and_loose("DECALS.BIN", r"\Vehicles\decals.bin") {
        let mut db = CVehicleDatabase::default();
        let mut r = BinReader::new(&n, &b);
        db.load_decals(&mut r).unwrap();
        assert!(r.at_end(), "{n}");
        assert_eq!(db.decals.len(), 245);
        assert!(db.decals.iter().all(|d| d.kind <= 3 && d.name.is_some()));
    }
}

#[test]
fn build_database() {
    let db = CVehicleDatabase::load().expect("vehicle database");
    // vehicle folders: 12 LA + 8 London + 9 Tokyo + 2 Teams + 1 HIN (no SUNB folder)
    assert_eq!(db.vehicles.len(), 32);
    assert!(db.find_vehicle_by_name("SUNBEAM").is_none());
    for s in &db.setups {
        for t in 0..NUM_BODYPART_TYPES {
            if s.part_hashes[t] != 0 {
                assert!(s.setup_bodyparts[t].is_some(), "setup part {t} unresolved");
            }
        }
    }
    for v in &db.vehicles {
        assert!(v.setup.is_some(), "{:?} has no setup", v.name);
        assert!(v.data.colours.iter().all(|c| c.colour.is_some()));
        let mut s = VehicleSetup::new();
        assert!(db.setup_default_vehicle(&mut s, v.hash));
        let pm = s.performance_modifiers(&db);
        assert!(pm.as_array().iter().all(|x| (0.0..=1.0).contains(x)), "{:?} {pm:?}", v.name);
    }
    assert!(db.bodypart_sets.iter().all(|s| s.vehicle.is_some()));

    let vi = db.find_vehicle_by_name("240sx").expect("240SX");
    let v = &db.vehicles[vi];
    println!("240SX entry: hash {:#010x} name {:?} path {} tier {} unlock {} hidden {}", v.hash, v.name, v.path, v.data.tier, v.unlock_value, v.hidden);
    println!("  unk {:#x} {:#x} {:#x} setups {:#010x} {:#010x} -> {:?} {:?}", v.unk_08, v.unk_0c, v.unk_10, v.setup_hash, v.setup_hash_2, v.setup, v.setup_2);
    println!("  lists a/b/c {}/{}/{} colours {} stats {:?} audio {:?}", v.data.list_a.len(), v.data.list_b.len(), v.data.list_c.len(), v.data.num_colours, v.stats.values, v.audio);
    println!("  list_a[0] {:?}", v.data.list_a[0]);
    println!("  data +0x14 {:?}", &v.data.unk_014[..16]);
    let mut s = VehicleSetup::new();
    db.setup_default_vehicle(&mut s, v.hash);
    for (t, p) in s.fitted_bodyparts.iter().enumerate() {
        if let Some(p) = p {
            let part = db.part(*p);
            println!("  fitted {:<11} hash {:#010x} level {} cost {}", velocity_physics::vehicle_database::vehicle_database::BODYPART_TYPE_NAMES[t], part.hash, part.level, part.cost);
        }
    }
    println!("  tyre {:?} brakes {:?}", s.tyre.map(|t| &db.tyres[t].name), s.brake_parts.map(|b| b.map(|b| db.brake_parts[b].level)));
    let pm = s.performance_modifiers(&db);
    println!("  default performance modifiers {pm:?}");
    println!("  top speed {} acceleration {}", s.get_top_speed(&db), s.get_acceleration(&db));
    let t = &db.traffic[db.find_traffic_by_name("LON_BUS01").unwrap()];
    println!("LON_BUS01 traffic: {t:?}");
}

/// RandomiseSetup 0x00182720 (attract mode) on every vehicle's default setup, 8 seeds each
#[test]
fn randomise_setup_every_vehicle() {
    let db = CVehicleDatabase::load().expect("vehicle database");
    let mut rand = visos::handy::crt_rand::CrtRand::default();
    let (mut priced, mut recoloured) = (0, 0);
    for v in &db.vehicles {
        for _ in 0..8 {
            let mut s = VehicleSetup::new();
            assert!(db.setup_default_vehicle(&mut s, v.hash));
            let body = s.colours[0].clone();
            let cost = db.randomise_setup(&mut s, true, &mut rand);
            assert!(cost >= 0, "{:?} cost {cost}", v.name);
            priced += (cost > 0) as usize;
            let decals = [s.decal_0, s.decal_1, s.decal_2_4, s.decal_2_5].into_iter()
                .chain(s.decals_3).chain(s.decals_3_1).chain(s.decals_3_2).chain(s.decals_3_3);
            assert!(decals.flatten().all(|d| d < db.decals.len()), "{:?}", v.name);
            recoloured += (s.colours[0].hash != body.hash) as usize;
        }
    }
    println!("randomise_setup: {priced} priced predefs, {recoloured} body colour changes");
    assert!(priced > 0 && recoloured > 0);
}
