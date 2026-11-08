//! The collision side of the spline barriers (src/Render/BarrierRender.cpp): every barrier line the AI loaded
//! (arrow barriers, then no-entry barriers) becomes a 4 m wall strip the physics' wall pass collides with. The
//! barrier rendering itself (arrow/X sprites, 0x00109080) belongs to graphics and is not here.

use crate::ai::armco::{first_vertex, ArmcoMesh, ARMCO_MATERIAL};
use crate::ai::vs_ai_road_line::VsAIRoadLine;
use crate::ai::vs_ai_tools::VsAITools;
use crate::ai::vs_ai_world::VsAIWorld;
use crate::ai::vs_math::{vs_distance_squared, VsVector4};

/// `dword_68656C`: wall height
const WALL_HEIGHT: f32 = 4.0;

/// the barrier lines in render-data order (BarrierRender 0x00107DA0: kind 0 then kind 1).
fn barriers(w: &VsAIWorld) -> Vec<&VsAIRoadLine> {
    (0..2u32).flat_map(|k| (0..w.get_num_barriers(k) as i32).filter_map(move |i| w.get_barrier(k, i))).collect()
}

/// the barrier collision mesh (`m_pFC_Mesh`, faces `[v0, v1, v2, material 21]`); the caller builds the
/// quadtree (`FCQuadTree::Build(8.0, 2.375, 0, 1)` + `Compact`). None when there are no barriers or the
/// `nobarriers` switch cleared flag 4.
/// BarrierRender.cpp 0x00108030
pub fn build_collision_mesh(w: &VsAIWorld) -> Option<ArmcoMesh> {
    let lines: Vec<&VsAIRoadLine> = barriers(w).into_iter().filter(|b| b.header.flags[0] != 1).collect();
    if lines.is_empty() {
        return None;
    }
    let nverts: usize = lines.iter().map(|b| 2 * b.points.len()).sum();
    let mut m = ArmcoMesh { vertices: Vec::with_capacity(nverts), faces: Vec::with_capacity(nverts - 2 * lines.len()) };
    for b in &lines {
        for p in &b.points {
            m.vertices.push([p.x, p.y, p.z]);
            m.vertices.push([p.x, p.y + WALL_HEIGHT, p.z]);
        }
    }
    debug_assert_eq!(m.vertices.len(), nverts, "m_pFC_Mesh->number_of_vertices == (int)dNumVerts");
    let sections = w.race_sections();
    let mut v = 0usize;
    for b in &lines {
        let n = b.points.len();
        // which side faces the road: 2 = +1, 3 = -1, 0 = a vote over the points against the nearest raceline
        // segment's midpoint
        let mut side: i32 = 0;
        match b.header.flags[0] {
            0 => {
                for i in 0..n {
                    let p = &b.points[i];
                    let mut best = f32::from_bits(1343554297);
                    let (mut s0, mut s1) = (VsVector4::default(), VsVector4::default());
                    for s in sections {
                        let seg = s.raceline.find_closest_segment(p);
                        let (a, e) = s.raceline.get_segment(seg);
                        // original: distance to the segment's start, not to the segment
                        let d = vs_distance_squared(&a, p);
                        if best > d {
                            best = d;
                            s0 = a;
                            s1 = e;
                        }
                    }
                    let mut mid = s0;
                    mid += s1;
                    mid *= 0.5;
                    if VsAITools::is_left_of_line(p, &b.points[(i + 1) % n], &mid) {
                        side -= 1;
                    } else {
                        side += 1;
                    }
                }
            }
            2 => side = 1,
            3 => side = -1,
            _ => debug_assert!(false, "false"),
        }
        let mut flip = side >= 0;
        // the original's unsigned 2n - 2 never wraps for shipped data (every line has points)
        for _ in 0..(2 * n).saturating_sub(2) {
            let tri = [v, v + 1, v + 2].map(|i| first_vertex(&m.vertices, m.vertices[i]));
            // CircuitEditorTools.cpp 0x0007F8C0
            m.faces.push(if flip { [tri[0], tri[1], tri[2], ARMCO_MATERIAL] } else { [tri[0], tri[2], tri[1], ARMCO_MATERIAL] });
            flip = !flip;
            v += 1;
        }
        v += 2;
    }
    Some(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// every race with barrier files builds a mesh whose faces index real vertices; the vote picks a side.
    #[test]
    fn barrier_meshes_all_races() {
        let Some(dir) = crate::game_dir() else { return };
        let mut built = 0;
        let mut stack = vec![dir.join("Tracks")];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if !p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("AISpline.spl")) {
                    continue;
                }
                let level = p.parent().unwrap();
                let mut w = VsAIWorld::new(dir.join("AI"));
                w.load_spline_data(level);
                w.process_spline_data(None);
                w.load_barrier_data(level);
                if let Some(m) = build_collision_mesh(&w) {
                    let lines = barriers(&w).into_iter().filter(|b| b.header.flags[0] != 1).count();
                    assert_eq!(m.faces.len(), m.vertices.len() - 2 * lines, "{}", level.display());
                    assert!(m.faces.iter().all(|f| (f[0].max(f[1]).max(f[2]) as usize) < m.vertices.len()));
                    built += 1;
                }
            }
        }
        eprintln!("{built} barrier meshes");
        assert!(built > 0);
    }
}
