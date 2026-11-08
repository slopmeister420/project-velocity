//! bundle loading as the game does it (`GameDataLoading.cpp` 0x0009C650 / 0x0009C7F0 / 0x0009C8C0,
//! `StringManager.cpp` 0x000C9C10 for heat haze): root "Chunk", then Typelib, Instances and Timeline children.
//! not an original file — one place for the loops every bundle consumer repeats.

use crate::chunk::file_chunk::{
    Chunk, ChunkError, CHUNK_ID_CHUNK, CHUNK_ID_CHUNK_NOCASE, CHUNK_ID_INSTANCES, CHUNK_ID_TIMELINE, CHUNK_ID_TYPELIB,
    CHUNK_ID_TYPELIB_NOCASE,
};
use crate::instances::file_instances::{read_instances, FileInstances, InstancesError};
use crate::timeline::file_tlf::{read_timeline, FileTimeline, TlfError};
use crate::type_::file_type::{load_type_library, FileTypeEntry};
use crate::type_::vs_type::{CEnvironment, TypeParseError};

#[derive(Debug, thiserror::Error)]
pub enum BundleError {
    #[error(transparent)]
    Chunk(#[from] ChunkError),
    #[error("Invalid bundle (does not contain a file header chunk)")]
    NoHeader,
    #[error(transparent)]
    Type(#[from] TypeParseError),
    #[error(transparent)]
    Instances(#[from] InstancesError),
    #[error(transparent)]
    Timeline(#[from] TlfError),
}

#[derive(Debug)]
pub struct Bundle {
    /// root id was the case-sensitive "Chunk" crc; the engine's header check rejects the others.
    pub engine_accepts: bool,
    pub types: Vec<FileTypeEntry>,
    pub instances: Option<FileInstances>,
    pub timelines: Vec<FileTimeline>,
    /// timeline chunks the engine skips ("incorrect version number for timeline file"): (name, version)
    pub rejected_timelines: Vec<(Option<String>, u32)>,
}

impl Bundle {
    /// parse a bundle, registering its types into `env`.
    pub fn load(file: &[u8], env: &mut CEnvironment) -> Result<Self, BundleError> {
        let root = Chunk::root(file)?;
        if root.id() != CHUNK_ID_CHUNK && root.id() != CHUNK_ID_CHUNK_NOCASE {
            return Err(BundleError::NoHeader);
        }
        let mut b = Bundle { engine_accepts: root.id() == CHUNK_ID_CHUNK, types: Vec::new(), instances: None, timelines: Vec::new(), rejected_timelines: Vec::new() };
        for c in root.child_list()? {
            match c.id() {
                CHUNK_ID_TYPELIB | CHUNK_ID_TYPELIB_NOCASE => b.types.extend(load_type_library(c.data_tail(), env)?),
                CHUNK_ID_INSTANCES => {
                    if b.instances.is_none() {
                        b.instances = Some(read_instances(c.data_tail(), env)?);
                    }
                }
                CHUNK_ID_TIMELINE => match read_timeline(c.data_tail(), env) {
                    Ok(t) => b.timelines.push(t),
                    Err(TlfError::Version(v)) => {
                        log::warn!("Error: incorrect version number for timeline file");
                        b.rejected_timelines.push((c.name().map(str::to_owned), v));
                    }
                },
                _ => {}
            }
        }
        Ok(b)
    }

    /// load with a fresh environment holding only the builtins.
    pub fn load_standalone(file: &[u8]) -> Result<(Self, CEnvironment), BundleError> {
        let mut env = CEnvironment::with_builtins();
        let b = Self::load(file, &mut env)?;
        Ok((b, env))
    }
}
