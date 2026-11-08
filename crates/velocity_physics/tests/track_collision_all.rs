//! every track's `Gfx\arena.msh` / `arena.qdt`, the material table and the bump map, from the install

use velocity_physics::c_bump_map::CBumpMap;
use velocity_physics::fastcar::mesh::{intersect_ray_triangle, triangle_box_overlap, vec_cross, vec_length, vec_sub, FcMesh};
use velocity_physics::fastcar::quadtree::{compact_count, compact_index, compact_is_leaf, FcQuadTree};
use velocity_physics::material_properties::{
    xbe_read, MaterialProperties, MaterialTxt, DEFAULT_MATERIALS, SURF_NAMES, SURF_TYPE_MAX_TYPES,
};
use visos::file::platform;

fn tracks() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(platform::resolve(r"\Tracks"))
        .expect("Tracks dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("Gfx").join("arena.msh").exists())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// highest hit of a downward ray from `top` at (x, z), searching the leaf under the point
fn ground(m: &FcMesh, q: &FcQuadTree, x: f32, z: f32, top: f32) -> Option<(f32, u32)> {
    let o = [x, top, z];
    let dir = [0.0, -1.0, 0.0];
    let mut best: Option<(f32, u32)> = None;
    for f in q.faces_near(o) {
        let [a, b, c] = m.triangle(f);
        if let Some(h) = intersect_ray_triangle(a, b, c, o, dir) {
            if h.t >= 0.0 && best.is_none_or(|(t, _)| h.t < t) {
                best = Some((h.t, f));
            }
        }
    }
    best.map(|(t, f)| (top - t, f))
}

#[test]
fn track_collision_all() {
    let list = tracks();
    assert!(list.len() >= 7, "tracks: {list:?}");
    for t in &list {
        let m = FcMesh::open(&FcMesh::track_path(t)).unwrap();
        let q = FcQuadTree::open(&FcQuadTree::track_path(t)).unwrap();
        // whole file consumed
        let msz = platform::read(&FcMesh::track_path(t)).unwrap().len();
        let mut need = 32 + 12 * m.num_verts as usize + (if m.short_faces != 0 { 8 } else { 16 }) * m.num_faces as usize;
        let stored_normals = msz > need;
        if stored_normals {
            need += 12 * m.num_faces as usize;
        }
        assert_eq!(msz, need, "{t} msh size");
        let qsz = platform::read(&FcQuadTree::track_path(t)).unwrap().len();
        let qn = if q.is_compact != 0 { 4 } else { 40 } * q.num_nodes as usize;
        let qi = if q.short_indices != 0 { 2 } else { 4 } * q.num_indices as usize;
        assert_eq!(qsz, 96 + qn + qi, "{t} qdt size");

        // mesh: indices in range, surfaces valid
        let mut attrs = std::collections::BTreeMap::new();
        let mut bad_surf = 0;
        for f in 0..m.num_faces {
            for c in 0..3 {
                assert!(m.index(f, c) < m.num_verts, "{t} face {f}");
            }
            if m.surface(f) as usize >= SURF_TYPE_MAX_TYPES {
                bad_surf += 1;
            }
            *attrs.entry(m.attribute(f)).or_insert(0u32) += 1;
        }
        assert_eq!(bad_surf, 0, "{t} surface types out of range");
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for v in &m.verts {
            for k in 0..3 {
                lo[k] = lo[k].min(v[k]);
                hi[k] = hi[k].max(v[k]);
            }
        }
        // stored normals agree with the recomputed ones
        if stored_normals {
            let mut r = m.clone();
            r.compute_normals();
            let (mut flipped, mut off, mut degen) = (0, 0, 0);
            for f in 0..m.num_faces {
                let (a, b) = (m.normal(f), r.normal(f));
                let d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
                if (b[0] * b[0] + b[1] * b[1] + b[2] * b[2] - 1.0).abs() > 1e-3 {
                    degen += 1;
                } else if d < -0.999 {
                    flipped += 1;
                } else if d < 0.999 {
                    off += 1;
                }
            }
            println!("  {t} stored vs recomputed normals: flipped {flipped} other {off} degenerate {degen}");
        }

        // quadtree header vs mesh
        assert_eq!(q.num_mesh_faces, m.num_faces, "{t}");
        assert_eq!((q.axis0, q.axis1, q.up_axis), (0, 2, 1), "{t}");
        for k in 0..3 {
            assert!(q.min[k] <= lo[k] && hi[k] <= q.max[k], "{t} bounds axis {k}");
        }
        for i in 0..q.num_indices {
            assert!(q.face_index(i) < m.num_faces, "{t} index {i}");
        }
        // leaves: runs inside the index array; every face referenced by some leaf; cell boxes overlap their faces
        let mut seen = vec![false; m.num_faces as usize];
        let mut leaves = 0;
        let mut miss = 0;
        let mut degenerate_miss = 0;
        let mut leaf = |first: u32, count: u32, cmin: [f32; 2], cmax: [f32; 2]| {
            assert!(first + count <= q.num_indices, "{t} leaf run");
            leaves += 1;
            let c = [(cmin[0] + cmax[0]) * 0.5, (q.min[1] + q.max[1]) * 0.5, (cmin[1] + cmax[1]) * 0.5];
            // the builder tests the cell grown by the bounds padding (FCQuadTree::BuildNode 0x00359870)
            let pad = q.bounds_padding + 0.01;
            let h = [(cmax[0] - cmin[0]) * 0.5 + pad, (q.max[1] - q.min[1]) * 0.5 + pad, (cmax[1] - cmin[1]) * 0.5 + pad];
            for i in first..first + count {
                let f = q.face_index(i);
                seen[f as usize] = true;
                let [a, b, cc] = m.triangle(f);
                if !triangle_box_overlap(a, b, cc, c, h) {
                    let n = vec_cross(vec_sub(b, a), vec_sub(cc, a));
                    // collinear slivers fail the plane test whatever the box
                    if vec_length(n) < 1e-2 {
                        degenerate_miss += 1;
                        continue;
                    }
                    miss += 1;
                }
            }
        };
        if q.is_compact != 0 {
            let mut st = vec![(0u32, [q.min[0], q.min[2]], [q.max[0], q.max[2]])];
            while let Some((n, mn, mx)) = st.pop() {
                let w = q.compact[n as usize];
                if compact_is_leaf(w) {
                    leaf(compact_index(w), compact_count(w), mn, mx);
                } else {
                    let mid = [(mn[0] + mx[0]) * 0.5, (mn[1] + mx[1]) * 0.5];
                    for k in 0..4u32 {
                        let (a0, b0) = if k & 1 != 0 { (mid[0], mx[0]) } else { (mn[0], mid[0]) };
                        let (a1, b1) = if k & 2 != 0 { (mid[1], mx[1]) } else { (mn[1], mid[1]) };
                        assert!(compact_index(w) + k < q.num_nodes);
                        st.push((compact_index(w) + k, [a0, a1], [b0, b1]));
                    }
                }
            }
        } else {
            for (i, n) in q.nodes.iter().enumerate() {
                assert!(n.parent == -1 || (n.parent as u32) < q.num_nodes, "{t} parent");
                assert_eq!(n.parent == -1, i == 0, "{t} root");
                if n.leaf != 0 {
                    leaf(n.child[0], n.child[1], n.min, n.max);
                } else {
                    for c in n.child {
                        assert!(c < q.num_nodes && q.nodes[c as usize].parent == i as i32, "{t} child");
                    }
                }
            }
        }
        let unseen = seen.iter().filter(|s| !**s).count();
        println!(
            "{t:9} verts {:6} faces {:6} short {} normals {} | qdt nodes {:6} {} indices {:7} short {} leaves {:6} \
             cell {} pad {} maxPerLeaf {} | unreferenced faces {unseen} leaf faces not touching cell {miss} (degenerate {degenerate_miss}) attributes {:?}",
            m.num_verts,
            m.num_faces,
            m.short_faces,
            if stored_normals { "file" } else { "computed" },
            q.num_nodes,
            if q.is_compact != 0 { "compact" } else { "40B" },
            q.num_indices,
            q.short_indices,
            leaves,
            q.min_cell_size,
            q.bounds_padding,
            q.max_faces_per_leaf,
            attrs
        );
        assert_eq!(miss, 0, "{t} faces outside their leaf cell");

        // find_leaf agrees with a hint-started search and the compact trace path
        let mut hint = 0;
        for s in 0..64 {
            let fx = (s % 8) as f32 / 8.0 + 0.03;
            let fz = (s / 8) as f32 / 8.0 + 0.05;
            let p = [lo[0] + (hi[0] - lo[0]) * fx, 0.0, lo[2] + (hi[2] - lo[2]) * fz];
            let a = q.find_leaf(p, 0, None);
            let b = q.find_leaf(p, hint, None);
            assert_eq!(a, b, "{t} hint");
            hint = a.2;
            if q.is_compact != 0 {
                let mut tr = Default::default();
                assert_eq!(q.find_leaf(p, 0, Some(&mut tr)), a, "{t} trace");
            }
        }

        // vertical rays down the middle of the track bounds
        if t == "LACoast" {
            let top = hi[1] + 10.0;
            let mut hits = 0;
            for s in 0..9 {
                let x = lo[0] + (hi[0] - lo[0]) * (0.1 + 0.1 * s as f32);
                let z = lo[2] + (hi[2] - lo[2]) * (0.1 + 0.1 * s as f32);
                match ground(&m, &q, x, z, top) {
                    Some((y, f)) => {
                        hits += 1;
                        println!(
                            "  LACoast ray ({x:9.2}, {z:9.2}) -> y {y:8.3} face {f} surf {} zone {}",
                            MaterialProperties::get_name(m.surface(f)),
                            m.zone(f)
                        )
                    }
                    None => println!("  LACoast ray ({x:9.2}, {z:9.2}) -> miss"),
                }
            }
            assert!(hits > 0, "no ray hits on LACoast");
        }
    }
}

#[test]
fn materials_and_bump_map() {
    let xbe = platform::read(r"\default.xbe").expect("default.xbe");
    // the table constants match the executable's static initialiser
    let raw = xbe_read(&xbe, 0x006879C0, 12 * SURF_TYPE_MAX_TYPES).unwrap();
    for (i, e) in DEFAULT_MATERIALS.iter().enumerate() {
        let w = |k: usize| u32::from_le_bytes(raw[12 * i + 4 * k..12 * i + 4 * k + 4].try_into().unwrap());
        assert_eq!(
            [w(0), w(1), w(2)],
            [e.friction_multiplier.to_bits(), e.drag.to_bits(), e.bump_map_height.to_bits()],
            "{}",
            SURF_NAMES[i]
        );
    }
    let mut mp = MaterialProperties::default();
    assert!(mp.set_tweak("tarmac_wet", "Friction multiplier", 0.25));
    assert_eq!(mp.get(17).friction_multiplier, 0.25);

    // the exporter table: every column is a surface name
    let txt = String::from_utf8_lossy(&platform::read(r"\Shaders\material.txt").unwrap()).into_owned();
    let mt = MaterialTxt::parse(&txt).unwrap();
    assert_eq!(mt.columns.len(), 25);
    for c in &mt.columns {
        assert!(MaterialTxt::surf_index(c).is_some(), "column {c}");
    }
    for (k, v) in &mt.rows {
        assert_eq!(v.len(), mt.columns.len(), "row {k}");
    }
    println!("material.txt rows {:?}", mt.rows.iter().map(|r| &r.0).collect::<Vec<_>>());

    let bm = CBumpMap::from_xbe(&xbe).unwrap();
    assert_eq!(bm.tile_size, 100.0);
    let (mn, mx) = bm.height.iter().fold((f32::MAX, f32::MIN), |(a, b), &h| (a.min(h), b.max(h)));
    println!("bump map 30x30 tile {} heights {mn}..{mx}", bm.tile_size);
    assert!(mn.is_finite() && mx.is_finite() && mx < 10.0 && mn > -10.0);
    // cell corners reproduce the table; mirrored and periodic
    assert_eq!(bm.get_height(0.0, 0.0), bm.height[0]);
    assert_eq!(bm.get_height(-12.5, 31.25), bm.get_height(12.5, -31.25));
    assert_eq!(bm.get_height(12.5, 31.25), bm.get_height(112.5, 231.25));
    for s in 0..50 {
        let h = bm.get_height(s as f32 * 7.3, s as f32 * 3.1);
        assert!(h >= mn - 1e-4 && h <= mx + 1e-4);
    }
}
