//! Traffic lights: two pairs of traffic blockers that take turns, with a 2s all-red gap
//! (src/Ai/VsAITrafficLight.cpp; reset and update sit in the VsAITools region of the idb).

use super::ai_context::AiHost;
use super::vs_ai_road_section::{VsAITrafficBlocker, VsAITrafficLightData};

/// flips both blockers of a pair; a blocker that turns off stops colliding.
/// VsAITrafficLight.cpp 0x00029710
pub fn toggle_group(blockers: &mut [VsAITrafficBlocker], group: [Option<usize>; 2], host: &mut dyn AiHost) {
    for b in group.into_iter().flatten() {
        let b = &mut blockers[b];
        b.flag = (b.flag == 0) as u32;
        if let Some(obj) = b.physics_object {
            host.traffic_blocker_object(obj, b.flag == 0);
        }
    }
}

impl VsAITrafficLightData {
    fn group(&self, pair: usize) -> [Option<usize>; 2] {
        [self.blockers[pair * 2], self.blockers[pair * 2 + 1]]
    }

    /// VsAITrafficLight.cpp 0x00029570
    pub fn reset(&mut self) {
        self.start_time = -1.0;
        self.state = 0;
    }

    /// VsAITrafficLight.cpp 0x000295A0
    pub fn update(&mut self, blockers: &mut [VsAITrafficBlocker], time: f32, host: &mut dyn AiHost) {
        let (Some(first), Some(_)) = (self.blockers[0], self.blockers[2]) else { return };
        if self.start_time == -1.0 {
            self.start_time = time;
            self.current_pair = Some(if blockers[first].flag != 0 { 1 } else { 0 });
        }
        let pair = self.current_pair.unwrap_or(0);
        match self.state {
            0 => {
                if time > self.start_time + self.cycle_time {
                    toggle_group(blockers, self.group(pair), host);
                    self.current_pair = Some(1 - pair);
                    self.state = 1;
                }
            }
            1 => {
                // start time is not reset on entering this state: the gap is 2s after the first switch
                if time > self.start_time + self.cycle_time + 2.0 {
                    toggle_group(blockers, self.group(pair), host);
                    self.state = 0;
                    self.start_time = time;
                }
            }
            _ => {}
        }
    }
}
