//! src\TimelineManager\Objects\TimelineCameraShaker.cpp — besides the shaker object (its camera envelope is in
//! `timeline_camera`), this file holds the `F3ScreenEffectController` handlers: screen flash, cinematic bars,
//! player camera shake, the story TFT movie (empty) and the trail blazer firewall.

use visos::type_::vs_type::Value;

use super::timeline_object::{field_bool, field_f32, EventPhase, HandlerCtx, TimelineObjectMessage};

pub fn screen_effect(name: &str, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    match (name, phase) {
        // TimelineCameraShaker.cpp 0x0017A490 — intensity and colour are not passed on
        ("F3EventScreenFlash", EventPhase::Begin { .. }) => ctx.out.push(TimelineObjectMessage::ScreenFlash {
            time_in: field_f32(value, "TimeIn"),
            time_out: field_f32(value, "TimeOut"),
            intensity: 0.5,
        }),
        // TimelineCameraShaker.cpp 0x0017A540 / 0x0017A580 — `On` is not read
        ("F3EventCinematicBars", EventPhase::Begin { .. }) => ctx.out.push(TimelineObjectMessage::CinematicBars(true)),
        ("F3EventCinematicBars", EventPhase::End) => ctx.out.push(TimelineObjectMessage::CinematicBars(false)),
        // TimelineCameraShaker.cpp 0x0017A5F0
        ("F3EventPlayerCameraShake", EventPhase::Begin { length, .. }) => ctx
            .out
            .push(TimelineObjectMessage::PlayerCameraShake { intensity: field_f32(value, "Intensity"), length }),
        // TimelineCameraShaker.cpp 0x0017A750
        ("F3EventTrailBlazerFirewall", EventPhase::Begin { .. }) => {
            ctx.out.push(TimelineObjectMessage::TrailBlazerFirewall(field_bool(value, "On")))
        }
        _ => {}
    }
}
