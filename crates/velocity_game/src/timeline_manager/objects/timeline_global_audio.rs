//! src\TimelineManager\Objects\TimelineGlobalAudio.cpp — `F3AudioController`: fader presets, ambience, music,
//! reverb and traffic audio.

use velocity_audio::AudioCommand;
use visos::handy::handy_crc::crc32;
use visos::type_::vs_type::Value;

use super::timeline_object::{field_bool, field_i32, EventPhase, HandlerCtx, TimelineObjectMessage};

fn first_crc(v: Option<&Value>) -> u32 {
    let s = match v {
        Some(Value::Struct(f)) => f.first().and_then(|(_, v)| v.as_str()),
        _ => None,
    };
    super::timeline_object::ccrc(s.unwrap_or(""))
}

pub fn event(name: &str, value: Option<&Value>, phase: EventPhase, ctx: &mut HandlerCtx) {
    let EventPhase::Begin { length, .. } = phase else { return };
    let on = field_bool(value, "Enable");
    match name {
        // TimelineGlobalAudio.cpp 0x0017B480
        "F3EventAudioFaderPreset" => {
            ctx.out.push(TimelineObjectMessage::FadeToVolumePreset { preset: field_i32(value, "Preset"), time: length })
        }
        // TimelineGlobalAudio.cpp 0x0017B520
        "F3EventAudioToggleEnvironment" => ctx.out.push(TimelineObjectMessage::CityAmbience(on)),
        // TimelineGlobalAudio.cpp 0x0017B5D0
        "F3EventAudioToggleMusic" => ctx.audio.push(if on { AudioCommand::ResumeMusic } else { AudioCommand::PauseMusic }),
        // TimelineGlobalAudio.cpp 0x0017B680
        "F3EventAudioToggleReverb" => ctx.audio.push(if on { AudioCommand::TunnelReverb } else { AudioCommand::DefaultReverb }),
        // TimelineGlobalAudio.cpp 0x0017B730
        "F3EventGlobalAudio" => {
            let c = first_crc(value);
            if c == crc32(b"disabletraffic", 0) {
                ctx.audio.push(AudioCommand::TrafficMuted(true));
            } else if c == crc32(b"enabletraffic", 0) {
                ctx.audio.push(AudioCommand::TrafficMuted(false));
            }
        }
        _ => {}
    }
}
