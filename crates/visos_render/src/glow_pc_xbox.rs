//! Glow (bloom), 0x0015EF70..0x0015F9B0 — the file after GameShaderProgramManager_PC_XBOX.cpp (no asserts; name
//! guessed from its "Glow*" shaders). Object `this` (0x0015F360): +0 128, +4 on, +8 blur passes, +12 display blur,
//! +13 stencil on read, +16 colour intensity, +20 shader set (0x0015EF70: GlowStencil, GlowScreenCopy, GlowBlur1,
//! GlowBlur2, GlowFinal, +20 the borrowed half buffer).
//!
//! The glow stencil (bit 1) is emulated in frame-buffer alpha (`shader_manager::apply_glow_stencil`), so the
//! GlowStencil pass needs no draw: the alpha it would clear is already 0.

use bevy::prelude::*;

use crate::full_screen_effects_manager_pc_xbox::{prog, FseImage, FseSet, FseShader, FullScreenEffects, OFF_SCREEN_HALF, WHITE};
use crate::global_mat_input_resolver::gmi;

/// "Graphics.Bloom" tweaks (0x0015F820), defaults from 0x0015F360
#[derive(Resource, Clone, Copy, Debug)]
pub struct GlowTweaks {
    /// +4 "On/Off"
    pub on: bool,
    /// +8 "Num Blur Passes" (0..10)
    pub passes: u32,
    /// +16 "Colour Intensity" (0..1)
    pub intensity: f32,
    /// +12 "Display blur On/Off" (debug text 0x000C8F20, not drawn)
    pub display_blur: bool,
    /// +13 "Use stencil on read On/Off"
    pub stencil_on_read: bool,
}

impl Default for GlowTweaks {
    /// GameShaderProgramManager_PC_XBOX.cpp 0x0015F360
    fn default() -> Self {
        Self { on: true, passes: 5, intensity: f32::from_bits(0x3e99999a), display_blur: false, stencil_on_read: true }
    }
}

/// "GlowScreenCopy" (blend 1, z off, alpha test func 6 ref 0 — passes everything the cleared buffer would not
/// differ from) and "GlowFinal" (blend 14 = one + one, z off)
const GLOW_SCREEN_COPY: FseShader = FseShader::new(1, false);
const GLOW_FINAL: FseShader = FseShader::new(14, false);

/// GameShaderProgramManager_PC_XBOX.cpp 0x0015F600 — the bloom source: GlowStencil clears back-buffer alpha off the glow stencil (emulated, see the
/// module doc; with "stencil on read" off the original keeps every pixel's alpha, which the emulation cannot), then
/// the cleared half buffer gets BloomStart (rgb·4 · a·a·intensity) of the view rectangle
pub fn screen_copy(fse: &mut FullScreenEffects, half: FseImage, rect: [f32; 4]) {
    // a3 = the view rectangle (pixels / back buffer size): the whole screen for one player, the view's half for two
    fse.draw(half, FseImage::BackBuffer, GLOW_SCREEN_COPY, prog::BLOOM_START, Some(rect), None, 0.0, WHITE);
}

/// GameShaderProgramManager_PC_XBOX.cpp 0x0015F4B0 — GameManager 0x000B9010 calls it after the barriers (render flag 0x2000), or after the stateful
/// particles when `byte_6F5B7F` (flag 0x1000000), once per view with the view (`fse`'s viewport = the view's)
pub fn render(fse: &mut FullScreenEffects, t: &GlowTweaks) {
    if !t.on {
        return;
    }
    let rect = fse.viewport();
    // 0x0015F2D0: borrow a half buffer
    let Some(half) = fse.get_off_screen_buffer(OFF_SCREEN_HALF) else { return };
    fse.set_vector(gmi::INTENSITY, Vec4::splat(t.intensity));
    screen_copy(fse, half, rect);
    fse.blur_passes(half, half, t.passes, true);
    // 0x002EEE20 back onto the back buffer re-applies its (the view's) viewport
    fse.draw(FseImage::BackBuffer, half, GLOW_FINAL, prog::COPY, None, None, 0.0, WHITE);
    fse.release_off_screen_buffer(half);
}

/// GameManager 0x000B9010: the bloom inside each active view in turn (flags -1 = `dword_685E84` on every world view)
fn glow_system(mut fse: ResMut<FullScreenEffects>, tweaks: Res<GlowTweaks>, split: Option<Res<crate::view_render::SplitScreen>>) {
    if !fse.active {
        return;
    }
    let player0 = fse.viewport();
    match split.filter(|s| s.on()) {
        Some(s) => {
            for (_, r) in &s.views {
                fse.set_viewport(r.to_array());
                render(&mut fse, &tweaks);
            }
            fse.set_viewport(player0);
        }
        None => render(&mut fse, &tweaks),
    }
}

/// bloom runs before motion blur (GameManager 0x000B9010 order)
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GlowSet;

pub struct GlowPlugin;

impl Plugin for GlowPlugin {
    fn build(&self, app: &mut App) {
        let t = GlowTweaks { on: std::env::var("VELOCITY_BLOOM").map(|v| v != "0").unwrap_or(true), ..default() };
        app.insert_resource(t).add_systems(PostUpdate, glow_system.in_set(FseSet::Build).in_set(GlowSet));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let t = GlowTweaks::default();
        assert_eq!((t.on, t.passes, t.stencil_on_read), (true, 5, true));
        assert!((t.intensity - 0.3).abs() < 1e-7);
    }

    /// GameShaderProgramManager_PC_XBOX.cpp 0x0015F4B0 with 5 passes: BloomStart, 5 x (Bloom to quarter, Copy back), GlowFinal = 12 draws
    #[test]
    fn pass_list() {
        let mut fse = FullScreenEffects::default();
        fse.active = true;
        fse.test_buffers();
        render(&mut fse, &GlowTweaks::default());
        let p = fse.queued();
        assert_eq!(p.len(), 12);
        assert_eq!(p[0].program, prog::BLOOM_START);
        assert_eq!(p[1].program, prog::BLOOM);
        assert_eq!(p[1].target, FseImage::OffScreen(2));
        assert_eq!(p[2].program, prog::COPY);
        assert_eq!(p[2].target, FseImage::OffScreen(0));
        assert_eq!(p[11].shader.blend, 14);
        assert_eq!(p[11].target, FseImage::BackBuffer);
        assert!((p[0].vectors[gmi::INTENSITY].x - 0.3).abs() < 1e-7);
    }

    /// two views: 0x0015F600 reads each view's rect of the back buffer, GlowFinal lands in that view's viewport;
    /// the off-screen passes keep their whole buffer
    #[test]
    fn per_view_bloom() {
        let mut fse = FullScreenEffects::default();
        fse.active = true;
        fse.test_buffers();
        let (top, bottom) = ([0.0, 0.0, 1.0, 0.5], [0.0, 0.5, 1.0, 0.5]);
        for r in [top, bottom] {
            fse.set_viewport(r);
            render(&mut fse, &GlowTweaks::default());
        }
        let p = fse.queued();
        assert_eq!(p.len(), 24);
        assert_eq!((p[0].src_rect, p[0].viewport), (top, crate::full_screen_effects_manager_pc_xbox::FULL_RECT));
        assert_eq!((p[11].target, p[11].viewport), (FseImage::BackBuffer, top));
        assert_eq!(p[12].src_rect, bottom);
        assert_eq!(p[23].viewport, bottom);
    }
}
