//! visos `Audio\` — the ViSOS sound engine: banks, groups/events, emitters, listeners, voices.
pub mod adpcm;
pub mod emitter;
pub mod event;
pub mod load;
pub mod output;
pub mod mixer;
pub mod pool;
pub mod sample;
pub mod system;
pub mod xbox;
pub mod xva;
pub mod xvg;

pub use system::{AudioInitParams, AudioSystem, EmitterId, EventId, EventSlot, ListenerId, SampleId};

#[cfg(test)]
mod data_tests;
#[cfg(test)]
mod engine_tests;
