//! src\TimelineManager\Objects\TimelineSkinnedModel.cpp — the "Model" object (index 12): visibility of a skinned
//! timeline model. The type table sends "Model" channels to the physics object, so it is never created.

use visos::type_::vs_type::Value;

use super::timeline_object::{field_bool, EventPhase, ObjectState, TimelineObject};

/// TimelineSkinnedModel.cpp 0x0017E4A0
pub fn event(obj: &mut TimelineObject, value: Option<&Value>, phase: EventPhase) {
    if let EventPhase::Begin { .. } = phase {
        obj.state = ObjectState::SkinnedModel { visible: field_bool(value, "MakeVisible") };
    }
}
