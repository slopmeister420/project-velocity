//! src/CarDamage: visual car damage (bone-weighted body dents, broken lights, cracked windows).
//! Nothing here feeds back into handling: the only callers in the original are CarRender, the AI vehicle
//! (allocation), the per-frame manager update and `GameManager::RestartRace` (repair).

pub mod car_damage;

pub use car_damage::*;
