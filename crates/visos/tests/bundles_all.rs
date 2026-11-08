//! positive control: every .bun / .bdl bundle parses, all types resolve, all typed data decodes.
use std::path::{Path, PathBuf};
use visos::chunk::Bundle;
use visos::file::game_dir;
use visos::timeline::file_tlf::FilePayload;

fn walk(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, exts, out);
        } else if p.extension().and_then(|x| x.to_str()).is_some_and(|x| exts.iter().any(|e| x.eq_ignore_ascii_case(e))) {
            out.push(p);
        }
    }
}

#[test]
fn parse_every_bundle() {
    if !game_dir().exists() {
        return;
    }
    let mut files = Vec::new();
    walk(game_dir(), &["bun", "bdl"], &mut files);
    assert_eq!(files.len(), 155);
    let (mut inst, mut items, mut typed, mut accepted) = (0, 0, 0, 0);
    for p in &files {
        let bytes = std::fs::read(p).unwrap();
        let (b, _env) = Bundle::load_standalone(&bytes).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        accepted += b.engine_accepts as usize;
        let i = b.instances.as_ref().unwrap_or_else(|| panic!("{} has no instances", p.display()));
        for x in &i.instances {
            inst += 1;
            assert!(x.name.is_some(), "{}", p.display());
            if x.type_ref.is_some() {
                assert!(x.value.is_some(), "{}: {:?} type unresolved", p.display(), x.type_name());
            }
        }
        for t in &b.timelines {
            assert!(t.length >= 0.0);
            for c in &t.channels {
                for tr in &c.tracks {
                    for it in &tr.items {
                        items += 1;
                        if let FilePayload::Typed { type_ref, value, .. } = &it.payload {
                            typed += 1;
                            assert!(value.is_some(), "{}: typed item {:?} unresolved", p.display(), type_ref);
                        }
                    }
                }
            }
        }
    }
    eprintln!("{} bundles ({} engine-accepted), {} instances, {} items ({} typed)", files.len(), accepted, inst, items, typed);
    assert!(inst > 0 && typed > 0);
}

#[test]
fn heat_haze_values() {
    if !game_dir().exists() {
        return;
    }
    let bytes = visos::file::platform::read(r"\Tracks\London\Circuit\Race05\heathaze.bdl").unwrap();
    let (b, _) = Bundle::load_standalone(&bytes).unwrap();
    assert!(b.engine_accepts);
    let i = b.instances.unwrap();
    assert_eq!(i.instances.len(), 3);
    let v = i.instances[0].value.as_ref().unwrap();
    assert_eq!(v.field("Radius").unwrap().as_f32(), Some(10.0));
    assert_eq!(v.field("FocalPoint").unwrap().as_vec4(), Some([0.0, 0.0, 0.0, 1.0]));
}
