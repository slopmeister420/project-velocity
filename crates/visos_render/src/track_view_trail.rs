//! trackview `--zonetrail <race dir>[,intensity[,section]]`: the race's trailblazer sections (AISpline.spl, read here
//! the way VsAIWorld 0x00030390 lays it out — velocity_ai cannot be linked from this crate) become zone trails over
//! the track's physics collision (ray casts as in game); the section's trail is switched on at start-up and updated
//! at 24 Hz of real time (logs the first point of each line)

use bevy::prelude::*;

use crate::danger_zone_render::DangerZoneInput;
use crate::trail_blazer_render::{TrailBlazerRender, TrailBlazerRenderCall, ZoneTrailLine};

const STEP: f32 = 1.0 / 24.0;

fn rd32(d: &[u8], p: usize) -> Option<u32> {
    Some(u32::from_le_bytes(d.get(p..p + 4)?.try_into().ok()?))
}

/// one road line at `*p` (16-byte header {kind, circular, flags, num_points, id} + vec4 points, w = distance)
fn road_line(d: &[u8], p: &mut usize) -> Option<(u32, Vec<Vec3>, f32)> {
    let circular = *d.get(*p + 4)? != 0;
    let n = rd32(d, *p + 8)? as usize;
    let id = rd32(d, *p + 12)?;
    *p += 16;
    let mut pts = Vec::with_capacity(n);
    let mut w = 0.0;
    for k in 0..n {
        let f = |o: usize| rd32(d, *p + 16 * k + o).map(f32::from_bits);
        pts.push(Vec3::new(f(0)?, f(4)?, f(8)?));
        w = f(12)?;
    }
    *p += 16 * n;
    // VsAIRoadLine setup: length = last w (+ the closing gap on circular lines)
    let length = if circular && n > 1 { pts[n - 1].distance(pts[0]) + w } else { w };
    Some((id, pts, length))
}

/// every lane line of every trailblazer section of `\Tracks\<track>\<race>\AISpline.spl`
pub fn trail_lines(track: &str, race: &str) -> Vec<ZoneTrailLine> {
    let path = format!("\\Tracks\\{track}\\{race}\\AISpline.spl");
    let Ok(d) = visos::file::platform::read(&path) else {
        log::error!("zonetrail: {path} not readable");
        return Vec::new();
    };
    let parse = || -> Option<Vec<ZoneTrailLine>> {
        let counts: Vec<u32> = (0..7).map(|k| rd32(&d, 0x10 + 4 * k)).collect::<Option<_>>()?;
        let mut p = 48usize;
        for _ in 0..counts[0] * 3 {
            road_line(&d, &mut p)?;
        }
        let mut out = Vec::new();
        // traffic, set B, set C, trailblazer
        for (set, &n) in [counts[1], counts[2], counts[3], counts[6]].iter().enumerate() {
            for s in 0..n {
                let lanes = rd32(&d, p)?;
                p += 16;
                for lane in 0..lanes {
                    let (id, points, length) = road_line(&d, &mut p)?;
                    if set == 3 {
                        out.push(ZoneTrailLine { section: s, lane, id, points, length });
                    }
                }
            }
        }
        Some(out)
    };
    parse().unwrap_or_default()
}

#[derive(Resource, Default)]
pub struct TrailDemo {
    started: bool,
    accum: f32,
}

/// start-up (first frame): build the trails, switch the chosen section's trail on; then 24 Hz updates
pub fn trail_demo(
    cfg: Res<crate::track_view::TrackViewConfig>,
    mut demo: ResMut<TrailDemo>,
    mut r: ResMut<TrailBlazerRender>,
    input: Option<ResMut<DangerZoneInput>>,
    time: Res<Time>,
) {
    let Some(spec) = cfg.zone_trail.as_deref() else { return };
    if !demo.started {
        demo.started = true;
        let mut parts = spec.split(',');
        let race = parts.next().unwrap_or_default();
        let intensity: f32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let section: u32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        let lines = trail_lines(&cfg.track, race);
        for l in &lines {
            log::info!("zonetrail: section {} lane {} id {:08X} {} points {:.1} m from {:?} via {:?}", l.section, l.lane, l.id, l.points.len(), l.length, l.points.first(), l.points.get(20));
        }
        let world = velocity_physics::c_config::load_global_config_with_fallback(0).and_then(|gc| {
            let track = velocity_physics::vehicle::vehicle_entity::cx::TrackWorld::load(&cfg.track).ok()?;
            Some(velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld::new(gc, Some(track)))
        });
        let Some(mut world) = world else {
            log::error!("zonetrail: no physics collision for {}", cfg.track);
            return;
        };
        let mut ray = |a: Vec3, b: Vec3| -> Option<(f32, Vec3)> {
            use velocity_physics::vehicle::vehicle_entity::sim::{RAY_NORMAL, RAY_TRACK};
            let h = world.ray_cast(a.to_array(), b.to_array(), RAY_TRACK | RAY_NORMAL, -1)?;
            Some((h.t, Vec3::from(h.normal)))
        };
        let id = lines.iter().find(|l| l.section == section).map(|l| l.id);
        r.apply(&TrailBlazerRenderCall::Init(lines), &mut ray);
        r.apply(&TrailBlazerRenderCall::Intensity(intensity), &mut ray);
        if let Some(id) = id {
            r.apply(&TrailBlazerRenderCall::Render { vehicle: 0, spline: Some(id) }, &mut ray);
        }
        log::info!("zonetrail: {} trails, {} blocks in the first", r.trails.len(), r.trails.first().map_or(0, |t| t.blocks.len()));
        return;
    }
    demo.accum += time.delta_secs();
    while demo.accum >= STEP {
        demo.accum -= STEP;
        r.update(STEP, false);
    }
    if let Some(mut i) = input {
        i.lerp = demo.accum / STEP;
    }
}
