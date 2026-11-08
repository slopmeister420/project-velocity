//! src\Replays\Cameras\
pub mod replay_camera_fixed;
pub mod replay_camera_manager;
pub mod replay_camera_orbit;
pub mod replay_camera_wheel;

use bevy::math::Vec3;

/// what the replay cameras ask of the race.
pub trait ReplayWorld {
    /// PhysicsCarInput.cpp 0x001BA270 — car position
    fn car_position(&self, vehicle: u32) -> Vec3;
    /// wheel position (FastCar wheel 0..3, 0x00361400)
    fn wheel_position(&self, vehicle: u32, wheel: u32) -> Vec3;
    /// car matrix rows 0, 1, 2 (VehicleEntity 0x001C66C0 / 0x001C6630 / 0x001C65A0): forward, up, side
    fn car_axes(&self, vehicle: u32) -> (Vec3, Vec3, Vec3);
    /// ReplayCamera_Orbit.cpp 0x00167680 — the line from the camera to the car is blocked
    fn blocked(&self, from: Vec3, to: Vec3, vehicle: u32) -> bool;
}
