//! src/Audio/VsAudioEvents.cpp — the player engine's driver (rpm, load, gear change and dump valve one-shots)
//! and the legacy physics-object sound ids.

use visos::handy::handy_crc::crc32;

use crate::audio::system::{AudioSystem, EmitterId, EventSlot};
use crate::input::CarAudioInput;
use crate::vs_audio_rev_bands::VsAudioRevBands;

/// `HANDY_CheckCRC("NAME")`: the crc the game compiles in for an event name.
pub fn crc(name: &str) -> u32 {
    crc32(name.as_bytes(), 0)
}

pub const GEAR_ENGAGE: u32 = 0xFAD4_2C70; // -86758288
pub const GEAR_DISENGAGE: u32 = 0x960F_D388; // -1777347704
pub const TURBO_DUMPVALVE: u32 = 0xAFAA_32A5; // -1347800411

/// event per old physics-object sound id 2000..2032 (`off_682FD0 + id`, i.e. 0x00684F10)
pub const OLD_IDS: [u32; 33] = [
    0x2EE3D3BB, 0x4FF8F984, 0xBFD41F71, 0xF87BF76B, 0x64F00259, 0x2B3101FD, 0x73DB9BEE, 0xA36117AE, 0x73DB9BEE,
    0x912982C7, 0xA36117AE, 0x73DB9BEE, 0x73DB9BEE, 0xA36117AE, 0x912982C7, 0x2B3101FD, 0x4454F278, 0x4454F278,
    0x4454F278, 0xABB5F55D, 0x4454F278, 0x4454F278, 0x4454F278, 0xA6AF9A6A, 0x0B00A76A, 0x2EE3D3BB, 0xA36117AE,
    0x2B3101FD, 0xEE9324F5, 0x2B3101FD, 0xABB5F55D, 0x4454F278, 0xB633BA65,
];

/// VsAudioEvents.cpp 0x0004C160: an old sound id to its event (IMPACT_CAR_MED outside 2000..2032).
pub fn map_old_id(id: i32) -> u32 {
    debug_assert!((2000..=2032).contains(&id), "oldId >= 2000 && oldId <= 2032");
    if !(2000..=2032).contains(&id) { 0x0C61_3DDC } else { OLD_IDS[(id - 2000) as usize] }
}

/// the VsAudioEvents.cpp globals (`dword_6D4D84..6D4DB4`).
pub struct VsAudioEvents {
    pub rev_bands: VsAudioRevBands, // dword_6D4DB4
    pub engine_running: bool,       // dword_6D4D28
    pub rpm: i32,                   // dword_6D4D84 (param 17)
    pub param26: i32,               // dword_6D4D88
    pub param21: i32,               // dword_6D4D8C
    pub ai_max_rpm: i32,            // dword_6D4D98
    pub last_load: f32,             // dword_6D4D9C
    pub last_gear: i32,             // dword_6D4DA0
    pub dump_valve: Option<EventSlot>, // dword_6D4DB0
}

impl Default for VsAudioEvents {
    fn default() -> Self {
        Self {
            rev_bands: VsAudioRevBands::default(),
            engine_running: false,
            rpm: 0,
            param26: 0,
            param21: 0,
            ai_max_rpm: 0,
            last_load: 0.0,
            last_gear: 0,
            dump_valve: None,
        }
    }
}

impl VsAudioEvents {
    /// VsAudioEvents.cpp 0x0004C340
    pub fn start_player_engine(&mut self, emitter: EmitterId) -> bool {
        self.rev_bands.emitter = Some(emitter);
        self.ai_max_rpm = 6000;
        self.rev_bands.set_rpm(1000);
        self.rev_bands.start();
        self.engine_running = true;
        true
    }

    /// VsAudioEvents.cpp 0x0004C3B0. returns true when the engine was running (VsAudio sets its restart flag).
    pub fn stop_player_engine(&mut self, audio: &mut AudioSystem) -> bool {
        if self.rev_bands.is_active() {
            self.rev_bands.stop(audio);
            self.engine_running = false;
            return true;
        }
        false
    }

    /// VsAudioEvents.cpp 0x0004C650
    pub fn set_engine_param(&mut self, id: u8, value: i32) -> bool {
        match id {
            17 => self.rpm = value,
            21 => self.param21 = value,
            26 => self.param26 = value,
            _ => {}
        }
        true
    }

    /// VsAudioEvents.cpp 0x0004C6A0
    pub fn engine_rpm(&self) -> i32 {
        self.rpm
    }

    /// VsAudioEvents.cpp 0x0004C400. `turbo_emitter` is the player car's second emitter.
    pub fn update_player_engine(&mut self, audio: &mut AudioSystem, car: &CarAudioInput, turbo_emitter: Option<EmitterId>) {
        if !self.rev_bands.is_active() {
            return;
        }
        let gear = car.gear;
        let mut load = car.throttle;
        if gear > 100 {
            load = 0.0;
        }
        self.rev_bands.set_load(load);
        self.rev_bands.update(audio, self.last_gear);
        self.rev_bands.set_rpm(self.engine_rpm() as f32 as i32);
        let emitter = self.rev_bands.emitter;
        if self.last_gear != gear {
            if self.last_gear >= gear {
                if self.last_gear > gear {
                    if let Some(e) = emitter {
                        audio.play_event(e, GEAR_ENGAGE, None, 256, None);
                    }
                }
            } else if let Some(e) = emitter {
                audio.play_event(e, GEAR_DISENGAGE, None, 256, None);
            }
            self.last_gear = gear;
        }
        if self.last_load > load {
            if self.dump_valve.is_none() {
                self.dump_valve = Some(audio.new_slot());
            }
            let slot = self.dump_valve.unwrap();
            if let Some(e) = turbo_emitter {
                let r = audio.play_event(e, TURBO_DUMPVALVE, None, 0, Some(slot));
                if r == 0 && audio.slot(slot).is_some() && audio.slot_sample(slot).is_some() {
                    let rbd_min = self.rev_bands.rbd.as_ref().map(|r| r.rpm_min).unwrap_or(0);
                    let d = (self.rev_bands.rpm - rbd_min) as u32;
                    audio.set_control_param(15, d as f32 / self.rev_bands.rpm_range);
                }
            }
        }
        self.last_load = load;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_crcs_match_names() {
        assert_eq!(crc("GEAR_ENGAGE"), GEAR_ENGAGE);
        assert_eq!(crc("GEAR_DISENGAGE"), GEAR_DISENGAGE);
        assert_eq!(crc("TURBO_DUMPVALVE"), TURBO_DUMPVALVE);
        assert_eq!(crc("IMPACT_CAR_MED"), 0x0C61_3DDC);
        assert_eq!(map_old_id(2032), crc("IMP_GLASS_VLRG"));
    }
}
