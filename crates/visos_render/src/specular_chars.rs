//! Render\SpecularChars.cpp — the Characters.pxu specular pass of skinned characters (object at Characters.cpp's
//! `unk_6D6CC8`, material input resolver `dword_6D6D58` vtable 0x0062B0D8). Per character index (`MAX_SPEC_CHARS`
//! 30) a 172-byte record: light rotation / elevation (degrees), colour, intensity, bloom and six materials (fabric,
//! skin, leather, plastic, metal, hair). Primitives whose shader name has "spec_" + a material name become kind
//! 8 + material (0x00143540). Characters.cpp 0x00079EC0 (front end) / 0x0007A460 (game) then draw: the base pass
//! every other primitive with SpecularLightColour (0, 0, 0, bloom), then per material in use its primitives with the
//! spec light colour; the vertex program maps the world normal through the reflection matrix (rows = the light/view
//! half vector and an up vector) into the spec texture: uv = 0.5 · (n · row)³.
//!
//! Settings: front end `\foyers\hangout\SpecChar.bin` (30 records), in game `\chars\FGlit<n>.bin` (not shipped) then
//! `\chars\FGlit.bin` (not shipped) then built-in defaults (5 records + the 72-byte diffuse light sets the game
//! lights its characters with, 0x00144BC0).

use bevy::prelude::*;

use crate::renderer::renderer_types::Model;
use crate::vs_render::VsLights;

/// `MAX_SPEC_CHARS`
pub const MAX_SPEC_CHARS: usize = 30;
/// `NUM_SPEC_MATERIALS`
pub const NUM_SPEC_MATERIALS: usize = 6;
/// `MAX_SPEC_MATERIALS_PER_CHAR`
pub const MAX_SPEC_MATERIALS_PER_CHAR: usize = 10;
/// first primitive kind of the spec materials (0x00143540: kind = material + 8)
pub const SPEC_KIND_BASE: u32 = 8;
/// `dword_6871AC`: no specular beyond this camera distance; the colour fades to 0 towards it
pub const SPEC_MAX_DISTANCE: f32 = 9.0;
/// `byte_6871A4` "Frontend.CharacterSpecular.Enabled"
pub const SPEC_ENABLED: bool = true;
/// `dword_687248`: blend mode of a glossy spec primitive while the pass is active
pub const SPEC_GLOSS_BLEND: u8 = 1;
/// 0x00143A40: distance used without a position (`0x3C23D70A`)
pub const SPEC_NO_POSITION_DISTANCE: f32 = 0.01;
/// `'10PS'`: settings without diffuse light sets (front end)
pub const SPEC_MAGIC: u32 = 0x5350_3031;
/// `'20PS'`: settings followed by one 72-byte diffuse light set per record (game)
pub const SPEC_MAGIC_DIFFUSE: u32 = 0x5350_3032;
/// `off_6871B0` + `off_6871B4`
pub const FRONT_END_SETTINGS: &str = "\\foyers\\hangout\\SpecChar.bin";

/// one 24-byte material record (+0 name, +4 power, +8 intensity, +12 texture, +16 kind, +20 uv stretch)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecMaterial {
    /// read by nothing on xbox (0x00143F80 ignores its power argument)
    pub power: f32,
    pub intensity: f32,
    /// index into the spec textures (front end only)
    pub texture: i32,
    /// +16: the primitive kind, written when the material is listed (0x00143740)
    pub kind: u32,
    /// read by nothing on xbox (0x00143F80 ignores it)
    pub uv_stretch: f32,
}

/// `off_6871B8`: material names and defaults (6 × 24 B)
pub const MATERIAL_NAMES: [&str; NUM_SPEC_MATERIALS] = ["fabric", "skin", "leather", "plastic", "metal", "hair"];
pub const DEFAULT_MATERIALS: [SpecMaterial; NUM_SPEC_MATERIALS] = [
    SpecMaterial { power: 0.7, intensity: 0.25, texture: 0, kind: 0, uv_stretch: 0.5 },
    SpecMaterial { power: 1.0, intensity: 0.5, texture: 0, kind: 0, uv_stretch: 0.5 },
    SpecMaterial { power: 1.5, intensity: 0.75, texture: 0, kind: 0, uv_stretch: 0.5 },
    SpecMaterial { power: 2.0, intensity: 1.0, texture: 0, kind: 0, uv_stretch: 0.5 },
    SpecMaterial { power: 1.0, intensity: 1.0, texture: 0, kind: 0, uv_stretch: 0.5 },
    SpecMaterial { power: 1.0, intensity: 0.5, texture: 0, kind: 0, uv_stretch: 0.5 },
];

/// one 172-byte character record
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecCharRecord {
    /// +0 light rotation (degrees)
    pub rotation: f32,
    /// +4 light elevation (degrees)
    pub elevation: f32,
    /// +8 rgb, +20 intensity
    pub colour: Vec3,
    pub intensity: f32,
    /// +24 SpecularLightColour alpha (the bloom amount)
    pub bloom: f32,
    /// +28 six materials
    pub materials: [SpecMaterial; NUM_SPEC_MATERIALS],
}

impl Default for SpecCharRecord {
    /// SpecularChars.cpp 0x001442F0 defaults: rotation 0, elevation 45, white, intensity 1, bloom 0.2
    fn default() -> Self {
        Self {
            rotation: 0.0,
            elevation: f32::from_bits(0x4234_0000),
            colour: Vec3::ONE,
            intensity: 1.0,
            bloom: f32::from_bits(0x3E4C_CCCD),
            materials: DEFAULT_MATERIALS,
        }
    }
}

/// one 72-byte diffuse light set (game): +0 three rgb colours, +36 ambient rgb, +48 three rotations, +60 three
/// elevations (degrees)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecDiffuse {
    pub colours: [Vec3; 3],
    pub ambient: Vec3,
    pub rotations: [f32; 3],
    pub elevations: [f32; 3],
}

impl Default for SpecDiffuse {
    /// 0x001442F0 defaults: colours 0.5, ambient 0.15, rotations 0/120/240, elevation 30
    fn default() -> Self {
        Self {
            colours: [Vec3::splat(0.5); 3],
            ambient: Vec3::splat(f32::from_bits(0x3E19_999A)),
            rotations: [0.0, f32::from_bits(0x42F0_0000), f32::from_bits(0x4370_0000)],
            elevations: [f32::from_bits(0x41F0_0000); 3],
        }
    }
}

/// the loaded settings (`dword_736534`: +0 magic, +4 version 6, +8 count, +12 offset of the records)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpecSettings {
    pub records: Vec<SpecCharRecord>,
    /// `dword_736538`: one per record, only with `SPEC_MAGIC_DIFFUSE`
    pub diffuse: Vec<SpecDiffuse>,
}

fn f32_at(d: &[u8], o: usize) -> Option<f32> {
    Some(f32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}

fn u32_at(d: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?))
}

impl SpecSettings {
    /// SpecularChars.cpp 0x001442F0, file branch. `None` when the count exceeds 30 (the original asserts and turns
    /// the pass off) or the file is short.
    pub fn parse(d: &[u8]) -> Option<Self> {
        let magic = u32_at(d, 0)?;
        let count = u32_at(d, 8)? as usize;
        if count > MAX_SPEC_CHARS {
            return None;
        }
        let base = u32_at(d, 12)? as usize;
        let mut records = Vec::with_capacity(count);
        for i in 0..count {
            let r = base + 172 * i;
            let mut materials = DEFAULT_MATERIALS;
            for (k, m) in materials.iter_mut().enumerate() {
                let o = r + 28 + 24 * k;
                *m = SpecMaterial {
                    power: f32_at(d, o + 4)?,
                    intensity: f32_at(d, o + 8)?,
                    texture: u32_at(d, o + 12)? as i32,
                    kind: u32_at(d, o + 16)?,
                    uv_stretch: f32_at(d, o + 20)?,
                };
            }
            records.push(SpecCharRecord {
                rotation: f32_at(d, r)?,
                elevation: f32_at(d, r + 4)?,
                colour: Vec3::new(f32_at(d, r + 8)?, f32_at(d, r + 12)?, f32_at(d, r + 16)?),
                intensity: f32_at(d, r + 20)?,
                bloom: f32_at(d, r + 24)?,
                materials,
            });
        }
        let mut diffuse = Vec::new();
        if magic == SPEC_MAGIC_DIFFUSE {
            let b = base + 172 * count;
            for i in 0..count {
                let o = b + 72 * i;
                let v = |k: usize| -> Option<Vec3> { Some(Vec3::new(f32_at(d, o + k)?, f32_at(d, o + k + 4)?, f32_at(d, o + k + 8)?)) };
                diffuse.push(SpecDiffuse {
                    colours: [v(0)?, v(12)?, v(24)?],
                    ambient: v(36)?,
                    rotations: [f32_at(d, o + 48)?, f32_at(d, o + 52)?, f32_at(d, o + 56)?],
                    elevations: [f32_at(d, o + 60)?, f32_at(d, o + 64)?, f32_at(d, o + 68)?],
                });
            }
        }
        Some(Self { records, diffuse })
    }

    /// 0x001442F0 without a file: 30 records (front end) or 5 + diffuse sets (game, `byte_736530`)
    pub fn defaults(in_game: bool) -> Self {
        let n = if in_game { 5 } else { MAX_SPEC_CHARS };
        Self {
            records: vec![SpecCharRecord::default(); n],
            diffuse: if in_game { vec![SpecDiffuse::default(); n] } else { Vec::new() },
        }
    }

    /// 0x001442F0: front end `SpecChar.bin`; game `\chars\FGlit<n>.bin` (`byte_6F7130`), then `\chars\FGlit.bin`,
    /// then the defaults. `None` = the pass is turned off (bad count).
    pub fn load(front_end: bool, fglit: u8) -> Option<Self> {
        let first = if front_end { FRONT_END_SETTINGS.to_string() } else { format!("\\chars\\FGlit{fglit}.bin") };
        let data = visos::file::platform::read(&first).ok().or_else(|| {
            log::info!("failed to load {first} , loading default settings");
            visos::file::platform::read("\\chars\\FGlit.bin").ok()
        });
        match data {
            Some(d) => Self::parse(&d),
            None => Some(Self::defaults(!front_end)),
        }
    }
}

/// SpecularChars.cpp 0x00143540 — material of a primitive: shader name containing "spec_" (case sensitive) and, lower
/// cased, a material name, for primitives of kind 0 or 2 (the kind becomes 8 + material)
pub fn spec_kind(shader_name: &str, kind: u32) -> u32 {
    if !shader_name.contains("spec_") {
        return kind;
    }
    let lower = shader_name.to_ascii_lowercase();
    for (i, m) in MATERIAL_NAMES.iter().enumerate() {
        if lower.contains(m) && (kind == 0 || kind == 2) {
            return SPEC_KIND_BASE + i as u32;
        }
    }
    kind
}

/// 0x00143540 over a whole model: the new kind of every primitive, in object/part/primitive order
pub fn classify_model(m: &Model) -> Vec<u32> {
    let mut out = Vec::new();
    for obj in &m.objects {
        for part in &obj.parts {
            for prim in &part.primitives {
                out.push(spec_kind(&m.shaders[prim.shader].name, prim.kind));
            }
        }
    }
    out
}

/// VsCameraManager 0x000613D0: degrees to radians
fn rad(d: f32) -> f32 {
    d * 0.017_453_292
}

/// VsVector4::Normalise: zero below length 1e-6
fn normalise(v: Vec3) -> Vec3 {
    let l = v.length();
    if l <= 1e-6 { Vec3::ZERO } else { v / l }
}

/// SpecularChars.cpp 0x00143A40 (light part): the record's light direction (`dword_7365A4`) and colour
/// (`byte_7365B4`: rgb · intensity · (9 − d)/9, w = bloom)
pub fn spec_light(rec: &SpecCharRecord, dist: f32) -> (Vec3, Vec4) {
    let el = rad(rec.elevation);
    let rot = rad(rec.rotation);
    let c = el.cos();
    let dir = normalise(Vec3::new(rot.sin() * c, el.sin(), rot.cos() * c));
    let col = rec.colour * rec.intensity * ((SPEC_MAX_DISTANCE - dist) / SPEC_MAX_DISTANCE);
    (dir, col.extend(rec.bloom))
}

/// SpecularChars.cpp 0x00143F80 — reflection matrix rows 0/1 (`unk_6D6D5C` after the caller's transpose: row 0 =
/// the half vector H of the light and the direction to the camera, row 1 = an up vector with its y negated) and the
/// returned H · V. The world matrix, power and uv stretch arguments are unused.
pub fn reflection(light_dir: Vec3, camera: Vec3, pos: Vec3) -> ([Vec3; 2], f32) {
    let v = normalise(camera - pos);
    let h = normalise(light_dir + v);
    let d = h.dot(v);
    let up = if h.x == 0.0 && h.z == 0.0 { Vec3::new(-h.y, 0.0, 0.0) } else { Vec3::Y };
    let side = normalise(up.cross(h));
    let mut u = side.cross(h);
    u.y = 0.0 - u.y;
    ([h, normalise(u)], d)
}

/// BoneBlend4.vxu oT1: the world normal (unnormalised) through rows 0/1 of the reflection matrix, cubed, × 0.5
pub fn spec_uv(n: Vec3, rows: &[Vec3; 2]) -> [f32; 2] {
    let x = n.dot(rows[0]);
    let y = n.dot(rows[1]);
    [x * x * x * 0.5, y * y * y * 0.5]
}

/// SpecularChars.cpp 0x00144C10 — the game's character light set from a diffuse record: direction (sin r · cos e,
/// sin e, cos r · cos e) with e the elevation (the original negates it twice), colours, count 3, ambient
pub fn diffuse_light_set(d: &SpecDiffuse) -> VsLights {
    let mut l = VsLights::default();
    for i in 0..3 {
        let e = rad(0.0 - d.elevations[i]);
        let r = rad(d.rotations[i]);
        let c = e.cos();
        l.dir[i] = Vec3::new(r.sin() * c, 0.0 - e.sin(), r.cos() * c);
        l.colour[i] = d.colours[i].clamp(Vec3::ZERO, Vec3::ONE).extend(1.0);
    }
    l.ambient = d.ambient.clamp(Vec3::ZERO, Vec3::ONE).extend(1.0);
    l
}

/// SpecularChars.cpp 0x00143740 — the materials listed for one character draw (by first appearance, at most 10)
/// and the blend each primitive's stage gets: glossy (stage +0x0C bit 0x10) spec primitives blend 1 while active,
/// 0 while not; the others keep theirs. `prims` = (kind, glossy).
pub fn spec_list(prims: &[(u32, bool)], active: bool) -> (Vec<u32>, Vec<Option<u8>>) {
    let mut list = Vec::new();
    let mut mask = 0u32;
    let mut blends = Vec::with_capacity(prims.len());
    for &(kind, gloss) in prims {
        let mut blend = None;
        if kind >= SPEC_KIND_BASE {
            if active {
                if gloss {
                    blend = Some(SPEC_GLOSS_BLEND);
                }
                let m = kind - SPEC_KIND_BASE;
                if mask & (1 << m) == 0 && list.len() < MAX_SPEC_MATERIALS_PER_CHAR {
                    list.push(kind);
                    mask |= 1 << m;
                }
            } else if gloss {
                blend = Some(0);
            }
        }
        blends.push(blend);
    }
    (list, blends)
}

/// what one primitive draws with this frame (Characters.pxu inputs, resolver 0x00079A10)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecDraw {
    /// c10 SpecularLightColour (clamped as a combiner constant)
    pub colour: Vec4,
    /// SpecularTexture: index into `dword_736478`
    pub texture: usize,
    /// blend override (0x002A2BE0 on the stage's blend record)
    pub blend: Option<u8>,
}

/// SpecularChars.cpp 0x00143A40 + 0x00143C70 for one character: per primitive the colour/texture it is drawn with.
/// `dist` = camera distance to the character's root bone (+ sphere radius up), `hv` = H · V of 0x00143F80.
pub fn character_draws(rec: &SpecCharRecord, prims: &[(u32, bool)], dist: f32, hv: f32, front_end: bool, enabled: bool) -> Vec<SpecDraw> {
    let mut active = enabled && dist <= SPEC_MAX_DISTANCE;
    let (list, blends) = spec_list(prims, active);
    if list.is_empty() {
        active = false;
    }
    let (_, light) = spec_light(rec, dist);
    // resolver 0x00079A10: rgb × min(2 · intensity of material `index`, 1), alpha = the record's bloom; base pass
    // (0xFFFF) rgb 0 and texture 0. the index is the list position, not the material (original)
    let resolve = |index: Option<usize>, c: Vec4| -> (Vec4, usize) {
        let mat = index.map(|i| rec.materials[i]);
        let k = mat.map(|m| (m.intensity * 2.0).min(1.0)).unwrap_or(0.0);
        let tex = match mat {
            Some(m) if front_end => m.texture.max(0) as usize,
            _ => 0,
        };
        ((c.truncate() * k).extend(rec.bloom).clamp(Vec4::ZERO, Vec4::ONE), tex)
    };
    prims
        .iter()
        .zip(blends)
        .map(|(&(kind, _), blend)| {
            let pos = if active { list.iter().position(|&k| k == kind) } else { None };
            let (colour, texture) = match pos {
                Some(i) => {
                    let m = rec.materials[(kind - SPEC_KIND_BASE) as usize];
                    resolve(Some(i), light * hv * m.intensity)
                }
                None => resolve(None, light),
            };
            SpecDraw { colour, texture, blend }
        })
        .collect()
}

/// the spec textures (`dword_736478`): [0] "Spec2", front end [1..11] spec_t_gk_001..011, game [1] spec_t_gk_005,
/// [2] spec_t_gk_011 (0x00143140)
pub fn texture_names(front_end: bool) -> Vec<String> {
    let mut v = vec!["spec2".to_string()];
    if front_end {
        for i in 1..12 {
            v.push(format!("spec_t_gk_{i:03}"));
        }
    } else {
        v.push("spec_t_gk_005".into());
        v.push("spec_t_gk_011".into());
    }
    v
}

/// the SpecularChars object: settings + spec textures. Made by CarDamageFX 0x00079C30 → 0x00143140 from
/// GameManager::CommonInit (arg = front end); here on the first skinned model: the `Chars` pack = front end,
/// `FlagGirl` = game.
#[derive(Resource)]
pub struct SpecularChars {
    pub front_end: bool,
    /// `None` = turned off by a bad settings file
    pub settings: Option<SpecSettings>,
    /// `dword_736478` (wrap addressing: CharacterSpecular pass 1, 0x002A1BE0(1, 1))
    pub textures: Vec<Option<Handle<Image>>>,
    /// `byte_6871A4` ("Enabled" tweak; `VELOCITY_SPEC_CHARS=0` turns it off for A/B checks)
    pub enabled: bool,
}

impl SpecularChars {
    /// SpecularChars.cpp 0x00143140: textures by name from the loaded packs (`\VFX\misc.vcd` and the character
    /// pack's textures), then the settings (0x001442F0)
    pub fn init(front_end: bool, vcds: &[&str], images: &mut Assets<Image>) -> Self {
        let names = texture_names(front_end);
        let mut tex = crate::model_builder::TexSet::default();
        for p in vcds {
            let Ok(v) = visos::file::Vcd::open(p) else { continue };
            for e in v.entries() {
                let stem = e.name.to_ascii_lowercase();
                let stem = stem.trim_end_matches(".vtf");
                if names.iter().any(|n| n == stem) && !tex.by_name.contains_key(stem) {
                    if let Some(t) = v.data(&e).and_then(|d| crate::renderer::texture_manager::decode_any_vtf(d).ok()) {
                        tex.add(e.name, t);
                    }
                }
            }
        }
        let textures: Vec<Option<Handle<Image>>> = names
            .iter()
            .map(|n| {
                let h = tex.image(n, 1, 1, images);
                if h.is_none() {
                    log::warn!("spec texture {n} not found");
                }
                h
            })
            .collect();
        let settings = SpecSettings::load(front_end, FGLIT_INDEX);
        if settings.is_none() {
            log::warn!("SpecularChars: settings refused, pass off");
        }
        let enabled = SPEC_ENABLED && std::env::var("VELOCITY_SPEC_CHARS").map_or(true, |v| v != "0");
        Self { front_end, settings, textures, enabled }
    }

    /// texture `i` of `dword_736478` (0 = none, as an unset slot)
    pub fn texture(&self, i: usize) -> Option<Handle<Image>> {
        self.textures.get(i).cloned().flatten()
    }
}

/// `byte_6F7130`: the FGlit file number (bss, written by nothing found)
pub const FGLIT_INDEX: u8 = 0;

/// optional on a skinned model root: its character index into the settings (Characters.cpp 0x00079EC0 a17 = FE
/// character element +764, 0x0007A460 a11 = the flag girl's index); 0 without it
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SpecCharIndex(pub u8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hangout_settings_parse() {
        let d = visos::file::platform::read(FRONT_END_SETTINGS).unwrap();
        let s = SpecSettings::parse(&d).unwrap();
        assert_eq!(s.records.len(), 30);
        assert!(s.diffuse.is_empty());
        let r = &s.records[0];
        assert_eq!(r.rotation, 360.0);
        assert_eq!(r.elevation, 33.0);
        assert!(r.materials.iter().all(|m| m.texture >= 0 && m.texture < 12));
        // in game no FGlit file ships: defaults with diffuse sets
        let g = SpecSettings::load(false, 0).unwrap();
        assert_eq!(g, SpecSettings::defaults(true));
        assert_eq!(g.diffuse.len(), 5);
    }

    #[test]
    fn light_and_reflection() {
        let rec = SpecCharRecord::default();
        let (dir, col) = spec_light(&rec, 0.0);
        assert!((dir - Vec3::new(0.0, 0.70710677, 0.70710677)).length() < 1e-5);
        assert_eq!(col, Vec4::new(1.0, 1.0, 1.0, rec.bloom));
        let (_, half) = spec_light(&rec, 4.5);
        assert!((half.x - 0.5).abs() < 1e-6);
        // camera straight along the light: H = L, H · V = 1
        let ([h, u], hv) = reflection(dir, dir * 5.0, Vec3::ZERO);
        assert!((h - dir).length() < 1e-5 && (hv - 1.0).abs() < 1e-5);
        // up' = side × H = (0, −0.707, 0.707), its y negated: H again (the flip breaks the basis, original)
        assert!((u - h).length() < 1e-5);
        let uv = spec_uv(h * 2.0, &[h, u]);
        assert!((uv[0] - 4.0).abs() < 1e-4 && (uv[1] - 4.0).abs() < 1e-4);
    }

    #[test]
    fn list_blends_and_draws() {
        let prims = [(0, false), (9, true), (13, false), (9, true)];
        let (list, blends) = spec_list(&prims, true);
        assert_eq!(list, vec![9, 13]);
        assert_eq!(blends, vec![None, Some(1), None, Some(1)]);
        let (_, off) = spec_list(&prims, false);
        assert_eq!(off, vec![None, Some(0), None, Some(0)]);
        let rec = SpecCharRecord::default();
        let d = character_draws(&rec, &prims, 0.0, 1.0, false, true);
        // base pass: colour 0, bloom alpha
        assert_eq!(d[0].colour, Vec4::new(0.0, 0.0, 0.0, rec.bloom));
        // skin listed first: light 1 · H·V 1 · skin 0.5 · min(2 · fabric 0.25, 1) (list index 0 = fabric)
        assert!((d[1].colour.x - 0.25).abs() < 1e-6);
        // hair second: 0.5 · min(2 · skin 0.5, 1)
        assert!((d[2].colour.x - 0.5).abs() < 1e-6);
        // beyond 9 m: everything in the base pass, glossy prims blend 0
        let far = character_draws(&rec, &prims, 10.0, 1.0, false, true);
        assert_eq!(character_draws(&rec, &prims, 0.0, 1.0, false, false), far);
        assert!(far.iter().all(|p| p.colour.truncate() == Vec3::ZERO));
        assert_eq!(far[1].blend, Some(0));
    }

    #[test]
    fn default_diffuse_set() {
        let l = diffuse_light_set(&SpecDiffuse::default());
        assert!((l.dir[0] - Vec3::new(0.0, 0.5, 0.8660254)).length() < 1e-5);
        assert!((l.ambient.x - 0.15).abs() < 1e-6);
    }

    /// census of the skinned primitives: shader name, kind, stage +0x0C / +0x08, command codes
    #[test]
    #[ignore]
    fn spec_census() {
        let mut s = String::new();
        for p in [r"\Chars\CHARS_M.VCD", r"\FlagGirl\FG_M.vcd"] {
            let v = visos::file::Vcd::open(p).unwrap();
            for e in v.entries() {
                if !e.name.to_ascii_lowercase().ends_with(".xmf") {
                    continue;
                }
                let d = v.data(&e).unwrap().to_vec();
                let m = Model::parse(&d).unwrap();
                s += &format!("== {p} {}\n", e.name);
                for obj in &m.objects {
                    for part in &obj.parts {
                        for prim in &part.primitives {
                            let sh = &m.shaders[prim.shader];
                            let st = &sh.stages[0];
                            let mut codes = Vec::new();
                            let mut i = 0;
                            while i + 1 < st.commands.len() && st.commands[i] != 0 {
                                let n = st.commands[i + 1].max(2) as usize;
                                codes.push(format!("{}:{:02X?}", st.commands[i], &st.commands[i + 2..(i + n).min(st.commands.len())]));
                                i += n;
                            }
                            s += &format!(
                                "  {:<40} kind {} -> {} st+8 {:X} st+C {:X} tex {} [{}]\n",
                                sh.name,
                                prim.kind,
                                spec_kind(&sh.name, prim.kind),
                                st.flags,
                                st.unk0c,
                                st.texture_name,
                                codes.join(" ")
                            );
                        }
                    }
                }
            }
        }
        for p in [r"\VFX\misc.vcd", r"\Chars\CHARS_T.VCD", r"\FlagGirl\FG_T.vcd"] {
            if let Ok(v) = visos::file::Vcd::open(p) {
                for e in v.entries() {
                    let n = e.name.to_ascii_lowercase();
                    if n.starts_with("spec") {
                        s += &format!("tex {p} {}\n", e.name);
                    }
                }
            }
        }
        std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/gfx/spec_census.txt"), s).unwrap();
    }
}
