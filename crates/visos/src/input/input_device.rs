//! InputDevice.cpp — the xbox pad physical key set ("InputXboxpadDevice PhysicalKeys") and per-device raw state.

/// Xboxpad physical keys, in the device's key order.
pub const XBOXPAD_KEYS: [&str; 20] = [
    "START", "BACK", "UP", "DOWN", "LEFT", "RIGHT", "A", "B", "X", "Y", "BLACK", "WHITE", "LTRIGGER", "RTRIGGER",
    "STICK_LX", "STICK_LY", "STICK_RX", "STICK_RY", "STICK_LBUTTON", "STICK_RBUTTON",
];
pub const NUM_KEYS: usize = 20;

/// indices into `XBOXPAD_KEYS`
pub mod key {
    pub const START: usize = 0;
    pub const BACK: usize = 1;
    pub const UP: usize = 2;
    pub const DOWN: usize = 3;
    pub const LEFT: usize = 4;
    pub const RIGHT: usize = 5;
    pub const A: usize = 6;
    pub const B: usize = 7;
    pub const X: usize = 8;
    pub const Y: usize = 9;
    pub const BLACK: usize = 10;
    pub const WHITE: usize = 11;
    pub const LTRIGGER: usize = 12;
    pub const RTRIGGER: usize = 13;
    pub const STICK_LX: usize = 14;
    pub const STICK_LY: usize = 15;
    pub const STICK_RX: usize = 16;
    pub const STICK_RY: usize = 17;
    pub const STICK_LBUTTON: usize = 18;
    pub const STICK_RBUTTON: usize = 19;
}

/// one device's key values (buttons 0/1, triggers 0..1, sticks -1..1 with y up).
pub type DeviceKeys = [f32; NUM_KEYS];
