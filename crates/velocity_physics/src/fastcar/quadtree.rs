//! FastCar `FCQuadTree` — the level collision quadtree (`Gfx\arena.qdt`). A 2D tree over the two
//! horizontal axes; leaves hold runs of the face-index array.
//!
//! File = the 96-byte object image followed by its arrays, all LE:
//! ```text
//! +00 numIndices   +04 numNodes   +08 shortIndices   +0C/+10 ptrs (stale)
//! +14 f32 min[3]   +20 f32 max[3]  (cubic: max = min + largest extent, padded)
//! +2C nodeCapacity +30 indexCapacity (overwritten on load)
//! +34 f32 minCellSize (8)  +38 f32 boundsPadding (2.5)  +3C axis0  +40 axis1  +44 upAxis (1 = Y)
//! +48 built  +4C maxFacesPerLeaf (8)  +50 numMeshFaces  +54/+58 ptrs  +5C compact
//! ```
//! then nodes (`u32 * numNodes` if compact, else 40-byte `FcQtNode`), then indices
//! (`u16 * numIndices` if shortIndices else `u32`).
//! Compact node word: top byte 0xFF = inner node, low 24 bits = first of 4 consecutive children;
//! otherwise leaf, low 24 bits = first index, top byte = count. Child order is the quadrant
//! `(p[axis0] > mid) | (p[axis1] > mid) << 1`.

use super::mem::V3;
use super::mesh::{MeshError, Reader};

/// 40-byte node of the uncompacted tree
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FcQtNode {
    pub parent: i32,   // +0x00 (-1 = root)
    pub min: [f32; 2], // +0x04 (axis0, axis1)
    pub max: [f32; 2], // +0x0C
    pub leaf: u32,     // +0x14
    /// leaf: `[first, count, ..]`; inner: child node indices
    pub child: [u32; 4], // +0x18
}

/// the per-level cell rectangles `FindLeaf` records for the debug drawing (`a7`, 324 bytes)
#[derive(Clone, Debug, Default)]
pub struct FcQtTrace {
    pub min: [[f32; 2]; 20], // +0
    pub max: [[f32; 2]; 20], // +160
    pub depth: i32,          // +320
}

#[derive(Clone, Debug, Default)]
pub struct FcQuadTree {
    pub num_indices: u32,   // +0x00
    pub num_nodes: u32,     // +0x04
    pub short_indices: u32, // +0x08
    pub indices32: Vec<u32>, // +0x0C
    pub indices16: Vec<u16>, // +0x10
    pub min: V3,            // +0x14
    pub max: V3,            // +0x20
    pub node_capacity: u32, // +0x2C
    pub index_capacity: u32, // +0x30
    pub min_cell_size: f32, // +0x34
    pub bounds_padding: f32, // +0x38
    pub axis0: u32,         // +0x3C
    pub axis1: u32,         // +0x40
    pub up_axis: u32,       // +0x44
    pub built: u32,         // +0x48
    pub max_faces_per_leaf: u32, // +0x4C
    pub num_mesh_faces: u32, // +0x50
    pub nodes: Vec<FcQtNode>, // +0x54
    pub compact: Vec<u32>,  // +0x58
    pub is_compact: u32,    // +0x5C
}

fn rd_u32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn rd_f32(b: &[u8], o: usize) -> f32 {
    f32::from_bits(rd_u32(b, o))
}

/// FCQTCompact_IsLeaf (FastCar) 0x0035A790
#[inline]
pub fn compact_is_leaf(w: u32) -> bool {
    (w >> 24) != 255
}
/// FCQTCompact_Index (FastCar) 0x0035A8C0
#[inline]
pub fn compact_index(w: u32) -> u32 {
    w & 0xFFFFFF
}
/// FCQTCompact_Count (FastCar) 0x0035B230
#[inline]
pub fn compact_count(w: u32) -> u32 {
    w >> 24
}

impl FcQuadTree {
    /// (FastCar) 0x000140B0
    pub fn new() -> Self {
        Self { axis0: u32::MAX, axis1: u32::MAX, up_axis: u32::MAX, ..Default::default() }
    }

    /// game path of a track's quadtree (`gfx\arena.msh` with the extension swapped, `PhysicsManager::Init` 0x0019CA80)
    pub fn track_path(track: &str) -> String {
        format!(r"\Tracks\{track}\gfx\arena.qdt")
    }

    /// load through the game-dir resolver
    pub fn open(game_path: &str) -> Result<Self, MeshError> {
        Self::load(&visos::file::platform::read(game_path)?)
    }

    /// (FastCar) 0x0019C7A0
    pub fn load(bytes: &[u8]) -> Result<Self, MeshError> {
        let mut r = Reader { b: bytes, pos: 0 };
        let h = r.take(96)?;
        let mut q = FcQuadTree {
            num_indices: rd_u32(h, 0x00),
            num_nodes: rd_u32(h, 0x04),
            short_indices: rd_u32(h, 0x08),
            min: [rd_f32(h, 0x14), rd_f32(h, 0x18), rd_f32(h, 0x1C)],
            max: [rd_f32(h, 0x20), rd_f32(h, 0x24), rd_f32(h, 0x28)],
            min_cell_size: rd_f32(h, 0x34),
            bounds_padding: rd_f32(h, 0x38),
            axis0: rd_u32(h, 0x3C),
            axis1: rd_u32(h, 0x40),
            up_axis: rd_u32(h, 0x44),
            built: rd_u32(h, 0x48),
            max_faces_per_leaf: rd_u32(h, 0x4C),
            num_mesh_faces: rd_u32(h, 0x50),
            is_compact: rd_u32(h, 0x5C),
            ..Default::default()
        };
        q.node_capacity = q.num_nodes;
        q.index_capacity = q.num_indices;
        let n = q.num_nodes as usize;
        if q.is_compact != 0 {
            q.compact = r.take(4 * n)?.chunks_exact(4).map(|c| rd_u32(c, 0)).collect();
        } else {
            q.nodes = r
                .take(40 * n)?
                .chunks_exact(40)
                .map(|c| FcQtNode {
                    parent: rd_u32(c, 0) as i32,
                    min: [rd_f32(c, 4), rd_f32(c, 8)],
                    max: [rd_f32(c, 12), rd_f32(c, 16)],
                    leaf: rd_u32(c, 20),
                    child: [rd_u32(c, 24), rd_u32(c, 28), rd_u32(c, 32), rd_u32(c, 36)],
                })
                .collect();
        }
        let ni = q.num_indices as usize;
        if q.short_indices != 0 {
            q.indices16 = r.take(2 * ni)?.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        } else {
            q.indices32 = r.take(4 * ni)?.chunks_exact(4).map(|c| rd_u32(c, 0)).collect();
        }
        Ok(q)
    }

    /// FCQuadTree::GetFaceIndex 0x0011F2E0 (CollisionMeshRender.cpp, inline accessor)
    #[inline]
    pub fn face_index(&self, i: u32) -> u32 {
        if self.short_indices != 0 {
            self.indices16[i as usize] as u32
        } else {
            self.indices32[i as usize]
        }
    }

    #[inline]
    fn outside_bounds(&self, p: V3) -> bool {
        let (a, b) = (self.axis0 as usize, self.axis1 as usize);
        self.min[a] > p[a] || p[a] > self.max[a] || self.min[b] > p[b] || p[b] > self.max[b]
    }

    /// FCQuadTree::FindLeaf (FastCar) 0x0035A8D0 — leaf containing `p` (horizontal axes only).
    /// Returns `(first, count, node)`: the run `first..first+count` of the index array, and the leaf
    /// node, which may be passed back as `hint` (uncompacted trees only; the game always passes 0).
    /// `trace` records the visited cells (compacted trees only, as in the original).
    pub fn find_leaf(&self, p: V3, hint: u32, trace: Option<&mut FcQtTrace>) -> (u32, u32, u32) {
        let (a0, a1) = (self.axis0 as usize, self.axis1 as usize);
        if self.is_compact != 0 {
            let mut node: u32 = 0;
            if let Some(t) = trace {
                t.depth = 0;
                t.min[0] = [self.min[a0], self.min[a1]];
                t.max[0] = [self.max[a0], self.max[a1]];
                if self.outside_bounds(p) {
                    return (0, 0, 0);
                }
                let mut d = 0usize;
                let mut mx = (t.min[d][0] + t.max[d][0]) * 0.5;
                let mut mz = (t.min[d][1] + t.max[d][1]) * 0.5;
                while !compact_is_leaf(self.compact[node as usize]) {
                    t.depth += 1;
                    if t.depth >= 20 {
                        return (0, 0, 0);
                    }
                    d = t.depth as usize;
                    let mut q = 0;
                    if p[a0] <= mx {
                        t.min[d][0] = t.min[d - 1][0];
                        t.max[d][0] = mx;
                    } else {
                        q |= 1;
                        t.min[d][0] = mx;
                        t.max[d][0] = t.max[d - 1][0];
                    }
                    if p[a1] <= mz {
                        t.min[d][1] = t.min[d - 1][1];
                        t.max[d][1] = mz;
                    } else {
                        q |= 2;
                        t.min[d][1] = mz;
                        t.max[d][1] = t.max[d - 1][1];
                    }
                    mx = (t.min[d][0] + t.max[d][0]) * 0.5;
                    mz = (t.min[d][1] + t.max[d][1]) * 0.5;
                    node = q + compact_index(self.compact[node as usize]);
                }
            } else {
                if self.outside_bounds(p) {
                    return (0, 0, 0);
                }
                let (mut lo0, mut lo1) = (self.min[a0], self.min[a1]);
                let (mut hi0, mut hi1) = (self.max[a0], self.max[a1]);
                let mut mx = (lo0 + hi0) * 0.5;
                let mut mz = (lo1 + hi1) * 0.5;
                while !compact_is_leaf(self.compact[node as usize]) {
                    let mut q = 0;
                    if p[a0] <= mx {
                        hi0 = mx;
                    } else {
                        q |= 1;
                        lo0 = mx;
                    }
                    if p[a1] <= mz {
                        hi1 = mz;
                    } else {
                        q |= 2;
                        lo1 = mz;
                    }
                    mx = (lo0 + hi0) * 0.5;
                    mz = (lo1 + hi1) * 0.5;
                    node = q + compact_index(self.compact[node as usize]);
                }
            }
            let w = self.compact[node as usize];
            return (compact_index(w), compact_count(w), node);
        }

        let mut n = hint;
        let nd = &self.nodes[n as usize];
        if nd.leaf != 0 && p[a0] >= nd.min[0] && p[a1] >= nd.min[1] && nd.max[0] >= p[a0] && nd.max[1] >= p[a1] {
            return (nd.child[0], nd.child[1], n);
        }
        while n != u32::MAX {
            let nd = &self.nodes[n as usize];
            if !(nd.min[0] > p[a0] || nd.min[1] > p[a1] || p[a0] > nd.max[0] || p[a1] > nd.max[1]) {
                break;
            }
            n = nd.parent as u32;
        }
        if n == u32::MAX {
            return (0, 0, 0);
        }
        while self.nodes[n as usize].leaf == 0 {
            let nd = &self.nodes[n as usize];
            let mx = (nd.min[0] + nd.max[0]) * 0.5;
            let mz = (nd.min[1] + nd.max[1]) * 0.5;
            let mut q = 0;
            if p[a0] > mx {
                q |= 1;
            }
            if p[a1] > mz {
                q |= 2;
            }
            n = nd.child[q];
        }
        let nd = &self.nodes[n as usize];
        (nd.child[0], nd.child[1], n)
    }

    /// mesh faces stored in the leaf under `p` (the loop `WheelCollision` runs over `FindLeaf`'s run)
    pub fn faces_near(&self, p: V3) -> impl Iterator<Item = u32> + '_ {
        let (first, count, _) = self.find_leaf(p, 0, None);
        (first..first + count).map(move |i| self.face_index(i))
    }
}

/// (FastCar) 0x00380EA0 — 2D separating-axis test of triangle `(a, b, c)` against the box at `centre`
/// with half sizes `(hx, hy)` (the two quadtree axes)
pub fn triangle_rect_overlap(a: [f32; 2], b: [f32; 2], c: [f32; 2], centre: [f32; 2], hx: f32, hy: f32) -> bool {
    use super::vsmath::fabs;
    let (p0x, p0y) = (a[0] - centre[0], a[1] - centre[1]);
    let (p1x, p1y) = (b[0] - centre[0], b[1] - centre[1]);
    let (p2x, p2y) = (c[0] - centre[0], c[1] - centre[1]);
    // edge normal (nx, ny) through p: separating when the box radius cannot reach the edge line;
    // x87 products, rounded where the original stores them
    let sep = |px: f32, py: f32, nx: f64, ny: f32| -> bool {
        let d = (-(px as f64) * nx - py as f64 * ny as f64) as f32;
        let r = (fabs(nx as f32) * hx as f64) as f32;
        d as f64 > fabs(ny) * hy as f64 + r as f64
    };
    let n01x = p1y - p0y;
    let n01y = -p1x + p0x;
    if (p2x - p1x) * n01x + (p2y - p1y) * n01y >= 0.0 {
        if sep(p0x, p0y, p2y as f64 - p0y as f64, -p2x + p0x)
            || sep(p2x, p2y, p1y as f64 - p2y as f64, -p1x + p2x)
            || sep(p1x, p1y, p0y as f64 - p1y as f64, -p0x + p1x)
        {
            return false;
        }
    } else if sep(p0x, p0y, n01x as f64, n01y)
        || sep(p1x, p1y, p2y as f64 - p1y as f64, -p2x + p1x)
        || sep(p2x, p2y, p0y as f64 - p2y as f64, -p0x + p2x)
    {
        return false;
    }
    (p0x <= hx || p1x <= hx || p2x <= hx)
        && (-hx <= p0x || -hx <= p1x || -hx <= p2x)
        && (p0y <= hy || p1y <= hy || p2y <= hy)
        && (-hy <= p0y || -hy <= p1y || -hy <= p2y)
}

impl FcQuadTree {
    /// FCQuadTree::Build (FastCar) 0x003593D0 — `mode` 0: axes (1, 2), 1: (0, 2), 2: (0, 1); the third
    /// axis is the up axis. Returns false for a mesh without faces (tree left unbuilt).
    pub fn build(&mut self, mesh: &super::mesh::FcMesh, min_cell_size: f32, bounds_padding: f32, max_faces_per_leaf: u32, mode: u32) -> bool {
        self.num_mesh_faces = mesh.num_faces;
        if mesh.num_faces == 0 {
            return false;
        }
        self.short_indices = (mesh.num_faces < 0xFFFF) as u32;
        self.max_faces_per_leaf = max_faces_per_leaf;
        self.min_cell_size = min_cell_size;
        self.bounds_padding = bounds_padding;
        self.up_axis = mode;
        match mode {
            0 => (self.axis0, self.axis1) = (1, 2),
            1 => (self.axis0, self.axis1) = (0, 2),
            2 => (self.axis0, self.axis1) = (0, 1),
            _ => {}
        }
        self.nodes.clear();
        self.compact.clear();
        self.is_compact = 0;
        self.indices16.clear();
        self.indices32.clear();
        self.num_indices = 0;
        self.min = mesh.verts[0];
        self.max = mesh.verts[0];
        for v in mesh.verts.iter().take(mesh.num_verts as usize).skip(1) {
            for k in 0..3 {
                if self.min[k] > v[k] {
                    self.min[k] = v[k];
                }
            }
            for k in 0..3 {
                if v[k] > self.max[k] {
                    self.max[k] = v[k];
                }
            }
        }
        for k in 0..3 {
            self.min[k] -= bounds_padding;
            self.max[k] += bounds_padding;
        }
        let e = [self.max[0] - self.min[0], self.max[1] - self.min[1], self.max[2] - self.min[2]];
        let mut s = if e[0] <= e[1] { e[1] } else { e[0] };
        if e[2] > s {
            s = e[2];
        }
        for k in 0..3 {
            self.max[k] = self.min[k] + s;
        }
        self.num_nodes = 0;
        let (a0, a1) = (self.axis0 as usize, self.axis1 as usize);
        self.nodes.push(FcQtNode { parent: -1, min: [self.min[a0], self.min[a1]], max: [self.max[a0], self.max[a1]], ..Default::default() });
        self.build_node(mesh, 0, None);
        self.built = 1;
        self.node_capacity = self.num_nodes;
        self.index_capacity = self.num_indices;
        true
    }

    fn push_index(&mut self, f: u32) {
        if self.short_indices != 0 {
            self.indices16.push(f as u16);
        } else {
            self.indices32.push(f);
        }
        self.num_indices += 1;
    }

    /// FCQuadTree::BuildNode (FastCar) 0x00359870 — `parent_faces` = the faces the parent kept (`None`
    /// at the root: every mesh face)
    fn build_node(&mut self, mesh: &super::mesh::FcMesh, n: usize, parent_faces: Option<&[u32]>) {
        self.num_nodes += 1;
        let (a0, a1, up) = (self.axis0 as usize, self.axis1 as usize, self.up_axis as usize);
        let nd = self.nodes[n];
        let mut h = [0.0f32; 3];
        h[a0] = nd.max[0] - nd.min[0];
        h[a1] = nd.max[1] - nd.min[1];
        h[up] = self.max[up] - self.min[up];
        let width = h[a0];
        for x in h.iter_mut() {
            *x = *x * 0.5 + self.bounds_padding;
        }
        let mut c = [0.0f32; 3];
        c[a0] = (nd.min[0] + nd.max[0]) * 0.5;
        c[a1] = (nd.min[1] + nd.max[1]) * 0.5;
        c[up] = (self.min[up] + self.max[up]) * 0.5;
        let overlaps = |f: u32| {
            let [p, q, r] = mesh.triangle(f);
            triangle_rect_overlap([p[a0], p[a1]], [q[a0], q[a1]], [r[a0], r[a1]], [c[a0], c[a1]], h[a0], h[a1])
        };
        let all: Vec<u32>;
        let cand: &[u32] = match parent_faces {
            Some(p) => p,
            None => {
                all = (0..mesh.num_faces).collect();
                &all
            }
        };
        let mut kept: Vec<u32> = Vec::new();
        if self.min_cell_size <= width {
            kept = cand.iter().copied().filter(|&f| overlaps(f)).collect();
            if kept.len() as u32 <= self.max_faces_per_leaf {
                let first = self.num_indices;
                for &f in &kept {
                    self.push_index(f);
                }
                let nd = &mut self.nodes[n];
                nd.leaf = 1;
                nd.child[0] = first;
                nd.child[1] = kept.len() as u32;
            } else {
                self.nodes[n].leaf = 0;
            }
        } else {
            let first = self.num_indices;
            let mut count = 0;
            for &f in cand {
                if overlaps(f) {
                    count += 1;
                    self.push_index(f);
                }
            }
            let nd = &mut self.nodes[n];
            nd.leaf = 1;
            nd.child[0] = first;
            nd.child[1] = count;
        }
        if self.nodes[n].leaf == 0 {
            for q in 0..4 {
                let nd = self.nodes[n];
                let (lo0, hi0) = if q & 1 != 0 { (c[a0], nd.max[0]) } else { (nd.min[0], c[a0]) };
                let (lo1, hi1) = if q & 2 != 0 { (c[a1], nd.max[1]) } else { (nd.min[1], c[a1]) };
                let k = self.num_nodes as usize;
                if self.nodes.len() <= k {
                    self.nodes.resize(k + 1, FcQtNode::default());
                }
                self.nodes[k] = FcQtNode { parent: n as i32, min: [lo0, lo1], max: [hi0, hi1], ..Default::default() };
                self.nodes[n].child[q] = k as u32;
                self.build_node(mesh, k, Some(&kept));
            }
        }
    }

    /// FCQuadTree::Compact (FastCar) 0x0035A3A0 — packs the nodes into words; false (tree left as it
    /// is) when it has more than 2^24 nodes or a leaf with 255 faces or more
    pub fn compact(&mut self) -> bool {
        let n = self.num_nodes as usize;
        if n > 0xFFFFFF {
            return false;
        }
        if self.nodes[..n].iter().any(|nd| nd.leaf != 0 && nd.child[1] >= 255) {
            return false;
        }
        // compact slot of each node; an inner node's children get four consecutive slots, handed out in
        // node order (children always come after their parent, so one pass assigns every slot)
        let mut slot = vec![0u32; n];
        let mut out = vec![0u32; n];
        let mut next = 1u32;
        for k in 0..n {
            let nd = self.nodes[k];
            let s = slot[k] as usize;
            if nd.leaf != 0 {
                out[s] = (nd.child[1] << 24) | (nd.child[0] & 0xFFFFFF);
            } else {
                out[s] = 0xFF00_0000 | (next & 0xFFFFFF);
                for q in 0..4 {
                    slot[nd.child[q] as usize] = next + q as u32;
                }
                next += 4;
            }
        }
        self.compact = out;
        self.nodes.clear();
        self.is_compact = 1;
        true
    }

    /// the file image `load` reads (96-byte object, then nodes, then indices)
    pub fn to_image(&self) -> Vec<u8> {
        let mut b = vec![0u8; 96];
        let put = |b: &mut Vec<u8>, o: usize, x: u32| b[o..o + 4].copy_from_slice(&x.to_le_bytes());
        put(&mut b, 0x00, self.num_indices);
        put(&mut b, 0x04, self.num_nodes);
        put(&mut b, 0x08, self.short_indices);
        for k in 0..3 {
            put(&mut b, 0x14 + 4 * k, self.min[k].to_bits());
            put(&mut b, 0x20 + 4 * k, self.max[k].to_bits());
        }
        put(&mut b, 0x2C, self.node_capacity);
        put(&mut b, 0x30, self.index_capacity);
        put(&mut b, 0x34, self.min_cell_size.to_bits());
        put(&mut b, 0x38, self.bounds_padding.to_bits());
        put(&mut b, 0x3C, self.axis0);
        put(&mut b, 0x40, self.axis1);
        put(&mut b, 0x44, self.up_axis);
        put(&mut b, 0x48, self.built);
        put(&mut b, 0x4C, self.max_faces_per_leaf);
        put(&mut b, 0x50, self.num_mesh_faces);
        put(&mut b, 0x5C, self.is_compact);
        if self.is_compact != 0 {
            for w in &self.compact {
                b.extend_from_slice(&w.to_le_bytes());
            }
        } else {
            for nd in &self.nodes[..self.num_nodes as usize] {
                let words = [nd.parent as u32, nd.min[0].to_bits(), nd.min[1].to_bits(), nd.max[0].to_bits(), nd.max[1].to_bits(), nd.leaf];
                for x in words.into_iter().chain(nd.child) {
                    b.extend_from_slice(&x.to_le_bytes());
                }
            }
        }
        if self.short_indices != 0 {
            for i in &self.indices16 {
                b.extend_from_slice(&i.to_le_bytes());
            }
        } else {
            for i in &self.indices32 {
                b.extend_from_slice(&i.to_le_bytes());
            }
        }
        b
    }
}
