//! dump every texture of a VCD to 32-bit .tga files: `vtfdump [--game=<dir>] <game path of vcd> <out dir> [name filter]`

use visos_render::renderer::{d3dx_tga, texture_manager};

fn main() {
    visos::file::platform::init_game_dir_from_args().expect("game dir");
    let a: Vec<String> = std::env::args().filter(|x| !x.starts_with("--game=")).collect();
    let vcd = visos::file::Vcd::open(&a[1]).expect("vcd");
    let out = std::path::PathBuf::from(&a[2]);
    std::fs::create_dir_all(&out).unwrap();
    let filter = a.get(3).map(|s| s.to_ascii_lowercase());
    for e in vcd.entries() {
        let l = e.name.to_ascii_lowercase();
        if filter.as_ref().is_some_and(|f| !l.contains(f.as_str())) {
            continue;
        }
        let d = vcd.data(&e).unwrap_or_default();
        let t = if l.ends_with(".vtf") {
            texture_manager::decode_any_vtf(d).ok()
        } else if l.ends_with(".tga") {
            d3dx_tga::decode_tga(d)
        } else {
            None
        };
        let Some(t) = t else { continue };
        let mut f = vec![0u8, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        f.extend((t.width as u16).to_le_bytes());
        f.extend((t.height as u16).to_le_bytes());
        f.extend([32, 0x28]);
        for p in t.levels[0].chunks_exact(4) {
            f.extend([p[2], p[1], p[0], p[3]]);
        }
        std::fs::write(out.join(format!("{}.tga", e.name)), f).unwrap();
    }
}
