//! `g_FadeRender` (object `flt_7190FC`; its functions sit in the DangerZoneRender.cpp range 0x00122FB0..0x00123420):
//! the full-screen colour fade of race starts, respawns, attract mode and the post-race sequence.
//!
//! API (core): `ResMut<FadeRender>` — `init` at CommonInit ("FadeRender Init"), `reset` at RestartRace, `start(t)` for
//! a fade over t seconds (negative = fade in from 0.999999), `set` / `alpha` / `fading`, and `update(clock, paused)`
//! once per game update (`g_FadeRender.Update`, GameManager::InGameUpdate with GameManager 0x000BF690's clock). The
//! quad is drawn every rendered frame by `fade_render_system` (GameManager 0x000B9010, after the lens spray, before
//! the OSD). The alpha moves with the 24 Hz update and is drawn unlerped, as the original.

use bevy::prelude::*;

use crate::full_screen_effects_manager_pc_xbox::{prog, FseImage, FseSet, FseShader, FullScreenEffects};

/// `dword_6869A0`: z of the quad (clamped to 0 by the sprite draw)
const FADE_Z: f32 = -1.0;
/// `0.999999f` start alpha of a fade-in
const FADE_IN_START: f32 = f32::from_bits(0x3F7F_FFEF);

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct FadeRender {
    /// +0
    pub alpha: f32,
    /// +4 alpha per second
    pub rate: f32,
    /// +8 colour bytes b, g, r, a (VISOS(0, 0, 0, 0) at init; nothing sets it)
    pub colour: [u8; 4],
    /// +12 texture handle (0)
    pub texture: u32,
    /// +16 clock of the last update
    pub last: f32,
    /// `dword_7190F4` "Graphics.Fullscreen Blur.Blur" tweak and its last seen value `dword_7190F8`
    pub blur: f32,
    blur_seen: f32,
}

impl FadeRender {
    /// DangerZoneRender.cpp 0x00122FB0
    pub fn init(&mut self) {
        self.texture = 0;
        self.colour = crate::car_damage_fx::visos_colour(0, 0, 0, 0);
        self.rate = 0.0;
        self.alpha = 0.0;
        self.last = 0.0;
    }

    /// DangerZoneRender.cpp 0x00123030
    pub fn reset(&mut self) {
        self.alpha = 0.0;
        self.rate = 0.0;
    }

    /// DangerZoneRender.cpp 0x00123070 — fade over `seconds` (rate 1/seconds); a negative time fades in from 0.999999
    pub fn start(&mut self, seconds: f32) {
        self.rate = 1.0 / seconds;
        if self.rate < 0.0 {
            self.alpha = FADE_IN_START;
        }
    }

    /// DangerZoneRender.cpp 0x001230C0
    pub fn set(&mut self, alpha: f32) {
        self.alpha = alpha;
    }

    /// FontHelper 0x000B3710
    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    /// DangerZoneRender.cpp 0x001231F0
    pub fn fading(&self) -> bool {
        self.rate.abs() > 0.0
    }

    /// DangerZoneRender.cpp 0x00123100 — advance by the clock's step since the last call (not while `paused`, GameBridge 0x00097E90); the
    /// fade stops at 0 or 1
    pub fn update(&mut self, clock: f32, paused: bool) {
        let mut dt = clock - self.last;
        self.last = clock;
        if dt < 0.0 {
            dt = 0.0;
        }
        if paused {
            return;
        }
        self.alpha = self.rate * dt + self.alpha;
        if self.alpha >= 0.0 {
            if self.alpha > 1.0 {
                self.alpha = 1.0;
                self.rate = 0.0;
            }
        } else {
            self.rate = 0.0;
            self.alpha = 0.0;
        }
    }

    /// DangerZoneRender.cpp 0x00123250 — the D3DCOLOR of the quad when it is drawn: VISOS(_ftol(alpha · 128), r, g, b)
    pub fn quad_colour(&mut self) -> Option<u32> {
        // a changed blur tweak calls the empty 0x001230E0
        if self.blur != self.blur_seen {
            self.blur_seen = self.blur;
        }
        if self.alpha <= 0.0 {
            return None;
        }
        let a = (self.alpha * 128.0) as i32 as u8;
        let c = crate::car_damage_fx::visos_colour(a, self.colour[2], self.colour[1], self.colour[0]);
        Some(u32::from_le_bytes(c))
    }
}

/// GameManager 0x000B9010 -> 0x00123250: the 512² sprite (Renderer 0x002AA770 begin-sprites state: SRCALPHA /
/// INVSRCALPHA, z off, MODULATE(texture 0, diffuse)) over the back buffer, drawn here with the Colour pair (oD0 = the
/// quad colour) and blend 2
fn fade_render_system(mut fse: ResMut<FullScreenEffects>, fade: Option<ResMut<FadeRender>>) {
    let Some(mut fade) = fade else { return };
    let Some(colour) = fade.quad_colour() else { return };
    if !fse.active {
        return;
    }
    // after the view loop: GameManager 0x000B3C30 on view 0 (the whole screen, also for two players)
    let view = fse.viewport();
    fse.set_viewport(crate::full_screen_effects_manager_pc_xbox::FULL_RECT);
    fse.draw(FseImage::BackBuffer, FseImage::BackBuffer, FseShader::new(2, false), prog::COLOUR, None, None, FADE_Z.max(0.0), colour);
    fse.set_viewport(view);
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FadeSet;

pub struct FadeRenderPlugin;

impl Plugin for FadeRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FadeRender>().add_systems(
            PostUpdate,
            fade_render_system.in_set(FseSet::Build).in_set(FadeSet).after(crate::lens_spray_manager::LensSpraySet),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fade_out_then_in() {
        let mut f = FadeRender::default();
        f.init();
        f.start(0.5);
        f.update(0.0, false);
        f.update(0.25, false);
        assert!((f.alpha - 0.5).abs() < 1e-6);
        f.update(1.0, false);
        assert_eq!((f.alpha, f.fading()), (1.0, false));
        assert_eq!(f.quad_colour(), Some(0xFF00_0000));
        f.start(-1.0);
        assert_eq!(f.alpha, FADE_IN_START);
        f.update(3.0, false);
        assert_eq!((f.alpha, f.fading(), f.quad_colour()), (0.0, false, None));
    }
}
