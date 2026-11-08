//! temporary probe: London / London2 crests on the racelines (jump cam take-off candidates) near the fixed cameras
use velocity_ai::ai::vs_ai_world::VsAISplineData;

#[test]
#[ignore]
fn london_crests() {
    let mut out = String::new();
    for track in ["LONDON", "LONDON2"] {
        for ty in ["CIRCUIT", "SPRINT"] {
            for n in 1..=10 {
                let dir = format!(r"\Tracks\{track}\{ty}\RACE{n:02}\");
                let Ok(d) = visos::file::platform::read(&format!("{dir}AISpline.spl")) else { continue };
                let Ok(s) = VsAISplineData::parse(&d) else { continue };
                let cams = visos::file::platform::read(&format!("{dir}cameras.cam"))
                    .ok()
                    .and_then(|b| velocity_game::replays::cameras::replay_camera_fixed::parse_cameras(&b).ok())
                    .map(|c| c.cameras.iter().map(|c| c.position).collect::<Vec<_>>())
                    .unwrap_or_default();
                for (i, r) in s.race_sections.iter().enumerate() {
                    let p = &r.raceline.points;
                    for k in 2..p.len().saturating_sub(2) {
                        let (a, b, c) = (p[k - 2], p[k], p[k + 2]);
                        let h1 = ((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt().max(0.1);
                        let h2 = ((c.x - b.x).powi(2) + (c.z - b.z).powi(2)).sqrt().max(0.1);
                        let (s1, s2) = ((b.y - a.y) / h1, (c.y - b.y) / h2);
                        if s1 - s2 > 0.12 && h1 < 30.0 && h2 < 30.0 {
                            let cam = cams
                                .iter()
                                .map(|q| ((q.x - b.x).powi(2) + (q.y - b.y).powi(2) + (q.z - b.z).powi(2)).sqrt())
                                .fold(f32::MAX, f32::min);
                            out += &format!(
                                "{track} {ty} {n:02} sec {i} pt {k}: ({:.1} {:.1} {:.1}) slope {s1:.2} -> {s2:.2} cams {} nearest {cam:.0}\n",
                                b.x, b.y, b.z,
                                cams.len()
                            );
                        }
                    }
                }
            }
        }
    }
    std::fs::write(std::env::var("PROBE_OUT").unwrap(), out).unwrap();
}
