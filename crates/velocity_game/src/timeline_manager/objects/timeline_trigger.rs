//! src\TimelineManager\Objects\TimelineTrigger.cpp — trigger volumes (`triggers.trg`, track-wide and per race).
//!
//! ```text
//! +0x00 f32 version (1.5 only; older files are rejected "wrong version.  REBUILD.")
//! +0x04 u32 count   +0x08 runtime pointer   +0x0C pad
//! +0x10 FileTrigger[count], 80 bytes:
//!   +0x00 VsVector position (w = speed limit in mph for speed clamps)   +0x10 VsQuaternion (xyz negated on load)
//!   +0x20 f32[3] dimensions   +0x2C f32 bounding radius squared   +0x30 u32 name crc   +0x34 u32 type crc
//!   +0x38 u32 target name crc   +0x3C runtime per-vehicle inside bits   +0x40..+0x4C type parameters
//! ```

use visos::chunk::file_chunk::rd32;

/// the only accepted version (TimelineTrigger.cpp 0x0017FC10)
pub const TRIGGER_FILE_VERSION: f32 = 1.5;
pub const TRIGGER_RECORD_SIZE: usize = 80;
/// `MAX_TRAILBLAZER_TRIGS`
pub const MAX_TRAILBLAZER_TRIGS: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct FileTrigger {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub dimensions: [f32; 3],
    pub radius_sq: f32,
    pub name_crc: u32,
    pub type_crc: u32,
    pub target_crc: u32,
    /// +0x40, +0x44, +0x48, +0x4C
    pub params: [u32; 4],
}

impl FileTrigger {
    pub fn is_trailblazer(&self) -> bool {
        self.type_crc == visos::handy::handy_crc::crc32(b"TRAILBLAZER", 0)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TriggerFileError {
    #[error("** Error loading trigger file, wrong version {0}.  REBUILD. **")]
    Version(f32),
    #[error("trigger file truncated")]
    Truncated,
}

fn f(b: &[u8], o: usize) -> f32 {
    f32::from_bits(rd32(b, o))
}

/// TimelineTrigger.cpp 0x0017FC10
pub fn parse_triggers(b: &[u8]) -> Result<Vec<FileTrigger>, TriggerFileError> {
    if b.len() < 16 {
        return Err(TriggerFileError::Truncated);
    }
    let v = f(b, 0);
    if v != TRIGGER_FILE_VERSION {
        return Err(TriggerFileError::Version(v));
    }
    let n = rd32(b, 4) as usize;
    if 16 + n * TRIGGER_RECORD_SIZE > b.len() {
        return Err(TriggerFileError::Truncated);
    }
    let mut out = Vec::with_capacity(n);
    let mut trailblazers = 0;
    for i in 0..n {
        let o = 16 + i * TRIGGER_RECORD_SIZE;
        // TimelineSequencer.cpp 0x002B73B0 negates x, y, z of the quaternion
        let t = FileTrigger {
            position: [f(b, o), f(b, o + 4), f(b, o + 8), f(b, o + 12)],
            rotation: [-f(b, o + 16), -f(b, o + 20), -f(b, o + 24), f(b, o + 28)],
            dimensions: [f(b, o + 32), f(b, o + 36), f(b, o + 40)],
            radius_sq: f(b, o + 44),
            name_crc: rd32(b, o + 48),
            type_crc: rd32(b, o + 52),
            target_crc: rd32(b, o + 56),
            params: [rd32(b, o + 64), rd32(b, o + 68), rd32(b, o + 72), rd32(b, o + 76)],
        };
        if t.is_trailblazer() {
            trailblazers += 1;
            if trailblazers == MAX_TRAILBLAZER_TRIGS {
                log::warn!("PERFORMANCE WARNING: Too many Trailblazer triggers, MAX_TRAILBLAZER_TRIGS needs increased");
            }
        }
        out.push(t);
    }
    Ok(out)
}

/// TimelineTrigger.cpp 0x0017F980 / 0x0017FE20 — track file first, race file second; if only the race one loads
/// it takes the track slot.
pub fn load_level_triggers(track_dir: &str, race_dir: &str) -> [Option<Vec<FileTrigger>>; 2] {
    let load = |p: String| match visos::file::platform::read(&p) {
        Ok(b) => match parse_triggers(&b) {
            Ok(t) => Some(t),
            Err(e) => {
                log::warn!("{e} ({p})");
                None
            }
        },
        Err(_) => {
            log::warn!("** Error loading trigger file, {p}**");
            None
        }
    };
    let mut r = [load(format!("{track_dir}triggers.trg")), load(format!("{race_dir}triggers.trg"))];
    if r[0].is_none() {
        r.swap(0, 1);
    }
    if r[0].is_none() {
        log::info!("No Level triggers found");
    }
    r
}

/// TimelineTrigger.cpp 0x0017F4E0 — `F3EventTrigger` on the trigger named like the object: switch it on/off and
/// optionally fire it
/// TriggerUpdate.cpp 0x001812C0 (the fire: fire engine / traffic spawners and timelines only)
pub fn event(
    obj: &super::timeline_object::TimelineObject,
    value: Option<&visos::type_::vs_type::Value>,
    phase: super::timeline_object::EventPhase,
    ctx: &mut super::timeline_object::HandlerCtx,
) {
    use super::timeline_object::{field_bool, set_trigger_enabled, EventPhase, TimelineAction};
    use crate::triggers::trigger_update::{types, TriggerMessage};
    let EventPhase::Begin { .. } = phase else { return };
    let Some(t) = ctx.triggers.find(obj.name_crc).map(|t| t.file.clone()) else {
        log::warn!("timeline F3TriggerEvent, trigger object {:?} not found", obj.name);
        return;
    };
    set_trigger_enabled(ctx.triggers, obj.name_crc, field_bool(value, "Enable"));
    if !field_bool(value, "Trigger") {
        return;
    }
    match t.type_crc {
        types::FIRE_ENGINE_SPAWNER => {
            log::info!("Player vehicle has activated a fire engine spawner");
            ctx.triggered.push(TriggerMessage::SpawnFireEngines { id: t.target_crc });
        }
        types::TRAFFIC_SPAWNER => {
            log::info!("Player vehicle has entered a traffic spawner");
            ctx.triggered.push(TriggerMessage::SpawnTraffic { id: t.target_crc });
        }
        types::TIMELINE => ctx.actions.push(TimelineAction::Play(t.target_crc)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_trg() {
        let root = visos::file::game_dir();
        if !root.exists() {
            return;
        }
        let mut files = Vec::new();
        fn walk(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for e in std::fs::read_dir(d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out)
                } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("trg")) {
                    out.push(p)
                }
            }
        }
        walk(root, &mut files);
        assert_eq!(files.len(), 68);
        let (mut ok, mut old, mut n) = (0, 0, 0);
        for p in &files {
            match parse_triggers(&std::fs::read(p).unwrap()) {
                Ok(t) => {
                    ok += 1;
                    n += t.len();
                    for x in &t {
                        assert!(x.position.iter().all(|c| c.is_finite()) && x.dimensions.iter().all(|&d| d > 0.0), "{}", p.display());
                    }
                }
                Err(TriggerFileError::Version(_)) => old += 1,
                Err(e) => panic!("{}: {e}", p.display()),
            }
        }
        // 63 v1.5 files load; 5 older ones are rejected like the original does
        assert_eq!((ok, old), (63, 5));
        assert!(n > 0);
    }
}
