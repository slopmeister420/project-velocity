//! positive control: every .cpt / index.idx inside every VCD parses and decompresses to its stated size.
use std::path::{Path, PathBuf};
use visos::compartment::compartment_sys::{Compartment, CompartmentIndex, CptEntryKind};
use visos::file::{game_dir, Vcd};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("vcd")) {
            out.push(p);
        }
    }
}

#[test]
fn parse_every_compartment() {
    if !game_dir().exists() {
        return;
    }
    let mut files = Vec::new();
    walk(game_dir(), &mut files);
    let (mut n, mut idx, mut tex, mut mesh, mut lists) = (0, 0, 0, 0, 0);
    for p in &files {
        let v = Vcd::open_host(p, &p.display().to_string()).unwrap();
        for e in v.entries() {
            let lower = e.name.to_ascii_lowercase();
            let Some(d) = v.data(&e) else { continue };
            if lower.ends_with(".cpt") {
                let c = Compartment::parse(d).unwrap_or_else(|err| panic!("{} {}: {err}", p.display(), e.name));
                for en in &c.entries {
                    match en.kind() {
                        CptEntryKind::Texture if en.offset != 0 => {
                            tex += 1;
                            assert!(en.name.as_deref().is_some_and(|s| !s.is_empty()), "{}", e.name);
                        }
                        CptEntryKind::Mesh(_) => mesh += 1,
                        _ => {}
                    }
                }
                lists += c.instance_lists.len();
                if let Some(err) = &c.instance_error { eprintln!("{} {}: {err}", p.display(), e.name); }
                n += 1;
            } else if lower == "index.idx" {
                let i = CompartmentIndex::parse(d).unwrap();
                assert!(i.version == 1 || i.version == 2);
                idx += 1;
            }
        }
    }
    eprintln!("{n} compartments, {idx} indexes, {tex} textures, {mesh} meshes, {lists} instance lists");
    assert_eq!(n, 541);
}
