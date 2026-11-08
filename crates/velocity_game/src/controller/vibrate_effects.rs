//! src\Controller\VibrateEffects.cpp — the rumble effects (per-pad effect lists, presets, per-update triggers,
//! output through visos `set_rumble`) and the race markers file (`g_Markers`, 0x0075C70C), named positions the
//! post-race sequence looks up ("POST_RACE_STUNT_AREA").
//!
//! `Markers.mfk`: f32 version (1.0), u32 count, 8 bytes pad, then 48-byte markers {vec4 position, quat
//! orientation, u32, u32 name crc, u32 x2}. Every shipped file is empty.

use bevy::math::{Quat, Vec3};
use bevy::prelude::*;

pub const MARKERS_VERSION: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Marker {
    pub position: Vec3,      // +0x00
    pub orientation: Quat,   // +0x10
    pub unknown_20: u32,     // +0x20
    pub name_crc: u32,       // +0x24
    pub unknown_28: [u32; 2], // +0x28
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CMarkers {
    pub markers: Vec<Marker>,
    /// +0x08 a file is loaded
    pub loaded: bool,
}

/// VibrateEffects.cpp 0x001EDEF0 — `<dir>\[<sub>\]Markers.mfk`, upper-cased
pub fn markers_path(dir: &str, sub: Option<&str>) -> String {
    // a relative directory gets a root prefix (the copied prefix is not recovered; "\" assumed)
    let mut p = if dir.starts_with('\\') { dir.to_string() } else { format!("\\{dir}") };
    if let Some(s) = sub {
        if !s.starts_with('\\') {
            p.push('\\');
        }
        p.push_str(s);
    }
    if !p.ends_with('\\') {
        p.push('\\');
    }
    p.push_str("Markers.mfk");
    p.to_ascii_uppercase()
}

fn f(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

impl CMarkers {
    /// VibrateEffects.cpp 0x001EDFE0 — "Markers Init" (the race directory)
    pub fn load(&mut self, race_dir: &str) {
        self.unload();
        let path = markers_path(race_dir, None);
        let Ok(b) = visos::file::platform::read(&path) else {
            log::warn!("ERROR: Failed to load the markers file (Markers.mfk)");
            return;
        };
        match Self::parse(&b) {
            Some(m) => *self = m,
            None => log::warn!("Marker File Version Incorrect  Expected v{MARKERS_VERSION} - Got v{}", b.get(0..4).map(|_| f(&b, 0)).unwrap_or(0.0)),
        }
    }

    pub fn parse(b: &[u8]) -> Option<Self> {
        if b.len() < 16 || f(b, 0) != MARKERS_VERSION {
            return None;
        }
        let n = u(b, 4) as usize;
        let mut markers = Vec::with_capacity(n);
        for i in 0..n {
            let r = b.get(16 + 48 * i..64 + 48 * i)?;
            markers.push(Marker {
                position: Vec3::new(f(r, 0), f(r, 4), f(r, 8)),
                orientation: Quat::from_xyzw(f(r, 16), f(r, 20), f(r, 24), f(r, 28)),
                unknown_20: u(r, 32),
                name_crc: u(r, 36),
                unknown_28: [u(r, 40), u(r, 44)],
            });
        }
        Some(CMarkers { markers, loaded: true })
    }

    /// VibrateEffects.cpp 0x001EE1C0
    pub fn unload(&mut self) {
        self.markers.clear();
        self.loaded = false;
    }

    /// VibrateEffects.cpp 0x001EE230 — marker by name crc
    pub fn find(&self, crc: u32) -> Option<&Marker> {
        if !self.loaded {
            return None;
        }
        self.markers.iter().find(|m| m.name_crc == crc)
    }
}

// ---- the rumble effects: per-pad effect lists, the preset table and the per-update triggers ----

pub const MAX_PADS: usize = 6; // GAMEINPUT_MAX_PADS
pub const MAX_EFFECTS: usize = 8; // `this+0x60` count cap in 0x001ED6D0
pub const PRESET_SKID: usize = 6;
pub const PRESET_TRAILBLAZER: usize = 7;

/// one row of `off_688C10` (name, motor, intensity, decay); `decay` is the effect's duration in seconds (the
/// tick divides the remaining time by it)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Preset {
    pub motor: i32,
    pub intensity: f32,
    pub decay: f32,
}

/// the preset table, from the shipped bits in `core_tweaks::VIBRATE_EFFECTS` (the debug menu's own copies of the
/// tweaks do not feed it yet)
#[derive(Resource, Clone, Debug)]
pub struct VibratePresets(pub [Preset; 8]);

impl Default for VibratePresets {
    fn default() -> Self {
        let mut p = [Preset { motor: 0, intensity: 0.0, decay: 1.0 }; 8];
        for (i, (_, motor, intensity, decay)) in crate::game_mode::core_tweaks::VIBRATE_EFFECTS.iter().enumerate() {
            p[i] = Preset { motor: *motor, intensity: f32::from_bits(*intensity), decay: f32::from_bits(*decay) };
        }
        Self(p)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Effect {
    pub preset: u32, // +0
    pub strength: f32, // +4
    pub time: f32, // +8 seconds left
}

/// one pad's `VibrateEffects` (100 bytes at `unk_75C1A0 + 100 * pad`: 8 x 12-byte effects, count at +0x60)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VibrateEffects {
    pub effects: [Effect; MAX_EFFECTS],
    pub count: usize,
}

impl VibrateEffects {
    /// VibrateEffects.cpp 0x001ED6D0 — a new effect with the preset's full duration; ignored when 8 are running
    pub fn add(&mut self, presets: &VibratePresets, preset: usize, strength: f32) {
        if self.count >= MAX_EFFECTS {
            log::warn!("Warning: VibrateEffects, maximum simultaneous effects exceeded, ignoring this request");
            return;
        }
        self.effects[self.count] = Effect { preset: preset as u32, strength, time: presets.0[preset].decay };
        self.count += 1;
    }

    /// VibrateEffects.cpp 0x001ED760 — the running effect of that preset gets the strength (its time keeps
    /// running), else a new one
    pub fn add_or_update(&mut self, presets: &VibratePresets, preset: usize, strength: f32) {
        for e in &mut self.effects[..self.count] {
            if e.preset as usize == preset {
                e.strength = strength;
                return;
            }
        }
        self.add(presets, preset, strength);
    }

    /// VibrateEffects.cpp 0x001ED7E0 — refresh a running effect: `time` < 0 = the preset's full duration
    pub fn update_existing(&mut self, presets: &VibratePresets, preset: usize, time: f32, strength: f32) {
        for e in &mut self.effects[..self.count] {
            if e.preset as usize == preset {
                e.time = if time < 0.0 { presets.0[preset].decay } else { time };
                e.strength = strength;
                return;
            }
        }
    }

    /// VibrateEffects.cpp 0x001ED460 / 0x001ED440
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// VibrateEffects.cpp 0x001ED480 — the two motor levels of this update (max over the live effects of
    /// intensity x strength x time / duration); every effect ages by `dt`, the finished ones are swapped out
    /// with the last one
    pub fn tick(&mut self, presets: &VibratePresets, dt: f32) -> [f32; 2] {
        let mut motors = [0.0f32; 2];
        let mut i = 0;
        while i < self.count {
            let e = self.effects[i];
            if e.time > 0.0 {
                let p = presets.0[(e.preset as usize).min(7)];
                let v = (p.intensity * e.strength) * (e.time / p.decay);
                let m = (p.motor.clamp(0, 1)) as usize;
                // VsFrameTimer 0x00057F90 = max
                motors[m] = if v <= motors[m] { motors[m] } else { v };
                self.effects[i].time -= dt;
                i += 1;
            } else {
                self.effects[i] = self.effects[self.count - 1];
                self.count -= 1;
            }
        }
        motors
    }
}

/// `unk_75C1A0`: the six pads' effects, plus what UserInput keeps of the rumble switch
#[derive(Resource, Clone, Debug, Default)]
pub struct Vibration {
    pub pads: [VibrateEffects; MAX_PADS],
    /// VehicleEntity +0x38 / +0x3C of each player's car (0x001D1680): skidding last update, seconds skidding
    pub skid_prev: [bool; MAX_PADS],
    pub skid_time: [f32; MAX_PADS],
}

impl Vibration {
    /// VibrateEffects.cpp 0x001EDD20 — every pad's list emptied (race init / restart / end / post race fade);
    /// port: the motors are zeroed too (the original's update loop is not running any more to do it)
    pub fn stop(&mut self) {
        for p in &mut self.pads {
            p.clear();
        }
        for port in 0..4 {
            visos::input::input_manager::set_rumble(port, 0, 0.0);
            visos::input::input_manager::set_rumble(port, 1, 0.0);
        }
    }

    /// VibrateEffects.cpp 0x001ED2D0 — the collision message (word 10 = impact): >= 1 shakes both players' pads,
    /// small (<= 6) / medium (<= 15) / large
    pub fn collision(&mut self, presets: &VibratePresets, players: [Option<usize>; 2], impact: f32) {
        if impact < 1.0 {
            return;
        }
        let preset = if impact <= 15.0 { (impact > 6.0) as usize } else { 2 };
        for p in players.into_iter().flatten() {
            if p < MAX_PADS {
                self.pads[p].add(presets, preset, 1.0);
            }
        }
    }

    /// VehicleEntity.cpp 0x001D1680 — the skid effect of a player's car: ramps over 1.5 s of skidding
    pub fn skid(&mut self, presets: &VibratePresets, pad: usize, dt: f32, skidding: bool) {
        if pad >= MAX_PADS {
            return;
        }
        if self.skid_prev[pad] {
            if skidding {
                self.skid_time[pad] += dt;
                let v = (self.skid_time[pad] / 1.5).min(1.0);
                self.pads[pad].update_existing(presets, PRESET_SKID, -1.0, v);
            } else {
                self.skid_time[pad] = 0.0;
            }
        } else if skidding {
            self.skid_time[pad] = 0.0;
            self.pads[pad].add_or_update(presets, PRESET_SKID, 0.0);
        }
        self.skid_prev[pad] = skidding;
    }
}

/// VehicleEntity.cpp 0x001D1770 — a wheel that is not flagged, slides on a contact (FastCar 0x00364A90 == 1), the
/// car goes faster than 1 m/s and the wheel has contacts (LightStreak 0x00133970)
fn car_skidding(w: &velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld, id: usize) -> bool {
    let s = w.slot(id);
    (0..4).any(|i| s.ve.m.b(24 + i) == 0 && s.car.wheel_contact_state(i) == 1 && s.ve.m.f(20) > 1.0 && s.car.n_contacts(i) > 0)
}

/// VibrateEffects.cpp 0x001ED8E0 + 0x001EDD70 — per update after the zone trail: each player's slipstream (VE+68
/// > 0.1), nitrous (VE+88) and zone (VE+89, not under a trailblazer) refresh their effects, the skid effect runs
/// (VehicleEntity 0x001C5800 -> 0x001D1680), the collisions of this update's physics frame add theirs, then the
/// pad's first device gets the two motor levels (UserInput 0x001E69F0 -> visos `set_rumble`)
#[allow(clippy::too_many_arguments)]
pub fn vibration_update(
    clock: Res<crate::vs_common_main::GameClock>,
    gi: Res<crate::game_input::game_input::GameInput>,
    gm: Res<crate::game_mode::game_manager::GameManager>,
    attract: Res<crate::font_helper::CAttractMode>,
    career: Option<Res<crate::game_mode::career::Career>>,
    trails: Option<Res<crate::game_mode::trail_blazer::PlayerTrailBlazers>>,
    physics: Option<Res<velocity_physics::plugin::PhysicsWorldRes>>,
    cars: Query<(&crate::game_mode::race_cars::RaceCar, &velocity_physics::plugin::PhysicsCar)>,
    mut hits: MessageReader<velocity_physics::plugin::PhysicsCollision>,
    presets: Res<VibratePresets>,
    mut vib: ResMut<Vibration>,
) {
    let dt = clock.timer.scaled_dt();
    let players = gm.race.as_ref().map_or(0, |r| r.num_players as usize).min(MAX_PADS);
    let player_of = |id: i32| -> Option<usize> {
        if id < 0 {
            return None;
        }
        cars.iter().find(|(c, p)| c.human && p.id == id as usize && c.slot < players).map(|(c, _)| c.slot)
    };
    for velocity_physics::plugin::PhysicsCollision(c) in hits.read() {
        vib.collision(&presets, [player_of(c.a), player_of(c.b)], c.impact);
    }
    // UserInput 0x001E6B00: rumble runs while the mode is the race (0x001ED880: not attract), the option is on
    // and the game is not paused (port: pause stops the update thread, so the motors are zeroed here)
    let enabled = !attract.playing && career.as_deref().is_none_or(|c| c.options.rumble) && !gm.paused;
    let guard = physics.as_deref().map(|p| p.lock());
    let world = guard.as_ref().and_then(|g| g.as_ref());
    for (c, p) in &cars {
        if !c.human || c.slot >= players {
            continue;
        }
        let Some(w) = world.filter(|w| w.vehicle_ids().contains(&p.id)) else { continue };
        let ve = &w.slot(p.id).ve.m;
        let pad = c.slot;
        if ve.f(68) > 0.1 {
            vib.pads[pad].add_or_update(&presets, 3, 1.0);
        }
        if ve.b(88) != 0 {
            vib.pads[pad].add_or_update(&presets, 4, 1.0);
        }
        let one_off = trails.as_deref().is_some_and(|t| t.0.get(pad).copied().flatten().is_some());
        if ve.b(89) != 0 && !one_off {
            vib.pads[pad].add_or_update(&presets, 5, 1.0);
        }
        let skidding = car_skidding(w, p.id);
        vib.skid(&presets, pad, dt, skidding);
    }
    for pad in 0..players {
        let mask = gi.device_mask(pad);
        if let Some(d) = (0..MAX_PADS).find(|d| mask & (1 << d) != 0) {
            let m = if enabled { vib.pads[pad].tick(&presets, dt) } else { [0.0; 2] };
            if d < 4 {
                visos::input::input_manager::set_rumble(d, 0, m[0]);
                visos::input::input_manager::set_rumble(d, 1, m[1]);
            }
        }
    }
}

/// 0x001EDD20 at race init (0x000B629F), restart (0x000BE10B), end of game (0x000BE5A9) and the post race fade
/// (0x000C37DD)
fn vibration_stop(
    mut vib: ResMut<Vibration>,
    mut modes: MessageReader<crate::game_mode::game_manager::GameModeMessage>,
    mut session: MessageReader<crate::game_mode::race_session::RaceSessionMessage>,
) {
    use crate::game_mode::game_manager::GameModeMessage as G;
    let mut stop = false;
    for m in modes.read() {
        stop |= matches!(m, G::InGameShutdown | G::RaceRestart);
    }
    for m in session.read() {
        stop |= matches!(m, crate::game_mode::race_session::RaceSessionMessage::PostRace(crate::game_mode::post_race_sequence::PostRaceEvent::Fade(t)) if *t > 0.0);
    }
    if stop {
        vib.stop();
    }
}

/// registers the rumble: effect lists, presets, the update after the zone trail, the stops
pub fn add_vibration(app: &mut App) {
    use crate::vs_common_main::{GameTick, InGameUpdateSet};
    // tests build the plugin without its neighbours
    if !app.world().contains_resource::<Messages<velocity_physics::plugin::PhysicsCollision>>() {
        app.add_message::<velocity_physics::plugin::PhysicsCollision>();
    }
    if !app.world().contains_resource::<Messages<crate::game_mode::game_manager::GameModeMessage>>() {
        app.add_message::<crate::game_mode::game_manager::GameModeMessage>();
    }
    if !app.world().contains_resource::<Messages<crate::game_mode::race_session::RaceSessionMessage>>() {
        app.add_message::<crate::game_mode::race_session::RaceSessionMessage>();
    }
    app.init_resource::<Vibration>()
        .init_resource::<VibratePresets>()
        .add_systems(
            GameTick,
            vibration_update
                .in_set(InGameUpdateSet::Effects)
                .run_if(in_state(crate::game_mode::game_manager::EGameMode::InGame))
                .run_if(|gm: Res<crate::game_mode::game_manager::GameManager>| !gm.paused),
        )
        .add_systems(OnEnter(crate::InRace), |mut v: ResMut<Vibration>| v.stop())
        .add_systems(Update, vibration_stop);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_tick_and_expire() {
        let p = VibratePresets::default();
        let mut e = VibrateEffects::default();
        e.add(&p, 0, 1.0); // small collision: motor 0, 0.3, 0.2 s
        e.add_or_update(&p, 5, 0.5); // zone: motor 1, 1.0, 0.5 s
        let m = e.tick(&p, 0.1);
        assert!((m[0] - 0.3).abs() < 1e-6 && (m[1] - 0.5).abs() < 1e-6, "{m:?}");
        e.update_existing(&p, 5, -1.0, 1.0);
        let m = e.tick(&p, 0.2);
        assert!((m[0] - 0.3 * 0.5).abs() < 1e-6 && (m[1] - 1.0).abs() < 1e-6, "{m:?}");
        e.tick(&p, 0.2);
        let m = e.tick(&p, 0.2);
        assert_eq!(m[0], 0.0);
        assert!(e.count < 2);
        for _ in 0..12 {
            e.add(&p, 1, 1.0);
        }
        assert_eq!(e.count, MAX_EFFECTS);
    }

    #[test]
    fn skid_ramps_and_collisions_pick_presets() {
        let p = VibratePresets::default();
        let mut v = Vibration::default();
        v.skid(&p, 0, 0.1, true);
        assert_eq!(v.pads[0].effects[0].strength, 0.0);
        v.skid(&p, 0, 0.75, true);
        assert!((v.pads[0].effects[0].strength - 0.5).abs() < 1e-6);
        v.skid(&p, 0, 1.0, true);
        assert_eq!(v.pads[0].effects[0].strength, 1.0);
        v.collision(&p, [Some(1), None], 7.0);
        assert_eq!(v.pads[1].effects[0].preset, 1);
        v.collision(&p, [Some(1), None], 0.5);
        assert_eq!(v.pads[1].count, 1);
        v.collision(&p, [None, Some(1)], 20.0);
        assert_eq!(v.pads[1].effects[1].preset, 2);
    }

    #[test]
    fn markers_all() {
        assert_eq!(markers_path("\\Tracks\\LADown\\Circuit\\Race04\\", None), "\\TRACKS\\LADOWN\\CIRCUIT\\RACE04\\MARKERS.MFK");
        if visos::file::platform::try_game_dir().is_err() {
            return;
        }
        for d in ["\\Tracks\\LADown\\Circuit\\Race04\\", "\\Tracks\\LADown\\Circuit\\Race05\\", "\\Tracks\\LADown\\Circuit\\Race06\\"] {
            let mut m = CMarkers::default();
            m.load(d);
            assert!(m.loaded && m.markers.is_empty(), "{d}");
        }
        let mut b = vec![0u8; 64];
        b[0..4].copy_from_slice(&1.0f32.to_le_bytes());
        b[4] = 1;
        b[16 + 36..16 + 40].copy_from_slice(&7u32.to_le_bytes());
        assert_eq!(CMarkers::parse(&b).unwrap().find(7).map(|m| m.name_crc), Some(7));
    }
}
