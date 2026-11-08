//! src\TimelineManager\Objects\TimelineHeatHaze.cpp — `F3HeatHaze`: the heat haze named like the object on or off.

use visos::type_::vs_type::Value;

use super::timeline_object::{field_bool, EventPhase, HandlerCtx, TimelineObject, TimelineObjectMessage};

/// TimelineHeatHaze.cpp 0x0017BD70
pub fn event(obj: &TimelineObject, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    if let EventPhase::Begin { .. } = phase {
        ctx.out.push(TimelineObjectMessage::HeatHaze { name: obj.name_crc, active: field_bool(value, "Activate") });
    }
}
