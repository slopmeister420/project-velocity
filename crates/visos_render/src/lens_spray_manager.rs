//! `g_LensSprayManager` (object `unk_71EC50`): water drops on the camera lens in wet races. Init 0x0012D400 / clear
//! 0x0012D490 / update 0x0012D520 / spawn 0x0012D960 sit in the GlobalMatInputResolver.cpp range, the shaders
//! 0x0015F9F0 and the draw 0x0015FB30 / 0x0015FC60 in GameShaderProgramManager_PC_XBOX's (file name unknown).
//!
//! API (core): `ResMut<LensSpray>` — `init` at GameManager::InGameInit, `update(&LensSprayInput)` once per game update
//! (`g_LensSprayManager.Update`, InGameUpdate after the skid smoke), `splash(1.0)` where the puddle code
//! (LightStreak 0x001326A0 / 0x00132A00 -> 0x00133910) splashes the camera's car. Drawn every rendered frame by
//! `lens_spray_system` (GameManager 0x000B9010 render flag 0x10000000, after the zone FX and before the fade).

use bevy::prelude::*;
use visos::handy::crt_rand::CrtRand;

use crate::full_screen_effects_manager_pc_xbox::{prog, FseImage, FseSet, FseShader, FseVertex, FullScreenEffects, OFF_SCREEN_QUARTER, WHITE};

pub const MAX_DROPS: usize = 128;
/// the original's pretransformed screen (`1132462080` = 256 = its centre)
const SCREEN: f32 = 512.0;
/// life lost per game update (`0.041666668f`)
const LIFE_STEP: f32 = 0.041666668;
/// camera speed: distance moved per update × 0.5; strength = speed / 2.5, at least 0.2 (VsFrameTimer 0x00057F90 =
/// max) and at most 1 from speed 1 on, else 0
const SPEED_SCALE: f32 = 0.5;
const SPEED_DIV: f32 = 2.5;
const STRENGTH_MIN: f32 = f32::from_bits(0x3E4C_CCCD);
/// rain: a burst every more than `dword_686B94` − `_ftol(dword_686BB4 · strength)` updates of
/// `dword_686B98` · `flt_725E20` (bss, never written: 0) + `dword_686BB8` · strength drops
const RAIN_PERIOD: i32 = 8;
const RAIN_PERIOD_SCALE: u32 = 5;
const RAIN_BASE: u32 = 1;
const RAIN_BASE_SCALE: f32 = 0.0;
const RAIN_COUNT: u32 = 3;
/// splash: min(speed / 2.5 · `dword_686BC4` · level, `flt_686BC8`) drops, level −`dword_686BCC` per update
const SPLASH_SCALE: f32 = 40.0;
const SPLASH_MAX: f32 = 10.0;
const SPLASH_DECAY: f32 = 0.25;
/// drop radius ranges and lives (`flt_686B9C`..`flt_686BA0`, `dword_686BA4` = 3 for rain; `flt_686BD0`..`flt_686BD4`,
/// `dword_686BD8` = 1.0 for splashes)
const RAIN_RADIUS: (f32, f32) = (2.0, 10.0);
const RAIN_LIFE: u32 = 3;
const SPLASH_RADIUS: (f32, f32) = (3.0, 4.0);
const SPLASH_LIFE: f32 = 1.0;
/// `dword_686BB0` segments per drop; `flt_686BA8` / `flt_686BAC` rim uv spread in x / y
const SEGMENTS: u32 = 8;
const RIM_U: f32 = 35.0;
const RIM_V: f32 = 20.0;

/// one drop, 32 bytes
#[derive(Clone, Copy, Debug, Default)]
pub struct Drop {
    /// +0 position in the 512² screen
    pub pos: Vec2,
    /// +16 life left, +20 full life, +24 radius
    pub life: f32,
    pub max_life: f32,
    pub radius: f32,
    /// +28
    pub active: bool,
}

/// what the update reads each game update (core fills it)
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct LensSprayInput {
    /// the current camera's position (`VsRender 0x0005ACD0` -> 0x00023E00), `None` = no camera
    pub camera: Option<Vec3>,
    /// forward speed of the camera's car (camera +96 -> VsAIRaceVehicle 0x00024870 = velocity · its x axis)
    pub forward_speed: f32,
    /// `VsAudio::IsTrackWet`
    pub wet: bool,
}

#[derive(Resource, Clone, Debug)]
pub struct LensSpray {
    pub drops: [Drop; MAX_DROPS],
    /// +4096
    pub enabled: bool,
    /// +4100 splash level
    pub splash: f32,
    /// `dword_71EC4C` updates since the last rain burst
    pub counter: i32,
    /// `dword_71FC58` camera position of the last update
    pub last: Vec3,
    /// VsRandUnit (the game's CRT `rand`)
    pub rand: CrtRand,
}

impl Default for LensSpray {
    fn default() -> Self {
        Self { drops: [Drop::default(); MAX_DROPS], enabled: false, splash: 0.0, counter: 0, last: Vec3::ZERO, rand: CrtRand::default() }
    }
}

impl LensSpray {
    /// GlobalMatInputResolver.cpp 0x0012D400 (shaders 0x0015F9F0 are built by the draw)
    pub fn init(&mut self) {
        if !self.enabled {
            self.clear();
            self.enabled = true;
            self.splash = 0.0;
        }
    }

    /// GlobalMatInputResolver.cpp 0x0012D490
    pub fn clear(&mut self) {
        self.splash = 0.0;
        for d in &mut self.drops {
            d.active = false;
        }
    }

    /// LightStreak.cpp 0x00133910 (+4100)
    pub fn splash(&mut self, level: f32) {
        self.splash = level;
    }

    /// GlobalMatInputResolver.cpp 0x0012D960 — first free drop at a random spot; `splash` = the short big ones
    pub fn spawn(&mut self, splash: bool) {
        let Some(d) = self.drops.iter_mut().find(|d| !d.active) else { return };
        d.pos.x = self.rand.rand_unit() as f32 * SCREEN;
        d.pos.y = self.rand.rand_unit() as f32 * SCREEN;
        if splash {
            d.radius = self.rand.rand_unit() as f32 * (SPLASH_RADIUS.1 - SPLASH_RADIUS.0) + SPLASH_RADIUS.0;
            d.life = SPLASH_LIFE;
            d.max_life = SPLASH_LIFE;
        } else {
            d.radius = self.rand.rand_unit() as f32 * (RAIN_RADIUS.1 - RAIN_RADIUS.0) + RAIN_RADIUS.0;
            d.life = RAIN_LIFE as f32;
            d.max_life = RAIN_LIFE as f32;
        }
        d.active = true;
    }

    /// GlobalMatInputResolver.cpp 0x0012D520 — one game update: drops age and slide outwards with the camera's speed, splashes and rain add
    /// new ones
    pub fn update(&mut self, input: &LensSprayInput) {
        if !self.enabled {
            return;
        }
        let Some(cam) = input.camera else { return };
        let mut speed = self.last.distance(cam);
        speed *= SPEED_SCALE;
        self.last = cam;
        if input.forward_speed < 0.0 {
            speed = 0.0;
        }
        let centre = Vec2::splat(SCREEN / 2.0);
        for d in self.drops.iter_mut().filter(|d| d.active) {
            d.life -= LIFE_STEP;
            if d.life >= 0.0 {
                let dir = (d.pos - centre).normalize_or_zero();
                d.pos += dir * speed;
            } else {
                d.active = false;
            }
        }
        let mut strength = speed / SPEED_DIV;
        let splash_k = speed / SPEED_DIV;
        if speed >= 1.0 {
            strength = strength.max(STRENGTH_MIN).min(1.0);
        } else {
            strength = 0.0;
        }
        let period = RAIN_PERIOD - (RAIN_PERIOD_SCALE as f32 * strength) as i32;
        if self.splash > 0.0 && splash_k > 0.0 {
            let n = (splash_k * SPLASH_SCALE * self.splash).min(SPLASH_MAX) as i32;
            for _ in 0..n {
                self.spawn(true);
            }
            self.splash -= SPLASH_DECAY;
        }
        if input.wet {
            self.counter += 1;
            if self.counter > period {
                let base = RAIN_BASE as f32 * RAIN_BASE_SCALE;
                let n = RAIN_COUNT as f32 * strength + base;
                for _ in 0..n as i32 {
                    self.spawn(false);
                }
                self.counter = 0;
            }
        }
    }

    /// GameShaderProgramManager_PC_XBOX.cpp 0x0015FC60 — one drop as 8 fan triangles (rim, next rim, centre; one 0x0015E7B0 draw each): positions around
    /// the drop, the rims sampling the copy mirrored and spread (35 / 20 × radius), the centre the spot itself;
    /// colour VISOS(_ftol(life / full · 128), 0x80, 0x80, 0x80). `size` = the copy's size in texels.
    pub fn drop_triangles(d: &Drop, size: Vec2, out: &mut Vec<[FseVertex; 3]>) {
        let (x, y, r) = (d.pos.x, d.pos.y, d.radius);
        let mut a = 0.0f32;
        let step = 6.2831855f32 / SEGMENTS as f32;
        let alpha = (d.life / d.max_life * 128.0) as i32 as u8;
        let colour = u32::from_le_bytes(crate::car_damage_fx::visos_colour(alpha, 0x80, 0x80, 0x80));
        let inv = f32::from_bits(0x3B00_0000);
        let rim = |a: f32| {
            let px = a.sin() * r + x;
            let py = a.cos() * r + y;
            let u = ((x - a.sin() * r * RIM_U) * inv).clamp(0.0, 1.0);
            let v = ((y - a.cos() * r * RIM_V) * inv).clamp(0.0, 1.0);
            FseVertex { pos: Vec2::new(px, py) / SCREEN, uv: Vec2::new(u * size.x, v * size.y), colour }
        };
        for _ in 0..SEGMENTS {
            let c = FseVertex { pos: Vec2::new(x, y) / SCREEN, uv: Vec2::new(x * inv * size.x, y * inv * size.y), colour };
            out.push([rim(a), rim(a + step), c]);
            a += step;
        }
    }

    /// GameShaderProgramManager_PC_XBOX.cpp 0x0015FB30 — copy the back buffer into a quarter buffer ("lensscreencopy": blend 1, clamp, z off, Copy), then
    /// every live drop from it with "lensspray" (blend 2, clamp, z off, stage alpha SELECTARG1 = diffuse)
    pub fn render(&self, fse: &mut FullScreenEffects) {
        if !self.enabled {
            return;
        }
        let Some(quarter) = fse.get_off_screen_buffer(OFF_SCREEN_QUARTER) else { return };
        fse.draw(quarter, FseImage::BackBuffer, FseShader::new(1, false), prog::COPY, None, None, 0.0, WHITE);
        let size = fse.image_size(quarter).as_vec2();
        let mut tris = Vec::new();
        for d in self.drops.iter().filter(|d| d.active) {
            Self::drop_triangles(d, size, &mut tris);
        }
        let shader = FseShader { ffp: crate::full_screen_effects_manager_pc_xbox::FFP_DIFFUSE_ALPHA, ..FseShader::new(2, false) };
        fse.draw_triangles(FseImage::BackBuffer, quarter, shader, prog::COPY, &tris);
        fse.release_off_screen_buffer(quarter);
    }
}

/// GameManager 0x000B9010 flag 0x10000000: 0x0015FB30 inside the view loop — the one drop set drawn once per view in
/// that view's viewport (each copying the whole back buffer, as the FSE quads sample it)
fn lens_spray_system(mut fse: ResMut<FullScreenEffects>, spray: Option<Res<LensSpray>>, split: Option<Res<crate::view_render::SplitScreen>>) {
    let Some(spray) = spray else { return };
    if !fse.active {
        return;
    }
    match split.filter(|s| s.on()) {
        Some(s) => {
            let restore = fse.viewport();
            for (_, r) in &s.views {
                fse.set_viewport(r.to_array());
                spray.render(&mut fse);
            }
            fse.set_viewport(restore);
        }
        None => spray.render(&mut fse),
    }
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LensSpraySet;

pub struct LensSprayPlugin;

impl Plugin for LensSprayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LensSpray>().init_resource::<LensSprayInput>().add_systems(
            PostUpdate,
            lens_spray_system.in_set(FseSet::Build).in_set(LensSpraySet).after(crate::zone_render_pc_xbox::ZoneFxSet),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rain_bursts_and_drops_slide_out() {
        let mut s = LensSpray::default();
        s.init();
        let mut cam = Vec3::ZERO;
        let mut inp = LensSprayInput { camera: Some(cam), forward_speed: 10.0, wet: true };
        // 2 m per update -> speed 1, strength 0.4: period 8 - 2 = 6, bursts of _ftol(1.2) = 1 drop
        for _ in 0..7 {
            cam.x += 2.0;
            inp.camera = Some(cam);
            s.update(&inp);
        }
        assert_eq!(s.drops.iter().filter(|d| d.active).count(), 1);
        let d = *s.drops.iter().find(|d| d.active).unwrap();
        assert_eq!((d.life, d.max_life), (3.0, 3.0));
        assert!((2.0..10.0).contains(&d.radius));
        cam.x += 2.0;
        inp.camera = Some(cam);
        s.update(&inp);
        let e = s.drops[0];
        let out = (d.pos - Vec2::splat(256.0)).normalize();
        assert!((e.pos - (d.pos + out)).length() < 1e-3);
        // splash: level 1 at speed 1 -> min(0.4 · 40, 10) = 10 big drops
        s.splash(1.0);
        cam.x += 2.0;
        inp.camera = Some(cam);
        s.update(&inp);
        assert_eq!(s.drops.iter().filter(|d| d.active && d.max_life == 1.0).count(), 10);
        assert_eq!(s.splash, 0.75);
        let mut tris = Vec::new();
        LensSpray::drop_triangles(&e, Vec2::new(160.0, 120.0), &mut tris);
        assert_eq!(tris.len(), 8);
    }
}
