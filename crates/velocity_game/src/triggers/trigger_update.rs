//! src\Triggers\TriggerUpdate.cpp — trigger volumes vs vehicles: enter / still inside / leave, per vehicle id
//! (0x00181920), and what each trigger type does (0x00180310 enter, 0x00180B70 inside, 0x00180D00 leave).
//! Effects outside the core are announced as `TriggerMessage`s; audio ones go straight to `AudioCommand`.

use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use velocity_audio::AudioCommand;

use crate::timeline_manager::objects::timeline_trigger::FileTrigger;

/// trigger type crcs (`FileTrigger::type_crc`, HANDY_CRC32 of the upper-case name where recovered)
pub mod types {
    pub const KILL: u32 = 0x10F8_64C1;
    pub const TUNNEL: u32 = 0x9447_3365;
    pub const SPEED_CLAMP: u32 = 0x71D9_1EB3;
    /// second speed clamp id (name not recovered)
    pub const SPEED_CLAMP_2: u32 = 0xD8BE_0920;
    /// physics speed scale in percent (names not recovered)
    pub const SPEED_A: u32 = 0x5423_6539;
    pub const SPEED_B: u32 = 0x547A_EAA7;
    pub const PARLIAMENT: u32 = 0xB442_684A;
    pub const TRAILBLAZER: u32 = 0x9C6D_28D5;
    pub const TIMELINE: u32 = 0xE5EF_8543;
    pub const TRAFFIC_BLOCKER: u32 = 0xC626_8A84;
    pub const TRAFFIC_SPAWNER: u32 = 0xD58A_6ACD;
    pub const FIRE_ENGINE_SPAWNER: u32 = 0x90A5_9546;
    pub const FIRE_ENGINE_SPAWNER_2: u32 = 0xD936_21D6;
    /// plays the sound event in `param`
    pub const SOUND: u32 = 0xC12F_13A9;
    /// world mode manager disable/enable (`CWorldModeManager::SetDisabled`)
    pub const WMM: u32 = 0x8E37_BDCF;
    /// ai-only: random speed variation
    pub const AI_SPEED: u32 = 0x84B2_BAA5;
}

/// 0x3EE4E248 (0.44704): mph to m/s
pub const MPH_TO_MS: f32 = f32::from_bits(0x3EE4_E248);

/// a vehicle the triggers test this update (filled by the physics / ai owners).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerVehicle {
    /// vehicle id (< 32)
    pub id: u32,
    pub position: Vec3,
    /// the human player's car (`VsAIDriver::IsPlayer`)
    pub player: bool,
    /// traffic car (ai race cars and players are `false`)
    pub traffic: bool,
}

#[derive(Resource, Debug, Default)]
pub struct TriggerVehicles(pub Vec<TriggerVehicle>);

/// effects for other crates.
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub enum TriggerMessage {
    /// player hit a kill trigger: respawn at the target trigger (+0.75 up), facing its rotation
    /// (`VsAITrafficVehicle::Respawn` 0x0002BB30)
    Respawn { vehicle: u32, position: Vec3, rotation: Quat },
    /// speed clamp on/off for a vehicle (limit in m/s; the original sets `vehicle+0x10` and the flag at +0x0C)
    SpeedClamp { vehicle: u32, limit: Option<f32> },
    /// physics speed scale (param percent / 100)
    SpeedScale(f32),
    WorldModeDisabled(bool),
    SpawnTraffic { id: u32 },
    SpawnFireEngines { id: u32 },
    TrafficBlocker { id: u32 },
    /// a trailblazer trigger (TrailBlazer.cpp 0x000C4870): the ai spline to follow, how far off it the car may
    /// get (param 0 / 1000), the race timeline played on success (param 1), the trigger to respawn at on
    /// failure (param 2), param 3 (restore the timeline on success too)
    Trailblazer { vehicle: u32, spline: u32, max_distance: f32, success_timeline: u32, respawn_trigger: u32, flag: bool },
    /// start a race timeline (`RaceTimelineSequencer::TriggerTimeline`, 0x001711E0)
    Timeline { name_crc: u32 },
    /// a TIMELINE trigger whose target is "TriggerBigParticles": TrailBlazer 0x000C63E0(1) (danger particles on)
    BigParticles,
    PlaySound { event: u32 },
    /// ai-only triggers: `VsAIRaceVehicle::SetDrivingHint(n)` (0x00021980)
    AiHint { vehicle: u32, hint: u32 },
}

/// runtime trigger state (`Triggers.h`: active flag and the per-vehicle inside bits)
#[derive(Clone, Debug)]
pub struct CTrigger {
    pub file: FileTrigger,
    pub active: bool,
    pub inside: u32,
}

#[derive(Resource, Debug, Default)]
pub struct CTriggers {
    /// track list then race list (0x00181B40 indexes across both)
    pub list: Vec<CTrigger>,
}

impl CTriggers {
    pub fn from_lists(track: Option<&Vec<FileTrigger>>, race: Option<&Vec<FileTrigger>>) -> Self {
        let mut list = Vec::new();
        for l in [track, race].into_iter().flatten() {
            list.extend(l.iter().cloned().map(|file| CTrigger { file, active: true, inside: 0 }));
        }
        CTriggers { list }
    }

    /// TriggerUpdate.cpp 0x00181AB0 — by name crc
    pub fn find(&self, name_crc: u32) -> Option<&CTrigger> {
        if name_crc == 0 {
            return None;
        }
        self.list.iter().find(|t| t.file.name_crc == name_crc)
    }
}

/// TriggerUpdate.cpp 0x00180F80 — bounding sphere, then the oriented box (strict inequalities).
pub fn contains(t: &FileTrigger, point: Vec3) -> bool {
    let d = point - Vec3::new(t.position[0], t.position[1], t.position[2]);
    if d.length_squared() > t.radius_sq {
        return false;
    }
    let q = Quat::from_xyzw(t.rotation[0], t.rotation[1], t.rotation[2], t.rotation[3]);
    let l = q.inverse() * d;
    let [w, h, dz] = t.dimensions;
    l.x > -w * 0.5 && w * 0.5 > l.x && l.z > -dz * 0.5 && dz * 0.5 > l.z && l.y > -h * 0.5 && h * 0.5 > l.y
}

/// TriggerUpdate.cpp 0x00180310 — a vehicle entered.
fn on_enter(ts: &CTriggers, t: &CTrigger, v: &TriggerVehicle, wet: bool, out: &mut Vec<TriggerMessage>, audio: &mut Vec<AudioCommand>) {
    let f = &t.file;
    if v.traffic {
        return;
    }
    if !v.player {
        // ai cars: hints by type (LABEL_88)
        let hint = match f.type_crc {
            0x97D2_0A81 => Some(1),
            0x0EDB_5B3B => Some(2),
            0x79DC_6BAD => Some(3),
            0xE7B8_E10E => Some(4),
            0x90BF_CE98 => Some(5),
            _ => None,
        };
        if let Some(hint) = hint {
            out.push(TriggerMessage::AiHint { vehicle: v.id, hint });
        }
        return;
    }
    match f.type_crc {
        types::KILL => {
            log::info!("Player vehicle has entered a kill trigger");
            match ts.find(f.target_crc) {
                Some(target) => {
                    let p = target.file.position;
                    out.push(TriggerMessage::Respawn {
                        vehicle: v.id,
                        position: Vec3::new(p[0], p[1], p[2]) + Vec3::new(0.0, 0.75, 0.0),
                        rotation: Quat::from_xyzw(
                            target.file.rotation[0],
                            target.file.rotation[1],
                            target.file.rotation[2],
                            target.file.rotation[3],
                        ),
                    });
                }
                None => log::info!("Kill trigger need target trigger name!"),
            }
        }
        types::SPEED_A | types::SPEED_B => {
            let pct = f.params[0];
            let s = if pct == 100 { 1.0 } else { pct as f32 * 0.01 };
            out.push(TriggerMessage::SpeedScale(s));
        }
        types::SPEED_CLAMP | types::SPEED_CLAMP_2 => {
            log::info!("Player vehicle has entered a speed clamp trigger");
            out.push(TriggerMessage::SpeedClamp { vehicle: v.id, limit: Some(f.position[3] * MPH_TO_MS) });
        }
        types::WMM => out.push(TriggerMessage::WorldModeDisabled(f.params[0] != 0)),
        types::FIRE_ENGINE_SPAWNER | types::FIRE_ENGINE_SPAWNER_2 => {
            log::info!("Player vehicle has entered a fire engine spawner");
            out.push(TriggerMessage::SpawnFireEngines { id: f.target_crc });
        }
        types::TUNNEL => {
            audio.push(AudioCommand::EnterCover { wet });
            log::info!("Player vehicle has entered a tunnel");
        }
        types::TRAILBLAZER => {
            out.push(TriggerMessage::Trailblazer {
                vehicle: v.id,
                spline: f.target_crc,
                max_distance: f.params[0] as f32 / 1000.0,
                success_timeline: f.params[1],
                respawn_trigger: f.params[2],
                flag: f.params[3] != 0,
            });
        }
        types::PARLIAMENT => {
            audio.push(AudioCommand::EnterShop { wet });
            log::info!("Player vehicle has entered houses of parliament");
        }
        types::SOUND => out.push(TriggerMessage::PlaySound { event: f.target_crc }),
        types::TRAFFIC_BLOCKER => {
            log::info!("Player vehicle has just entered a Traffic Blocker trigger");
            out.push(TriggerMessage::TrafficBlocker { id: f.target_crc });
        }
        types::TRAFFIC_SPAWNER => {
            log::info!("Player vehicle has entered a traffic spawner");
            out.push(TriggerMessage::SpawnTraffic { id: f.target_crc });
        }
        types::TIMELINE => {
            out.push(TriggerMessage::Timeline { name_crc: f.target_crc });
            // VsStringAnsi::Hash = case-sensitive HANDY_CRC32(s, len, 0)
            if visos::handy::handy_crc::crc32(b"TriggerBigParticles", 0) == f.target_crc {
                out.push(TriggerMessage::BigParticles);
            }
        }
        _ => log::debug!("Player vehicle has entered other trigger box type"),
    }
}

/// TriggerUpdate.cpp 0x00180B70 — still inside.
fn on_inside(t: &CTrigger, v: &TriggerVehicle, out: &mut Vec<TriggerMessage>) {
    if v.player && matches!(t.file.type_crc, types::SPEED_CLAMP | types::SPEED_CLAMP_2) {
        out.push(TriggerMessage::SpeedClamp {
            vehicle: v.id,
            limit: Some(t.file.position[3] * MPH_TO_MS),
        });
    }
}

/// TriggerUpdate.cpp 0x00180D00 — left.
fn on_leave(t: &CTrigger, v: &TriggerVehicle, wet: bool, out: &mut Vec<TriggerMessage>, audio: &mut Vec<AudioCommand>) {
    if !v.player || v.traffic {
        return;
    }
    match t.file.type_crc {
        types::PARLIAMENT => {
            audio.push(AudioCommand::ExitShop { wet });
            log::info!("Player vehicle has left houses of parliament");
        }
        types::TUNNEL => {
            audio.push(AudioCommand::ExitCover { wet });
            log::info!("Player vehicle has left a tunnel");
        }
        types::SPEED_A | types::SPEED_B => log::info!("Player vehicle has left a speed trigger"),
        types::SPEED_CLAMP | types::SPEED_CLAMP_2 => {
            log::info!("Player vehicle has left a speed clamp trigger");
            out.push(TriggerMessage::SpeedClamp { vehicle: v.id, limit: None });
        }
        _ => log::debug!("Player vehicle has left other trigger box type"),
    }
}

/// a tunnel trigger the player's car was tested against (TriggerUpdate 0x00181120 → camera rain 0x00134EF0)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RainCover {
    /// trigger +0 (`VsAIRaceVehicle::23D30`)
    pub position: Vec3,
    /// trigger +40 (`LightStreak::135D30`, the box depth)
    pub length: f32,
    /// row 2 of Renderer 0x002AC0C0(trigger +16 quaternion)
    pub dir: Vec3,
    /// 0x00180F80's result
    pub inside: bool,
}

/// Renderer 0x002AC0C0's row 2 for the quaternion (x, y, z, w)
fn quat_row2(q: [f32; 4]) -> Vec3 {
    let [x, y, z, w] = q;
    let (x2, y2, z2) = (2.0 * x, 2.0 * y, 2.0 * z);
    Vec3::new(z2 * x - y2 * w, z2 * y + x2 * w, 1.0 - (x2 * x + y2 * y))
}

/// TriggerUpdate.cpp 0x00181920 — one vehicle against every trigger. `covers` gets 0x00181120's camera rain call
/// (non-traffic player car, TUNNEL type) for every tunnel trigger, inside or not.
pub fn update_vehicle(
    ts: &mut CTriggers,
    v: &TriggerVehicle,
    wet: bool,
    out: &mut Vec<TriggerMessage>,
    audio: &mut Vec<AudioCommand>,
    covers: &mut Vec<RainCover>,
) {
    if v.id >= 32 {
        debug_assert!(false, "u32ID < 32");
        return;
    }
    let bit = 1u32 << v.id;
    for i in 0..ts.list.len() {
        let inside = contains(&ts.list[i].file, v.position);
        let t = &ts.list[i];
        // 0x00181120: VsAIRaceVehicle 0x00024A40 (+124) == 0, VsAIDriver 0x00015D30 (+844 player), type 0x94473365
        if !v.traffic && v.player && t.file.type_crc == types::TUNNEL {
            let p = t.file.position;
            covers.push(RainCover { position: Vec3::new(p[0], p[1], p[2]), length: t.file.dimensions[2], dir: quat_row2(t.file.rotation), inside });
        }
        if inside {
            // inactive kill triggers do nothing
            if t.file.type_crc == types::KILL && !t.active {
                continue;
            }
            if t.inside & bit != 0 {
                on_inside(t, v, out);
            } else {
                ts.list[i].inside |= bit;
                let t = ts.list[i].clone();
                on_enter(ts, &t, v, wet, out, audio);
            }
        } else if t.inside & bit != 0 {
            ts.list[i].inside &= !bit;
            let t = ts.list[i].clone();
            on_leave(&t, v, wet, out, audio);
        }
    }
}

/// per game tick: every vehicle the owners reported.
pub fn update_triggers(
    mut ts: ResMut<CTriggers>,
    vehicles: Res<TriggerVehicles>,
    gm: Res<crate::game_mode::game_manager::GameManager>,
    mut tw: MessageWriter<TriggerMessage>,
    mut aw: MessageWriter<AudioCommand>,
    cams: Option<Res<crate::camera::game_cameras::GameCameras>>,
    mut rain: Option<ResMut<visos_render::camera_rain::CameraRain>>,
) {
    let wet = gm.race.as_ref().is_some_and(|r| r.wet);
    let (mut out, mut audio, mut covers) = (Vec::new(), Vec::new(), Vec::new());
    for v in &vehicles.0 {
        update_vehicle(&mut ts, v, wet, &mut out, &mut audio, &mut covers);
        // 0x00134EF0 per call: the camera manager's current camera (VsRender 0x0005ACD0: +212 position, +280 dir)
        if let (Some(r), Some(c)) = (rain.as_deref_mut(), cams.as_deref()) {
            let pos = crate::camera::game_cameras::current_camera_position(&c.manager).unwrap_or(Vec3::ZERO);
            let dir = crate::camera::game_cameras::current_camera_dir(&c.manager).unwrap_or(Vec3::ZERO);
            for k in covers.drain(..) {
                r.cover(wet, pos, dir, k.position, k.length, k.dir, k.inside);
            }
        }
        covers.clear();
    }
    for m in out {
        tw.write(m);
    }
    for a in audio {
        aw.write(a);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trig(ty: u32) -> FileTrigger {
        FileTrigger {
            position: [10.0, 0.0, 0.0, 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            dimensions: [4.0, 4.0, 10.0],
            radius_sq: 100.0,
            name_crc: 1,
            type_crc: ty,
            target_crc: 0,
            params: [0; 4],
        }
    }

    #[test]
    fn tunnel_enter_leave() {
        let mut ts = CTriggers::from_lists(Some(&vec![trig(types::TUNNEL)]), None);
        let mut v = TriggerVehicle { id: 0, position: Vec3::ZERO, player: true, traffic: false };
        let (mut o, mut a, mut c) = (Vec::new(), Vec::new(), Vec::new());
        update_vehicle(&mut ts, &v, false, &mut o, &mut a, &mut c);
        assert!(a.is_empty());
        // the rain cover call goes out every update, inside or not
        assert_eq!(c, vec![RainCover { position: Vec3::new(10.0, 0.0, 0.0), length: 10.0, dir: Vec3::Z, inside: false }]);
        v.position = Vec3::new(10.0, 0.0, 4.0);
        update_vehicle(&mut ts, &v, false, &mut o, &mut a, &mut c);
        update_vehicle(&mut ts, &v, false, &mut o, &mut a, &mut c);
        assert_eq!(a.len(), 1);
        assert!(c[2].inside);
        v.position = Vec3::new(10.0, 0.0, 6.0);
        update_vehicle(&mut ts, &v, false, &mut o, &mut a, &mut c);
        assert!(matches!(a[1], AudioCommand::ExitCover { .. }));
        // ai cars make no cover call
        v.player = false;
        c.clear();
        update_vehicle(&mut ts, &v, false, &mut o, &mut a, &mut c);
        assert!(c.is_empty());
    }

    #[test]
    fn cover_dir_is_renderer_row2() {
        // 90 degrees about y (x, y, z, w): Renderer 0x002AC0C0 row 2 = (2xz - 2yw, 2yz + 2xw, 1 - 2x² - 2y²)
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let d = quat_row2([0.0, s, 0.0, s]);
        assert!((d - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-6);
    }

    #[test]
    fn big_particles_timeline_trigger() {
        // LADown Circuit Race02 Triggers.trg record 17 targets 0xCACBD0F4
        assert_eq!(visos::handy::handy_crc::crc32(b"TriggerBigParticles", 0), 0xCACB_D0F4);
        let mut t = trig(types::TIMELINE);
        t.target_crc = 0xCACB_D0F4;
        let mut ts = CTriggers::from_lists(Some(&vec![t]), None);
        let v = TriggerVehicle { id: 0, position: Vec3::new(10.0, 0.0, 0.0), player: true, traffic: false };
        let (mut o, mut a, mut c) = (Vec::new(), Vec::new(), Vec::new());
        update_vehicle(&mut ts, &v, false, &mut o, &mut a, &mut c);
        assert_eq!(o, vec![TriggerMessage::Timeline { name_crc: 0xCACB_D0F4 }, TriggerMessage::BigParticles]);
    }
}
