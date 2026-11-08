//! src\TimelineManager\Objects\TimelineParticleGroup.cpp — `F3ParticleGroupController`: starts and stops particle
//! systems, and stops the ones it started when the timeline lets go of it (onUnbind 0x0017C260).

use visos::type_::vs_type::Value;

use super::timeline_object::{field_bool, field_crc, EventPhase, HandlerCtx, ObjectState, TimelineObject, TimelineObjectMessage};

/// TimelineParticleGroup.cpp 0x0017C390
pub fn event(obj: &mut TimelineObject, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    let EventPhase::Begin { .. } = phase else { return };
    let system = field_crc(value, "System");
    let active = field_bool(value, "Activate");
    ctx.out.push(TimelineObjectMessage::ParticleSystem { system, active });
    if active {
        if let ObjectState::ParticleGroup { systems } = &mut obj.state {
            systems.push(system);
        }
    }
}
