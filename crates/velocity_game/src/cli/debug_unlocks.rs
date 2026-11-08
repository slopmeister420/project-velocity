//! the debug build's unlock tweaks. F3_D registers three bool tweaks in the debug menu and applies them through
//! the tweak-changed callback `FECK2Manager.cpp 0x00202EA0`; only "Unlock ALL Vehicles" is on by default
//! (`byte_6877F0` = 1 in .data, everything else is bss). The port's `-u` / `--unlock` boots with all three on,
//! which is the debug build with its unlock options switched on; without it they are off (retail behaviour).

use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DebugUnlocks {
    /// "Unlock ALL Vehicles" (`byte_6877F0`, FEAlloyEditor_Basic.cpp 0x00258770): every vehicle reads as
    /// unlocked (`CVehicleDatabase::GetEntryLockState` 0x0018D2A0)
    pub vehicles: bool,
    /// "Scan Track Folder" (`byte_6F5B75`, registered at 0x000B38DC): every race unlocked
    /// (`EventDatabaseTrackLoading.cpp 0x00094990`), incl. the races shipped `Available: NO`
    pub races: bool,
    /// "Unlock ALL Events" (`byte_6F0D70`, FEOnlineMatchInfoScreen.cpp 0x00218390): every career event
    /// unlocked (`EventDatabaseEvents.cpp 0x00092CD0` with 1)
    pub events: bool,
}

impl DebugUnlocks {
    pub fn all() -> Self {
        Self { vehicles: true, races: true, events: true }
    }
}
