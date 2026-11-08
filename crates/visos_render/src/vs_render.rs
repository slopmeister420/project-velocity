//! src\Core\VsRender.cpp — `VsRender`: the scene's directional light set (three lights + ambient) and the light
//! matrix the vertex shaders consume.

use bevy::math::{Vec3, Vec4};

/// three directional lights and an ambient colour.
#[derive(Clone, Copy, Debug)]
pub struct VsLights {
    /// +0x04 + 16*i light direction (normalised)
    pub dir: [Vec3; 3],
    /// +0x34 + 16*i light colour
    pub colour: [Vec4; 3],
    /// +0x64 ambient colour
    pub ambient: Vec4,
}

impl Default for VsLights {
    fn default() -> Self {
        Self { dir: [Vec3::NEG_Y; 3], colour: [Vec4::ZERO; 3], ambient: Vec4::ONE }
    }
}

/// `VsVector4::Normalise` 0x000135B0: xyz divided by the xyz length, left as is at length <= 1e-6
fn vs_normalise(v: Vec3) -> Vec3 {
    let l = v.length();
    if l.abs() <= 1e-6 { v } else { Vec3::new(v.x / l, v.y / l, v.z / l) }
}

impl VsLights {
    /// VsRender.cpp 0x000583F0 — the set VsRender starts with (constructor 0x00058140) and the one a foyer without
    /// a lights.txt keeps (FECK2Element3D::LoadFoyerLights 0x00298CD0): sources (−6, 1, 3), (0, 1, 0), (3, 1, −6)
    /// normalised, directions = their negation normalised again, colours 0.2 / 0.3 / 0.1 grey, ambient 0.4 grey
    pub fn vs_render_defaults() -> Self {
        let src = [Vec3::new(-6.0, 1.0, 3.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(3.0, 1.0, -6.0)];
        let grey = |bits: u32| Vec4::new(f32::from_bits(bits), f32::from_bits(bits), f32::from_bits(bits), 1.0);
        Self {
            dir: src.map(|s| vs_normalise(-vs_normalise(s))),
            colour: [grey(0x3E4C_CCCD), grey(0x3E99_999A), grey(0x3DCC_CCCD)],
            ambient: grey(0x3ECC_CCCD),
        }
    }

    /// VsRender.cpp 0x00058A10 — `SetDirectionalLight(index, dir, colour, normalise)`
    pub fn set_directional_light(&mut self, index: usize, dir: Vec4, colour: Vec4, normalise: bool) {
        if index < 3 {
            let d = dir.truncate();
            self.dir[index] = if normalise { d.normalize_or_zero() } else { d };
            self.colour[index] = colour;
        }
    }

    /// VsRender.cpp 0x00058AC0
    pub fn set_ambient(&mut self, colour: Vec4) {
        self.ambient = colour;
    }

    /// `lights.txt`: eight `( x, y, z, w )` rows, read column-wise (FECK2Element3D.cpp 0x00298CD0 loader):
    /// rows 0..3 are the x/y/z/w components of directions 0..3, rows 4..7 the r/g/b/a components of colours
    /// 0..2 and the ambient colour. direction 3 is read and unused.
    pub fn parse_lights_txt(text: &str) -> Option<Self> {
        let rows: Vec<[f32; 4]> = text
            .lines()
            .filter_map(|l| {
                let i = l.find('(')?;
                let j = l[i..].find(')')? + i;
                let v: Vec<f32> = l[i + 1..j].split(',').filter_map(|x| x.trim().parse().ok()).collect();
                (v.len() == 4).then(|| [v[0], v[1], v[2], v[3]])
            })
            .collect();
        if rows.len() < 8 {
            return None;
        }
        let col = |base: usize, k: usize| Vec4::new(rows[base][k], rows[base + 1][k], rows[base + 2][k], rows[base + 3][k]);
        let mut l = Self::default();
        for i in 0..3 {
            l.set_directional_light(i, col(0, i), col(4, i), true);
        }
        l.set_ambient(col(4, 3));
        Some(l)
    }

    /// VsRender.cpp 0x00058AF0 builds the matrix of negated directions; lighting a normal with it gives
    /// `ambient + sum(max(0, n . -dir_i) * colour_i)`, saturated like an `oD0` output.
    pub fn light_vertex(&self, n: Vec3) -> Vec4 {
        let mut c = self.ambient.truncate();
        for i in 0..3 {
            c += self.colour[i].truncate() * n.dot(-self.dir[i]).max(0.0);
        }
        c.clamp(Vec3::ZERO, Vec3::ONE).extend(1.0)
    }
}

/// point light record (32 bytes): position with the radius in w, colour with w = 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointLight {
    pub pos: Vec3,
    pub radius: f32,
    pub colour: Vec4,
}

/// `PtLights.txt` (GameDataLoading.cpp 0x0009DE90): `Point lights: ( n )`, then per light two quoted names,
/// `Pos: ( x, y, z )`, `Col: ( r, g, b )`, `Radius: ( r )`.
pub fn parse_ptlights_txt(text: &str) -> Vec<PointLight> {
    let toks = crate::fog_render::setup_tokens(text);
    let mut vals = toks.iter().filter_map(|t| match t {
        crate::fog_render::SetupToken::Values(v) => Some(v.clone()),
        _ => None,
    });
    let n = vals.next().and_then(|v| v.first().copied()).unwrap_or(0.0) as usize;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let (Some(p), Some(c), Some(r)) = (vals.next(), vals.next(), vals.next()) else { break };
        let g = |v: &Vec<f32>, i: usize| v.get(i).copied().unwrap_or(0.0);
        out.push(PointLight {
            pos: Vec3::new(g(&p, 0), g(&p, 1), g(&p, 2)),
            radius: g(&r, 0),
            colour: Vec4::new(g(&c, 0), g(&c, 1), g(&c, 2), 1.0),
        });
    }
    out
}

/// `dword_629238`: share of the smallest overlap taken off both lights of a triple (0x0005A040)
pub const LIGHT_OVERLAP_REDUCE: f32 = f32::from_bits(0x3F02_8F5C);
/// `MAX_OVERLAPPING_LIGHTS`
pub const MAX_OVERLAPPING_LIGHTS: usize = 0x100;

/// VsRender.cpp 0x0005B200 — squared distance of two lights in x, colour w (the ground height TrackFX 0x0014DEC0
/// stores there) and z
fn light_dist2(a: &PointLight, b: &PointLight) -> f32 {
    let dx = a.pos.x - b.pos.x;
    let dy = a.colour.w - b.colour.w;
    let dz = a.pos.z - b.pos.z;
    dx * dx + dy * dy + dz * dz
}

/// VsRender.cpp 0x00059FB0 — r_a + r_b − distance
fn light_overlap(l: &[PointLight], a: usize, b: usize) -> f32 {
    let d = light_dist2(&l[a], &l[b]).sqrt();
    l[a].radius + l[b].radius - d
}

/// VsRender.cpp 0x0005A3B0 — the first three mutually overlapping lights around light `i`: records (light, other,
/// overlap) for (i, ja), (i, jb), (ja, jb); empty when there is none
fn overlapping_triple(l: &[PointLight], i: usize) -> Vec<(usize, usize, f32)> {
    let ri = l[i].radius;
    if ri <= 0.0 {
        return Vec::new();
    }
    let mut rec: Vec<(usize, usize, f32)> = Vec::new();
    for j in 0..l.len() {
        if j == i {
            continue;
        }
        let rj = l[j].radius;
        if rj > 0.0 {
            let d2 = light_dist2(&l[i], &l[j]);
            let sum = ri + rj;
            if sum * sum > d2 {
                let ov = sum - d2.sqrt();
                if ov < 0.0 {
                    log::warn!("ftOverlap >= 0.f");
                }
                rec.push((i, j, ov));
                if rec.len() >= MAX_OVERLAPPING_LIGHTS {
                    log::warn!("nOLights < MAX_OVERLAPPING_LIGHTS");
                }
            }
        }
    }
    for a in 0..rec.len().saturating_sub(1) {
        for b in a + 1..rec.len() {
            let (ja, jb) = (rec[a].1, rec[b].1);
            let ab = light_overlap(l, ja, jb);
            if ab > 0.0 {
                return vec![(i, ja, light_overlap(l, i, ja)), (i, jb, light_overlap(l, i, jb)), (ja, jb, ab)];
            }
        }
    }
    Vec::new()
}

/// VsRender.cpp 0x0005A040 (TrackFX 0x00149020 in game, after the ground heights) — no spot may be lit by more
/// than two point lights (MAX_PT_LIGHT_SOURCES): while light i is in a triple of mutually overlapping lights, the
/// pair with the smallest overlap (sort 0x0005B2E0, predicate 0x0005B780 ascending) both lose 0.51 of it
pub fn reduce_light_overlaps(l: &mut [PointLight]) {
    for i in 0..l.len() {
        loop {
            let mut rec = overlapping_triple(l, i);
            if rec.is_empty() {
                break;
            }
            rec.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
            let reduce = rec[0].2 * LIGHT_OVERLAP_REDUCE;
            if reduce < 0.0 {
                log::warn!("ftReduce >= 0.f");
            }
            let (a, b) = (rec[0].0, rec[0].1);
            l[a].radius -= reduce;
            l[b].radius -= reduce;
        }
    }
    for i in 0..l.len() {
        if overlapping_triple(l, i).len() >= 2 {
            log::warn!("nOLights < MAX_PT_LIGHT_SOURCES");
        }
    }
}

#[cfg(test)]
mod overlap_tests {
    use super::*;

    fn light(x: f32, ground: f32, r: f32) -> PointLight {
        PointLight { pos: Vec3::new(x, 50.0, 0.0), radius: r, colour: Vec4::new(1.0, 1.0, 1.0, ground) }
    }

    /// three lights 4 apart with radius 3: the pair with the smallest overlap shrinks until no triple is left;
    /// the height comes from colour w, not the light's y
    #[test]
    fn reduce_overlaps() {
        let mut l = [light(0.0, 0.0, 3.0), light(4.0, 0.0, 3.0), light(8.0, 0.0, 3.5)];
        assert_eq!(overlapping_triple(&l, 0).len(), 0, "0 and 2 do not overlap");
        let mut l2 = [light(0.0, 0.0, 3.0), light(4.0, 0.0, 3.0), light(2.0, 0.0, 3.0)];
        assert_eq!(overlapping_triple(&l2, 0).len(), 3);
        reduce_light_overlaps(&mut l2);
        assert!((0..3).all(|i| overlapping_triple(&l2, i).is_empty()));
        // smallest overlap = 0/1 (2): both lose 1.02
        assert!((l2[0].radius - (3.0 - 2.0 * LIGHT_OVERLAP_REDUCE)).abs() < 1e-5, "{:?}", l2);
        assert!((l2[2].radius - 3.0).abs() < 1e-6);
        // far apart in colour w: no overlap at all
        l[2] = light(2.0, 100.0, 3.0);
        reduce_light_overlaps(&mut l);
        assert_eq!(l[0].radius, 3.0);
    }
}

/// `dword_685064`: point light attenuation scale
pub const POINT_LIGHT_ATTEN_SCALE: f32 = 2.0;

/// `flt_6866A0`: in-game scale of a car's two point light colours (CarRender.cpp 0x00119EF0)
pub const CAR_POINT_LIGHT_SCALE: f32 = 0.25;
/// `flt_686000`: the 3d hud's scale of the player car's two point light colours (OSD3dInterface.cpp 0x000EA2B0)
pub const HUD_POINT_LIGHT_SCALE: f32 = 1.5;

/// `flt_706534` "Brightness" (CarPalette.cpp 0x0010E3F0 tweak): CarPalette 0x0010D7E0 stores `dword_686668` in game,
/// `dword_68666C` in the front end, both 1.0
pub const CAR_BRIGHTNESS: f32 = f32::from_bits(0x3F80_0000);
/// `flt_686798`: cap of each summed point light colour channel (CarRender.cpp 0x001123E0)
pub const CAR_POINT_COLOUR_MAX: f32 = f32::from_bits(0x3FFE_B852);
/// `flt_686794`: second factor of colour 0 (CarRender.cpp 0x001123E0)
pub const CAR_POINT_COLOUR_SCALE: f32 = f32::from_bits(0x3F80_0000);
/// `dword_686790`: share of the summed point colour in the ambient (CarRender.cpp 0x001123E0)
pub const CAR_POINT_AMBIENT_SCALE: f32 = f32::from_bits(0x3F40_0000);
/// `dword_686670`: ambient alpha at full weight (CarRender.cpp 0x001123E0)
pub const CAR_POINT_AMBIENT_ALPHA: f32 = f32::from_bits(0x3E80_0000);
/// `dword_625678`: smallest weight a point light adds (CarRender.cpp 0x001123E0)
pub const CAR_POINT_MIN_WEIGHT: f32 = f32::from_bits(0x3A83_126F);
/// `dword_685074`: height added to the position ClosestLights measures from (VsRender.cpp 0x0005A710)
pub const CLOSEST_LIGHTS_Y: f32 = f32::from_bits(0x3F40_0000);

/// OSD3dInterface.cpp 0x000EB0C0 — a packed light colour scaled; packing (Armco 0x00013CA0) clamps to 0..1
fn scale_packed(c: Vec4, s: f32) -> Vec4 {
    (c * s).clamp(Vec4::ZERO, Vec4::ONE)
}

impl VsLights {
    /// CarRender.cpp 0x00119EF0 — a car's 32-byte light set (+1028: three packed directions, three colours,
    /// ambient, count 3). Front end: scene lights 0/1 with their colours, light 2 off (dir (0,1,0), black), scene
    /// ambient. In game: `object_lights` (VsRender 0x00059140; `search` = not LOD 2), then light 2 = scene colour 0
    /// (+880), ambient = scene ambient (+864), and the two point light colours x 0.25.
    /// `point_scale` = `flt_6866A0` ("Graphics.Lights.Point Light Intensity", `CAR_POINT_LIGHT_SCALE`)
    pub fn car_light_set(&self, points: &[PointLight], pos: Vec3, slots: &mut [i16; 2], search: bool, frontend: bool, point_scale: f32) -> VsLights {
        if frontend {
            let mut l = *self;
            l.dir[2] = Vec3::Y;
            l.colour[2] = Vec4::ZERO;
            return l;
        }
        let mut l = self.object_lights(points, pos, slots, search);
        l.colour[2] = self.colour[0];
        l.ambient = self.ambient;
        l.colour[0] = scale_packed(l.colour[0], point_scale);
        l.colour[1] = scale_packed(l.colour[1], point_scale);
        l
    }

    /// CarRender.cpp 0x001123E0 — end of the light pass (CarLightInputs 0x00112A50, skipped for a car the view sits
    /// in): overwrite the car's set (+1028) with one light from the point lights around `pos` (car matrix row 3).
    /// Lights from last to first, radius 0 skipped; weight t = 1 − d²/r² with d² on the **xz plane** only, kept when
    /// > 0.001; Σt, Σ light·t, Σ colour·t; colour channels capped at 1.99, w 0. Direction 0 = normalise(Σpos/Σt − pos),
    /// colour 0 = colour · `brightness` · 1, colours 1/2 black, ambient = colour · (0.75 · `brightness`) with alpha
    /// 0.25 · min(Σt, 1); directions 1/2 and the count are left. Also returns (direction, capped colour) =
    /// `unk_718208` / `unk_7181C4` (read only by unreferenced code at 0x00113B70 / 0x00114680 and Characters 0x0007A460).
    /// Σt = 0 divides 0 by 0 (a NaN direction under a black colour in the original); zero here.
    pub fn car_point_light_set(&self, points: &[PointLight], pos: Vec3, brightness: f32) -> (VsLights, Vec3, Vec4) {
        let mut sum_t = 0.0f32;
        let mut sum_pos = Vec3::ZERO;
        let mut colour = Vec4::ZERO;
        for l in points.iter().rev() {
            if l.radius == 0.0 {
                continue;
            }
            let d = l.pos - pos;
            let x2 = d.x * d.x;
            let d2 = d.z * d.z + x2;
            let r2 = l.radius * l.radius;
            if r2 > d2 {
                let t = 1.0 - d2 / r2;
                if t > CAR_POINT_MIN_WEIGHT {
                    sum_t += t;
                    sum_pos += l.pos * t;
                    colour += l.colour * t;
                }
            }
        }
        colour.x = colour.x.min(CAR_POINT_COLOUR_MAX);
        colour.y = colour.y.min(CAR_POINT_COLOUR_MAX);
        colour.z = colour.z.min(CAR_POINT_COLOUR_MAX);
        colour.w = 0.0;
        let dir = if sum_t == 0.0 { Vec3::ZERO } else { vs_normalise(sum_pos / sum_t - pos) };
        let mut l = *self;
        l.dir[0] = dir;
        l.colour[0] = (colour * brightness * CAR_POINT_COLOUR_SCALE).clamp(Vec4::ZERO, Vec4::ONE);
        l.colour[1] = Vec4::ZERO;
        l.colour[2] = Vec4::ZERO;
        let mut amb = colour * (CAR_POINT_AMBIENT_SCALE * brightness);
        amb.w = if sum_t <= 1.0 { CAR_POINT_AMBIENT_ALPHA * sum_t } else { CAR_POINT_AMBIENT_ALPHA };
        l.ambient = amb.clamp(Vec4::ZERO, Vec4::ONE);
        (l, dir, colour)
    }

    /// VsRender.cpp 0x0005A710 — `ClosestLights(pos, dirs, colours, ambient)` as its only caller (CarRender 0x00118D70)
    /// uses it: null outputs = VsRender's own set (+4 / +0x34 / +0x64), directions negated. Nothing without point
    /// lights (+0x124); p = pos + (0, 0.75, 0); d² 3D, kept when r² > d², atten = 1 − √d²/r inserted into three slots
    /// sorted by atten (init 0 / −1); a slot with a light: dir = −normalise(light − p), colour = light.rgb · atten (w 1);
    /// an empty slot: colour zero (`unk_7805C0`), dir kept; the ambient is assigned to itself.
    pub fn closest_lights(&mut self, points: &[PointLight], pos: Vec3) {
        if points.is_empty() {
            return;
        }
        let mut p = pos;
        p.y += CLOSEST_LIGHTS_Y;
        let mut slots = [(0.0f32, -1i32); 3];
        for (i, l) in points.iter().enumerate() {
            let d = l.pos - p;
            let d2 = d.x * d.x + d.y * d.y + d.z * d.z;
            let r = l.radius;
            if r * r > d2 {
                let atten = 1.0 - d2.sqrt() / r;
                if atten > slots[2].0 {
                    for s in 0..3 {
                        if atten > slots[s].0 {
                            for k in (s + 1..3).rev() {
                                slots[k] = slots[k - 1];
                            }
                            slots[s] = (atten, i as i32);
                            break;
                        }
                    }
                }
            }
        }
        for (s, &(atten, i)) in slots.iter().enumerate() {
            if i < 0 {
                self.colour[s] = Vec4::ZERO;
            } else {
                let l = &points[i as usize];
                self.dir[s] = -vs_normalise(l.pos - p);
                self.colour[s] = (l.colour.truncate() * atten).extend(1.0);
            }
        }
    }

    /// OSD3dInterface.cpp 0x000EA2B0 — the 3d hud's light set (+2824): the player car's set with the two point light
    /// colours x 1.5 and the ambient = lerp(white, flash, flash.w) (FuryInstance 0x0012A4F0 with t = 1 - flash.w;
    /// `flash` = +2912, VsPoliceManager::Flash 0x0003A1C0, zero when no police car flashes).
    pub fn hud_light_set(&self, flash: Vec4) -> VsLights {
        let mut l = *self;
        l.colour[0] = scale_packed(l.colour[0], HUD_POINT_LIGHT_SCALE);
        l.colour[1] = scale_packed(l.colour[1], HUD_POINT_LIGHT_SCALE);
        let t = 1.0 - flash.w;
        let white = Vec4::ONE;
        let a = flash * (1.0 - t) + white * t;
        l.ambient = a.clamp(Vec4::ZERO, Vec4::ONE);
        l
    }

    /// Renderer.cpp 0x002A8BE0 + D3D fixed-function lighting (material: diffuse = vertex colour, ambient = white,
    /// emissive 0; Renderer 0x002A81C0): rgb = saturate(ambient + sum(vertex rgb * colour_i * max(0, n . -dir_i))),
    /// alpha = vertex alpha. A model without a normal stream has n = 0 (nv2a default attribute): ambient only.
    pub fn light_fixed_function(&self, n: Option<Vec3>, diffuse: Vec4) -> Vec4 {
        let mut c = self.ambient.truncate();
        if let Some(n) = n {
            for i in 0..3 {
                c += diffuse.truncate() * self.colour[i].truncate() * n.dot(-self.dir[i]).max(0.0);
            }
        }
        c.clamp(Vec3::ZERO, Vec3::ONE).extend(diffuse.w)
    }
}

/// first compartment version whose instances get their own light set (CompartmentManager.cpp 0x00087D00)
pub const INSTANCE_OWN_SET_VERSION: u32 = 6;

impl VsLights {
    /// VsRender.cpp 0x000598D0 — an instance's own set: slots from its count (0) to 1 black with dir (0,0,1), slot 2
    /// = scene light 0, count 3, ambient = `ambient` (the caller's scene ambient x 0.5; None = scene ambient)
    pub fn instance_own_set(&self, ambient: Option<Vec4>) -> VsLights {
        let mut l = VsLights::default();
        for i in 0..2 {
            l.dir[i] = Vec3::Z;
            l.colour[i] = Vec4::ZERO;
        }
        l.dir[2] = self.dir[0];
        l.colour[2] = self.colour[0].clamp(Vec4::ZERO, Vec4::ONE);
        l.ambient = ambient.unwrap_or(self.ambient).clamp(Vec4::ZERO, Vec4::ONE);
        l
    }

    /// CompartmentManager.cpp 0x00087D00 — the set bound to a compartment's visual/physics instances: version >= 6
    /// packs get `instance_own_set(scene ambient x 0.5)`; older ones (every shipped pack: 5, stale 3) point at the
    /// global set `unk_718218` (static zero, count 0) whose ambient only CSkyRender::Render 0x0013BF30 writes, last
    /// with unk_730DE8 (1,1,1) x "Buildings Brightness" before the world draws (`sky_ambient`; zero without a sky):
    /// no light is enabled, so lit rgb = that ambient.
    pub fn instance_light_set(&self, version: u32, sky_ambient: Vec4) -> VsLights {
        if version >= INSTANCE_OWN_SET_VERSION {
            return self.instance_own_set(Some(self.ambient * 0.5));
        }
        VsLights {
            dir: [Vec3::ZERO; 3],
            colour: [Vec4::ZERO; 3],
            ambient: sky_ambient.clamp(Vec4::ZERO, Vec4::ONE),
        }
    }
}

impl VsLights {
    /// VsRender.cpp 0x00059140 — lights for one object at `pos`: up to two point lights in range fill slots 0-1
    /// (direction light → object, colour scaled by clamp((r - d) / r * 2, 0, 1)), unused point slots are blanked,
    /// slot 2 is scene light 0. `slots` keeps the object's lights (-1 empty) while in range; new ones are searched
    /// only when `search` is set. Ambient as the scene.
    pub fn object_lights(&self, points: &[PointLight], pos: Vec3, slots: &mut [i16; 2], search: bool) -> VsLights {
        let mut found: Vec<(usize, Vec3, f32)> = Vec::new();
        for s in slots.iter_mut() {
            if *s < 0 {
                continue;
            }
            let Some(l) = points.get(*s as usize) else {
                *s = -1;
                continue;
            };
            let d2 = (pos - l.pos).length_squared();
            if l.radius * l.radius < d2 {
                *s = -1;
            } else {
                found.push((*s as usize, (pos - l.pos).normalize_or_zero(), d2.sqrt()));
            }
        }
        if search {
            for (i, l) in points.iter().enumerate() {
                if found.len() >= 2 {
                    break;
                }
                if l.radius == 0.0 {
                    continue;
                }
                let d2 = (pos - l.pos).length_squared();
                if l.radius * l.radius >= d2 && !found.iter().any(|f| f.0 == i) {
                    found.push((i, (pos - l.pos).normalize_or_zero(), d2.sqrt()));
                    if slots[0] == -1 {
                        slots[0] = i as i16;
                    } else {
                        slots[1] = i as i16;
                    }
                }
            }
        }
        let mut out = VsLights { ambient: self.ambient, ..*self };
        let mut k = 0;
        for (i, dir, dist) in &found {
            let l = points[*i];
            let a = ((l.radius - dist) / l.radius * POINT_LIGHT_ATTEN_SCALE).clamp(0.0, 1.0);
            out.colour[k] = (l.colour.truncate() * a).extend(l.colour.w);
            // the slot stores the negated light→object direction as the light's direction
            out.dir[k] = *dir;
            k += 1;
        }
        // free point slots are blanked (colour 0, direction (0,0,1)); slot 2 takes scene light 0
        while k < 2 {
            out.colour[k] = Vec4::ZERO;
            out.dir[k] = -Vec3::Z;
            k += 1;
        }
        if found.len() < 3 {
            out.dir[2] = self.dir[0];
            out.colour[2] = self.colour[0];
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_track_lights_txt() {
        let dir = visos::file::resolve("\\Tracks");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path().join("Lights.txt");
            if let Ok(t) = std::fs::read_to_string(&p) {
                let l = VsLights::parse_lights_txt(&t).unwrap_or_else(|| panic!("{}", p.display()));
                assert!((l.dir[0].length() - 1.0).abs() < 1e-4);
                n += 1;
            }
        }
        assert!(n >= 3);
    }

    #[test]
    fn every_track_ptlights_txt() {
        let dir = visos::file::resolve("/Tracks");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            if let Ok(t) = std::fs::read_to_string(e.path().join("PtLights.txt")) {
                let header = t.split(['(', ')']).nth(1).and_then(|v| v.trim().parse::<usize>().ok()).unwrap();
                let l = parse_ptlights_txt(&t);
                assert_eq!(l.len(), header, "{}", e.path().display());
                assert!(l.iter().all(|p| p.radius >= 0.0)); // radius 0 lights exist and are skipped by the search
                n += 1;
            }
        }
        assert!(n >= 3);
    }

    #[test]
    fn car_point_light_set_xz_weights() {
        let pts = [
            PointLight { pos: Vec3::new(3.0, 40.0, 0.0), radius: 6.0, colour: Vec4::new(1.0, 0.5, 0.0, 7.0) },
            PointLight { pos: Vec3::new(0.0, 0.0, 9.0), radius: 0.0, colour: Vec4::ONE },
            PointLight { pos: Vec3::new(-20.0, 0.0, 0.0), radius: 6.0, colour: Vec4::ONE },
        ];
        let base = VsLights::vs_render_defaults();
        let (l, dir, col) = base.car_point_light_set(&pts, Vec3::ZERO, CAR_BRIGHTNESS);
        // only the first light reaches: t = 1 − 9/36 (its height is ignored)
        let t = 1.0 - 9.0 / 36.0;
        assert_eq!(col, Vec4::new(t, 0.5 * t, 0.0, 0.0));
        assert!((dir - Vec3::new(3.0, 40.0, 0.0).normalize()).length() < 1e-6);
        assert_eq!(l.colour[1], Vec4::ZERO);
        assert_eq!(l.dir[1], base.dir[1]);
        assert_eq!(l.ambient, Vec4::new(0.75 * t, 0.375 * t, 0.0, 0.25 * t));
        let (none, d0, _) = base.car_point_light_set(&[], Vec3::ZERO, CAR_BRIGHTNESS);
        assert_eq!((none.colour[0], none.ambient, d0), (Vec4::ZERO, Vec4::ZERO, Vec3::ZERO));
    }

    #[test]
    fn closest_lights_sorted_slots() {
        let pts = [
            PointLight { pos: Vec3::new(0.0, 0.75, 4.0), radius: 8.0, colour: Vec4::new(1.0, 0.0, 0.0, 1.0) },
            PointLight { pos: Vec3::new(0.0, 0.75, -1.0), radius: 4.0, colour: Vec4::new(0.0, 1.0, 0.0, 1.0) },
            PointLight { pos: Vec3::new(30.0, 0.0, 0.0), radius: 4.0, colour: Vec4::ONE },
            PointLight { pos: Vec3::new(9.0, 9.0, 9.0), radius: 0.0, colour: Vec4::ONE },
        ];
        let base = VsLights::vs_render_defaults();
        let mut l = base;
        l.closest_lights(&pts, Vec3::ZERO);
        // atten 0.75 (green, 1 m of 4) before 0.5 (red, 4 m of 8); slot 2 empty: black, direction kept
        assert_eq!(l.colour[0], Vec4::new(0.0, 0.75, 0.0, 1.0));
        assert_eq!(l.dir[0], Vec3::Z);
        assert_eq!(l.colour[1], Vec4::new(0.5, 0.0, 0.0, 1.0));
        assert_eq!(l.dir[1], Vec3::NEG_Z);
        assert_eq!((l.colour[2], l.dir[2], l.ambient), (Vec4::ZERO, base.dir[2], base.ambient));
        let mut none = base;
        none.closest_lights(&[], Vec3::ZERO);
        assert_eq!(none.colour, base.colour);
    }

    #[test]
    fn vs_render_default_set() {
        let l = VsLights::vs_render_defaults();
        assert_eq!(l.dir[1], Vec3::NEG_Y);
        assert!((l.dir[0] - Vec3::new(6.0, -1.0, -3.0).normalize()).length() < 1e-6);
        // an upward normal: 0.4 + 0.2·(1/√46) + 0.3 + 0.1·(1/√46)
        let c = l.light_vertex(Vec3::Y).x;
        assert!((c - (0.4 + 0.3 + 0.3 / 46f32.sqrt())).abs() < 1e-5, "{c}");
    }

    #[test]
    fn object_lights_slots() {
        let scene = VsLights::default();
        let pts = [PointLight { pos: Vec3::ZERO, radius: 10.0, colour: Vec4::ONE }];
        let mut slots = [-1i16; 2];
        let l = scene.object_lights(&pts, Vec3::new(5.0, 0.0, 0.0), &mut slots, true);
        assert_eq!(slots, [0, -1]);
        assert_eq!(l.colour[0], Vec4::new(1.0, 1.0, 1.0, 1.0)); // (10-5)/10*2 = 1
        assert_eq!(l.colour[1], Vec4::ZERO);
        let l = scene.object_lights(&pts, Vec3::new(20.0, 0.0, 0.0), &mut slots, true);
        assert_eq!(slots, [-1, -1]);
        assert_eq!(l.colour[0], Vec4::ZERO);
    }

    #[test]
    fn car_and_hud_light_sets() {
        let mut scene = VsLights::default();
        scene.colour = [Vec4::splat(0.5), Vec4::splat(0.25), Vec4::splat(0.75)];
        scene.ambient = Vec4::splat(0.2);
        let pts = [PointLight { pos: Vec3::ZERO, radius: 10.0, colour: Vec4::ONE }];
        let mut slots = [-1i16; 2];
        // in game: point light x 0.25, slot 2 = scene light 0, scene ambient
        let l = scene.car_light_set(&pts, Vec3::new(5.0, 0.0, 0.0), &mut slots, true, false, CAR_POINT_LIGHT_SCALE);
        assert_eq!(l.colour[0], Vec4::splat(0.25));
        assert_eq!(l.colour[1], Vec4::ZERO);
        assert_eq!(l.colour[2], Vec4::splat(0.5));
        assert_eq!(l.ambient, Vec4::splat(0.2));
        // front end: scene lights 0/1, light 2 off, no point lights
        let f = scene.car_light_set(&pts, Vec3::ZERO, &mut [-1; 2], true, true, CAR_POINT_LIGHT_SCALE);
        assert_eq!(f.colour[1], Vec4::splat(0.25));
        assert_eq!((f.colour[2], f.dir[2]), (Vec4::ZERO, Vec3::Y));
        // hud: x 1.5 clamped, ambient white without a flash, half blue at flash w 0.5
        let h = l.hud_light_set(Vec4::ZERO);
        assert_eq!(h.colour[0], Vec4::splat(0.375));
        assert_eq!(h.ambient, Vec4::ONE);
        let h = l.hud_light_set(Vec4::new(0.0, 0.0, 1.0, 0.5));
        assert_eq!(h.ambient.truncate(), Vec3::new(0.5, 0.5, 1.0));
        // no normals: ambient only, vertex alpha kept
        assert_eq!(h.light_fixed_function(None, Vec4::new(0.1, 0.1, 0.1, 0.3)), Vec4::new(0.5, 0.5, 1.0, 0.3));
    }

    /// shipped packs (< 6): ambient-only grey from the sky, vertex rgb dropped even with a normal; >= 6: slot 2 =
    /// scene light 0, half scene ambient
    #[test]
    fn instance_light_sets() {
        let mut scene = VsLights::default();
        scene.dir[0] = Vec3::NEG_Y;
        scene.colour[0] = Vec4::splat(0.5);
        scene.ambient = Vec4::splat(0.4);
        let l = scene.instance_light_set(5, Vec4::splat(0.6));
        let c = l.light_fixed_function(Some(Vec3::Y), Vec4::new(0.2, 0.9, 0.3, 0.7));
        assert_eq!(c, Vec4::new(0.6, 0.6, 0.6, 0.7));
        let o = scene.instance_light_set(6, Vec4::ZERO);
        assert_eq!((o.colour[0], o.colour[1], o.colour[2]), (Vec4::ZERO, Vec4::ZERO, Vec4::splat(0.5)));
        assert_eq!(o.ambient, Vec4::splat(0.2));
        let c = o.light_fixed_function(Some(Vec3::Y), Vec4::ONE);
        assert!((c.x - 0.7).abs() < 1e-6);
    }
}
