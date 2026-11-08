//! src\GameMode\TrailBlazer.cpp — the trailblazer challenge: a trigger hands the player car a spline to follow;
//! time slows down, the screen tints with the distance off the line, and the car must reach the end without
//! straying further than the trigger allows, slowing below 20 m/s or turning away. Success plays a race
//! timeline; failure restores the race timeline's restore point and respawns the car. One per ai race car.
//!
//! The spline is the ai's road line, the car is the physics car: both are read through `TrailWorld`.
//! Effects on the renderer, physics globals, audio, OSD and time go out as `TrailEvent`s.

use bevy::math::Vec3;

/// the ai road line ops the trailblazer uses (VsAIRoadLine)
pub trait TrailSpline {
    fn find_closest_segment(&self, p: Vec3) -> u32;
    fn find_segment(&self, p: Vec3, hint: u32) -> u32;
    fn distance_along(&self, p: Vec3, seg: u32) -> f32;
    fn length(&self) -> f32;
    fn point_at(&self, d: f32, seg: u32) -> Vec3;
}

/// the car as the trailblazer reads it
#[derive(Clone, Copy, Debug, Default)]
pub struct TrailCar {
    /// `VsAIDriver+0x98`
    pub position: Vec3,
    /// +0x90 m/s
    pub speed: f32,
    /// +0xC8 heading vector of the ai car
    pub heading: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrailEvent {
    /// RaceTimelineSequencer 0x00171490 / 0x001713D0 restore / 0x00171620 restore and clear / 0x00171580 keep
    TimelineSetRestorePoint,
    TimelineRestore,
    TimelineRestoreAndClear,
    TimelineKeep,
    PlayTimeline(u32),
    /// WMM time stretch (`dword_6FA8D0`+0x10)
    TimeStretch(f32),
    /// physics globals: `GC->visual_zone_velocity`, `zone_sensitivities_multiplier`,
    /// `SS_friction_limited_SA_multiplier`, `SS_steering_bias_when_steering`, `SS_steering_bias`, `dword_686B58`
    PhysicsGlobals { zone_velocity: Option<f32>, zone_sensitivities: Option<f32>, friction_sa: Option<f32>, steer_bias_steering: Option<f32>, steer_bias: Option<f32>, g_686b58: Option<f32> },
    /// the physics car's trailblazer flag (+0x10) and its reset (VehicleEntity 0x001C8B90 "C::VehicleEntity")
    CarFlag(bool),
    CarReset,
    /// TrailBlazerRender: on (spline crc) / off, and per update intensity
    Render { spline: Option<u32> },
    /// 0x000C5F90 → DangerZoneRender `unk_718AC0`: +28 on, +64 = &TB+0x60 (prev[5], cur[5]; a pointer, so the
    /// renderer sees the record as the whole update left it), +48 = TB+0x5C success (by value)
    RenderParams { on: bool, prev: [f32; 5], cur: [f32; 5], success: bool },
    /// DangerZoneRender `unk_718AC0` +29: 1 at the intro (0x000C6080), 0 when the outro ends (0x000C6120)
    DangerEnabled(bool),
    Intensity(f32),
    /// DangerZoneRender 0x001234A0 / 0x001238A0 (off)
    DangerZone(Option<f32>),
    /// vibration effect 7 (VibrateEffects 0x001ED760 / 0x001ED7E0)
    Vibrate(f32),
    /// OSD message (0 = clear)
    Message(u32),
    /// VsAIRaceVehicle 0x00022C90: respawn at a trigger, at this speed (m/s)
    Respawn { trigger: u32, speed: f32 },
    AudioStart,
    AudioStop,
    AudioFadeIn(f32),
}

/// a `TrailEvent` of the trailblazer of physics vehicle `vehicle`, written by its runner (the ai race vehicle) in the
/// GameTick Ai set; core reads the danger zone ones in the same tick (`race_effects_bridge::danger_zone_record`)
#[derive(bevy::prelude::Message, Clone, Copy, Debug, PartialEq)]
pub struct TrailBlazerMessage {
    pub vehicle: u32,
    pub event: TrailEvent,
}

/// TrailBlazer.cpp 0x000C57E0 render parameters (`+0x00..+0x4C`)
#[derive(Clone, Debug, PartialEq)]
pub struct TrailParams {
    pub p: [f32; 20],
}

impl Default for TrailParams {
    fn default() -> Self {
        let b = |x: u32| f32::from_bits(x);
        TrailParams {
            p: [
                b(0x4020_0000),
                8.0,
                8.0,
                10.0,
                5.0,
                f32::from_bits(4),
                60.0,
                40.0,
                b(0x3E19_999A),
                1.0,
                b(0x3DCC_CCCD),
                b(0x3C23_D70A),
                0.25,
                0.5,
                0.5,
                b(0x3E99_999A),
                1.0,
                b(0x3E99_999A),
                b(0x3E4C_CCCD),
                b(0x3F4C_CCCD),
            ],
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (b - a) * t + a
}

/// per player: the race vehicle's trailblazer (VsAIRaceVehicle +0x3C8, "one-off object") while it runs —
/// VsAIRaceVehicle 0x00023310 (+0x3C8 set and 0x00024F10: +0x8C spline != 0) → its +0xA8 progress. Read by the
/// chase camera (0x00067510) and the zone FX (0x00162D10); the binary fills it from the ai each update
#[derive(bevy::prelude::Resource, Clone, Copy, Debug, Default)]
pub struct PlayerTrailBlazers(pub [Option<f32>; 2]);

/// `TrailBlazer` (inside VsAIRaceVehicle)
#[derive(Clone, Debug, Default)]
pub struct TrailBlazer {
    pub params: TrailParams,
    /// +0x50 0 off, 1 intro, 2 main, 3 outro
    pub mode: u32,
    pub effect_time: f32,    // +0x54
    pub outro_start: f32,    // +0x58
    pub success: bool,       // +0x5C
    pub prev: [f32; 5],      // +0x60
    pub render: [f32; 5],    // +0x74
    pub vehicle: u32,        // +0x88
    pub spline: Option<u32>, // +0x8C
    pub success_timeline: u32, // +0x90
    pub respawn_trigger: u32,  // +0x94
    pub flag: bool,          // +0x98
    pub ending: bool,        // +0x99
    pub segment: u32,        // +0x9C
    pub max_distance: f32,   // +0xA0
    pub off_ratio: f32,      // +0xA4
    pub progress: f32,       // +0xA8
    pub time: f32,           // +0xAC
    pub end_timer: f32,      // +0xB0
    /// +0xB4..+0xBC time stretch: target, start, length
    pub stretch: [f32; 3],
    /// +0xC0..+0xD4 the physics globals saved at the start
    pub saved: [f32; 6],
}

impl TrailBlazer {
    /// TrailBlazer.cpp 0x000C46C0
    pub fn new(vehicle: u32) -> Self {
        TrailBlazer { vehicle, stretch: [1.0, 1.0, 1.0], ..Default::default() }
    }

    pub fn active(&self) -> bool {
        self.spline.is_some()
    }

    /// TrailBlazer.cpp 0x000C4870 — `saved` = the current physics globals (zone velocity, zone sensitivities,
    /// friction sa, steer bias steering, steer bias, dword_686B58)
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        &mut self,
        spline: u32,
        s: &dyn TrailSpline,
        max_distance: f32,
        success_timeline: u32,
        respawn_trigger: u32,
        flag: bool,
        car: &TrailCar,
        saved: [f32; 6],
        out: &mut Vec<TrailEvent>,
    ) {
        if max_distance < 0.0 {
            log::error!("assert: maxDistance >= 0.0f");
        }
        out.push(TrailEvent::TimelineSetRestorePoint);
        self.saved = saved;
        out.push(TrailEvent::PhysicsGlobals {
            zone_velocity: None,
            zone_sensitivities: Some(1.5),
            friction_sa: Some(1.0),
            steer_bias_steering: Some(1.0),
            steer_bias: Some(1.0),
            g_686b58: None,
        });
        self.ending = false;
        self.spline = Some(spline);
        self.max_distance = max_distance;
        self.success_timeline = success_timeline;
        self.respawn_trigger = respawn_trigger;
        self.flag = flag;
        self.off_ratio = 0.0;
        self.progress = 0.0;
        self.segment = s.find_closest_segment(car.position);
        self.end_timer = 0.0;
        self.time = 0.0;
        self.stretch = [f32::from_bits(0x3DCC_CCCD), f32::from_bits(0x3E4C_CCCD), f32::from_bits(0x3DCC_CCCD)];
        out.push(TrailEvent::TimeStretch(self.stretch[0]));
        out.push(TrailEvent::CarFlag(true));
        out.push(TrailEvent::Render { spline: Some(spline) });
        // 0x000C6080
        self.mode = 1;
        self.render[0] = 0.0;
        out.push(TrailEvent::DangerEnabled(true));
        // (also `byte_719168` +29 = 0: the danger particles' gate of 0x001234A0)
        self.success = false;
        out.push(TrailEvent::AudioStart);
    }

    /// TrailBlazer.cpp 0x000C4E50 — leave the challenge (either way)
    fn end(&mut self, out: &mut Vec<TrailEvent>) {
        out.push(TrailEvent::CarFlag(false));
        out.push(TrailEvent::Render { spline: None });
        let s = self.saved;
        out.push(TrailEvent::PhysicsGlobals {
            zone_velocity: Some(s[0]),
            zone_sensitivities: Some(s[1]),
            friction_sa: Some(s[2]),
            steer_bias_steering: Some(s[3]),
            steer_bias: Some(s[4]),
            g_686b58: Some(s[5]),
        });
        out.push(TrailEvent::CarReset);
        self.spline = None;
        self.max_distance = 0.0;
        self.progress = 0.0;
        self.segment = 0;
        out.push(TrailEvent::TimeStretch(1.0));
        // TrailBlazer.cpp 0x000C60E0
        self.mode = 3;
        self.outro_start = self.effect_time;
        out.push(TrailEvent::DangerZone(None));
    }

    /// TrailBlazer.cpp 0x000C4BD0
    fn succeed(&mut self, out: &mut Vec<TrailEvent>) {
        self.end(out);
        self.end_timer = 5.0;
        self.ending = true;
        out.push(TrailEvent::Message(visos::handy::handy_crc::crc32(b"OSD_TRAILBLAZER_SUCCESS", 0)));
        if self.success_timeline != 0 {
            out.push(TrailEvent::PlayTimeline(self.success_timeline));
        }
        self.success = true;
    }

    /// TrailBlazer.cpp 0x000C4C90 — `respawn_speed` = the respawn trigger's param / 1000
    fn fail(&mut self, respawn_speed: f32, out: &mut Vec<TrailEvent>) {
        self.end(out);
        out.push(TrailEvent::TimelineRestore);
        if self.respawn_trigger != 0 {
            out.push(TrailEvent::Respawn { trigger: self.respawn_trigger, speed: respawn_speed });
        }
        self.stretch = [f32::from_bits(0x3C23_D70A), f32::from_bits(0x3C23_D70A), f32::from_bits(0x3DCC_CCCD)];
        out.push(TrailEvent::TimeStretch(self.stretch[0]));
        self.end_timer = 5.0;
        self.ending = true;
        out.push(TrailEvent::Message(visos::handy::handy_crc::crc32(b"OSD_IN_A_TRAILBLAZER_FAR_FAR_AWAY", 0)));
        self.success = false;
    }

    /// TrailBlazer.cpp 0x000C5740
    fn finish(&mut self, out: &mut Vec<TrailEvent>) {
        if !self.success || self.flag {
            out.push(TrailEvent::TimelineRestoreAndClear);
        } else {
            out.push(TrailEvent::TimelineKeep);
        }
        out.push(TrailEvent::Message(0));
        self.success_timeline = 0;
        self.end_timer = 0.0;
        self.ending = false;
        self.success = false;
    }

    /// TrailBlazer.cpp 0x000C4FF0 — per update (`respawn_speed` for a failure, see `fail`)
    pub fn update(&mut self, dt: f32, s: Option<&dyn TrailSpline>, car: &TrailCar, respawn_speed: f32, out: &mut Vec<TrailEvent>) {
        self.update_render(out);
        let at = out.len();
        self.push_render(out);
        self.update_main(dt, s, car, respawn_speed, out);
        // the danger renderer holds a pointer to +0x60: it draws the record as this update left it
        if let TrailEvent::RenderParams { on: true, prev, cur, .. } = &mut out[at] {
            *prev = self.prev;
            *cur = self.render;
        }
    }

    fn update_main(&mut self, dt: f32, s: Option<&dyn TrailSpline>, car: &TrailCar, respawn_speed: f32, out: &mut Vec<TrailEvent>) {
        if self.spline.is_some() {
            let sp = self.progress.sqrt();
            let st = (self.time / 2.0).clamp(0.0, 1.0);
            out.push(TrailEvent::PhysicsGlobals {
                zone_velocity: Some(lerp(26.8224, 67.056, sp)),
                zone_sensitivities: None,
                friction_sa: None,
                steer_bias_steering: Some(lerp(1.0, 0.0, st)),
                steer_bias: Some(lerp(1.0, 0.0, st)),
                g_686b58: Some(lerp(40.0, 110.0, sp)),
            });
        }
        if self.ending {
            self.end_timer -= dt;
            if self.end_timer <= 0.0 {
                self.finish(out);
                return;
            }
        }
        self.time += dt;
        let k = 1.0 - ((self.time - self.stretch[1]) / self.stretch[2]).clamp(0.0, 1.0);
        let mut v = lerp(1.0, self.stretch[0], k);
        v = v.max(0.0).min(1.0);
        out.push(TrailEvent::TimeStretch(v));
        let (Some(_), Some(s)) = (self.spline, s) else { return };
        let pos = car.position;
        self.segment = s.find_segment(pos, self.segment);
        let d = s.distance_along(pos, self.segment);
        self.progress = d / s.length();
        if self.progress >= 1.0 - 1e-6 {
            self.succeed(out);
            return;
        }
        let p = s.point_at(d, self.segment);
        let dist = (p - pos).length_squared().sqrt();
        self.off_ratio = dist / self.max_distance;
        let mut level = self.off_ratio;
        let too_long = self.time > 3.0;
        if self.off_ratio <= 1.0 {
            let speed = car.speed;
            if 30.0 > speed && speed > 20.0 {
                level = level.max(1.0 - (speed - 20.0) / (30.0 - 20.0));
            }
            if 20.0 <= speed {
                let a = s.point_at((d - 0.1).max(0.0), self.segment);
                let b = s.point_at(s.length().min(d + 0.1), self.segment);
                let dir = crate::camera::vs_base_camera::normalise(b - a);
                let dot = dir.dot(car.heading);
                // `dword_6FA3E8` is never written: 0
                if 0.5 > dot {
                    level = level.max(1.0 - (dot - 0.0) / (0.5 - 0.0));
                }
                if 0.0 > dot {
                    if too_long {
                        self.fail(respawn_speed, out);
                    }
                    level = 1.0;
                }
            } else {
                if too_long {
                    self.fail(respawn_speed, out);
                }
                level = 1.0;
            }
        } else {
            if too_long {
                self.fail(respawn_speed, out);
            }
            level = 1.0;
        }
        out.push(TrailEvent::Intensity(level));
        out.push(TrailEvent::Vibrate(level.sqrt()));
        self.render[1] = 1.0 - level;
        out.push(TrailEvent::DangerZone(Some(level)));
    }

    /// TrailBlazer.cpp 0x000C5930
    fn update_render(&mut self, out: &mut Vec<TrailEvent>) {
        self.prev = self.render;
        self.effect_time = self.time * 2.0;
        let p = &self.params.p;
        let t = self.effect_time;
        match self.mode {
            0 => {}
            1 => {
                // TrailBlazer.cpp 0x000C5A80 intro
                let v3 = p[12] + p[13];
                if t < v3 + p[14] {
                    if p[12] <= t {
                        if v3 <= t {
                            let v1 = ((t - v3) / p[14]).clamp(0.0, 1.0);
                            let fade = lerp(p[8], p[9], v1);
                            self.render = [1.0, self.render[1], 1.0, v1, v1];
                            out.push(TrailEvent::AudioFadeIn(fade));
                        } else {
                            self.render = [1.0, self.render[1], 1.0, 0.0, 0.0];
                        }
                    } else {
                        let v2 = (t / p[12]).clamp(0.0, 1.0);
                        self.render = [v2, self.render[1], v2, 0.0, 0.0];
                    }
                } else {
                    self.mode = 2;
                    self.render = [1.0, self.render[1], 1.0, 1.0, 1.0];
                }
            }
            3 => {
                // TrailBlazer.cpp 0x000C5D00 outro
                let v7 = t - self.outro_start;
                let v5 = if self.success { 0.0 } else { p[15] };
                if v5 <= v7 {
                    if v5 + p[16] <= v7 {
                        let v3 = 1.0 - ((v7 - (v5 + p[16])) / p[17]).clamp(0.0, 1.0);
                        let v2 = 1.0 - ((v7 - (v5 + p[16])) / p[18]).clamp(0.0, 1.0);
                        self.render = [0.0, self.render[1], v3, v3, v2];
                        if v3 <= 0.0 {
                            // TrailBlazer.cpp 0x000C6120
                            self.mode = 0;
                            self.render = [0.0, 0.0, 0.0, self.render[3], self.render[4]];
                            out.push(TrailEvent::DangerEnabled(false));
                            out.push(TrailEvent::AudioStop);
                        }
                    } else {
                        self.render = [0.0, self.render[1], 1.0, 1.0, 1.0];
                    }
                } else {
                    let v4 = (v7 / v5).clamp(0.0, 1.0);
                    self.render = [1.0 - v4, self.render[1], 1.0, 1.0, 1.0];
                }
            }
            _ => {
                if self.mode != 2 {
                    log::error!("assert: m_mode == MAIN");
                }
                self.render = [1.0, self.render[1], 1.0, 1.0, 1.0];
            }
        }
    }

    /// TrailBlazer.cpp 0x000C5F90
    fn push_render(&self, out: &mut Vec<TrailEvent>) {
        if self.mode != 0 {
            out.push(TrailEvent::RenderParams { on: true, prev: self.prev, cur: self.render, success: self.success });
        } else {
            out.push(TrailEvent::RenderParams { on: false, prev: [0.0; 5], cur: [0.0; 5], success: false });
        }
    }
}

/// one read position over `TrailBlazerMessage` shared by both `apply_trail_blazer_calls` instances, so every
/// message is applied exactly once whether it was written in the Ai set or by the ai's trigger pass after Physics
#[derive(bevy::prelude::Resource, Default)]
pub struct TrailBlazerCalls(bevy::ecs::message::MessageCursor<TrailBlazerMessage>);

/// the trailblazer's immediate calls outside the ai, in message order: physics VE+0x10 (via
/// `VsAIPoliceVehicle__17DE0`), `C::VehicleEntity` 0x001CC2E0, the `::GC` / `dword_686B58` stores, the WMM time
/// stretch 0x000C61D0 (`dword_6FA8D0`+0x10), the OSD message 0x000C6200 (GameManager+0x108C) and the TB audio
/// (VsAudio::StartTb 0x0004BEA0, StopTb 0x0004C080, SetTbFadeIn(0) 0x0004C050 — the intro's lerp result is
/// discarded, the call passes 0), vibration effect 7 (0x001ED760 strength 1, then 0x001ED7E0 default time,
/// strength sqrt(level); the pad = the driver's player number). Runs before the physics
/// frame (Ai-set messages) and again after the ai's trigger pass, before the next update's physics.
#[allow(clippy::too_many_arguments)]
pub fn apply_trail_blazer_calls(
    mut vib: bevy::prelude::ResMut<crate::controller::vibrate_effects::Vibration>,
    presets: bevy::prelude::Res<crate::controller::vibrate_effects::VibratePresets>,
    cars: bevy::prelude::Query<(&crate::game_mode::race_cars::RaceCar, &velocity_physics::plugin::PhysicsCar)>,
    msgs: bevy::prelude::Res<bevy::ecs::message::Messages<TrailBlazerMessage>>,
    mut cursor: bevy::prelude::ResMut<TrailBlazerCalls>,
    physics: Option<bevy::prelude::Res<velocity_physics::plugin::PhysicsWorldRes>>,
    mut wmm: bevy::prelude::ResMut<crate::game_mode::update_thread::CWorldModeManager>,
    mut gm: bevy::prelude::ResMut<crate::game_mode::game_manager::GameManager>,
    mut audio_cmd: bevy::prelude::MessageWriter<velocity_audio::AudioCommand>,
    mut audio: Option<bevy::prelude::ResMut<velocity_audio::VisosAudio>>,
) {
    use velocity_physics::vehicle::vehicle_entity::sim_ai::TrailBlazerGlobals;
    let mut world = physics.as_deref().map(|p| p.lock());
    for m in cursor.0.read(&msgs) {
        let id = m.vehicle as usize;
        let w = world.as_mut().and_then(|w| w.as_mut());
        match m.event {
            TrailEvent::CarFlag(b) => match w.filter(|w| w.vehicle_ids().contains(&id)) {
                Some(w) => w.set_trail_blazer_flag(id, b),
                None => log::warn!("trailblazer: no physics vehicle {id}"),
            },
            TrailEvent::CarReset => {
                if let Some(w) = w.filter(|w| w.vehicle_ids().contains(&id)) {
                    w.trail_blazer_reset(id);
                }
            }
            TrailEvent::PhysicsGlobals { zone_velocity, zone_sensitivities, friction_sa, steer_bias_steering, steer_bias, g_686b58 } => {
                if let Some(w) = w {
                    w.set_trail_blazer_globals(&TrailBlazerGlobals {
                        zone_velocity,
                        zone_sensitivities,
                        friction_sa,
                        steer_bias_steering,
                        steer_bias,
                        g_686b58,
                    });
                }
            }
            TrailEvent::Vibrate(level) => {
                use crate::controller::vibrate_effects::{MAX_PADS, PRESET_TRAILBLAZER};
                if let Some(pad) = cars.iter().find(|(c, p)| c.human && p.id == id).map(|(c, _)| c.slot).filter(|p| *p < MAX_PADS) {
                    vib.pads[pad].add_or_update(&presets, PRESET_TRAILBLAZER, 1.0);
                    vib.pads[pad].update_existing(&presets, PRESET_TRAILBLAZER, -1.0, level);
                }
            }
            TrailEvent::TimeStretch(v) => wmm.time_stretch = v,
            TrailEvent::Message(crc) => gm.trail_blazer_message = crc,
            TrailEvent::AudioStart => {
                audio_cmd.write(velocity_audio::AudioCommand::StartTb);
            }
            TrailEvent::AudioStop => {
                audio_cmd.write(velocity_audio::AudioCommand::StopTb);
            }
            TrailEvent::AudioFadeIn(_) => {
                if let Some(a) = audio.as_deref_mut() {
                    a.game.set_tb_fade_in(0);
                }
            }
            _ => {}
        }
        if log::log_enabled!(log::Level::Debug) && !matches!(m.event, TrailEvent::RenderParams { .. } | TrailEvent::TimeStretch(_) | TrailEvent::Intensity(_) | TrailEvent::Vibrate(_) | TrailEvent::DangerZone(Some(_)) | TrailEvent::PhysicsGlobals { zone_sensitivities: None, .. }) {
            log::debug!("trailblazer {}: {:?}", m.vehicle, m.event);
        }
    }
}

/// GameManager::InGameUpdate 0x000B7839 — first thing every in-game update: the WMM's stretch to the frame timer
/// (UpdateThread 0x000C6B70 -> VsFrameTimer 0x00057E40)
pub fn apply_world_mode_stretch(
    wmm: bevy::prelude::Res<crate::game_mode::update_thread::CWorldModeManager>,
    mut clock: bevy::prelude::ResMut<crate::vs_common_main::GameClock>,
) {
    wmm.apply(&mut clock.timer);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Line;
    impl TrailSpline for Line {
        fn find_closest_segment(&self, _: Vec3) -> u32 {
            0
        }
        fn find_segment(&self, _: Vec3, _: u32) -> u32 {
            0
        }
        fn distance_along(&self, p: Vec3, _: u32) -> f32 {
            p.z.clamp(0.0, 100.0)
        }
        fn length(&self) -> f32 {
            100.0
        }
        fn point_at(&self, d: f32, _: u32) -> Vec3 {
            Vec3::new(0.0, 0.0, d)
        }
    }

    #[test]
    fn follow_to_the_end() {
        let mut t = TrailBlazer::new(0);
        let mut out = Vec::new();
        let mut car = TrailCar { position: Vec3::ZERO, speed: 35.0, heading: Vec3::Z };
        t.start(1, &Line, 5.0, 77, 0, false, &car, [0.0; 6], &mut out);
        for _ in 0..200 {
            car.position.z += 1.0;
            t.update(1.0 / 24.0, Some(&Line), &car, 0.0, &mut out);
            if !t.active() {
                break;
            }
        }
        assert!(t.success && t.ending);
        assert!(out.contains(&TrailEvent::PlayTimeline(77)));
    }
}
