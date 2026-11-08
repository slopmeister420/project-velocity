//! src\GameMode\UpdateThread.cpp — two classes:
//! - `CUpdateThread` (GameManager+0x1F8): a worker that keeps presenting frames every 0.1 s while a blocking load runs.
//!   Loading is not blocking in this port; the worker's loop is kept as a tick the caller polls.
//! - `CWorldModeManager` ("WMM", global 0x006FA8D0): one exclusive gameplay mode at a time (crash cam, slow motion...),
//!   with a cooldown after leaving and a time stretch applied to the frame timer every update.

/// `CUpdateThread` (0x1C bytes)
#[derive(Clone, Debug)]
pub struct CUpdateThread {
    pub running: bool,     // +0x0C
    /// +0x0D the worker is asked to stop
    pub stop: bool,
    /// +0x10 time of the last frame presented
    pub last: f32,
    /// +0x14 a frame was presented since the last stop (the debug graph is closed on stop)
    pub presented: bool,
    pub interval: f32,     // +0x18 (0.1)
}

impl Default for CUpdateThread {
    /// UpdateThread.cpp 0x000C6400
    fn default() -> Self {
        CUpdateThread { running: false, stop: false, last: 0.0, presented: false, interval: f32::from_bits(0x3DCC_CCCD) }
    }
}

impl CUpdateThread {
    /// UpdateThread.cpp 0x000C64D0 — start (at `now`) / stop the keep-alive worker
    pub fn enable(&mut self, on: bool, now: f32) {
        if on == self.running {
            log::error!("assert: update thread already in that state");
            return;
        }
        if on {
            self.last = now;
        } else {
            // UpdateThread.cpp 0x000C65C0
            self.stop = true;
            self.presented = false;
        }
        self.running = on;
    }

    /// UpdateThread.cpp 0x000C65E0 — one pass of the worker loop (it sleeps 100 ms between passes): true when a
    /// frame should be presented (VehicleDatabaseWheelBinLoading 0x0018F2A0)
    pub fn tick(&mut self, now: f32) -> bool {
        if self.stop {
            self.stop = false;
            self.running = false;
            return false;
        }
        if now - self.last >= self.interval {
            self.presented = true;
            self.last = now;
            true
        } else {
            false
        }
    }
}

/// a WMM mode id (2 and 3 seen; 3 is reference counted)
pub type WorldMode = i32;

/// `CWorldModeManager` (0x18 bytes, `dword_6FA8D0`)
#[derive(Clone, Debug, bevy::prelude::Resource)]
pub struct CWorldModeManager {
    pub mode: WorldMode,     // +0x00 (0 = none)
    pub sleep_until: f32,    // +0x04 game time before which new modes are refused
    pub handler: Option<String>, // +0x08 mode object (name only here)
    pub disabled: bool,      // +0x0C
    pub time_stretch: f32,   // +0x10
    pub ref_count: i32,      // +0x14 (mode 3)
}

impl Default for CWorldModeManager {
    /// UpdateThread.cpp 0x000C6790
    fn default() -> Self {
        CWorldModeManager { mode: 0, sleep_until: 0.0, handler: None, disabled: false, time_stretch: 1.0, ref_count: 0 }
    }
}

impl CWorldModeManager {
    /// UpdateThread.cpp 0x000C6810
    pub fn reset(&mut self) {
        if self.mode != 0 {
            log::info!("WMM: Resetting: Leaving {:?}", self.handler);
            self.mode = 0;
        }
        self.handler = None;
        self.sleep_until = 0.0;
        self.time_stretch = 1.0;
        self.ref_count = 0;
    }

    /// UpdateThread.cpp 0x000C68B0 — returns whether the mode was entered.
    pub fn request(&mut self, mode: WorldMode, handler: &str, game_time: f32) -> bool {
        if self.disabled {
            return false;
        }
        if mode == 3 {
            self.ref_count += 1;
            if self.ref_count > 1 {
                return true;
            }
        }
        if mode != 3 {
            if self.sleep_until > game_time {
                log::info!("WMM: Can't set mode during sleep time, request denied");
                return false;
            }
            if self.mode != 0 {
                log::info!("WMM: Can't set mode to {} from {:?}, request denied", handler, self.handler);
                return false;
            }
        }
        if self.mode != 0 {
            log::info!("WMM: Cancelling {:?}", self.handler);
        }
        self.mode = mode;
        self.handler = Some(handler.to_string());
        log::info!("WMM: Entering {}", handler);
        true
    }

    /// UpdateThread.cpp 0x000C6A90 — leave `mode`, refusing new ones for `sleep` seconds.
    pub fn leave(&mut self, mode: WorldMode, sleep: f32, game_time: f32) {
        if self.mode != mode {
            return;
        }
        if mode == 3 {
            self.ref_count -= 1;
            if self.ref_count > 0 {
                return;
            }
        }
        log::info!("WMM: Leaving {:?}", self.handler);
        if game_time + sleep > self.sleep_until {
            self.sleep_until = game_time + sleep;
        }
        self.mode = 0;
        self.handler = None;
    }

    /// UpdateThread.cpp 0x000C6A40
    pub fn set_disabled(&mut self, d: bool, game_time: f32) {
        self.disabled = d;
        if d && self.mode != 0 {
            let m = self.mode;
            self.leave(m, 0.0, game_time);
        }
    }

    /// UpdateThread.cpp 0x000C6B70 — the stretch the frame timer runs the next update at.
    pub fn apply(&self, timer: &mut crate::core_::vs_frame_timer::VsFrameTimer) {
        timer.set_time_stretch(self.time_stretch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_alive_presents_every_tenth_of_a_second() {
        let mut t = CUpdateThread::default();
        t.enable(true, 0.0);
        assert!(!t.tick(0.05));
        assert!(t.tick(0.1));
        t.enable(false, 0.2);
        assert!(!t.tick(0.5));
        assert!(!t.running);
    }

    #[test]
    fn modes() {
        let mut w = CWorldModeManager::default();
        assert!(w.request(2, "crash", 0.0));
        assert!(!w.request(2, "other", 0.0));
        w.leave(2, 1.0, 5.0);
        assert!(!w.request(2, "again", 5.5));
        assert!(w.request(2, "again", 6.5));
    }
}
