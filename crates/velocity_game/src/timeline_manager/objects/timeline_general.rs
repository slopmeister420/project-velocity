//! src\TimelineManager\Objects\TimelineGeneral.cpp — `F3TimelineController`: one timeline plays or stops another.

use visos::type_::vs_type::Value;

use super::timeline_object::{field_bool, field_crc, EventPhase, HandlerCtx, TimelineAction};

pub fn event(value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    if !field_bool(value, "Activate") {
        return;
    }
    let name = field_crc(value, "Timeline");
    match phase {
        // TimelineGeneral.cpp 0x0017B130
        EventPhase::Begin { .. } => ctx.actions.push(TimelineAction::Play(name)),
        // TimelineGeneral.cpp 0x0017B1E0 -> RaceTimelineSequencer.cpp 0x001723B0 StopTimeline, empty in F3_D
        EventPhase::End | EventPhase::Update { .. } => {}
    }
}
