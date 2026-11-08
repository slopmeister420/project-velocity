//! src\Camera\ — the in-game cameras.
pub mod game_cameras;
pub mod vs_base_camera;
pub mod vs_camera_manager;
pub mod vs_crash_camera;
pub mod vs_crash_camera_manager;
pub mod vs_fly_camera;
pub mod vs_vehicle_follow_camera;
pub mod vs_vehicle_follow_camera_point_of_interest;
#[cfg(test)]
mod vs_vehicle_follow_camera_golden;
