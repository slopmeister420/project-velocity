//! src\TimelineManager\Objects\TimelinePhysics.cpp — physics objects (`F3PhysicsObject`, collision, immovable and
//! "Model" channels): swap, visibility, moves with velocities, attached particle systems and resets. The objects
//! themselves are the physics crate's, addressed by the channel's name crc.

use visos::type_::vs_type::Value;

use super::timeline_object::{
    field_bool, field_crc, field_vec3, EventPhase, HandlerCtx, TimelineObject, TimelineObjectMessage,
};

fn first(v: Option<&Value>) -> Option<&Value> {
    match v {
        Some(Value::Struct(f)) => f.first().map(|(_, v)| v),
        _ => None,
    }
}

pub fn event(obj: &TimelineObject, name: &str, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    let EventPhase::Begin { .. } = phase else { return };
    let object = obj.name_crc;
    match name {
        // TimelinePhysics.cpp 0x0017D1A0 — the named object shows, this one hides
        "F3EventInstanceSwap" => {
            let other = super::timeline_object::ccrc(first(value).and_then(Value::as_str).unwrap_or(""));
            ctx.out.push(TimelineObjectMessage::InstanceVisibility { object: other, visible: true });
            ctx.out.push(TimelineObjectMessage::InstanceVisibility { object, visible: false });
        }
        // TimelinePhysics.cpp 0x0017D3A0
        "F3EventInstanceVisibility" => {
            ctx.out.push(TimelineObjectMessage::InstanceVisibility { object, visible: field_bool(value, "MakeVisible") })
        }
        // TimelinePhysics.cpp 0x0017D540 — the location's z is negated (the event data keeps the exporter's axes)
        "F3EventInstanceMove" => {
            let mut location = field_vec3(value, "Location");
            location.z = -location.z;
            ctx.out.push(TimelineObjectMessage::InstanceMove {
                object,
                location,
                velocity: field_vec3(value, "Velocity"),
                angular_velocity: field_vec3(value, "AngularVelocity"),
            });
        }
        // TimelinePhysics.cpp 0x0017D8A0
        "F3EventPhysicsObjectParticleSys" => ctx.out.push(TimelineObjectMessage::PhysicsObjectParticles {
            object,
            system: field_crc(value, "System"),
            active: field_bool(value, "Activate"),
        }),
        // TimelinePhysics.cpp 0x0017DAA0 — a set first field resets them all
        "F3EventResetPhysicsObject" => {
            let all = first(value).and_then(Value::as_bool).unwrap_or(false);
            ctx.out.push(TimelineObjectMessage::ResetPhysicsObject { object: (!all).then_some(object) });
        }
        _ => {}
    }
}
