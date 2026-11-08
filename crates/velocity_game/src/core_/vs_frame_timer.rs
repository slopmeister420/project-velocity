//! src\Core\VsFrameTimer.cpp — fixed-step game updates (24 per second) with render interpolation. The main loop
//! (VsCommonMain.cpp 0x0016DD20) calls `tick` repeatedly: `true` = run one game update now, `false` = render once.
//! The vblank-locked path (0x00057A40, `vblank_mode > 0`) is not used on PC; time comes from a monotonic clock
//! instead of the xbox performance counter.

/// the game rate passed at boot (`VsBounds__572A0(24.0)`)
pub const GAME_RATE: f32 = 24.0;
/// `m_maxUpdatesPerFrame` set by `VsFrameTimer__574F0(2)` after construction
pub const MAX_UPDATES_PER_FRAME: i32 = 2;

#[derive(Clone, Debug)]
pub struct VsFrameTimer {
    pub dt: f32,                  // +0x00 1 / rate
    pub max_updates: i32,         // +0x04
    pub single_step: bool,        // +0x08
    pub vblank_mode: i32,         // +0x0C
    pub updates_this_frame: i32,  // +0x14
    pub total_updates: i32,       // +0x18
    pub frames: i32,              // +0x1C
    pub lerp: f32,                // +0x20
    pub render_time: f32,         // +0x24
    pub render_delta: f32,        // +0x28
    pub time_stretch: f32,        // +0x2C
    pub stretched: bool,          // +0x30
    pub stretch_changed: bool,    // +0x31
    pub stretched_updates: i32,   // +0x34
    pub stretched_time: f32,      // +0x38
    pub updates_last_frame: i32,  // +0x40
    pub ticks_per_update: u64,    // +0x48
    pub now: u64,                 // +0x50
    pub last_update: u64,         // +0x58
}

impl VsFrameTimer {
    /// VsFrameTimer.cpp 0x000572A0 (ctor) with `ticks_per_second` of the clock.
    pub fn new(rate: f32, ticks_per_second: u64, now: u64) -> Self {
        let mut t = VsFrameTimer {
            dt: 0.0,
            max_updates: 4,
            single_step: false,
            vblank_mode: 0,
            updates_this_frame: 0,
            total_updates: 0,
            frames: 0,
            lerp: 0.0,
            render_time: 0.0,
            render_delta: 0.0,
            time_stretch: 1.0,
            stretched: false,
            stretch_changed: false,
            stretched_updates: 0,
            stretched_time: 0.0,
            updates_last_frame: 0,
            ticks_per_update: 0,
            now: 0,
            last_update: 0,
        };
        t.set_rate(rate, ticks_per_second);
        t.reset(now);
        t
    }

    /// VsFrameTimer.cpp 0x00057450
    pub fn set_rate(&mut self, rate: f32, ticks_per_second: u64) {
        self.dt = 1.0 / rate;
        self.ticks_per_update = (ticks_per_second as f64 / rate as f64) as u64;
    }

    /// VsFrameTimer.cpp 0x00057310 — the next update is due immediately.
    pub fn reset(&mut self, now: u64) {
        self.now = u64::MAX;
        self.last_update = now.wrapping_sub(self.ticks_per_update * 2);
        self.lerp = 1.0;
        self.total_updates = 0;
        self.frames = 0;
        self.updates_this_frame = 0;
        self.render_time = 0.0;
        self.render_delta = 0.0;
        self.time_stretch = 1.0;
        self.stretched = false;
        self.stretch_changed = false;
        self.stretched_time = 0.0;
        self.stretched_updates = 0;
        self.updates_last_frame = 0;
    }

    /// VsFrameTimer.cpp 0x00057E40 — time stretch for the next update (slow motion).
    pub fn set_time_stretch(&mut self, s: f32) {
        debug_assert!(!self.stretch_changed, "timestretch factor already changed this update tick");
        self.stretch_changed = true;
        self.time_stretch = s;
        self.stretched = s != 1.0;
    }

    /// VsFrameTimer.cpp 0x00057F10
    fn count_update(&mut self) {
        if self.stretched {
            self.stretched_updates += 1;
            self.stretched_time += self.dt * self.time_stretch;
        }
    }

    /// VsFrameTimer.cpp 0x00057580
    pub fn tick(&mut self, now: u64) -> bool {
        self.stretch_changed = false;
        if self.single_step {
            return self.tick_single_step();
        }
        self.tick_realtime(now)
    }

    /// VsFrameTimer.cpp 0x000575E0 — one update per rendered frame.
    fn tick_single_step(&mut self) -> bool {
        if self.updates_this_frame != 0 {
            self.lerp = 0.0;
            self.render_time = self.total_updates as f32 * self.dt;
            self.render_delta = self.dt;
            self.updates_last_frame = self.updates_this_frame;
            self.updates_this_frame = 0;
            self.frames += 1;
            false
        } else {
            self.updates_this_frame += 1;
            self.total_updates += 1;
            self.count_update();
            true
        }
    }

    /// VsFrameTimer.cpp 0x00057690
    fn tick_realtime(&mut self, now: u64) -> bool {
        self.now = now;
        let elapsed = now.wrapping_sub(self.last_update);
        let prev_lerp = self.lerp;
        if self.updates_this_frame >= self.max_updates {
            if elapsed >= self.ticks_per_update {
                // too far behind: drop the time
                self.last_update = now;
                self.lerp = 0.0;
            } else {
                self.lerp = (elapsed as f64 / self.ticks_per_update as f64) as f32;
            }
        } else {
            if elapsed >= self.ticks_per_update * 2 {
                self.last_update = now;
                self.updates_this_frame += 1;
                self.total_updates += 1;
                self.count_update();
                return true;
            }
            if elapsed > self.ticks_per_update {
                self.last_update += self.ticks_per_update;
                self.updates_this_frame += 1;
                self.total_updates += 1;
                self.count_update();
                return true;
            }
            self.lerp = (elapsed as f64 / self.ticks_per_update as f64) as f32;
        }
        debug_assert!(self.lerp >= 0.0 && self.lerp <= 1.0);
        self.render_delta = ((self.lerp - prev_lerp) + self.updates_this_frame as f32) * self.dt;
        self.render_time += self.render_delta;
        self.updates_last_frame = self.updates_this_frame;
        self.updates_this_frame = 0;
        self.frames += 1;
        false
    }

    /// GameManager.cpp 0x000BF650 — real update step
    pub fn update_dt(&self) -> f32 {
        self.dt
    }

    /// GameManager.cpp 0x000BF670 — stretched update step
    pub fn scaled_dt(&self) -> f32 {
        self.dt * self.time_stretch
    }

    /// GameManager.cpp 0x000BF690 — unstretched time since reset
    pub fn update_time(&self) -> f32 {
        self.total_updates as f32 * self.dt
    }

    /// VsAIRaceVehicle.cpp-attributed 0x00024390 — game time with stretched periods counted at their rate
    pub fn game_time(&self) -> f32 {
        (self.total_updates - self.stretched_updates) as f32 * self.dt + self.stretched_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_steps() {
        let tps = 24_000u64;
        let mut t = VsFrameTimer::new(GAME_RATE, tps, 0);
        t.max_updates = MAX_UPDATES_PER_FRAME;
        // first call: the reset leaves two updates owed, the catch-up drops the backlog
        assert!(t.tick(0));
        assert!(!t.tick(0));
        // 2.5 update periods later at most max_updates run, then a render
        let now = 2500;
        let mut n = 0;
        while t.tick(now) {
            n += 1;
        }
        assert_eq!(n, 1); // >= 2 periods: jumps straight to now
        assert_eq!(t.total_updates, 2);
        let now = now + 1500;
        assert!(t.tick(now));
        assert!(!t.tick(now));
        assert!((t.lerp - 0.5).abs() < 1e-6);
        assert!((t.dt - 1.0 / 24.0).abs() < 1e-9);
    }
}
