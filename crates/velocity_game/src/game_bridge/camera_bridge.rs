//! src\GameBridge\CameraBridge.cpp — per player camera hand-over (0x00096C70 returns player `n`'s record):
//! what the update decided the view is, read by the renderer and the replay camera manager.

use bevy::math::Vec3;

/// 52 bytes: position, look-at point, up, vertical field of view
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraBridgeData {
    pub position: Vec3, // +0x00
    pub target: Vec3,   // +0x10
    pub up: Vec3,       // +0x20
    pub fov: f32,       // +0x30
}
