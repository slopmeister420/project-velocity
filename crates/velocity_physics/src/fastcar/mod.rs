//! FastCar — Eugene Laptev's rigid-body car library, linked without asserts or `__FILE__`
//! strings into the unattributed range 0x355B60-0x3C63D0 (inside the `_AUDIO_Resample` region).
pub mod expression;
pub mod mem;
pub mod rigid_body;
pub mod vsmath;
pub mod mesh;
pub mod quadtree;
pub mod solver;
pub mod car;
pub mod car_contacts;
pub mod car_evolve;
pub mod car_tyre;
pub mod car_wheel;
#[cfg(feature = "tr")]
pub mod tr;
