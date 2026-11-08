//! GameDataLoading.cpp — the track / foyer lighting loaders (directional `lights.txt`, `PtLights.bin` / `.txt`),
//! kept apart from `game_data_loading.rs` for size. The original pushes the values into VsRender as it reads them
//! (SetDirectionalLight / SetAmbient / BuildLightMatrix, AllocPointLights / SetPointLight, TrackFX 0x0014AEC0 per
//! point light); here they come back as `GdlLights` for the render crate.

use bevy::math::{Vec3, Vec4};
use visos::handy::vs_text_file::VsTextFile;
use visos_render::vs_render::{PointLight, VsLights};

/// what the loaders hand to VsRender / TrackFX
#[derive(Clone, Debug, Default)]
pub struct GdlLights {
    /// SetDirectionalLight(0..2, dir, colour, 1) + SetAmbient
    pub lights: VsLights,
    /// the raw columns as read: directions 0..3 (the 4th is read and never used), colours 0..2 + ambient
    pub dirs: [Vec4; 4],
    pub colours: [Vec4; 4],
    /// BuildLightMatrix's argument (the caller's float, passed through)
    pub scale: f32,
    /// AllocPointLights(n) + SetPointLight(i): position xyz, radius in w; colour xyz, w = 1
    pub point_lights: Vec<PointLight>,
    /// TrackFX 0x0014AEC0(i, flare, reflection) per point light
    pub flare_ids: Vec<(i32, i32)>,
    /// the point lights came from `PtLights.bin`
    pub from_bin: bool,
}

/// GameDataLoading.cpp 0x000A0B40 — `\<dir>lights.txt`: eight `( a, b, c, d )` lines, the first four the x / y / z / w
/// rows of directions 0..3, the next four those of colours 0..2 and the ambient; then the point lights, binary first
/// (0x0009DA10), else text (0x0009DE90). `None` when lights.txt does not load (the point lights are not tried).
pub fn load_lights(scale: f32, dir: &str) -> Option<GdlLights> {
    let path = format!("\\{}{}.{}", dir, "lights", "txt");
    let mut f = VsTextFile::default();
    if !f.load(None, &path) {
        return None;
    }
    let mut dirs = [Vec4::ZERO; 4];
    let mut colours = [Vec4::ZERO; 4];
    for k in 0..4 {
        if let Some((_, v)) = f.read_f32n::<4>() {
            for (i, x) in v.iter().enumerate() {
                dirs[i][k] = *x;
            }
        }
    }
    for k in 0..4 {
        if let Some((_, v)) = f.read_f32n::<4>() {
            for (i, x) in v.iter().enumerate() {
                colours[i][k] = *x;
            }
        }
    }
    let mut out = GdlLights { dirs, colours, scale, ..Default::default() };
    for i in 0..3 {
        out.lights.set_directional_light(i, dirs[i], colours[i], true);
    }
    out.lights.set_ambient(colours[3]);
    if !load_point_lights_bin(&mut out, dir, false) {
        load_point_lights_txt(&mut out, dir, false);
    }
    Some(out)
}

fn point_lights_path(dir: &str, foyer: bool, ext: &str) -> String {
    let name = dir.to_ascii_uppercase();
    if foyer {
        format!("\\Foyers\\{}\\{}.{}", name, "PtLights", ext)
    } else {
        format!("\\{}{}.{}", name, "PtLights", ext)
    }
}

/// GameDataLoading.cpp 0x0009DA10 — `PtLights.bin`: u16 count at +12, u16 record offset at +14, 32-byte records
/// (position f32[3], colour f32[3], radius f32, u8 flare +28, u8 reflection +29). `false` when there is no file
/// (no shipped track has one).
pub fn load_point_lights_bin(out: &mut GdlLights, dir: &str, foyer: bool) -> bool {
    let path = point_lights_path(dir, foyer, "bin");
    let Ok(data) = visos::file::platform::read(&path) else {
        return false;
    };
    parse_point_lights_bin(out, &data);
    true
}

/// the record walk of 0x0009DA10
pub fn parse_point_lights_bin(out: &mut GdlLights, data: &[u8]) {
    if data.len() < 16 {
        log::error!("assert: (pfileData!=NULL) && (fileSize>0)");
        return;
    }
    let u16_at = |o: usize| u16::from_le_bytes([data[o], data[o + 1]]) as usize;
    let f32_at = |o: usize| data.get(o..o + 4).map_or(0.0, |b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let base = u16_at(14);
    let n = u16_at(12);
    out.point_lights = Vec::with_capacity(n);
    out.flare_ids = Vec::with_capacity(n);
    for i in 0..n {
        let r = base + 32 * i;
        let pos = Vec3::new(f32_at(r), f32_at(r + 4), f32_at(r + 8));
        let colour = Vec4::new(f32_at(r + 12), f32_at(r + 16), f32_at(r + 20), 1.0);
        let radius = f32_at(r + 24);
        out.flare_ids.push((data.get(r + 28).copied().unwrap_or(0) as i32, data.get(r + 29).copied().unwrap_or(0) as i32));
        out.point_lights.push(PointLight { pos, radius, colour });
    }
    out.from_bin = true;
    log::info!("Point light source loading from binary file - {} lights", n);
}

/// GameDataLoading.cpp 0x0009DE90 — `PtLights.txt`: `( n )`, then per light two quoted names (the second one names
/// the flare / reflection, 0x0009D860), `( x, y, z )` position, `( r, g, b )` colour, `( radius )` (asserted < 100).
/// Position and colour are read into the same two vectors every light, so a short line keeps the last light's values.
pub fn load_point_lights_txt(out: &mut GdlLights, dir: &str, foyer: bool) -> bool {
    let path = point_lights_path(dir, foyer, "txt");
    let mut f = VsTextFile::default();
    if !f.load(None, &path) {
        return false;
    }
    let mut pos = Vec4::ZERO;
    let mut colour = Vec4::new(0.0, 0.0, 0.0, 1.0);
    let n = f.read_s32().map_or(0, |(_, v)| v);
    out.point_lights = Vec::with_capacity(n.max(0) as usize);
    out.flare_ids = Vec::with_capacity(n.max(0) as usize);
    // the name buffer is the path buffer (v14[49]); EOF leaves it as it was
    let mut name = path.clone();
    for _ in 0..n.max(0) {
        if let Some(s) = f.read_string() {
            name = s;
        }
        if let Some(s) = f.read_string() {
            name = s;
        }
        out.flare_ids.push(light_name_ids(&name));
        if let Some((_, v)) = f.read_f32n::<3>() {
            pos = Vec4::new(v[0], v[1], v[2], pos.w);
        }
        if let Some((_, v)) = f.read_f32n::<3>() {
            colour = Vec4::new(v[0], v[1], v[2], colour.w);
        }
        let rad = f.read_f32().map_or(0.0, |(_, v)| v);
        if rad >= 100.0 {
            log::error!("assert: rad < 100.f");
        }
        pos.w = rad;
        out.point_lights.push(PointLight { pos: pos.truncate(), radius: pos.w, colour });
    }
    log::info!("Point light source loading - {} lights", n);
    true
}

/// GameDataLoading.cpp 0x0009D860 — flare / reflection ids from a light name: from the second `_` counting from
/// the end, `_<digits>` gives the flare id, the next `_<digits>` the reflection id (`_W` / `_w` = 99). Both stay 0
/// when absent. At most 256 characters are scanned.
pub fn light_name_ids(name: &str) -> (i32, i32) {
    let b = name.as_bytes();
    let at = |i: usize| b.get(i).copied().unwrap_or(0);
    let (mut flare, mut reflection) = (0, 0);
    let mut i = b.len();
    let mut seen = 0;
    while i > 0 {
        if at(i) == b'_' {
            seen += 1;
            if seen == 2 {
                break;
            }
        }
        i -= 1;
    }
    let (mut state, mut next) = (0, 1);
    let mut j = i;
    while at(j) != 0 && j < 256 {
        let c = at(j);
        if state == 0 {
            if c == b'_' && (at(j + 1).is_ascii_digit() || at(j + 1) == b'W' || at(j + 1) == b'w') {
                state = next;
            }
        } else if state == 1 {
            if c.is_ascii_digit() {
                flare = atoi(&b[j..]);
                state = 0;
                next = 2;
            }
        } else if c.is_ascii_digit() {
            reflection = atoi(&b[j..]);
            return (flare, reflection);
        } else if c == b'w' || c == b'W' {
            reflection = 99;
            return (flare, reflection);
        }
        j += 1;
    }
    (flare, reflection)
}

fn atoi(s: &[u8]) -> i32 {
    let mut v: i32 = 0;
    for &c in s.iter().take_while(|c| c.is_ascii_digit()) {
        v = v.wrapping_mul(10).wrapping_add((c - b'0') as i32);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_ids() {
        assert_eq!(light_name_ids("LGT_Streetlamp26"), (0, 0));
        assert_eq!(light_name_ids("LGT_lamp_3_2"), (3, 2));
        assert_eq!(light_name_ids("LGT_lamp_12_W"), (12, 99));
        assert_eq!(light_name_ids("X_lamp_7"), (7, 0));
        assert_eq!(light_name_ids("Omni02"), (0, 0));
    }

    #[test]
    fn bin_records() {
        let mut d = vec![0u8; 16 + 32];
        d[12] = 1;
        d[14] = 16;
        for (k, v) in [1.0f32, 2.0, 3.0, 0.5, 0.25, 0.125, 9.0].iter().enumerate() {
            d[16 + 4 * k..20 + 4 * k].copy_from_slice(&v.to_le_bytes());
        }
        d[16 + 28] = 4;
        d[16 + 29] = 5;
        let mut o = GdlLights::default();
        parse_point_lights_bin(&mut o, &d);
        assert_eq!(o.point_lights[0], PointLight { pos: Vec3::new(1.0, 2.0, 3.0), radius: 9.0, colour: Vec4::new(0.5, 0.25, 0.125, 1.0) });
        assert_eq!(o.flare_ids, vec![(4, 5)]);
    }

    /// positive control: every shipped track's lights load; the point lights agree with the render crate's reader
    #[test]
    fn every_track_lights() {
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        let mut n = 0;
        for t in ["ExecTest", "LACoast", "LADown", "London", "London2"] {
            let dir = format!("Tracks\\{t}\\");
            let l = load_lights(1.0, &dir).unwrap_or_else(|| panic!("{t}: lights.txt"));
            assert!(!l.from_bin);
            assert_eq!(l.flare_ids.len(), l.point_lights.len());
            let txt = visos::file::platform::read(&format!("\\{dir}PtLights.txt")).unwrap();
            let theirs = visos_render::vs_render::parse_ptlights_txt(&String::from_utf8_lossy(&txt));
            assert_eq!(l.point_lights, theirs, "{t}");
            let raw = visos::file::platform::read(&format!("\\{dir}lights.txt")).unwrap();
            let g = visos_render::vs_render::VsLights::parse_lights_txt(&String::from_utf8_lossy(&raw)).unwrap();
            assert_eq!(l.lights.dir, g.dir, "{t}");
            assert_eq!(l.lights.ambient, g.ambient, "{t}");
            n += l.point_lights.len();
        }
        assert!(n > 0);
        let mut f = GdlLights::default();
        assert!(load_point_lights_txt(&mut f, "hangout", true));
        assert!(!f.point_lights.is_empty());
    }
}
