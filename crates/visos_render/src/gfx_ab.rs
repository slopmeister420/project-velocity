//! debug harness (no original counterpart): frame-matched A/B captures of the primary window.
//! `VELOCITY_GFX_AB=<seconds>[,<prefix>]`: `seconds` after the race's RainSurface init the next frames run the
//! `PHASES` one per frame (each turns some draws off) and capture the window; the stats against frame 0 (and the
//! frame-to-frame noise, 0 vs 2) go to the log, amplified diffs to `research/screens/<prefix>_diff_<phase>.png`.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::sync::{Arc, Mutex};

/// what a phase turns off
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbOff {
    None,
    /// `rain_surface_system` (= `VELOCITY_RAIN_SURFACE=0`)
    RainSurface,
    /// `EffectDraw` batches with the PARTTEX `tyremist` texture (tyre spray "mistspray", puddle "splash")
    Mist,
    /// every `EffectDraw` batch (particles, spray, rain, skids, flares ...)
    Effects,
    /// RainSurface and every `EffectDraw` batch
    All,
    /// the beam lights' volume texture swapped for the black placeholder (only the headlight pool goes)
    BeamTexture,
    /// the instance primitives on their plain stage materials instead of the "HeadLightBeam(Lit)" override
    BeamOverride,
}

/// the beam phases sit between plain frames (6/8/10) so each can be held against both neighbours
pub const PHASES: [(&str, AbOff); 11] = [
    ("0_on", AbOff::None),
    ("1_rs_off", AbOff::RainSurface),
    ("2_on", AbOff::None),
    ("3_mist_off", AbOff::Mist),
    ("4_fx_off", AbOff::Effects),
    ("5_all_off", AbOff::All),
    ("6_on", AbOff::None),
    ("7_beam_off", AbOff::BeamTexture),
    ("8_on", AbOff::None),
    ("9_override_off", AbOff::BeamOverride),
    ("10_on", AbOff::None),
];

#[derive(Resource, Default)]
pub struct GfxAb {
    at: Option<f32>,
    prefix: String,
    /// real time of the RainSurface init
    armed: Option<f32>,
    /// current phase index (None = idle / done)
    pub phase: Option<usize>,
    done: bool,
    shots: Arc<Mutex<Vec<(usize, Image)>>>,
    reported: bool,
}

impl GfxAb {
    fn off(&self) -> AbOff {
        self.phase.map(|p| PHASES[p].1).unwrap_or(AbOff::None)
    }
    /// `VELOCITY_GFX_AB` is set
    pub fn enabled(&self) -> bool {
        self.at.is_some()
    }
    pub fn beam_texture_off(&self) -> bool {
        self.off() == AbOff::BeamTexture
    }
    pub fn beam_override_off(&self) -> bool {
        self.off() == AbOff::BeamOverride
    }
    pub fn rain_surface_off(&self) -> bool {
        matches!(self.off(), AbOff::RainSurface | AbOff::All)
    }
    /// is an effect batch with this texture hidden this frame
    pub fn effect_off(&self, t: &crate::effect_draw::EffectTexture) -> bool {
        match self.off() {
            AbOff::Effects | AbOff::All => true,
            AbOff::Mist => matches!(t, crate::effect_draw::EffectTexture::Particle(n) if n.eq_ignore_ascii_case("tyremist")),
            _ => false,
        }
    }
}

fn ab_step(
    mut commands: Commands,
    mut ab: ResMut<GfxAb>,
    time: Res<Time<Real>>,
    rs: Option<Res<crate::rain_surface::RainSurface>>,
) {
    let Some(at) = ab.at else { return };
    let now = time.elapsed_secs();
    if ab.armed.is_none() {
        if rs.is_some_and(|r| r.on || r.wet) {
            ab.armed = Some(now);
            log::info!("gfx ab: armed, capturing in {at} s");
        }
        return;
    }
    if ab.done {
        report(&mut ab);
        return;
    }
    let next = match ab.phase {
        None if now - ab.armed.unwrap() >= at => 0,
        None => return,
        Some(p) => p + 1,
    };
    if next >= PHASES.len() {
        ab.phase = None;
        ab.done = true;
        return;
    }
    ab.phase = Some(next);
    let path = std::path::PathBuf::from(format!("research/screens/{}_{}.png", ab.prefix, PHASES[next].0));
    let shots = ab.shots.clone();
    log::info!("gfx ab: frame {} {} -> {}", next, PHASES[next].0, path.display());
    commands.spawn(Screenshot::primary_window()).observe(move |c: On<ScreenshotCaptured>| {
        if let Ok(d) = c.image.clone().try_into_dynamic() {
            let _ = d.to_rgb8().save(&path);
        }
        shots.lock().unwrap().push((next, c.image.clone()));
    });
}

/// per-pixel stats of `b` against `a`: (mean abs diff per channel, pixels differing by > 8, mean luma a, mean luma b)
pub fn diff_stats(a: &[[u8; 3]], b: &[[u8; 3]]) -> (f32, usize, f32, f32) {
    let mut sum = 0u64;
    let mut changed = 0;
    let (mut la, mut lb) = (0f64, 0f64);
    for (p, q) in a.iter().zip(b) {
        let d: u32 = (0..3).map(|i| (p[i] as i32 - q[i] as i32).unsigned_abs()).sum();
        sum += d as u64;
        if (0..3).any(|i| (p[i] as i32 - q[i] as i32).abs() > 8) {
            changed += 1;
        }
        la += (p[0] as f64 + p[1] as f64 + p[2] as f64) / 3.0;
        lb += (q[0] as f64 + q[1] as f64 + q[2] as f64) / 3.0;
    }
    let n = a.len().max(1) as f64;
    ((sum as f64 / n / 3.0) as f32, changed, (la / n) as f32, (lb / n) as f32)
}

fn report(ab: &mut GfxAb) {
    if ab.reported {
        return;
    }
    let shots = ab.shots.lock().unwrap();
    if shots.len() < PHASES.len() {
        return;
    }
    // (phase, width, height, rgb pixels)
    let mut rgb: Vec<(usize, u32, u32, Vec<[u8; 3]>)> = Vec::new();
    for (k, img) in shots.iter() {
        if let Ok(d) = img.clone().try_into_dynamic() {
            let b = d.to_rgb8();
            rgb.push((*k, b.width(), b.height(), b.pixels().map(|p| p.0).collect()));
        }
    }
    drop(shots);
    ab.reported = true;
    rgb.sort_by_key(|s| s.0);
    let Some((_, w, h, base)) = rgb.iter().find(|s| s.0 == 0) else { return };
    for (k, _, _, px) in rgb.iter().filter(|s| s.0 != 0) {
        let (mean, changed, la, lb) = diff_stats(base, px);
        log::info!(
            "gfx ab: {} vs 0_on: mean |d| {mean:.2}, {changed} px > 8 of {}, luma {la:.1} -> {lb:.1}",
            PHASES[*k].0,
            base.len()
        );
        // |a − b| × 4 as an image
        let mut data = Vec::with_capacity(base.len() * 4);
        for (p, q) in base.iter().zip(px) {
            for i in 0..3 {
                data.push(((p[i] as i32 - q[i] as i32).abs() * 4).min(255) as u8);
            }
            data.push(255);
        }
        let img = Image::new(
            bevy::render::render_resource::Extent3d { width: *w, height: *h, depth_or_array_layers: 1 },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            // (try_into_dynamic takes Rgba8UnormSrgb, not Rgba8Unorm; the bytes go through as they are)
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::all(),
        );
        if let Ok(d) = img.try_into_dynamic() {
            let _ = d.to_rgb8().save(format!("research/screens/{}_diff_{}.png", ab.prefix, PHASES[*k].0));
        }
    }
}

pub struct GfxAbPlugin;

impl Plugin for GfxAbPlugin {
    fn build(&self, app: &mut App) {
        let mut ab = GfxAb::default();
        if let Ok(v) = std::env::var("VELOCITY_GFX_AB") {
            let mut it = v.split(',');
            ab.at = it.next().and_then(|s| s.trim().parse().ok());
            ab.prefix = it.next().unwrap_or("gfx_ab").trim().to_string();
        }
        app.insert_resource(ab).add_systems(Update, ab_step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_stats_counts() {
        let a = vec![[10u8, 10, 10]; 4];
        let mut b = a.clone();
        b[0] = [30, 10, 10];
        let (mean, changed, la, lb) = diff_stats(&a, &b);
        assert_eq!(changed, 1);
        assert!((mean - 20.0 / 12.0).abs() < 1e-4);
        assert!(lb > la);
    }

    #[test]
    fn phases_gate() {
        let mut ab = GfxAb::default();
        let mist = crate::effect_draw::EffectTexture::Particle("tyremist".into());
        let other = crate::effect_draw::EffectTexture::Misc("spec2".into());
        ab.phase = Some(3);
        assert!(ab.effect_off(&mist) && !ab.effect_off(&other) && !ab.rain_surface_off());
        ab.phase = Some(1);
        assert!(ab.rain_surface_off() && !ab.effect_off(&mist));
        ab.phase = Some(5);
        assert!(ab.rain_surface_off() && ab.effect_off(&other));
    }
}
