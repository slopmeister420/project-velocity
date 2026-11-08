//! Render\LicencePlateRender.cpp (0x0012DC60..0x0012E5A0) — the player car's licence plate: a 64x32 "LPlate"
//! render target (linear A8R8G8B8) holding the plate style's background ("la_plate" / "london_plate" /
//! "tokyo_plate", `\VFX\misc.vcd`) and the plate text, drawn on the rear bumper through the "customPlate" shader.
//!
//! The render target is composed on the cpu into one 64x32 image (two draws, no depth). The text is drawn by the
//! frontend's fonts (`Fonts\FontManager.cpp`, owned by the frontend crate) through [`PlateFontHook`]; without the
//! hook the plate shows the background only.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::car_render::CarModels;
use crate::light_streak::visos_colour;
use crate::renderer::shader_manager::{StageState, XboxStageMaterial, XboxStageParams};
use crate::renderer::shader_program_manager::ProgramMaterial;
use crate::renderer::texture_manager::{decode_any_vtf, DecodedTexture};

/// `NUM_LPLATE_TYPES`; plate style names `off_686BDC`
pub const PLATE_TEXTURES: [&str; 3] = ["la_plate", "london_plate", "tokyo_plate"];
/// render target size (0x0012E1E0 `RenderTargetManager 0x002EEA10("LPlate", 64, 32, 18, …)`)
pub const PLATE_W: u32 = 64;
pub const PLATE_H: u32 = 32;
/// `dword_686BE8`: text drawn at y = 16 − 5
pub const TEXT_Y_OFFSET: f32 = 5.0;
/// widest text before it is squeezed (0x0012DF40)
pub const MAX_TEXT_WIDTH: i32 = 58;
/// font helper ids `dword_686BEC` (game: 6 = game set FEFont11) / `dword_686BF0` (front end: 2 = FEFont12)
pub const FONT_GAME: u32 = 6;
pub const FONT_FRONT_END: u32 = 2;
/// text colours per style `dword_71FF04` (static init 0x00425790, VISOS_ColourXBOX a, r, g, b): LA dark blue,
/// London black, Tokyo green
pub const TEXT_COLOURS: [[u8; 4]; 3] = [[0x80, 0x0C, 0x12, 0x21], [0x80, 0x08, 0x08, 0x08], [0x80, 0x00, 0x2E, 0x00]];
/// the plate primitive's stage texture (0x0012E1E0)
pub const PLATE_TEXTURE: &str = "plate01";

/// what the game sets: GamePlayer.cpp 0x000C1930 initialises style `dword_6F71DC` = 0 and text `unk_6F71E0` =
/// "OCTANE"; FEOnlineSignInScreen 0x0023AE10 / 0x0023AEF0 change them. `front_end` = GameManager::CommonInit's flag
/// (0x0012DC60 `a2`: all three backgrounds loaded and the front end font).
#[derive(Resource, Clone, Debug)]
pub struct LicencePlate {
    pub style: u32,
    pub text: String,
    pub front_end: bool,
}

impl Default for LicencePlate {
    fn default() -> Self {
        Self { style: 0, text: "OCTANE".into(), front_end: false }
    }
}

/// one glyph quad in render target pixels (x0, y0, x1, y1), uv in the font page (0..1), D3DCOLOR
#[derive(Clone, Copy, Debug)]
pub struct PlateGlyph {
    pub rect: [f32; 4],
    pub uv: [f32; 4],
    pub colour: u32,
}

/// the font calls 0x0012DF40 makes (`CFontHelper::GetFontIndex`, `CFontManager::GetFont`, `CBaseFont` +0x40 scale,
/// `GetStringWidth`, FontHelper 0x000B2F60 set scale, `DrawStringAt` with flags 0). Implemented by the frontend.
pub trait PlateFont: Send + Sync + 'static {
    /// the font's x scale (`CBaseFont` +0x40, LicencePlateRender 0x0012E5A0); None = font not loaded
    fn scale(&self, font: u32) -> Option<f32>;
    /// `CBaseFont::GetStringWidth(text, 0)` at the font's own scale
    fn string_width(&self, font: u32, text: &[u16]) -> i32;
    /// `DrawStringAt(x, y, text, colour, 0, 0)` with the scale set to `scale`: quads in render target pixels
    fn draw_string_at(&self, font: u32, scale: f32, x: f32, y: f32, text: &[u16], colour: u32) -> Vec<PlateGlyph>;
    /// the font's glyph page, rgba8 rows top-down (as the font texture is sampled)
    fn page(&self, font: u32) -> Option<&DecodedTexture>;
}

#[derive(Resource)]
pub struct PlateFontHook(pub Box<dyn PlateFont>);

/// the plate primitives of one car (0x0012E1E0: every primitive of the rear bumper's LOD 0 model using the shader
/// of the first `plate01` primitive); they get the customPlate material
#[derive(Component, Clone, Debug)]
pub struct CarPlatePrims(pub Vec<Entity>);

/// the render target object `unk_71FEE4`: +4 customPlate shader, +8/+12.. background textures, +24 target
#[derive(Resource)]
pub struct LicencePlateRender {
    pub image: Handle<Image>,
    /// "customPlate": texture = the target, stage flags 1, blend 1, vertex program LinearCopy (oD0 = v3, white for
    /// car models without a diffuse stream)
    pub custom_plate: Handle<XboxStageMaterial>,
    backgrounds: [Option<DecodedTexture>; 3],
    /// the background in use (+8: in game fixed at init, in the front end picked by style every render)
    current: Option<usize>,
    built: Option<(u32, String, bool, bool)>,
}

pub struct LicencePlatePlugin;

impl Plugin for LicencePlatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LicencePlate>().add_systems(Update, (licence_plate_render, apply_custom_plate).chain());
    }
}

/// LicencePlateRender.cpp 0x0012E1E0 — the plate primitives of the car's rear bumper (LOD 0)
pub fn find_plate_prims(m: &CarModels) -> Option<CarPlatePrims> {
    let model = m.model(m.part("r_bumper")?, 0);
    // stage texture names are kept lowercase (the original's strstr is case sensitive)
    let shader = &model.iter().find(|r| r.texture.contains(PLATE_TEXTURE))?.shader_name;
    let v: Vec<Entity> = model.iter().filter(|r| &r.shader_name == shader).map(|r| r.entity).collect();
    Some(CarPlatePrims(v))
}

fn plate_image() -> Image {
    let mut img = Image::new(
        Extent3d { width: PLATE_W, height: PLATE_H, depth_or_array_layers: 1 },
        TextureDimension::D2,
        vec![0; (PLATE_W * PLATE_H * 4) as usize],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::all(),
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    img
}

/// LicencePlateRender.cpp 0x0012DC60 — background textures (by name from the loaded `\VFX\misc.vcd`) and the two
/// shaders
fn init(images: &mut Assets<Image>, materials: &mut Assets<XboxStageMaterial>) -> LicencePlateRender {
    let mut backgrounds: [Option<DecodedTexture>; 3] = Default::default();
    if let Ok(v) = visos::file::Vcd::open("\\VFX\\misc.vcd") {
        for (i, name) in PLATE_TEXTURES.iter().enumerate() {
            backgrounds[i] = v.get(&format!("{name}.vtf")).and_then(|d| decode_any_vtf(d).ok());
            if backgrounds[i].is_none() {
                log::warn!("licence plate: no {name}");
            }
        }
    }
    let image = images.add(plate_image());
    let state = StageState { blend_mode: 1, ..StageState::default() };
    let mut params = XboxStageParams::new(&state);
    params.arg.x = 0.0;
    let custom_plate = materials.add(XboxStageMaterial { params, texture: Some(image.clone()), stencil: None, spec_texture: None, state });
    LicencePlateRender { image, custom_plate, backgrounds, current: None, built: None }
}

/// bilinear sample of a texture at uv (texel centres at +0.5), wrapping
fn sample(t: &DecodedTexture, u: f32, v: f32) -> [f32; 4] {
    let (w, h) = (t.width as i32, t.height as i32);
    let x = u * w as f32 - 0.5;
    let y = v * h as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let px = |xi: i32, yi: i32| {
        let (xi, yi) = (xi.rem_euclid(w), yi.rem_euclid(h));
        let o = ((yi * w + xi) * 4) as usize;
        let p = &t.levels[0][o..o + 4];
        [p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32]
    };
    let (a, b, c, d) = (px(x0 as i32, y0 as i32), px(x0 as i32 + 1, y0 as i32), px(x0 as i32, y0 as i32 + 1), px(x0 as i32 + 1, y0 as i32 + 1));
    let mut o = [0.0; 4];
    for k in 0..4 {
        o[k] = (a[k] * (1.0 - fx) + b[k] * fx) * (1.0 - fy) + (c[k] * (1.0 - fx) + d[k] * fx) * fy;
    }
    o
}

/// GFXHelper.cpp 0x0012B720 — the background quad (x, y, w, h) with vertex colour VISOS(0x80,0x80,0x80,0x80) =
/// white, uv 0..1 (4096 in the 4.12 vertex format), blend 1: a straight copy
fn draw_background(px: &mut [u8], t: &DecodedTexture, x: f32, y: f32, w: f32, h: f32) {
    for j in 0..PLATE_H {
        for i in 0..PLATE_W {
            let (cx, cy) = (i as f32 + 0.5, j as f32 + 0.5);
            if cx < x || cy < y || cx >= x + w || cy >= y + h {
                continue;
            }
            let s = sample(t, (cx - x) / w, (cy - y) / h);
            let o = ((j * PLATE_W + i) * 4) as usize;
            for k in 0..4 {
                px[o + k] = s[k].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// a font quad: texture modulated by the D3DCOLOR, alpha blended (src alpha, inv src alpha)
fn draw_glyph(px: &mut [u8], page: &DecodedTexture, g: &PlateGlyph) {
    let [x0, y0, x1, y1] = g.rect;
    let c = [(g.colour >> 16) & 255, (g.colour >> 8) & 255, g.colour & 255, g.colour >> 24].map(|v| v as f32 / 255.0);
    for j in (y0.floor().max(0.0) as u32)..(y1.ceil().min(PLATE_H as f32) as u32) {
        for i in (x0.floor().max(0.0) as u32)..(x1.ceil().min(PLATE_W as f32) as u32) {
            let (cx, cy) = (i as f32 + 0.5, j as f32 + 0.5);
            if cx < x0 || cy < y0 || cx >= x1 || cy >= y1 {
                continue;
            }
            let u = g.uv[0] + (g.uv[2] - g.uv[0]) * (cx - x0) / (x1 - x0);
            let v = g.uv[1] + (g.uv[3] - g.uv[1]) * (cy - y0) / (y1 - y0);
            let s = sample(page, u, v);
            let a = s[3] / 255.0 * c[3];
            let o = ((j * PLATE_W + i) * 4) as usize;
            for k in 0..3 {
                let src = s[k] * c[k];
                px[o + k] = (src * a + px[o + k] as f32 * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// LicencePlateRender.cpp 0x0012DF40 — draw the plate into the target: background, then the text centred in the
/// 64 pixels (squeezed to 58 when wider), at y 11, in the style's colour
pub fn render_plate(bg: Option<&DecodedTexture>, plate: &LicencePlate, font: Option<&dyn PlateFont>) -> Vec<u8> {
    let mut px = vec![0u8; (PLATE_W * PLATE_H * 4) as usize];
    let style = plate.style.min(2) as usize;
    // dword_71FED8 / dword_71FEDC (never written: 0), dword_686BF4/686BF8 = 64, 32
    if let Some(t) = bg {
        draw_background(&mut px, t, 0.0, 0.0, PLATE_W as f32, PLATE_H as f32);
    }
    let Some(f) = font else { return px };
    let id = if plate.front_end { FONT_FRONT_END } else { FONT_GAME };
    let (Some(base), Some(page)) = (f.scale(id), f.page(id)) else { return px };
    let text: Vec<u16> = plate.text.encode_utf16().collect();
    let mut width = f.string_width(id, &text);
    let mut scale = base;
    if width > MAX_TEXT_WIDTH {
        scale = MAX_TEXT_WIDTH as f32 / width as f32 * base;
        width = MAX_TEXT_WIDTH;
    }
    let x = ((PLATE_W as i32 - width) as u32 as i64) as f32 * 0.5;
    let y = 16.0 - TEXT_Y_OFFSET;
    let colour = visos_colour(TEXT_COLOURS[style]);
    for g in f.draw_string_at(id, scale, x, y, &text, colour) {
        draw_glyph(&mut px, page, &g);
    }
    px
}

/// CarRender.cpp 0x00119900 -> LicencePlateRender 0x0012DF40: the target is redrawn while a car has a plate; here
/// only when the plate, the style or the font changes
fn licence_plate_render(
    mut commands: Commands,
    plate: Res<LicencePlate>,
    font: Option<Res<PlateFontHook>>,
    lpr: Option<ResMut<LicencePlateRender>>,
    cars: Query<(), With<CarPlatePrims>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<XboxStageMaterial>>,
) {
    let Some(mut lpr) = lpr else {
        if !cars.is_empty() {
            commands.insert_resource(init(&mut images, &mut materials));
        }
        return;
    };
    // 0x0012DC60: in game only the style at init is loaded; the front end picks it per render
    let style = plate.style.min(2) as usize;
    if plate.front_end || lpr.current.is_none() {
        lpr.current = Some(style);
    }
    let key = (lpr.current.unwrap_or(0) as u32, plate.text.clone(), plate.front_end, font.is_some());
    if lpr.built.as_ref() == Some(&key) {
        return;
    }
    let bg = lpr.current.and_then(|i| lpr.backgrounds[i].as_ref());
    let px = render_plate(bg, &plate, font.as_ref().map(|f| f.0.as_ref()));
    if let Some(mut img) = images.get_mut(&lpr.image) {
        img.data = Some(px);
    }
    lpr.built = Some(key);
}

/// LicencePlateRender.cpp 0x0012E1E0 / FuryInstance.cpp 0x0012AD70 — swap the plate primitives' shader for
/// customPlate
fn apply_custom_plate(mut commands: Commands, lpr: Option<Res<LicencePlateRender>>, cars: Query<(Entity, &CarPlatePrims), Without<PlateApplied>>) {
    let Some(lpr) = lpr else { return };
    for (car, p) in &cars {
        for &e in &p.0 {
            if let Ok(mut ec) = commands.get_entity(e) {
                ec.remove::<MeshMaterial3d<ProgramMaterial>>().insert(MeshMaterial3d(lpr.custom_plate.clone()));
            }
        }
        commands.entity(car).insert(PlateApplied);
    }
}

#[derive(Component)]
struct PlateApplied;

#[cfg(test)]
mod tests {
    use super::*;

    /// positive control: the three plate backgrounds are in `\VFX\misc.vcd`, and every car with a rear bumper
    /// carrying `plate01` is found
    #[test]
    fn plate_backgrounds_and_bumpers() {
        let v = visos::file::Vcd::open("\\VFX\\misc.vcd").unwrap();
        for n in PLATE_TEXTURES {
            let t = decode_any_vtf(v.get(&format!("{n}.vtf")).unwrap_or_else(|| panic!("no {n}"))).unwrap();
            assert!(t.width >= 16 && t.height >= 8, "{n}");
            let px = render_plate(Some(&t), &LicencePlate::default(), None);
            assert!(px.chunks(4).any(|p| p[3] > 0), "{n}: empty plate");
        }
        let (mut bumpers, mut plates) = (0, 0);
        let root = visos::file::game_dir().join("Vehicles");
        for city in ["LA", "London", "Tokyo"] {
            let Ok(rd) = std::fs::read_dir(root.join(city)) else { continue };
            for d in rd.flatten() {
                let car = d.file_name().to_string_lossy().to_string();
                let Ok(v) = visos::file::Vcd::open(&format!("\\Vehicles\\{city}\\{car}\\VEHICLEM.VCD")) else { continue };
                let Some(m) = v.get("r_bumper0.xmf").and_then(|d| crate::renderer::renderer_types::Model::parse(d).ok()) else { continue };
                bumpers += 1;
                let has = m.shaders.iter().any(|s| s.stages.first().is_some_and(|st| st.texture_name.to_ascii_lowercase().contains(PLATE_TEXTURE)));
                plates += has as usize;
            }
        }
        assert!(bumpers > 20, "{bumpers} rear bumpers");
        assert!(plates * 2 > bumpers, "{plates} of {bumpers} rear bumpers carry plate01");
    }
}
