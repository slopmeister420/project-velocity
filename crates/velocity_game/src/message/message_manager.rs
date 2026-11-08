//! src\Message\MessageManager.cpp — `g_MessageManager` (0x006FAA34): five broadcast channels, each a list of
//! handlers called in turn (0x000CA0B0 add, 0x000CA1B0 remove, 0x000CA2C0 send). Here a channel is one Bevy
//! message type: senders write `GameMessage`, handlers are systems reading it. The rest of the file is the
//! network actor replication (`CActorNetDataHandler`), not ported with the network game.

use bevy::prelude::*;

use crate::camera::vs_crash_camera_manager::CollisionInfo;

/// `eMSG_*` channel ids
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EMessage {
    /// physics contact: car / car or car / scenery (PhysicsManager 0x001A0570, 0x001A3A30, ...)
    Collision = 0,
    /// a tweak changed (GameTweaksMenu 0x000B07E0)
    TweakChanged = 1,
    /// a car was put back / reset (PhysicsManager 0x0019FD80): race timeline restore points listen
    VehicleReset = 2,
    /// the camera cut (VsAIVehicle 0x0002C620, replay cameras)
    CameraCut = 3,
    /// a physics object was reset (PhysicsObjectParticleTracker 0x001E4710)
    PhysicsObjectReset = 4,
}

#[derive(Message, Clone, Debug, PartialEq)]
pub enum GameMessage {
    Collision(CollisionInfo),
    /// the tweak's variable (as a debug name)
    TweakChanged(String),
    VehicleReset { vehicle: u32 },
    CameraCut,
    PhysicsObjectReset { object: u32 },
}

impl GameMessage {
    pub fn id(&self) -> EMessage {
        match self {
            GameMessage::Collision(_) => EMessage::Collision,
            GameMessage::TweakChanged(_) => EMessage::TweakChanged,
            GameMessage::VehicleReset { .. } => EMessage::VehicleReset,
            GameMessage::CameraCut => EMessage::CameraCut,
            GameMessage::PhysicsObjectReset { .. } => EMessage::PhysicsObjectReset,
        }
    }
}
