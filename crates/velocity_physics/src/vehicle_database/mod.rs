//! src/VehicleDatabase — vehicle, setup, upgrade, wheel, traffic, colour and decal databases.

pub mod vehicle_database;
pub mod vehicle_database_colours;
pub mod vehicle_database_decal_loading;
pub mod vehicle_database_physics;
pub mod vehicle_database_setup_bin_loading;
pub mod vehicle_database_setups;
pub mod vehicle_database_traffic_bin_loading;
pub mod vehicle_database_upgrade_bin_loading;
pub mod vehicle_database_vehicle_bin_loading;
pub mod vehicle_database_wheel_bin_loading;

pub use vehicle_database::{CVehicleDatabase, PartRef, VehicleDatabaseError, VehicleSetup, string_hash};
pub use vehicle_database_physics::PerformanceModifiers;
pub use vehicle_database_traffic_bin_loading::TrafficSetup;
pub use vehicle_database_vehicle_bin_loading::VehicleEntry;
