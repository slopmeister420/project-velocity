//! BoneAnimation\BoneAnimBlendTree.cpp — per-bone QTS from the skeleton's bind pose blended with the layer's
//! anim (UpdateLeaf 0x002DB610, blend 0x002DBB40), local matrices (0x002DBE60) and the depth-stack world /
//! skin matrices (0x002DBCE0). Matrices are glam column matrices of the d3d row-vector ones.

use bevy::prelude::*;

use crate::bone_anim::{quat_slerp, BoneAnim, BoneQts};
use crate::bone_skeleton_model::BoneSkeletonModel;

/// Renderer.cpp 0x002AC2C0 — rotation matrix of a quaternion (the engine rotates by the conjugate)
pub fn quat_matrix(q: Quat) -> Mat4 {
    Mat4::from_quat(Quat::from_xyzw(-q.x, -q.y, -q.z, q.w))
}

/// BoneAnimBlendTree.cpp 0x002DBB40 — blend `b` into `a` by `s`
pub fn blend(a: &BoneQts, b: &BoneQts, s: f32) -> BoneQts {
    let mut translation = a.translation * (1.0 - s) + b.translation * s;
    translation.w = 1.0;
    BoneQts { rotation: quat_slerp(s, a.rotation, b.rotation), scale: a.scale * (1.0 - s) + b.scale * s, translation }
}

/// one anim layer: anim, weight, normalised time
pub struct Layer<'a> {
    pub anim: &'a BoneAnim,
    pub weight: f32,
    pub time: f32,
}

/// BoneAnimBlendTree.cpp 0x002DB610 UpdateLeaf for a list of leaf layers — bind QTS (rotation +0x10, scale +0x20, translation +0x60),
/// then each layer's active bones blended in by its weight (bones beyond the anim's count keep the bind pose).
pub fn update_leaf(skel: &BoneSkeletonModel, layers: &[Layer]) -> Vec<BoneQts> {
    let mut q: Vec<BoneQts> = skel
        .bones
        .iter()
        .map(|b| BoneQts { rotation: b.rotation, scale: b.scale, translation: b.translation() })
        .collect();
    for l in layers {
        if l.anim.checksum != skel.checksum {
            log::warn!("bone anim {}: checksum {:08X} != skeleton {:08X}", l.anim.name, l.anim.checksum, skel.checksum);
        }
        for (i, tr) in l.anim.bones.iter().enumerate() {
            if i >= q.len() || !tr.active || tr.keys.is_empty() {
                continue;
            }
            let s = l.anim.sample(i, l.time);
            q[i] = blend(&q[i], &s, l.weight);
        }
    }
    q
}

/// BoneAnimBlendTree.cpp 0x002DBE60 CreateLocalMatrices — rotation, rows scaled (Renderer 0x002AC200), translation row
pub fn local_matrix(q: &BoneQts) -> Mat4 {
    let mut m = quat_matrix(q.rotation);
    m.x_axis *= q.scale.x;
    m.y_axis *= q.scale.y;
    m.z_axis *= q.scale.z;
    m.x_axis.w = 0.0;
    m.y_axis.w = 0.0;
    m.z_axis.w = 0.0;
    m.w_axis = q.translation;
    m
}

/// BoneAnimBlendTree.cpp 0x002DBCE0 — model-space world matrices through the depth stack (`world = local * stack[depth - 1]`, 32 deep)
pub fn world_matrices(skel: &BoneSkeletonModel, qts: &[BoneQts]) -> Vec<Mat4> {
    let mut stack = [Mat4::IDENTITY; 32];
    let mut out = Vec::with_capacity(qts.len());
    for (b, q) in skel.bones.iter().zip(qts) {
        let d = (b.depth as usize).min(31);
        let l = local_matrix(q);
        stack[d] = if d != 0 { stack[d - 1] * l } else { l };
        out.push(stack[d]);
    }
    out
}

/// BoneAnimBlendTree.cpp 0x002DBCE0 tail — skin matrices `invBind * world` (what BoneBlend4 multiplies the bind-pose vertices by)
pub fn skin_matrices(skel: &BoneSkeletonModel, world: &[Mat4]) -> Vec<Mat4> {
    skel.bones.iter().zip(world).map(|(b, w)| *w * b.inv_world).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn models() -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for p in [r"\Chars\CHARS_M.VCD", r"\FlagGirl\FG_M.vcd"] {
            let v = visos::file::Vcd::open(p).unwrap();
            for e in v.entries() {
                if e.name.to_ascii_lowercase().ends_with(".xmf") {
                    out.push((e.name.to_string(), v.data(&e).unwrap().to_vec()));
                }
            }
        }
        out
    }

    /// the bind QTS rebuilds every bone's stored bind local and world matrices
    #[test]
    fn bind_pose_matches_skeleton() {
        let ms = models();
        assert!(ms.len() > 40);
        for (name, d) in &ms {
            let s = BoneSkeletonModel::parse(d).unwrap().unwrap_or_else(|| panic!("{name}: not skinned"));
            let q = update_leaf(&s, &[]);
            let w = world_matrices(&s, &q);
            for (i, b) in s.bones.iter().enumerate() {
                assert!(local_matrix(&q[i]).abs_diff_eq(b.local, 2e-3), "{name} bone {i} local");
                assert!(w[i].abs_diff_eq(b.world, 5e-3), "{name} bone {i} world\n{}\n{}", w[i], b.world);
            }
            // skin matrices are identity in the bind pose
            for m in skin_matrices(&s, &w) {
                assert!(m.abs_diff_eq(Mat4::IDENTITY, 5e-3), "{name} skin");
            }
        }
    }

    /// every .gaf parses and samples; anims match a shipped skeleton by checksum and bone count
    #[test]
    fn every_anim() {
        let ms = models();
        let skels: Vec<BoneSkeletonModel> = ms.iter().filter_map(|(_, d)| BoneSkeletonModel::parse(d).unwrap()).collect();
        let mut n = 0;
        for p in [r"\Chars\CHARS_A.VCD", r"\FlagGirl\FG_A.vcd"] {
            let v = visos::file::Vcd::open(p).unwrap();
            for e in v.entries() {
                if !e.name.to_ascii_lowercase().ends_with(".gaf") {
                    continue;
                }
                for a in BoneAnim::parse_file(v.data(&e).unwrap()).unwrap_or_else(|x| panic!("{}: {x}", e.name)) {
                    let s = skels.iter().find(|s| s.checksum == a.checksum);
                    if let Some(s) = s {
                        assert_eq!(s.bones.len(), a.bones.len(), "{}", e.name);
                    }
                    for t in [0.0, 0.37, 0.999, 1.0] {
                        for i in 0..a.bones.len() {
                            if a.bones[i].active && !a.bones[i].keys.is_empty() {
                                let q = a.sample(i, t);
                                assert!(q.rotation.is_finite() && (q.rotation.length() - 1.0).abs() < 1e-3, "{} {i}", e.name);
                            }
                        }
                    }
                    n += 1;
                }
            }
        }
        assert!(n > 100, "{n}");
    }
}
