//! Vehicle and world physics (src/Physics, src/CarDamage, src/VehicleDatabase).


pub mod c_bump_map;
pub mod car_damage;
pub mod c_config;
pub mod c_config_mem;
pub mod fastcar;
pub mod material_properties;
pub mod physics_manager;
pub mod physics_objects;
pub mod physics_tweaks;
#[cfg(feature = "tr")]
pub mod plugin;
#[cfg(feature = "tr")]
pub mod tr_prelude;
pub mod vehicle;
pub mod vehicle_database;

#[cfg(feature = "tr")]
pub use plugin::PhysicsPlugin;

#[cfg(not(feature = "tr"))]
pub struct PhysicsPlugin;

#[cfg(not(feature = "tr"))]
impl bevy::prelude::Plugin for PhysicsPlugin {
    fn build(&self, _app: &mut bevy::prelude::App) {}
}
