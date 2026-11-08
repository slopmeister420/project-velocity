//! src\TimelineManager\Objects\TimelineAI.cpp — the `F3AIController` handlers: race sections on/off, respawns and
//! the player's ai control. The race cars are the ai's; the trigger volumes the core's.

use bevy::math::{Quat, Vec3};
use visos::type_::vs_type::Value;

use super::timeline_object::{field_bool, field_crc, field_f32, EventPhase, HandlerCtx, TimelineObjectMessage};

/// first field as a float (`*(float*)data`)
fn first_f32(v: Option<&Value>) -> f32 {
    match v {
        Some(Value::Struct(f)) => f.first().and_then(|(_, v)| v.as_f32()).unwrap_or(0.0),
        _ => 0.0,
    }
}

pub fn event(name: &str, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    let EventPhase::Begin { .. } = phase else { return };
    match name {
        // TimelineAI.cpp 0x00176B50
        "F3EventEnableRaceSection" => ctx.out.push(TimelineObjectMessage::EnableRaceSection {
            raceline: field_crc(value, "RaceSection"),
            enable: field_bool(value, "Enable"),
        }),
        // TimelineAI.cpp 0x00176D90
        "F3EventRespawnPlayerOnRaceline" => {
            ctx.out.push(TimelineObjectMessage::RespawnRacersOnRaceline { value: first_f32(value) })
        }
        // TimelineAI.cpp 0x00176F00 — only with exactly one race car
        "F3EventPlayerAIControl" => {
            if ctx.world.racers == 1 {
                ctx.out.push(TimelineObjectMessage::PlayerHumanControl { human: !field_bool(value, "AIControl") });
            }
        }
        "F3EventRespawn" => respawn(value, ctx),
        _ => {}
    }
}

/// TimelineAI.cpp 0x00177150 — the area check trigger's inside bits pick the cars (human driven ones always);
/// the respawn points are triggers, taken in order
fn respawn(value: Option<&Value>, ctx: &mut HandlerCtx) {
    let area = field_crc(value, "AreaCheck");
    let Some(t) = ctx.triggers.find(area) else {
        log::warn!("Failed to find area check trigger");
        log::error!("assert: false");
        return;
    };
    let inside = t.inside;
    let points = (1..=16)
        .map(|n| {
            let c = field_crc(value, &format!("RespawnPoint{n}"));
            ctx.triggers.find(c).map(|s| {
                let (p, r) = (s.file.position, s.file.rotation);
                (Vec3::new(p[0], p[1], p[2]), Quat::from_xyzw(r[0], r[1], r[2], r[3]))
            })
        })
        .collect();
    ctx.out.push(TimelineObjectMessage::Respawn { inside, points, speed: field_f32(value, "RespawnSpeed") });
}
