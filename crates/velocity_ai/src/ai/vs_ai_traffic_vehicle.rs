//! A traffic car: follows one lane, queues behind the car ahead, stops at red blockers, slows at
//! the lane end and gives way to racers (src/Ai/VsAITrafficVehicle.cpp; most of it sits in the
//! VsAITrafficLight region of the idb, 0x29830..0x29EA0).

use super::ai_context::AiContext;
use super::vs_ai_race_vehicle::{AiScene, AiVehicleView, VehicleRef};
use super::vs_ai_road_section::LaneRef;
use super::vs_ai_vehicle::{AiPhysicsVehicle, VehicleControls, VsAIVehicle};
use super::vs_ai_world::{VsAIHandling, VsAIWorld};
use super::vs_math::*;

/// what one traffic car needs to know about another (`m_pVehicles[i]`).
#[derive(Debug, Clone, Default)]
pub struct TrafficView {
    pub active: bool,
    pub lane: Option<LaneRef>,
    pub dist_along: f32,
    pub position: VsVector4,
    pub speed: f32,
    pub max_speed: f32,
    pub physics: Option<usize>,
}

/// per-frame inputs of a traffic update.
pub struct TrafficFrame<'a, 'c> {
    pub ctx: &'a mut AiContext<'c>,
    pub world: &'a mut VsAIWorld,
    pub scene: &'a AiScene,
    /// every traffic car, indexed like `VsTrafficManager::vehicles`
    pub traffic: &'a [TrafficView],
}

/// what an update asks the owning manager to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficEvent {
    None,
    /// reached the end of its lane and removed itself; the manager's active count drops
    Despawned { police: bool },
}

#[derive(Debug, Clone)]
pub struct VsAITrafficVehicle {
    pub base: VsAIVehicle,
    pub corner_tightness: f32,       // +0x008
    pub cruise_speed: f32,           // +0x018
    pub max_speed: f32,              // +0x01C current speed limit
    pub id: u32,                     // +0x078
    pub is_traffic: bool,            // +0x07C
    /// +0x0F0 the car's traffic.bin entry, whose `+0x0C` type bits (1 traffic, 2 fire engine) are read
    pub type_flags: Option<u32>,
    pub lane: Option<LaneRef>,       // +0x0F4
    pub seg: u32,                    // +0x0FC
    pub look_ahead: [VsVector4; 3],  // +0x100
    pub dist_along: f32,             // +0x130
    pub next_blocker: Option<usize>, // +0x134 index into the lane's blocker list
    pub unk_138: u32,                // +0x138
    pub behaviour: u32,              // +0x13C 1 ambient, 2 triggered (bit 1 times arrival with the player)
    pub hazard: bool,                // +0x140
    pub police: bool,                // +0x141 a police manager car in traffic disguise
    pub end_mode: u32,               // +0x144 1 stop at the lane end, 2 despawn there
    pub lane_offset: f32,            // +0x148 sideways shift of the lookahead points
}

impl Default for VsAITrafficVehicle {
    /// VsAITrafficLight.cpp 0x00029830
    fn default() -> Self {
        Self {
            base: VsAIVehicle::default(),
            corner_tightness: 0.0,
            cruise_speed: 0.0,
            max_speed: 0.0,
            id: 0,
            is_traffic: false,
            type_flags: None,
            lane: None,
            seg: 0,
            look_ahead: [VsVector4::default(); 3],
            dist_along: 0.0,
            next_blocker: None,
            unk_138: 0,
            behaviour: 0,
            hazard: false,
            police: false,
            end_mode: 0,
            lane_offset: 0.0,
        }
    }
}

impl VsAITrafficVehicle {
    /// VsAITrafficLight.cpp 0x000298A0
    pub fn init(&mut self, id: u32) {
        self.id = id;
        self.base.physics = None;
        self.type_flags = None;
        self.lane = None;
        self.police = false;
        self.reset();
    }

    /// VsAITrafficLight.cpp 0x00029920
    pub fn reset(&mut self) {
        // VsAITrafficVehicle.cpp 0x0002B750
        self.base.flag_004 = false;
        self.base.flag_00c = false;
        self.base.unk_010 = 0;
        self.base.flag_014 = false;
        self.max_speed = 0.0;
        self.base.damage = 0.0;
        self.base.max_damage = 100.0;
        self.deactivate_unbound();
        self.unk_138 = 0;
        self.base.flag_004 = false;
        self.base.flag_014 = true;
        self.cruise_speed = 14.0;
        self.max_speed = self.cruise_speed;
        self.base.flag_00c = false;
        self.base.unk_010 = 0;
        self.base.debug_selected = false;
        self.hazard = false;
        self.end_mode = 2;
        self.next_blocker = None;
        self.lane_offset = 0.0;
    }

    fn deactivate_unbound(&mut self) {
        self.lane = None;
        self.next_blocker = None;
        self.base.active = false;
    }

    /// leaves its lane and the world.
    /// VsAITrafficLight.cpp 0x00029BE0
    pub fn deactivate(&mut self, world: &mut VsAIWorld, physics: Option<&mut dyn AiPhysicsVehicle>) {
        if let Some(l) = self.lane {
            let lane = world.lane_mut(l);
            lane.num_vehicles = lane.num_vehicles.wrapping_sub(1);
        }
        self.lane = None;
        self.next_blocker = None;
        self.base.deactivate(physics);
    }

    /// type bits of its traffic.bin entry, 1 without one.
    /// VsAITrafficVehicle.cpp 0x0002B540
    pub fn get_type(&self) -> u32 {
        self.type_flags.unwrap_or(1)
    }
    /// VsAITrafficVehicle.cpp 0x0002B5D0
    pub fn is_triggered_type(&self) -> bool {
        (self.get_type() & 2) != 0
    }

    pub fn view(&self) -> TrafficView {
        TrafficView {
            active: self.base.active,
            lane: self.lane,
            dist_along: self.dist_along,
            position: self.base.position,
            speed: self.base.speed,
            max_speed: self.max_speed,
            physics: self.base.physics,
        }
    }

    /// as race cars see it.
    pub fn race_view(&self, i: usize) -> AiVehicleView {
        AiVehicleView {
            r: VehicleRef::Traffic(i),
            active: self.base.active,
            is_traffic: true,
            physics: self.base.physics,
            id: self.id,
            position: self.base.position,
            velocity: self.base.velocity,
            forward: self.base.forward,
            speed: self.base.speed,
            ..Default::default()
        }
    }

    fn phys<'p>(&self, f: &'p mut TrafficFrame) -> &'p mut dyn AiPhysicsVehicle {
        f.ctx.physics.vehicle(self.base.physics.expect("m_pPhysicsVehicle"))
    }
    fn controls<'p>(&self, f: &'p mut TrafficFrame) -> &'p mut VehicleControls {
        self.phys(f).controls()
    }
    fn handling<'w>(&self, w: &'w VsAIWorld) -> &'w VsAIHandling {
        &w.handlings[self.base.handling.expect("m_pHandling")]
    }
    fn lane_ref(&self) -> LaneRef {
        self.lane.expect("m_pLane")
    }

    /// VsAITrafficLight.cpp 0x00029A10
    pub fn update(&mut self, f: &mut TrafficFrame) -> TrafficEvent {
        if !self.base.active {
            return TrafficEvent::None;
        }
        self.max_speed = self.cruise_speed;
        if self.is_triggered_type() {
            f.ctx.host.traffic_trigger_update(self.id);
        }
        self.base.debug_selected = f.world.debug_vehicle_id == self.id;
        // the debug curve and lookahead drawing of the original is not ported
        self.update_state(f);
        self.update_next_blocker(f.world);
        self.update_look_ahead(f);
        self.drive(f)
    }

    /// VsAITrafficLight.cpp 0x00029C50
    fn update_state(&mut self, f: &mut TrafficFrame) {
        let (spd, fwd, pos, ori, vel) = {
            let v = self.phys(f);
            (v.speed(), v.forward(), v.position(), v.orientation(), v.velocity())
        };
        self.base.speed = spd;
        self.base.forward.x = fwd.x;
        self.base.forward.y = fwd.y;
        self.base.forward.z = fwd.z;
        self.base.position = pos;
        self.base.orientation = ori;
        self.base.velocity.x = vel.x;
        self.base.velocity.y = vel.y;
        self.base.velocity.z = vel.z;
        let line = &f.world.lane(self.lane_ref()).line;
        self.seg = line.find_segment(&self.base.position, self.seg);
        self.dist_along = line.get_distance_along_line(&self.base.position, self.seg);
    }

    /// first blocker ahead on the lane.
    /// VsAITrafficLight.cpp 0x00029DE0
    fn update_next_blocker(&mut self, w: &VsAIWorld) {
        let lane = w.lane(self.lane_ref());
        if lane.blockers.is_empty() {
            return;
        }
        if self.next_blocker.is_none_or(|b| self.dist_along > lane.blockers[b].1) {
            for (i, b) in lane.blockers.iter().enumerate() {
                if b.1 > self.dist_along {
                    self.next_blocker = Some(i);
                    return;
                }
            }
        }
    }

    /// lookahead points, the ambush speed and the red-blocker stop.
    /// VsAITrafficLight.cpp 0x00029EA0
    fn update_look_ahead(&mut self, f: &mut TrafficFrame) {
        self.controls(f).unk_18 = false;
        let h = self.handling(f.world).clone();
        let v = f.world.response_curves[h.speed_ratio_rc].get_value(self.base.speed / h.max_speed);
        let step = (h.look_ahead_max - h.look_ahead_min) * v + h.look_ahead_min;
        let l = self.lane_ref();
        let (d, seg) = (self.dist_along, self.seg);
        self.set_look_ahead(f.world, l, d, step, seg, false);
        if !self.is_triggered_type() {
            self.apply_lane_offset(f.world);
        }
        if (self.behaviour & 2) != 0 && !self.is_triggered_type() {
            // arrive at the crossing together with the player
            if let Some(p0) = f.scene.players.first() {
                let lane = f.world.lane(l);
                if p0.race_section.is_some() && p0.race_section == lane.race_section {
                    let gap = lane.race_section_dist - p0.dist_along;
                    if gap > 0.0 {
                        let t = gap / p0.speed;
                        let dist = lane.line.get_length() * 0.5 - self.dist_along;
                        if dist / self.base.speed > 0.0 && t > 0.0 {
                            self.max_speed = dist / t;
                            if self.max_speed <= 50.0 {
                                if self.max_speed < 2.0 {
                                    self.max_speed = 2.0;
                                }
                            } else {
                                self.max_speed = 50.0;
                            }
                        }
                    }
                }
            }
        }
        if self.is_triggered_type() {
            return;
        }
        let Some(bi) = self.next_blocker else { return };
        let (blocker, bdist) = f.world.lane(l).blockers[bi];
        let red = f.world.spline.as_ref().unwrap().traffic_blockers[blocker].flag != 0;
        if !red {
            return;
        }
        let ext = self.phys(f).extent();
        let gap = bdist - (self.dist_along + ext);
        if gap > 0.0 {
            if gap >= 2.0 {
                if gap >= 10.0 {
                    if gap < 20.0 {
                        self.max_speed = self.cruise_speed * 0.5;
                    }
                } else {
                    self.max_speed = 4.0;
                }
            } else {
                self.max_speed = 0.0;
            }
        }
    }

    /// any racer within `dist_sq` whose direction from us is within the cone.
    /// VsAITrafficVehicle.cpp 0x0002A1A0
    fn racer_in_cone(&self, racer: &AiVehicleView, dir: &VsVector4, dist_sq: f32, min_dot: f32) -> bool {
        if dist_sq > vs_distance_squared(&self.base.position, &racer.position) {
            let mut d = racer.position - self.base.position;
            d.normalise();
            return vs_dot3(dir, &d) >= min_dot;
        }
        false
    }

    /// players first, then the ai cars.
    /// VsAITrafficVehicle.cpp 0x0002A2D0
    fn any_racer_in_cone(&self, scene: &AiScene, dir: &VsVector4, dist_sq: f32, min_dot: f32) -> bool {
        scene.players.iter().chain(scene.ai.iter()).any(|r| self.racer_in_cone(r, dir, dist_sq, min_dot))
    }

    /// queueing, lane-end handling and giving way to racers; true when it steered round a racer.
    /// VsAITrafficVehicle.cpp 0x0002A420
    fn avoidance(&mut self, f: &mut TrafficFrame) -> (bool, TrafficEvent) {
        let l = self.lane_ref();
        let mut avoided = false;
        let mut despawn = false;
        self.base.flag_004 = false;
        if let Some(a) = self.find_vehicle_ahead(f) {
            let ahead = f.traffic[a].clone();
            let d = vs_distance(&self.base.position, &ahead.position);
            let ext_a = ahead.physics.map_or(0.0, |h| f.ctx.physics.vehicle_ref(h).extent());
            let ext_me = self.phys(f).extent();
            if d - (ext_a + ext_me) < 8.0 {
                if ahead.max_speed == 0.0 {
                    self.max_speed = 0.0;
                } else {
                    self.max_speed = ahead.speed - 1.0;
                    if self.max_speed < 0.0 {
                        self.max_speed = 0.0;
                    }
                }
            }
        }
        let cone_dist = 900.0f32;
        let cone_dot = f32::from_bits(0x3F77_0A3D);
        let fwd = self.base.forward;
        let mut blocked = self.any_racer_in_cone(f.scene, &fwd, cone_dist, cone_dot);
        if self.is_triggered_type() && !blocked {
            let mut d = self.look_ahead[0];
            d -= self.base.position;
            d.normalise();
            blocked = self.any_racer_in_cone(f.scene, &d, cone_dist, cone_dot);
        }
        let mut end_limit = f32::MAX;
        let line = &f.world.lane(l).line;
        if !line.is_circular() {
            let remain = line.get_length() - self.dist_along;
            if self.end_mode == 1 {
                if remain < 40.0 {
                    end_limit = self.cruise_speed * 0.5;
                }
                if remain < 20.0 {
                    end_limit = 20.0;
                }
                if remain < 10.0 {
                    end_limit = 10.0;
                }
                if remain < 5.0 {
                    end_limit = 0.0;
                    let c = self.controls(f);
                    c.handbrake = 1.0;
                    c.unk_18 = true;
                }
            }
            if self.end_mode == 2 && remain < 0.5 {
                despawn = true;
            }
        }
        if blocked {
            let mut found = false;
            if self.is_triggered_type() {
                let step = f32::from_bits(0x3DB2_B8C3);
                let dist_sq = 400.0f32;
                let mut k = 1;
                while !found && k < 12 {
                    for s in [-1i32, 1] {
                        let ang = (k * s) as f32 * step;
                        let mut d = self.look_ahead[0] - self.base.position;
                        d.normalise();
                        let mut r = vs_rotate_y(&d, ang);
                        r *= 8.0;
                        if !self.any_racer_in_cone(f.scene, &r, dist_sq, cone_dot) {
                            log::debug!("Found a way at angle {:8.2} degrees", ang);
                            let mut target = self.base.position;
                            target += r;
                            let h = self.base.physics.unwrap();
                            let a = f.ctx.physics.steer_angle_to(h, &target);
                            self.phys(f).set_steer_angle(a);
                            avoided = true;
                            let s2 = self.base.get_steer_towards(&target, 1.0);
                            self.corner_tightness = s2 + s2;
                            self.corner_tightness = (self.corner_tightness * self.corner_tightness) * self.corner_tightness;
                            found = true;
                            break;
                        }
                    }
                    k += 1;
                }
            }
            if !found {
                self.set_hazard(f, true);
                self.max_speed = 0.0;
            }
        } else {
            self.set_hazard(f, false);
        }
        self.max_speed = if end_limit < self.max_speed { end_limit } else { self.max_speed };
        let mut ev = TrafficEvent::None;
        if despawn {
            ev = TrafficEvent::Despawned { police: self.police };
            let h = self.base.physics;
            let w = &mut *f.world;
            match h {
                Some(h) => self.deactivate(w, Some(f.ctx.physics.vehicle(h))),
                None => self.deactivate(w, None),
            }
        }
        (avoided, ev)
    }

    /// nearest traffic car ahead on the same lane.
    /// VsAITrafficVehicle.cpp 0x0002AB00
    fn find_vehicle_ahead(&self, f: &TrafficFrame) -> Option<usize> {
        let l = self.lane_ref();
        let line = &f.world.lane(l).line;
        let len = line.get_length();
        let mut look = line.wrap_distance(self.dist_along + 50.0);
        if look > len {
            if line.is_circular() {
                look -= len;
            } else {
                look = len;
            }
        }
        let mut best = 10_000_000.0f32;
        let mut found = None;
        let me = self.dist_along;
        for (i, o) in f.traffic.iter().enumerate() {
            if !o.active || o.lane != Some(l) || o.physics == self.base.physics {
                continue;
            }
            let od = o.dist_along;
            if od <= me {
                if me > od && line.is_circular() && me > look && look > od {
                    let gap = len - me + od;
                    if best > gap {
                        found = Some(i);
                        best = gap;
                    }
                }
            } else if me > look || look > od {
                let gap = od - me;
                if best > gap {
                    found = Some(i);
                    best = gap;
                }
            }
        }
        found
    }

    /// steering at the lookahead, throttle/brake from the corner and the speed limit.
    /// VsAITrafficVehicle.cpp 0x0002AD80
    fn drive(&mut self, f: &mut TrafficFrame) -> TrafficEvent {
        if !self.base.active {
            return TrafficEvent::None;
        }
        {
            let c = self.controls(f);
            c.handbrake = 0.0;
            c.horn = false;
        }
        let (avoided, ev) = self.avoidance(f);
        if !self.base.active {
            return ev;
        }
        let h = self.handling(f.world).clone();
        let sr = f.world.response_curves[h.speed_ratio_rc].get_value(self.base.speed / h.max_speed);
        if !avoided {
            let a = self.base.get_steer_towards(&self.look_ahead[0], 1.0);
            self.phys(f).set_steer_angle(a);
            let s1 = self.base.get_steer_towards(&self.look_ahead[1], 1.0);
            let s2 = self.base.get_steer_towards(&self.look_ahead[2], 1.0);
            self.corner_tightness = s2 + s1;
            self.corner_tightness = (self.corner_tightness * self.corner_tightness) * self.corner_tightness;
        }
        let c = self.corner_tightness;
        let scale = 1.0f32;
        let throttle = f.world.response_curves[h.throttle_rc].get_value(vs_abs(c) * sr * h.throttle_factor * scale);
        let brake = f.world.response_curves[h.brake_rc].get_value(vs_abs(c) * sr * h.brake_factor * scale);
        let (spd, max) = (self.base.speed, self.max_speed);
        let limit = self.base.flag_014;
        let ctl = self.controls(f);
        ctl.throttle = throttle;
        ctl.brake = brake;
        if limit {
            let diff = spd - max;
            let ratio = spd / max;
            if max == 0.0 {
                ctl.throttle = 0.0;
                ctl.brake = 1.0;
                ctl.handbrake = 1.0;
            } else if diff > 0.0 {
                ctl.throttle = 0.0;
                ctl.brake = 1.0;
                if ratio > 1.0 && ratio <= 1.1 {
                    ctl.brake = (ratio - 1.0) * 10.0;
                }
            }
        }
        ev
    }

    /// three lookahead points 1, 3 and 4 steps along the lane.
    /// VsAITrafficVehicle.cpp 0x0002B100
    pub fn set_look_ahead(&mut self, w: &mut VsAIWorld, l: LaneRef, dist: f32, step: f32, seg: u32, backwards: bool) {
        let line = &mut w.lane_mut(l).line;
        for i in 0..3u32 {
            let mut k = i + 1;
            if i != 0 {
                k += 1;
            }
            let mut d = if backwards { dist - k as f32 * step } else { k as f32 * step + dist };
            if line.is_circular() {
                if d > line.get_length() {
                    d -= line.get_length();
                }
                if d < 0.0 {
                    d += line.get_length();
                }
            }
            let mut out = 0;
            self.look_ahead[i as usize] = line.get_point_at_distance(d, seg, &mut out);
        }
    }

    /// shifts the first two lookahead points sideways by `lane_offset`.
    /// VsAITrafficVehicle.cpp 0x0002B250
    fn apply_lane_offset(&mut self, w: &VsAIWorld) {
        if self.lane_offset == 0.0 {
            return;
        }
        let line = &w.lane(self.lane_ref()).line;
        for i in 0..2 {
            let (a, b) = line.get_segment(self.seg);
            let mut dir = b - a;
            dir.normalise();
            // original quirk: the offset's y is the point's own height, so y grows too
            let mut n = if self.lane_offset >= 0.0 {
                VsVector4::new(-dir.z, self.look_ahead[i].y, dir.x, 0.0)
            } else {
                VsVector4::new(dir.z, self.look_ahead[i].y, -dir.x, 0.0)
            };
            n *= vs_abs(self.lane_offset);
            self.look_ahead[i] += n;
        }
    }

    /// VsAITrafficVehicle.cpp 0x0002B4B0
    fn set_hazard(&mut self, f: &mut TrafficFrame, on: bool) {
        if on != self.hazard {
            self.hazard = on;
            f.ctx.host.traffic_hazard(self.id, on);
        }
    }
}

