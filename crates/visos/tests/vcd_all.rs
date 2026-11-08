//! positive control: every .vcd in the install parses and validates.
use std::path::Path;
use visos::file::{game_dir, Vcd};

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
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
fn parse_every_vcd() {
    let root = game_dir();
    if !root.exists() {
        eprintln!("game dir missing, skipping");
        return;
    }
    let mut files = Vec::new();
    walk(root, &mut files);
    assert_eq!(files.len(), 302);
    let mut bad = Vec::new();
    let mut entries = 0;
    for p in &files {
        let v = Vcd::open_host(p, &p.display().to_string()).unwrap_or_else(|e| panic!("{e}"));
        entries += v.len();
        let problems = v.validate();
        if !problems.is_empty() {
            bad.push(p.file_name().unwrap().to_string_lossy().to_uppercase());
            continue;
        }
        for e in v.entries() {
            assert_eq!(v.find(e.name).unwrap().index, e.index, "{}", e.name);
            assert!(v.get(e.name).is_some());
        }
    }
    // PHYSOBJM.VCD ships with its crc toc overwritten by file data (3 garbage entries); unused by the game.
    assert_eq!(bad, vec!["PHYSOBJM.VCD".to_string()]);
    assert!(entries > 10000, "{entries}");
}

#[test]
fn lookup_by_game_path() {
    if !game_dir().exists() {
        return;
    }
    let v = Vcd::open(r"\ASP\ASP.vcd").unwrap();
    let d = v.get("ASP.GAS").unwrap();
    assert_eq!(&d[..4], b"0SAG");
}
