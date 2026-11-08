//! src\Osd\Speedo.cpp + OSD3dInterfaceConfiguration.cpp (the hud part) — the speedo of the digital HUDs
//! (`OSD\HUDS\<name>.vcd`: `<name>-config.txt`, `-base`, `-revs`, `-nos`, `-zone`, `-boost`, `-font`, `-thumb`).
//!
//! Geometry is built in HUD units exactly as the original (pixel values of the 256x256 design converted with
//! `px / texture size * hud size`, y up); the gauge vtable is `[shape + 5 * format]` (0x000F42E0).
//! The quads are projected onto the 512x512 2D target through the 3d interface camera
//! (`osd_3d_interface::project`: eye z 30, fov 45, the camera 0x000EA2B0 draws with). The `+44` gauge pass
//! (blend mode 4, 0x000F3F60) is not drawn. Fed from `SpeedoInput`, which game flow fills from the car.

use crate::render2d::{Colour, DrawList, Quad, TextureId, TextureStore};
use velocity_game::timeline_manager::race_timeline_sequencer::vs_string_hash;

/// one gauge of the config (`speedo <x> gauge ...`, 11 dwords at config +168/+212/+256/+300).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GaugeConfig {
    pub shape: u32,        // 0 needle, 1 horizontal bar, 2 vertical bar, 3 circle segment, 4 offset gauge
    pub format: u32,       // 0 analogue, 1 digital
    pub kind: u32,         // 0 revs, 1 nos, 2 zone, 3 turbo
    pub offset: [f32; 3],  // pixels
    pub start: i32,        // degrees or pixels
    pub range: i32,        // degrees or pixels
    pub radius: u32,       // pixels
    pub segments: u32,
}

/// `OSD3dInterfaceConfiguration` hud block (400 bytes per player).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudConfig {
    pub version: i32,
    pub filename: String,           // +40
    pub name: String,               // +104
    pub position: [[f32; 3]; 3],    // single, splitscreen p1, p2
    pub height: f32,                // +16
    pub width: f32,                 // +20
    pub texture_width: u32,         // +28
    pub texture_height: u32,        // +24
    pub font_spacing: u32,          // +32
    pub font_width: u32,            // +36
    pub gauges: [GaugeConfig; 4],   // revs, nos, zone, turbo blocks in file order
    pub speed_offset: [f32; 3],     // +344
    pub speed_height: u32,          // +364
    pub speed_width: u32,           // +360
    pub speed_spacing: u32,         // +368
    pub gear_offset: [f32; 3],      // +372
    pub gear_height: u32,           // +392
    pub gear_width: u32,            // +388
    pub gear_spacing: u32,          // +396
    pub tft: [[f32; 3]; 6],         // single pos/rot, p1 pos/rot, p2 pos/rot
    pub map: [[f32; 3]; 6],         // single pos/rot, p1 pos/rot, p2 pos/rot
    pub map_grid_texture: String,   // +1408
    pub map_colour: [u8; 3],        // +1792
}

#[derive(Debug)]
pub struct ConfigError(pub String);

/// the `#` commented token stream the `OSD3dInterface` readers (0x000EB6C0 int, 0x000EB780 float,
/// 0x000EB840 vector, 0x000EB940 string) walk in order.
struct Tokens<'a> {
    toks: Vec<&'a str>,
    i: usize,
}

impl<'a> Tokens<'a> {
    fn new(text: &'a str) -> Self {
        let mut toks = Vec::new();
        for line in text.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') {
                continue;
            }
            toks.push(l);
        }
        Self { toks, i: 0 }
    }
    fn next(&mut self, what: &str) -> Result<&'a str, ConfigError> {
        let t = self.toks.get(self.i).ok_or_else(|| ConfigError(format!("missing {}", what)))?;
        self.i += 1;
        Ok(t)
    }
    fn nums(&mut self, what: &str) -> Result<Vec<f32>, ConfigError> {
        let t = self.next(what)?;
        let inner = t.strip_prefix('(').and_then(|t| t.strip_suffix(')')).ok_or_else(|| ConfigError(format!("{}: not a value: {}", what, t)))?;
        inner.split(',').map(|v| v.trim().parse::<f32>().map_err(|_| ConfigError(format!("{}: bad number {}", what, v)))).collect()
    }
    fn int(&mut self, what: &str) -> Result<i32, ConfigError> {
        Ok(self.nums(what)?[0] as i32)
    }
    fn float(&mut self, what: &str) -> Result<f32, ConfigError> {
        Ok(self.nums(what)?[0])
    }
    fn vec3(&mut self, what: &str) -> Result<[f32; 3], ConfigError> {
        let v = self.nums(what)?;
        Ok([v[0], *v.get(1).unwrap_or(&0.0), *v.get(2).unwrap_or(&0.0)])
    }
    fn string(&mut self, what: &str) -> Result<String, ConfigError> {
        let t = self.next(what)?;
        Ok(t.trim_matches('"').to_string())
    }
}

impl HudConfig {
    /// OSD3dInterfaceConfiguration.cpp 0x000EB9F0 — `Load` (single player reading order).
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let mut t = Tokens::new(text);
        let mut c = HudConfig { version: t.int("version number")?, ..Default::default() };
        if c.version != 1 {
            return Err(ConfigError(format!("hud config version is incorrect. Expected version 1, got version {}", c.version)));
        }
        c.filename = t.string("filename")?;
        c.name = t.string("hud name")?;
        for p in 0..3 {
            c.position[p] = t.vec3("speedo position")?;
        }
        c.height = t.float("speedo height")?;
        c.width = t.float("speedo width")?;
        c.texture_width = t.int("speedo texture width")? as u32;
        c.texture_height = t.int("speedo texture height")? as u32;
        c.font_spacing = t.int("speedo font texture spacing")? as u32;
        c.font_width = t.int("speedo font texture width")? as u32;
        for g in c.gauges.iter_mut() {
            g.shape = t.int("gauge shape")? as u32;
            g.format = t.int("gauge format")? as u32;
            g.kind = t.int("gauge type")? as u32;
            g.offset = t.vec3("gauge offset")?;
            g.start = t.int("gauge start angle")?;
            g.range = t.int("gauge range")?;
            g.radius = t.int("gauge radius")? as u32;
            g.segments = t.int("gauge num segments")? as u32;
        }
        c.speed_offset = t.vec3("speedo speed text offset")?;
        c.speed_height = t.int("speedo speed text height")? as u32;
        c.speed_width = t.int("speedo speed text width")? as u32;
        c.speed_spacing = t.int("speedo speed text spacing")? as u32;
        c.gear_offset = t.vec3("speedo gear text offset")?;
        c.gear_height = t.int("speedo gear text height")? as u32;
        c.gear_width = t.int("speedo gear text width")? as u32;
        c.gear_spacing = t.int("speedo gear text spacing")? as u32;
        for v in c.tft.iter_mut() {
            *v = t.vec3("TFT")?;
        }
        for v in c.map.iter_mut() {
            *v = t.vec3("map")?;
        }
        c.map_grid_texture = t.string("map grid texture name")?;
        let col = t.vec3("map colour")?;
        c.map_colour = [col[0] as u8, col[1] as u8, col[2] as u8];
        Ok(c)
    }

    /// OSD3dInterfaceConfiguration.cpp 0x000EB9F0 `Load(name, player)` with `byte_6F7157` set: player p's block
    /// (+400 p) gets the "splitscreen player p+1" speedo position, its TFT (+800 + 224 p) and map (+1248 + 548 p)
    /// slots the player's position / rotation lines; returned here as the block's single-player fields
    pub fn split_player(&self, player: usize) -> Self {
        let p = player.min(1);
        let mut c = self.clone();
        c.position[0] = self.position[1 + p];
        c.tft[0] = self.tft[2 + 2 * p];
        c.tft[1] = self.tft[3 + 2 * p];
        c.map[0] = self.map[2 + 2 * p];
        c.map[1] = self.map[3 + 2 * p];
        c
    }

    /// `\OSD\HUDS\<name>.vcd` + `<name>-config.txt`
    /// OSD3dInterfaceConfiguration.cpp 0x000EB001 / FECarCustomisationScreen.cpp 0x002515D0 — the `\OSD\HUDS\*.vcd`
    /// stems in directory order (sorted here), as both the hud lookup by hash and the gauges list scan them
    pub fn list() -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(visos::file::platform::resolve(r"\OSD\HUDS")) {
            for e in rd.flatten() {
                let n = e.file_name().to_string_lossy().to_string();
                if n.len() > 4 && n[n.len() - 4..].eq_ignore_ascii_case(".vcd") {
                    v.push(n[..n.len() - 4].to_string());
                }
            }
        }
        v.sort_by_key(|s| s.to_ascii_lowercase());
        v
    }

    /// OSD3dInterfaceConfiguration.cpp 0x000EB001 — the hud a `VsStringAnsi::Hash` (the setup's +0x188) names; 0 =
    /// "digione", an unknown hash = no hud (the original logs the load failure)
    pub fn name_for_hash(hash: u32) -> Option<String> {
        let hash = if hash == 0 { vs_string_hash("digione") } else { hash };
        Self::list().into_iter().find(|n| vs_string_hash(n) == hash)
    }

    pub fn load(name: &str) -> Result<Self, ConfigError> {
        let path = format!("\\OSD\\HUDS\\{}.vcd", name);
        let v = visos::file::Vcd::open(&path).map_err(|e| ConfigError(format!("{}: {:?}", path, e)))?;
        let file = format!("{}-config.txt", name);
        let data = v.get(&file).ok_or_else(|| ConfigError(format!("Can't open hud config file - {}", file)))?;
        Self::parse(&String::from_utf8_lossy(data))
    }
}

/// what the speedo reads from the player car each frame.
#[derive(Clone, Debug, Default)]
pub struct SpeedoInput {
    pub speed: f32,  // VsAIRaceVehicle 0x00024220 (shown * 2.25, capped 999)
    pub rpm: f32,    // OSDBridge 0x00098770 (+0)
    pub gear: i32,   // VsConfigFileRead 0x001B74A0: > 90 = -100, < 0 reverse, 0 neutral
    pub nos: f32,    // 0x00024720 ratio
    pub zone: f32,   // 0x000F78D0 ratio
    pub turbo: f32,  // 0x000F79A0 ratio (1 when the car has no turbo, 0x000F7970)
}

/// gauge runtime record (56 bytes, Speedo 0x000F4590)
#[derive(Clone, Debug, Default)]
struct Gauge {
    offset: [f32; 3],
    shape: u32,
    format: u32,
    kind: u32,
    start: f32,   // radians for needle / segment / offset, pixels for bars
    range: f32,
    radius: f32,
    segments: u32,
    texture: TextureId,
}

/// `CSpeedo`
pub struct Speedo {
    pub config: HudConfig,
    pos: [f32; 3],   // +40
    height: f32,     // +384
    width: f32,      // +388
    tex_h: f32,      // +396
    tex_w: f32,      // +392
    font_spacing: f32, // +400
    font_width: f32,   // +404
    gauges: [Gauge; 4],
    base: TextureId,   // +408
    font: TextureId,   // +412
    pub visible: bool, // +416
}

const GAUGE_TEXTURES: [&str; 4] = ["revs", "nos", "zone", "boost"];

impl Speedo {
    /// Speedo.cpp 0x000F4360 — `Setup` from the hud config, textures from the hud pack.
    pub fn new(config: HudConfig, tex: &mut TextureStore) -> Self {
        let name = config.filename.clone();
        tex.load_vcd(&format!("\\OSD\\HUDS\\{}.vcd", name));
        let find = |tex: &TextureStore, part: &str| tex.find(visos::handy::handy_crc::name_hash(&format!("{}-{}", name, part).to_ascii_lowercase()));
        let mut gauges: [Gauge; 4] = Default::default();
        for (g, c) in gauges.iter_mut().zip(config.gauges.iter()) {
            // Speedo.cpp 0x000F4590 — SetupGaugeData
            let (start, range) = match c.shape {
                0 | 3 | 4 => (c.start as f32 * 0.017453292, c.range as f32 * 0.017453292),
                _ => (c.start as f32, c.range as f32),
            };
            *g = Gauge {
                offset: c.offset,
                shape: c.shape,
                format: c.format,
                kind: c.kind,
                start,
                range,
                radius: c.radius as f32,
                segments: c.segments,
                texture: find(tex, GAUGE_TEXTURES[c.kind as usize & 3]),
            };
        }
        Self {
            pos: config.position[0],
            height: config.height,
            width: config.width,
            tex_h: config.texture_height as f32,
            tex_w: config.texture_width as f32,
            font_spacing: config.font_spacing as f32,
            font_width: config.font_width as f32,
            gauges,
            base: find(tex, "base"),
            font: find(tex, "font"),
            visible: true,
            config,
        }
    }

    /// Speedo.cpp 0x000F4F40 — continuous ratio of a gauge (the frame-timer smoothing 0x00057F90 is omitted).
    fn ratio(kind: u32, i: &SpeedoInput) -> f32 {
        match kind {
            0 => (i.rpm.min(6000.0) / 6000.0).min(1.0),
            1 => i.nos.min(1.0),
            2 => i.zone.min(1.0),
            3 => i.turbo.min(1.0),
            _ => 0.0,
        }
    }

    /// Speedo.cpp 0x000F5220 — lit segment count, clamped to 0..segments.
    fn lit_segments(g: &Gauge, i: &SpeedoInput) -> u32 {
        let n = g.segments.max(1) as f32;
        let v = match g.kind {
            0 => (i.rpm.min(6000.0) / (6000.0 / n)) as i32,
            1 => (i.nos / (1.0 / n)) as i32,
            2 => (i.zone / (1.0 / n)) as i32,
            3 => (i.turbo / (1.0 / n)) as i32,
            _ => 0,
        };
        v.clamp(0, g.segments as i32) as u32
    }

    fn px_x(&self, px: f32) -> f32 {
        px / self.tex_w * self.width
    }
    fn px_y(&self, px: f32) -> f32 {
        px / self.tex_h * self.height
    }

    /// Speedo.cpp 0x000F46F0 — +40 = +56 (the config position) + `off`
    pub fn set_offset(&mut self, off: [f32; 3]) {
        let b = self.config.position[0];
        self.pos = [b[0] + off[0], b[1] + off[1], b[2] + off[2]];
    }

    /// Speedo.cpp 0x000F4760 / 0x000F47C0 / 0x000F4A50 — `Render`: base, the four gauges, then speed and gear digits.
    pub fn render(&self, input: &SpeedoInput, out: &mut DrawList) {
        if !self.visible {
            return;
        }
        let mut hud = Vec::<HudQuad>::new();
        // 0x000F47C0 base
        hud.push(HudQuad::rect(self.pos, self.width, self.height, 0.0, self.base, [0.0, 0.0, 1.0, 1.0]));
        for g in &self.gauges {
            self.render_gauge(g, input, &mut hud);
        }
        // 0x000F4A50 digits
        self.render_speed(input, &mut hud);
        self.render_gear(input, &mut hud);
        for q in hud {
            out.push(q.to_screen());
        }
    }

    /// Speedo.cpp 0x000F42E0 / 0x000F5100 / 0x000F5D20 / 0x000F59F0 / 0x000F6330 / 0x000F5470 / 0x000F60A0 / 0x000F5730 / 0x000F65C0 —
    /// one gauge through the `[shape + 5 * format]` table (needle, circle segments, bars, offset dial)
    fn render_gauge(&self, g: &Gauge, input: &SpeedoInput, hud: &mut Vec<HudQuad>) {
        let centre = self.pos;
        let r = self.px_y(g.radius);
        match (g.format, g.shape) {
            // 0x000F5100 needle: the gauge texture rotated to start + range * ratio
            (0, 0) => {
                let a = g.range * Self::ratio(g.kind, input) + g.start;
                hud.push(HudQuad::rect(centre, self.width, self.height, a, g.texture, [0.0, 0.0, 1.0, 1.0]));
            }
            // 0x000F5D20 analogue circle segment: 15 degree wedges, then the remainder
            (0, 3) => {
                let mut step = 15.0 * 0.017453292f32;
                if g.range < 0.0 {
                    step = 0.0 - step;
                }
                let sweep = g.range * Self::ratio(g.kind, input);
                let n = (sweep / step).abs() as u32;
                let rest = sweep - n as f32 * step;
                let mut a = g.start;
                for _ in 0..n {
                    hud.push(HudQuad::wedge(centre, r, a, a + step, g.texture));
                    a += step;
                }
                hud.push(HudQuad::wedge(centre, r, a, a + rest, g.texture));
            }
            // 0x000F59F0 digital circle segment: one wedge per lit segment
            (1, 3) => {
                let n = Self::lit_segments(g, input);
                let step = g.range / g.segments.max(1) as f32;
                let mut a = g.start;
                for _ in 0..n {
                    hud.push(HudQuad::wedge(centre, r, a, a + step, g.texture));
                    a += step;
                }
            }
            // 0x000F6330 / 0x000F5470 horizontal bar, 0x000F60A0 / 0x000F5730 vertical bar
            (f, 1) | (f, 2) => {
                let fill = if f == 0 {
                    Self::ratio(g.kind, input) * g.range
                } else {
                    Self::lit_segments(g, input) as f32 * (g.range / g.segments.max(1) as f32)
                };
                let x0 = centre[0] - 0.5 * self.width;
                let y0 = centre[1] - 0.5 * self.height;
                if g.shape == 1 {
                    let edge = self.tex_w * 0.5 + g.offset[0];
                    let w = (edge + fill) / self.tex_w * self.width;
                    hud.push(HudQuad::from_corner([x0, y0, centre[2]], w, self.height, g.texture, [0.0, 0.0, w / self.width, 1.0]));
                } else {
                    let edge = self.tex_h * 0.5 + g.offset[1];
                    let h = (edge + fill) / self.tex_h * self.height;
                    hud.push(HudQuad::from_corner([x0, y0, centre[2]], self.width, h, g.texture, [0.0, 1.0 - h / self.height, 1.0, 1.0]));
                }
            }
            // 0x000F65C0 offset gauge: a small dial at the offset with its own rotation
            (0, 4) => {
                let p = [centre[0] + self.px_x(g.offset[0]), centre[1] + self.px_y(g.offset[1]), centre[2]];
                let s = self.px_y(g.radius);
                let a = g.range * Self::ratio(g.kind, input) + g.start;
                hud.push(HudQuad::rect(p, s, s, a, g.texture, [0.0, 0.0, 1.0, 1.0]));
            }
            _ => {}
        }
    }

    /// Speedo.cpp 0x000F4B90 — speed digits, right to left from the configured offset.
    fn render_speed(&self, input: &SpeedoInput, hud: &mut Vec<HudQuad>) {
        let v = ((input.speed.abs() * 2.25) as u32).min(999);
        let s = format!("{}", v);
        let mut p = [self.pos[0] + self.px_x(self.config.speed_offset[0]), self.pos[1] + self.px_y(self.config.speed_offset[1]), self.pos[2]];
        let w = self.px_x(self.config.speed_width as f32);
        let h = self.px_y(self.config.speed_height as f32);
        let step = self.px_x(self.config.speed_spacing as f32);
        for c in s.bytes().rev() {
            hud.push(self.digit(p, w, h, (c - b'0') as u32));
            p[0] -= step;
        }
    }

    /// Speedo.cpp 0x000F4DC0 — gear digit: 1..9, 10 = R, 11 = N.
    fn render_gear(&self, input: &SpeedoInput, hud: &mut Vec<HudQuad>) {
        let mut gear = input.gear;
        if gear > 90 {
            gear -= 100;
        }
        let glyph = if gear >= 0 { if gear != 0 { gear.min(9) } else { 11 } } else { 10 };
        let p = [self.pos[0] + self.px_x(self.config.gear_offset[0]), self.pos[1] + self.px_y(self.config.gear_offset[1]), self.pos[2]];
        hud.push(self.digit(p, self.px_x(self.config.gear_width as f32), self.px_y(self.config.gear_height as f32), glyph as u32));
    }

    /// Speedo.cpp 0x000F73F0 — one glyph of the font strip, top-left at `p`.
    fn digit(&self, p: [f32; 3], w: f32, h: f32, glyph: u32) -> HudQuad {
        let k = ((4096.0 / self.font_width) * self.font_spacing) as i32;
        let u0 = (k * glyph as i32) as f32 / 4096.0;
        let u1 = (k * glyph as i32 + k) as f32 / 4096.0;
        HudQuad {
            texture: self.font,
            pos: [[p[0], p[1]], [p[0] + w, p[1]], [p[0], p[1] - h], [p[0] + w, p[1] - h]],
            uv: [[u0, 0.0], [u1, 0.0], [u0, 1.0], [u1, 1.0]],
            z: p[2],
        }
    }
}

/// a quad in HUD units (y up) in the plane z.
struct HudQuad {
    texture: TextureId,
    pos: [[f32; 2]; 4],
    uv: [[f32; 2]; 4],
    z: f32,
}

impl HudQuad {
    /// Speedo.cpp 0x000F68A0 — a `w` x `h` quad centred on `c`, rotated by `angle`.
    fn rect(c: [f32; 3], w: f32, h: f32, angle: f32, texture: TextureId, uv: [f32; 4]) -> Self {
        let (hw, hh) = (w * 0.5, h * 0.5);
        let (s, co) = angle.sin_cos();
        let rot = |x: f32, y: f32| [c[0] + x * co - y * s, c[1] + x * s + y * co];
        HudQuad {
            texture,
            pos: [rot(-hw, hh), rot(hw, hh), rot(-hw, -hh), rot(hw, -hh)],
            uv: [[uv[0], uv[1]], [uv[2], uv[1]], [uv[0], uv[3]], [uv[2], uv[3]]],
            z: c[2],
        }
    }

    /// Speedo.cpp 0x000F6CF0 — quad from its bottom-left corner.
    fn from_corner(p: [f32; 3], w: f32, h: f32, texture: TextureId, uv: [f32; 4]) -> Self {
        HudQuad {
            texture,
            pos: [[p[0], p[1] + h], [p[0] + w, p[1] + h], [p[0], p[1]], [p[0] + w, p[1]]],
            uv: [[uv[0], uv[1]], [uv[2], uv[1]], [uv[0], uv[3]], [uv[2], uv[3]]],
            z: p[2],
        }
    }

    /// Speedo.cpp 0x000F70F0 — one fan triangle (centre, a, b); uv maps the speedo circle onto the texture.
    fn wedge(c: [f32; 3], r: f32, a: f32, b: f32, texture: TextureId) -> Self {
        let pa = [c[0] + a.cos() * r, c[1] + a.sin() * r];
        let pb = [c[0] + b.cos() * r, c[1] + b.sin() * r];
        let uva = [a.cos() * 0.5 + 0.5, 1.0 - (a.sin() * 0.5 + 0.5)];
        let uvb = [b.cos() * 0.5 + 0.5, 1.0 - (b.sin() * 0.5 + 0.5)];
        // degenerate fourth corner: the rasteriser's second triangle collapses
        HudQuad { texture, pos: [[c[0], c[1]], pa, pb, pa], uv: [[0.5, 0.5], uva, uvb, uva], z: c[2] }
    }

    /// through the 3d interface camera (the quads are parallel to its image plane, so projecting the corners is exact)
    fn to_screen(&self) -> Quad {
        let z = self.z;
        let map = |p: [f32; 2]| crate::osd::osd_3d_interface::project([p[0], p[1], z]);
        let white: Colour = crate::feck2::feck2_manager::COLOUR_WHITE;
        Quad { texture: self.texture, pos: self.pos.map(map), uv: self.uv, colour: [white; 4] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_hash_round_trip() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let l = HudConfig::list();
        assert!(l.len() >= 9);
        for n in &l {
            assert_eq!(HudConfig::name_for_hash(vs_string_hash(n)).as_deref(), Some(n.as_str()));
        }
        assert_eq!(HudConfig::name_for_hash(0).as_deref(), Some("digione"));
    }

    #[test]
    fn parse_every_hud_config() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let dir = visos::file::platform::resolve(r"\OSD\HUDS");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            let Some(stem) = name.strip_suffix(".vcd") else { continue };
            let c = HudConfig::load(stem).unwrap_or_else(|e| panic!("{}: {:?}", stem, e));
            assert_eq!(c.filename.to_ascii_lowercase(), stem);
            assert!(c.gauges.iter().all(|g| g.shape <= 4 && g.format <= 1 && g.kind <= 3), "{}", stem);
            n += 1;
        }
        assert!(n >= 9, "{}", n);
    }
}
