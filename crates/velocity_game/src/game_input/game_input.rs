//! src\GameInput\GameInput.cpp — which input devices belong to which player (`+0x608` device masks, 6 players).
//! The rest of the file is the network vehicle-input relay, not ported.

use bevy::prelude::*;

pub const MAX_PLAYERS: usize = 6;

#[derive(Resource, Debug, Clone, Default)]
pub struct GameInput {
    /// +0x608 per player device mask
    pub device_masks: [u32; MAX_PLAYERS],
}

impl GameInput {
    /// GameInput.cpp 0x000A59A0 — set by the front end when a pad presses start.
    pub fn set_device_mask(&mut self, player: usize, mask: u32) {
        if player < MAX_PLAYERS {
            self.device_masks[player] = mask;
        }
    }

    /// GameInput.cpp 0x000A59D0
    pub fn device_mask(&self, player: usize) -> u32 {
        if player >= MAX_PLAYERS { 0 } else { self.device_masks[player] }
    }
}
