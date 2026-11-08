//! src\GameMode\PostRaceSequence.cpp — `g_PostRaceSequence` (0x006FA070): after a race the first three cars are
//! lined up at the track's `POST_RACE_STUNT_AREA` marker (Markers.mfk) and the winner does a stunt, filmed
//! from a camera fixed to the marker. States: 0 idle, 1 fade out, 2 set up, 3 stunt (entry / update / exit
//! table at 0x000C3420; the object's own code sits in GameVars.cpp's range, 0x000C3290..0x000C3590).
//! Every shipped Markers.mfk is empty, so the marker is never found.

use bevy::math::{Mat3, Quat, Vec3};

use crate::camera::vs_base_camera::quat_matrix;
use crate::controller::vibrate_effects::CMarkers;
use crate::game_bridge::camera_bridge::CameraBridgeData;

/// the static tables (initialised at 0x00422FF0..0x004230A5): camera and look offsets, the three cars'
/// offsets and headings (degrees) round the marker
pub const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 2.0, 10.0);
pub const TARGET_OFFSET: Vec3 = Vec3::new(0.0, 1.0, 0.0);
pub const CAR_OFFSETS: [(Vec3, f32); 3] =
    [(Vec3::new(0.5, 0.0, 1.5), 210.0), (Vec3::new(-0.5, 0.0, -5.0), -50.0), (Vec3::new(-3.5, 0.0, -4.0), -30.0)];
/// stunt phase lengths (`dword_685EB0..685EBC`)
pub const STUNT_TIMES: [f32; 4] = [3.0, 1.0, 1.0, 5.0];
/// camera fov (0x3EC90FDB)
pub const CAMERA_FOV: f32 = f32::from_bits(0x3EC9_0FDB);

/// what the sequence does to the race cars (ai / physics side)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PostRaceEvent {
    /// DangerZoneRender 0x00123070 fade (1 out, -1 back in) and vibration off (0x001EDD20)
    Fade(f32),
    /// VsAIVehicle 0x0002C670: put a car at a position / orientation (and 0x0002C5D0 stop it)
    PlaceCar { car: u32, position: Vec3, rotation: Quat, stop: bool },
    /// VsAIRaceVehicle 0x000234F0 control mode (0 normal, 1 stunt hold, 2 stunt go, 3 parked)
    CarMode { car: u32, mode: u32 },
    /// physics car stunt inputs (+0x5C4..+0x5D0 = 1)
    StuntInputs { car: u32 },
    /// physics car inputs cleared (0x000C3F40 with 0)
    ClearInputs { car: u32 },
    /// every other ai racer parked at the origin (0x0002BDA0)
    ParkOthers,
    DisableTraffic,
    /// camera cut (0x0002C620) and render cut
    CameraCut,
}

#[derive(Clone, Debug, Default)]
pub struct PostRaceSequence {
    /// +0x30 the marker's orientation
    pub orientation: Mat3,
    /// +0x70 the marker position (on the ground)
    pub marker: Vec3,
    pub dt: f32,          // +0x80
    pub time: f32,        // +0x84
    pub stunt_time: f32,  // +0x88
    pub state: u32,       // +0x90
    pub stunt_state: u32, // +0x94
    pub counter: u32,     // +0x98
    /// +0x9C..+0xA4 the cars lined up (winner first)
    pub cars: [Option<u32>; 3],
    pub initialised: bool, // +0xA8
    pub first: bool,       // +0xA9
    /// +0xAA the sequence camera is the view
    pub camera: bool,
    /// `byte_6FA068` (bss): re-place the other two cars every update
    pub reposition: bool,
}

impl PostRaceSequence {
    /// GameVars.cpp 0x000C32D0 (InGameInit) and 0x000C3390
    pub fn init(&mut self) {
        if !self.initialised {
            self.reset();
            self.initialised = true;
        }
    }

    /// GameVars.cpp 0x000C3390
    pub fn reset(&mut self) {
        self.first = true;
        self.time = 0.0;
        self.dt = 0.0;
        self.state = 0;
        self.camera = false;
    }

    /// PostRaceSequence.cpp 0x000C35C0 — exit the current state, enter the new one
    pub fn set_state(&mut self, new: u32, ctx: &PostRaceContext, out: &mut Vec<PostRaceEvent>) {
        if new >= 4 {
            log::error!("assert: _newState >= 0 && _newState < PrS_NumItems");
            return;
        }
        match self.state {
            2 => out.push(PostRaceEvent::Fade(-1.0)),
            3 => self.release_cars(ctx, out),
            _ => {}
        }
        self.state = new;
        match new {
            1 => out.push(PostRaceEvent::Fade(1.0)),
            2 => {
                self.setup(ctx, out);
                // PostRaceSequence.cpp 0x000C4460
                self.camera = true;
                out.push(PostRaceEvent::CameraCut);
                self.counter = 0;
            }
            3 => {
                self.stunt_state = 0;
                self.stunt_time = 0.0;
            }
            _ => {}
        }
    }

    /// GameVars.cpp 0x000C3560 — start (from MonitorSessions)
    pub fn start(&mut self, ctx: &PostRaceContext, out: &mut Vec<PostRaceEvent>) {
        self.set_state(1, ctx, out);
    }

    /// GameVars.cpp 0x000C34B0 — per update (`fade` = the danger zone fade level, 0x000B3710)
    pub fn update(&mut self, dt: f32, fade: f32, ctx: &PostRaceContext, out: &mut Vec<PostRaceEvent>) {
        if !self.initialised {
            return;
        }
        if self.first {
            self.first = false;
        } else {
            self.time += dt;
            self.dt = dt;
        }
        match self.state {
            1 => {
                if fade > 0.9999 {
                    self.set_state(2, ctx, out);
                }
            }
            2 => {
                if self.counter == 4 {
                    self.set_state(3, ctx, out);
                }
                self.counter += 1;
            }
            3 => self.stunt(out),
            _ => {}
        }
    }

    fn place(&self, i: usize, car: u32, out: &mut Vec<PostRaceEvent>, stop: bool) {
        let (off, deg) = CAR_OFFSETS[i];
        let m = self.orientation * Mat3::from_rotation_y(deg.to_radians());
        let position = self.marker + self.orientation * off;
        out.push(PostRaceEvent::PlaceCar { car, position, rotation: Quat::from_mat3(&m), stop });
    }

    /// PostRaceSequence.cpp 0x000C3F70 — line the first three up at the marker
    fn setup(&mut self, ctx: &PostRaceContext, out: &mut Vec<PostRaceEvent>) {
        let crc = visos::handy::handy_crc::crc32(b"POST_RACE_STUNT_AREA", 0);
        let Some(m) = ctx.markers.find(crc) else {
            // the original dereferences the missing marker (no shipped track has one)
            log::warn!("PostRaceSequence: no POST_RACE_STUNT_AREA marker");
            return;
        };
        self.marker = m.position;
        if let Some(y) = (ctx.ground_y)(self.marker) {
            self.marker.y = y;
        }
        self.orientation = quat_matrix(m.orientation);
        if !ctx.split_screen {
            let order = &ctx.order;
            if ctx.player_position >= 4 {
                self.cars = [order.first().copied(), order.get(1).copied(), Some(ctx.player)];
            } else {
                self.cars = [Some(ctx.player), None, None];
                let mut k = 1;
                for &c in order.iter().take(3) {
                    if Some(c) != self.cars[0] && k < 3 {
                        self.cars[k] = Some(c);
                        k += 1;
                    }
                }
            }
            for i in 0..3 {
                if let Some(c) = self.cars[i] {
                    self.place(i, c, out, true);
                }
            }
            out.push(PostRaceEvent::ParkOthers);
            for c in self.cars.iter().flatten() {
                out.push(PostRaceEvent::CarMode { car: *c, mode: 1 });
            }
            out.push(PostRaceEvent::DisableTraffic);
        }
    }

    /// PostRaceSequence.cpp 0x000C3950 — the winner's stunt
    fn stunt(&mut self, out: &mut Vec<PostRaceEvent>) {
        for i in 1..3 {
            if self.reposition {
                if let Some(c) = self.cars[i] {
                    self.place(i, c, out, false);
                }
            }
        }
        self.stunt_time += self.dt;
        let car = self.cars[0];
        match self.stunt_state {
            0 => {
                if self.reposition {
                    if let Some(c) = car {
                        self.place(0, c, out, false);
                    }
                }
                if self.stunt_time > STUNT_TIMES[0] {
                    self.stunt_time = 0.0;
                    self.stunt_state = 1;
                    if let Some(c) = car {
                        out.push(PostRaceEvent::CarMode { car: c, mode: 2 });
                    }
                }
            }
            1 => {
                if let Some(c) = car {
                    out.push(PostRaceEvent::StuntInputs { car: c });
                }
                if self.stunt_time > STUNT_TIMES[1] {
                    self.stunt_time = 0.0;
                    self.stunt_state = 2;
                }
            }
            2 => {
                if self.stunt_time > STUNT_TIMES[2] {
                    self.stunt_time = 0.0;
                    self.stunt_state = 3;
                }
            }
            3 => {
                if self.stunt_time > STUNT_TIMES[3] {
                    self.stunt_time = 0.0;
                    self.stunt_state = 4;
                    if let Some(c) = car {
                        out.push(PostRaceEvent::CarMode { car: c, mode: 1 });
                    }
                }
            }
            4 => {}
            _ => log::error!("ERROR: PostRaceSequence::UpdateState_PerformStunt - invalid case for m_stuntState"),
        }
    }

    /// PostRaceSequence.cpp 0x000C3DE0 (exit of the stunt state)
    fn release_cars(&mut self, _ctx: &PostRaceContext, out: &mut Vec<PostRaceEvent>) {
        for c in self.cars.iter().flatten() {
            out.push(PostRaceEvent::CarMode { car: *c, mode: 0 });
            out.push(PostRaceEvent::ClearInputs { car: *c });
        }
        out.push(PostRaceEvent::ParkOthers);
    }

    /// PostRaceSequence.cpp 0x000C44A0 — "ApplyCurrentCamera": the view while the sequence camera is on
    pub fn camera_view(&self) -> Option<CameraBridgeData> {
        if !self.initialised || !self.camera {
            return None;
        }
        Some(CameraBridgeData {
            position: self.marker + self.orientation * CAMERA_OFFSET,
            target: self.marker + self.orientation * TARGET_OFFSET,
            up: Vec3::Y,
            fov: CAMERA_FOV,
        })
    }

    /// PostRaceSequence.cpp 0x000C4650
    pub fn camera_active(&self) -> bool {
        self.initialised && self.camera
    }
}

/// what the sequence reads at set-up
pub struct PostRaceContext<'a> {
    pub markers: &'a CMarkers,
    pub ground_y: &'a dyn Fn(Vec3) -> Option<f32>,
    pub split_screen: bool,
    pub player: u32,
    /// the player's finishing place (1-based)
    pub player_position: u32,
    /// race order (ids)
    pub order: Vec<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_through_its_states_without_a_marker() {
        let markers = CMarkers::default();
        let ctx = PostRaceContext { markers: &markers, ground_y: &|_| None, split_screen: false, player: 0, player_position: 1, order: vec![0, 1, 2] };
        let mut p = PostRaceSequence::default();
        p.init();
        let mut out = Vec::new();
        p.start(&ctx, &mut out);
        p.update(0.04, 1.0, &ctx, &mut out);
        p.update(0.04, 1.0, &ctx, &mut out);
        assert_eq!(p.state, 2);
        for _ in 0..6 {
            p.update(0.04, 1.0, &ctx, &mut out);
        }
        assert_eq!(p.state, 3);
        assert!(p.camera_active());
    }
}
