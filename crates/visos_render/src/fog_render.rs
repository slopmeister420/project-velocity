//! fog renderer (no source file of its own in the debug build; its functions sit in the
//! Render\Pc_xbox\CarShaderManager_PC_XBOX.cpp range at 0x0015A470..0x0015ADB0). Fixed-function D3D fog set up from
//! the track's Setup.txt fog block.

use bevy::math::Vec3;

/// fog values of the track setup (`byte_6F5C78` +0x24..+0x5D), filled by GameDataLoading.cpp 0x0009E300.
#[derive(Clone, Debug, PartialEq)]
pub struct SetupFog {
    pub colour_top: Vec3,    // +0x24
    pub colour_bottom: Vec3, // +0x34
    pub distance: f32,       // +0x44
    pub alpha_top: f32,      // +0x48
    pub alpha_bottom: f32,   // +0x4C
    pub top_elevation: f32,  // +0x50
    pub bot_elevation: f32,  // +0x54
    pub unk58: f32,          // +0x58 always 1.0 once a fog block is read
    pub sky_fogged: bool,    // +0x5C
    pub gradient: bool,      // +0x5D
}

impl Default for SetupFog {
    /// GameDataLoading.cpp 0x0009E300 defaults
    fn default() -> Self {
        Self {
            colour_top: Vec3::new(f32::from_bits(0x3D4C_CCCD), f32::from_bits(0x3CA3_D70A), 0.0),
            colour_bottom: Vec3::ZERO,
            distance: 1.0,
            alpha_top: f32::from_bits(0x3F26_6666),
            alpha_bottom: f32::from_bits(0x3F26_6666),
            top_elevation: f32::from_bits(0x3E99_999A),
            bot_elevation: f32::from_bits(0xBDCC_CCCD),
            unk58: 0.0,
            sky_fogged: false,
            gradient: false,
        }
    }
}

/// `VsTextFile` token stream: quoted strings and `( a, b, ... )` groups in file order (labels are skipped).
#[derive(Debug, Clone, PartialEq)]
pub enum SetupToken {
    Str(String),
    Values(Vec<f32>),
}

pub fn setup_tokens(text: &str) -> Vec<SetupToken> {
    let mut out = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with("//") {
            continue;
        }
        let mut rest = l;
        while !rest.is_empty() {
            match (rest.find('"'), rest.find('(')) {
                (Some(q), p) if p.is_none_or(|p| q < p) => {
                    let Some(e) = rest[q + 1..].find('"') else { break };
                    out.push(SetupToken::Str(rest[q + 1..q + 1 + e].to_string()));
                    rest = &rest[q + e + 2..];
                }
                (_, Some(p)) => {
                    let Some(e) = rest[p..].find(')') else { break };
                    out.push(SetupToken::Values(rest[p + 1..p + e].split(',').filter_map(|v| v.trim().parse().ok()).collect()));
                    rest = &rest[p + e + 1..];
                }
                _ => break,
            }
        }
    }
    out
}

impl SetupFog {
    /// GameDataLoading.cpp 0x0009E300 — the "FOG" / "GRADIENT FOG" block at the top of Setup.txt.
    pub fn parse(text: &str) -> Self {
        let mut s = Self::default();
        let toks = setup_tokens(text);
        let mut it = toks.iter().peekable();
        let mut next_vals = || loop {
            match it.next() {
                Some(SetupToken::Values(v)) => return v.clone(),
                Some(SetupToken::Str(_)) => continue,
                None => return Vec::new(),
            }
        };
        let f = |v: Vec<f32>| v.first().copied().unwrap_or(0.0);
        let v3 = |v: Vec<f32>| Vec3::new(v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0));
        match toks.first() {
            Some(SetupToken::Str(h)) if h == "FOG" => {
                s.colour_top = v3(next_vals());
                s.distance = f(next_vals());
                s.alpha_top = f(next_vals());
                s.sky_fogged = f(next_vals()) as i32 != 0;
                s.unk58 = 1.0;
            }
            Some(SetupToken::Str(h)) if h == "GRADIENT FOG" => {
                s.colour_top = v3(next_vals());
                s.colour_bottom = v3(next_vals());
                s.alpha_top = f(next_vals());
                s.alpha_bottom = f(next_vals());
                s.top_elevation = f(next_vals());
                s.bot_elevation = f(next_vals());
                s.distance = f(next_vals());
                s.sky_fogged = f(next_vals()) as i32 != 0;
                s.gradient = true;
                s.unk58 = 1.0;
            }
            _ => {}
        }
        s
    }
}

/// fog render object (`dword_71A028`); a resource from the track build on ("Graphics.Fog" tweaks edit it, the
/// track's materials bake its render states at build time).
#[derive(bevy::prelude::Resource, Clone, Debug)]
pub struct FogRender {
    pub enabled: bool,    // +0x00
    pub colour: Vec3,     // +0x0C
    pub alpha: f32,       // +0x18
    pub fade_std: f32,    // +0x20
    pub distance: f32,    // +0x24
    pub fade_start: i32,  // +0x28
    pub fade_end: i32,    // +0x2C
    pub rebloom: bool,    // +0x30
    pub fog_sky: bool,    // +0x31
    pub fog_type: i32,    // +0x34 0 off, 1 std, 2 fade
    pub intensity: f32,   // +0x38
    pub unk3c: f32,       // +0x3C
}

impl FogRender {
    /// CarShaderManager_PC_XBOX.cpp 0x0015A470 (init: enabled) + 0x0015A4E0 (copy from the setup).
    pub fn init(setup: &SetupFog) -> Self {
        Self {
            enabled: true,
            colour: setup.colour_top,
            alpha: setup.alpha_top,
            fade_std: setup.unk58,
            distance: setup.distance,
            fade_start: 30,
            fade_end: 60,
            rebloom: true,
            fog_sky: setup.sky_fogged,
            fog_type: 1,
            intensity: f32::from_bits(0x3E99_999A),
            unk3c: 1.0,
        }
    }

    /// CarShaderManager_PC_XBOX.cpp 0x0015A990 — FOGENABLE, FOGTABLEMODE = D3DFOG_EXP, FOGDENSITY = intensity/1000,
    /// FOGCOLOR = argb(alpha, colour). Returns (colour, density) or None when fog is off.
    pub fn render_states(&self) -> Option<(Vec3, f32)> {
        self.enabled.then(|| (self.colour, self.intensity / 1000.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_track_setup_fog() {
        let dir = visos::file::resolve("\\Tracks");
        let mut n = 0;
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            for f in ["Setup.txt", "SetupWET.txt"] {
                if let Ok(t) = std::fs::read_to_string(e.path().join(f)) {
                    let s = SetupFog::parse(&t);
                    assert_eq!(s.unk58, 1.0, "{}", e.path().display());
                    n += 1;
                }
            }
        }
        assert!(n > 0);
        let la = SetupFog::parse(&std::fs::read_to_string(visos::file::resolve("\\Tracks\\LACoast\\Setup.txt")).unwrap());
        assert!(la.gradient && la.sky_fogged);
        assert_eq!(la.colour_top, Vec3::new(0.11, 0.11, 0.141));
        assert_eq!(la.distance, 0.2);
    }
}
