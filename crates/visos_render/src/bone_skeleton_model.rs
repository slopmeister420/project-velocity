//! BoneAnimation\BoneSkeletonModel.cpp — the skeleton of a skinned XMF2 model (`skin_count == 1`): header +0x30
//! points at `{u32 checksum, u32 numBones, Bone* bones}`, bones are 240 bytes in depth-first order.

use bevy::prelude::*;

use crate::renderer::renderer_types::{cstr, rd16, rd32, rdf, ModelError};

/// one 240-byte bone. Matrices are d3d row-vector matrices as glam column matrices (`from_cols_array`).
#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,   // +0x00
    pub parent: i32,    // +0x04 (-1 for the root)
    pub depth: u16,     // +0x08 hierarchy depth (matrix stack slot)
    pub rotation: Quat, // +0x10 bind local rotation
    pub scale: Vec4,    // +0x20 bind local scale
    pub local: Mat4,    // +0x30 bind local matrix (translation row = +0x60)
    pub world: Mat4,    // +0x70 bind world matrix
    pub inv_world: Mat4, // +0xB0 inverse bind world matrix
}

impl Bone {
    /// BoneSkeletonModel.cpp 0x002E1E10 — the bind translation (last row of the local matrix)
    pub fn translation(&self) -> Vec4 {
        self.local.w_axis
    }
}

#[derive(Clone, Debug)]
pub struct BoneSkeletonModel {
    pub checksum: u32,
    pub bones: Vec<Bone>,
}

fn mat(b: &[u8], o: usize) -> Result<Mat4, ModelError> {
    let mut m = [0f32; 16];
    for (k, v) in m.iter_mut().enumerate() {
        *v = rdf(b, o + 4 * k)?;
    }
    Ok(Mat4::from_cols_array(&m))
}

impl BoneSkeletonModel {
    /// the skeleton of a model file (`None` when it is not skinned)
    pub fn parse(b: &[u8]) -> Result<Option<Self>, ModelError> {
        if rd32(b, 44)? != 1 {
            return Ok(None);
        }
        let info = rd32(b, 48)? as usize;
        if info == 0 {
            return Ok(None);
        }
        let (checksum, n, at) = (rd32(b, info)?, rd32(b, info + 4)? as usize, rd32(b, info + 8)? as usize);
        let mut bones = Vec::with_capacity(n);
        for i in 0..n {
            let o = at + 240 * i;
            let name_off = rd32(b, o)? as usize;
            bones.push(Bone {
                name: if name_off != 0 { cstr(b, name_off)? } else { String::new() },
                parent: rd32(b, o + 4)? as i32,
                depth: rd16(b, o + 8)?,
                rotation: Quat::from_xyzw(rdf(b, o + 16)?, rdf(b, o + 20)?, rdf(b, o + 24)?, rdf(b, o + 28)?),
                scale: Vec4::new(rdf(b, o + 32)?, rdf(b, o + 36)?, rdf(b, o + 40)?, rdf(b, o + 44)?),
                local: mat(b, o + 48)?,
                world: mat(b, o + 112)?,
                inv_world: mat(b, o + 176)?,
            });
        }
        Ok(Some(Self { checksum, bones }))
    }

    /// BoneSkeletonModel.cpp 0x002E1EC0 — bone index by name (case insensitive, 0 when absent)
    pub fn find_bone(&self, name: &str) -> usize {
        self.bones.iter().position(|b| b.name.eq_ignore_ascii_case(name)).unwrap_or(0)
    }
}
