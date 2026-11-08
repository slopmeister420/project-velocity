//! src\TimelineManager\Objects\TimelineSound.cpp — `VSAudioEmitter` objects (vtable 0x00641A58, code at
//! 0x0017E6E0..0x0017EBD0 under the skinned model's label): a 3D emitter (distances 75 / 150, volume 1) and a 2D
//! one, created on first use. The 3D path needs the object's +0x10 instance, which nothing ever sets (the
//! emitter has no onBind), so every sound plays on the 2D emitter by the event's own name.

use visos::type_::vs_type::Value;

use super::timeline_object::{field_crc, field_str, EventPhase, HandlerCtx, TimelineObject, TimelineObjectMessage};

pub fn event(obj: &TimelineObject, name: &str, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    let EventPhase::Begin { .. } = phase else { return };
    let object = obj.name_crc;
    let event = field_crc(value, "AudioEvent");
    match name {
        // TimelineSound.cpp 0x0017EC30 -> 0x0017E8A0 -> 0x0017EA00, and 0x0017ED30 -> 0x0017EA00
        "F3EventSound" | "F3Event2DSound" => ctx.out.push(TimelineObjectMessage::Sound { object, event, position: None }),
        // TimelineSound.cpp 0x0017EE30 — 0x0017EBD0 the named event, 0x0017EB80 everything (null name)
        "F3EventSoundStop" => {
            let named = field_str(value, "AudioEvent").is_some();
            ctx.out.push(TimelineObjectMessage::SoundStop { object, event: named.then_some(event) });
        }
        _ => {}
    }
}
