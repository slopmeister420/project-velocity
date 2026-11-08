//! Positive control for `FCQuadTree::Build` / `Compact`: rebuilding every shipped `arena.qdt` from its
//! `arena.msh` with the file's own parameters. ExecTest, London2 and Tokyo2 come out word for word (stale
//! pointers and the builder's array capacities aside, one ulp on the cubic bounds); on the other four the
//! exporter's arithmetic put up to 3 borderline faces into a leaf differently, with identical trees.

use velocity_physics::fastcar::mesh::FcMesh;
use velocity_physics::fastcar::quadtree::FcQuadTree;

#[test]
fn rebuild_every_track_quadtree() {
    let root = visos::file::platform::game_dir().join("Tracks");
    if !root.is_dir() {
        return;
    }
    let mut n = 0;
    let mut exact = Vec::new();
    for e in std::fs::read_dir(&root).unwrap().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let (Ok(mb), Ok(qb)) = (
            visos::file::platform::read(&FcMesh::track_path(&name)),
            visos::file::platform::read(&FcQuadTree::track_path(&name)),
        ) else {
            continue;
        };
        let mesh = FcMesh::load(&mb).unwrap();
        let stock = FcQuadTree::load(&qb).unwrap();
        let mut q = FcQuadTree::new();
        assert!(q.build(&mesh, stock.min_cell_size, stock.bounds_padding, stock.max_faces_per_leaf, stock.up_axis));
        if stock.is_compact != 0 {
            assert!(q.compact(), "{name}: compact refused");
        }
        let img = q.to_image();
        let mut diff = Vec::new();
        assert_eq!(q.num_nodes, stock.num_nodes, "{name}: tree shape");
        assert!(q.num_indices.abs_diff(stock.num_indices) <= 4, "{name}: {} indices, stock {}", q.num_indices, stock.num_indices);
        if img.len() != qb.len() {
            eprintln!("{name}: {} nodes, {} indices (stock {})", q.num_nodes, q.num_indices, stock.num_indices);
            n += 1;
            continue;
        }
        for o in (0..img.len()).step_by(4) {
            // stale pointers and capacities in the object header
            if o < 96 && matches!(o, 0x0C | 0x10 | 0x2C | 0x30 | 0x54 | 0x58) {
                continue;
            }
            let w = |b: &[u8]| b[o..(o + 4).min(b.len())].iter().rev().fold(0u32, |a, &x| a << 8 | x as u32);
            // max = min + largest extent: the exporter kept the extent in x87 precision, the XBE stores it
            // as a float first (FCVec_Sub), so the cubic max may differ by one ulp
            if o < 96 && (0x20..0x2C).contains(&o) && w(&img).abs_diff(w(&qb)) <= 1 {
                continue;
            }
            if w(&img) != w(&qb) {
                diff.push(o);
            }
        }
        if let Some(&o) = diff.first() {
            let nodes_end = 96 + 4 * q.num_nodes as usize;
            let w = |b: &[u8], o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
            eprintln!("{name}: first diff {o:#x} (nodes end {nodes_end:#x}) built {:#x} stock {:#x}", w(&img, o), w(&qb, o));
        }
        eprintln!("{name}: {} nodes, {} indices, {} differing words", q.num_nodes, q.num_indices, diff.len());
        if diff.is_empty() {
            exact.push(name.clone());
        } else {
            assert!(diff.len() < q.num_nodes as usize + q.num_indices as usize / 2, "{name}");
        }

        n += 1;
    }
    assert!(n > 0);
    for t in ["ExecTest", "London2", "Tokyo2"] {
        assert!(exact.iter().any(|e| e == t), "{t} not rebuilt exactly");
    }
}
