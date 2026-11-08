//! Names the translated code (`*/gen.rs`, tools/phys_c2rs4.py) refers to: the memory model, the
//! numeric helpers, the hand-written overrides and every translated function.
pub use crate::fastcar::vsmath;
pub use crate::vehicle::vehicle_entity::cx::{Cx, IntOps, Ptr, Sp};
pub use crate::vehicle::vehicle_entity::ov::*;

pub use crate::fastcar::tr::*;
pub use crate::physics_manager::tr::*;
pub use crate::physics_manager::vdispatch::vdispatch;
pub use crate::physics_objects::physics_object_manager::tr::*;
pub use crate::physics_objects::physics_object_particle_tracker::tr::*;
pub use crate::physics_objects::physics_objects::tr::*;
pub use crate::vehicle::physics_car_input::tr::*;
pub use crate::vehicle::vehicle_entity::tr::*;
pub use crate::vehicle::vehicle_entity::tr_ext::*;
pub use crate::vehicle::vs_config_file_read::tr::*;
