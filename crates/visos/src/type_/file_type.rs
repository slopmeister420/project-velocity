//! Type\file_type.cpp — Typelib chunks and type references inside bundle chunks.
//!
//! Typelib data: `{u32 count, u32 entries}` then `FileTypeEntry[count]` (20 bytes):
//! `{u32 crc, u32 name, u32 description, CType* type (runtime), u32 pad}`. Offsets are relative to the
//! Typelib data. `description` is VSTYPE text (`struct X {...}`), parsed and registered on load.
//! A `FileTypeRef` (12 bytes) `{u32 crc, u32 name, CType* (runtime)}` names a type for instance data.

use std::sync::Arc;

use crate::chunk::file_chunk::{cstr, rd32};
use crate::type_::vs_type::{CEnvironment, CType, CTypeKind, TypeParseError};

/// one Typelib entry.
#[derive(Clone, Debug)]
pub struct FileTypeEntry {
    pub crc: u32,
    pub name: String,
    pub description: Option<String>,
}

/// file_type.cpp 0x002ED580 (FixupTypeLibrary) — read the entries and register their types in `env`.
pub fn load_type_library(data: &[u8], env: &mut CEnvironment) -> Result<Vec<FileTypeEntry>, TypeParseError> {
    let count = rd32(data, 0) as usize;
    let off = rd32(data, 4) as usize;
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let e = off + 20 * i;
        let entry = FileTypeEntry {
            crc: rd32(data, e),
            name: cstr(data, rd32(data, e + 4) as usize).unwrap_or("").to_string(),
            description: cstr(data, rd32(data, e + 8) as usize).map(str::to_owned),
        };
        // file_type.cpp 0x002ED6D0
        if let Some(d) = &entry.description {
            let (t, _) = env.parse(d, false)?;
            env.add(t);
        }
        out.push(entry);
    }
    Ok(out)
}

/// a resolved `FileTypeRef`.
#[derive(Clone, Debug)]
pub struct FileTypeRef {
    pub crc: u32,
    pub name: String,
    pub ty: Option<Arc<CType>>,
}

/// file_type.cpp 0x002ED690 — read a 12-byte type ref at `at` and resolve its name.
pub fn read_type_ref(base: &[u8], at: usize, env: &CEnvironment) -> FileTypeRef {
    let name = cstr(base, rd32(base, at + 4) as usize).unwrap_or("").to_string();
    FileTypeRef { crc: rd32(base, at), ty: env.find(&name), name }
}

/// file_type.cpp 0x002ED740 (FixupDataOfType) — offsets of the pointer-like members (string, array,
/// vararray kinds) that the loader rebases. the original rebases the first dword of array/vararray
/// members too; only strings occur in shipped data.
pub fn pointer_members(ty: &CType) -> impl Iterator<Item = u32> + '_ {
    ty.members
        .iter()
        .filter(|m| matches!(m.ty.kind, CTypeKind::String | CTypeKind::Array | CTypeKind::VarArray))
        .map(|m| m.offset)
}
