//! src\TimelineManager\Objects\TimelineExplosion.cpp — `F3ExplosionObject`: an `F3EventExplosion` blows up at the
//! object with its attributes.

use super::timeline_object::{field_bool, field_f32, EventPhase, HandlerCtx, TimelineObject, TimelineObjectMessage};

/// `byte_74B298` (never set): explosions off
pub const EXPLOSIONS_DISABLED: bool = false;

/// TimelineExplosion.cpp 0x0017AE60 -> 0x0017ACD0 ("BOOM!")
pub fn event(obj: &TimelineObject, phase: EventPhase, ctx: &mut HandlerCtx) {
    let EventPhase::Begin { .. } = phase else { return };
    if EXPLOSIONS_DISABLED {
        return;
    }
    let a = obj.data.as_ref();
    if a.is_none() {
        log::error!("assert: explosionAttribs");
    }
    log::debug!("BOOM!");
    ctx.out.push(TimelineObjectMessage::Explosion {
        position: obj.world.w_axis.truncate(),
        blast_radius: field_f32(a, "BlastRadius"),
        force_objects: field_f32(a, "Force_Objects"),
        angular_force_objects: field_f32(a, "Angular_Force_Objects"),
        force_vehicles: field_f32(a, "Force_Vehicles"),
        angular_force_vehicles: field_f32(a, "Angular_Force_Vehicles"),
        sector_angle: field_f32(a, "SectorAngle"),
        ignore_player: if field_bool(a, "Ignore_Player_Car") { 1 } else { -1 },
    });
}
