//! Ambient traffic: a fixed pool of cars spawned onto traffic lanes out of the players' sight and
//! removed once far away, plus the traffic lights and trigger-spawned cars
//! (src/Ai/VsTrafficManager.cpp). Police cars driving as traffic are not handled here yet.

use super::ai_context::AiContext;
use super::vs_ai_race_vehicle::AiScene;
use super::vs_ai_road_section::{LaneRef, RoadSectionSet};
use super::vs_ai_tools::VsAITools;
use super::vs_ai_traffic_light::toggle_group;
use super::vs_ai_traffic_vehicle::{TrafficEvent, TrafficFrame, TrafficView, VsAITrafficVehicle};
use super::vs_ai_world::VsAIWorld;
use super::vs_math::*;
use super::vs_ai_police_vehicle::PoliceVehicleMode;
use super::vs_police_manager::VsPoliceManager;

/// MAX_TRAFFIC_VEHICLES
pub const MAX_TRAFFIC_VEHICLES: u32 = 14;

/// a car the manager drives: one of its own, or a police car's traffic part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficSlot {
    Own(usize),
    Police(usize),
}

fn slot_mut<'a>(own: &'a mut [VsAITrafficVehicle], police: Option<&'a mut VsPoliceManager>, s: TrafficSlot) -> &'a mut VsAITrafficVehicle {
    match s {
        TrafficSlot::Own(i) => &mut own[i],
        TrafficSlot::Police(i) => &mut police.expect("police manager").vehicles[i].traffic,
    }
}

/// byte_6D4018: read before every spawn, never written by any code
const NO_SPAWNING: bool = false;

/// where a car goes when spawned: the lane point, the point 10m on, the start velocity.
#[derive(Debug, Clone, Copy, Default)]
pub struct SpawnPoint {
    pub pos: VsVector4,
    pub ahead: VsVector4,
    pub velocity: VsVector4,
}

#[derive(Debug, Clone, Default)]
pub struct VsTrafficManager {
    pub vehicles: Vec<VsAITrafficVehicle>, // +0x00 count, +0x10 array (332 bytes each)
    pub num_active: u32,                   // +0x04
    pub enabled: bool,                     // +0x08
    pub lane_cursor: u32,                  // +0x14 next lane of section 0 to spawn on
    pub paused: bool,                      // +0x18 no spawning, everything removed
    pub first_id: u32,                     // +0x1C
}

/// proximity of a point to the players: 0 within 140m (scaled up over the first 10s), 2 beyond 220m.
/// VsAIWorld.cpp 0x00034600 / 0x000344E0 / 0x00034690
pub fn closest_player_proximity(scene: &AiScene, time: f32, p: &VsVector4) -> u32 {
    let mut best = 2;
    for pl in &scene.players {
        let d = vs_distance_squared(&pl.position, p);
        let mut t = time * 0.1;
        if t > 1.0 {
            t = 1.0;
        }
        let prox = if d <= 19600.0 * t {
            0
        } else if d > 48400.0 {
            2
        } else {
            1
        };
        best = if best == 0 || prox == 0 {
            0
        } else if best == 1 || prox == 1 {
            1
        } else {
            2
        };
    }
    best
}

impl VsTrafficManager {
    /// VsTrafficManager.cpp 0x0003E010
    pub fn new(world: &mut VsAIWorld, first_id: u32, host: &mut dyn super::ai_context::AiHost) -> Self {
        let mut m = Self { first_id, ..Default::default() };
        m.create();
        m.reset(world, host);
        m
    }

    /// the 14 cars.
    /// VsTrafficManager.cpp 0x0003FC10
    fn create(&mut self) {
        self.vehicles = vec![VsAITrafficVehicle::default(); MAX_TRAFFIC_VEHICLES as usize];
        let mut id = self.first_id;
        for v in self.vehicles.iter_mut() {
            v.init(id);
            v.is_traffic = true;
            id += 1;
        }
    }

    /// VsTrafficManager.cpp 0x0003E0C0
    pub fn reset(&mut self, world: &mut VsAIWorld, host: &mut dyn super::ai_context::AiHost) {
        self.enabled = true;
        self.lane_cursor = 0;
        for v in self.vehicles.iter_mut() {
            v.reset();
        }
        self.num_active = 0;
        self.remove_all(world, None);
        if let Some(s) = world.spline.as_mut() {
            for l in s.traffic_lights.iter_mut() {
                l.reset();
            }
        }
        self.reset_blockers(world, host);
        self.paused = false;
    }

    /// blockers back to their file state.
    /// VsTrafficManager.cpp 0x0003E230
    fn reset_blockers(&mut self, world: &mut VsAIWorld, host: &mut dyn super::ai_context::AiHost) {
        let Some(s) = world.spline.as_mut() else { return };
        for b in s.traffic_blockers.iter_mut() {
            b.flag = b.active as u32;
            if let Some(obj) = b.physics_object
                && !b.active
            {
                host.traffic_blocker_object(obj, true);
            }
        }
    }

    /// VsTrafficManager.cpp 0x00040740
    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn views(&self) -> Vec<TrafficView> {
        self.vehicles.iter().map(|v| v.view()).collect()
    }

    /// VsTrafficManager.cpp 0x0003E2E0
    pub fn update(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, scene: &AiScene, mut police: Option<&mut VsPoliceManager>) {
        self.update_spawning(ctx, world, scene, police.as_deref_mut());
        self.update_lights(ctx, world);
        for i in 0..self.vehicles.len() {
            let views = self.views();
            let mut f = TrafficFrame { ctx, world, scene, traffic: &views };
            if let TrafficEvent::Despawned { police } = self.vehicles[i].update(&mut f)
                && !police
            {
                self.num_active = self.num_active.wrapping_sub(1);
            }
        }
    }

    /// VsTrafficManager.cpp 0x0003E350
    fn update_lights(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld) {
        let time = world.time;
        let Some(s) = world.spline.as_mut() else { return };
        for l in s.traffic_lights.iter_mut() {
            l.update(&mut s.traffic_blockers, time, ctx.host);
        }
    }

    /// toggles a light pair; exposed for triggers.
    pub fn toggle_blockers(world: &mut VsAIWorld, group: [Option<usize>; 2], ctx: &mut AiContext) {
        if let Some(s) = world.spline.as_mut() {
            toggle_group(&mut s.traffic_blockers, group, ctx.host);
        }
    }

    /// one spawn attempt per frame on the next lane of traffic section 0, then far cars removed.
    /// VsTrafficManager.cpp 0x0003F670
    fn update_spawning(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, scene: &AiScene, mut police: Option<&mut VsPoliceManager>) {
        if ctx.game.race_clock <= 2.0 || self.paused {
            self.remove_all(world, Some(ctx));
            return;
        }
        let Some(sec) = world.road_sections(RoadSectionSet::Traffic).first() else { return };
        let n = sec.get_num_lanes();
        self.lane_cursor += 1;
        if self.lane_cursor >= n {
            self.lane_cursor = 0;
        }
        let lane = LaneRef { set: RoadSectionSet::Traffic, section: 0, lane: self.lane_cursor as usize };
        let v = self.find_free_vehicle(world, scene, ctx, 1, 1, police.as_deref_mut());
        if let Some(v) = v
            && !NO_SPAWNING
        {
            let off = ctx.rand.range(-1.0, 1.0);
            let veh = slot_mut(&mut self.vehicles, police.as_deref_mut(), v);
            veh.end_mode = 2;
            veh.behaviour = 1;
            veh.lane_offset = off;
            if let Some((pt, sp)) = self.find_spawn_point(ctx, world, scene, lane, v, true, true, police.as_deref_mut()) {
                self.spawn(ctx, world, lane, pt, v, sp, police.as_deref_mut());
            }
        }
        self.remove_far(ctx, world, scene, police);
    }

    /// a lane point clear of ai cars (30m) and of other traffic on the lane, walking the lane in a
    /// random direction; 0 when none.
    /// VsTrafficManager.cpp 0x0003E4E0
    #[allow(clippy::too_many_arguments)]
    fn find_spawn_point(
        &mut self,
        ctx: &mut AiContext,
        world: &mut VsAIWorld,
        scene: &AiScene,
        lane: LaneRef,
        v: TrafficSlot,
        check_proximity: bool,
        ambush: bool,
        mut police: Option<&mut VsPoliceManager>,
    ) -> Option<(u32, SpawnPoint)> {
        let n = world.lane(lane).line.get_num_points() as i32;
        let (start, end, step) = if ctx.rand.unit() <= 0.5 { (1, n - 1, 1) } else { (n - 2, 0, -1) };
        let mut i = start;
        while i != end {
            // original quirk: walking backwards starts at n-2 > n-3, so that direction never spawns
            if i > n - 3 {
                return None;
            }
            let p = *world.lane(lane).line.get_point(i as u32);
            if !check_proximity || closest_player_proximity(scene, world.time, &p) == 1 {
                let mut ok = !scene.ai.iter().any(|a| vs_distance_squared(&p, &a.position) < 900.0);
                if ok {
                    let gap = if ambush { ctx.rand.range(0.0, 20.0) + 50.0 } else { 5.0 };
                    for o in &self.vehicles {
                        if o.base.active && o.lane == Some(lane) && gap > vs_abs(p.w - o.dist_along) {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok
                    && world.police_enabled
                    && let Some(pm) = police.as_deref()
                {
                    let gap = if ambush { ctx.rand.range(0.0, 20.0) + 50.0 } else { 5.0 };
                    for pv in &pm.vehicles {
                        if pv.is_active()
                            && pv.mode == PoliceVehicleMode::Traffic
                            && pv.traffic.lane == Some(lane)
                            && gap > vs_abs(p.w - pv.traffic.dist_along)
                        {
                            ok = false;
                            break;
                        }
                    }
                }
                if ok && let Some(sp) = self.check_spawn_point(world, scene, lane, i as u32, v, ambush, police.as_deref_mut()) {
                    return Some((i as u32, sp));
                }
            }
            i += step;
        }
        None
    }

    /// sets the car on the lane at `point`; with `ambush`, only where the player is heading towards it.
    /// VsTrafficManager.cpp 0x0003E940
    #[allow(clippy::too_many_arguments)]
    fn check_spawn_point(
        &mut self,
        world: &mut VsAIWorld,
        scene: &AiScene,
        lane: LaneRef,
        point: u32,
        v: TrafficSlot,
        ambush: bool,
        police: Option<&mut VsPoliceManager>,
    ) -> Option<SpawnPoint> {
        debug_assert!(point != 0, "dPoint != 0");
        let mut ok = true;
        let veh = slot_mut(&mut self.vehicles, police, v);
        veh.seg = point;
        veh.lane = Some(lane);
        let line = &mut world.lane_mut(lane).line;
        let mut d = line.get_point(point).w + 10.0;
        if d > line.get_length() {
            d = line.get_length();
        }
        let mut out = 0;
        let ahead = line.get_point_at_distance(d, point, &mut out);
        veh.look_ahead[0] = ahead;
        let p = *line.get_point(point);
        let mut dir = ahead - p;
        dir.normalise();
        dir *= veh.cruise_speed * f32::from_bits(0x3E8E_38C9);
        let lane_speed = world.lane(lane).speed;
        if ambush {
            let pl = closest_player(scene, &ahead);
            debug_assert!(pl.is_some(), "pPlayerVehicle");
            if let Some(pl) = pl {
                let mut vel = pl.velocity;
                if vel.length_squared() < 1.0 {
                    vel = pl.forward;
                }
                let mut to = p;
                to -= pl.position;
                to.normalise();
                if vs_dot3(&to, &pl.forward) >= f32::from_bits(0x3F34_FDF4) {
                    if dir.length_squared() >= f32::from_bits(0x3DCC_CCCD) {
                        if lane_speed > 0.0 {
                            let t = VsAITools::line_parameter(&VsVector4::xyz(0.0, 0.0, 0.0), &dir, &vel);
                            let p0 = &scene.players[0];
                            let mut d2 = p - p0.position;
                            d2.normalise();
                            let ang = VsAITools::angle_between(&p0.forward, &d2);
                            ok = if ang > f32::from_bits(0x3FC9_0FDB) && ang < f32::from_bits(0x4096_CBE4) {
                                (0.0..=1.0).contains(&t)
                            } else {
                                !(0.0..=1.0).contains(&t)
                            };
                        }
                    } else {
                        ok = false;
                    }
                } else {
                    ok = false;
                }
            }
        }
        ok.then_some(SpawnPoint { pos: p, ahead, velocity: dir })
    }

    /// places the car on the ground at the spawn point, facing along the lane.
    /// VsTrafficManager.cpp 0x0003F050
    #[allow(clippy::too_many_arguments)]
    fn spawn(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, lane: LaneRef, point: u32, v: TrafficSlot, mut sp: SpawnPoint, police: Option<&mut VsPoliceManager>) {
        debug_assert!(point != 0, "dPoint != 0");
        let mut dir = sp.ahead;
        dir -= sp.pos;
        dir.w = 1.0;
        dir.normalise();
        let ang = std::f32::consts::FRAC_PI_2 - VsAITools::angle_between(&VsVector4::new(0.0, 0.0, 1.0, 1.0), &dir);
        let q = vs_quat_axis_angle(ang, &VsVector4::xyz(0.0, 1.0, 0.0));
        let mut below = sp.pos;
        let mut above = sp.pos;
        below.y -= 3.0;
        above.y += 3.0;
        match VsAITools::line_check(&*ctx.physics, &above, &below) {
            Some(h) => {
                sp.pos = h;
                sp.pos.y += 0.5;
            }
            None => log::error!(
                "Error finding AI Traffic Vehicle spawn position between - ({}, {}, {}) and ({}, {}, {})",
                above.x, above.y, above.z, below.x, below.y, below.z
            ),
        }
        match v {
            TrafficSlot::Police(i) => {
                let pm = police.expect("police manager");
                pm.spawn_as_traffic(ctx, i, &sp.pos, &q, &sp.velocity);
                pm.last_spawn = world.time;
                debug_assert!(pm.vehicles[i].traffic.lane.is_some(), "pVehicle->GetLane() != NULL");
            }
            TrafficSlot::Own(i) => {
                let veh = &mut self.vehicles[i];
                let h = veh.base.physics.expect("m_pPhysicsVehicle");
                veh.base.teleport(ctx.physics.vehicle(h), &sp.pos, &q, &sp.velocity);
                ctx.physics.vehicle(h).set_control_mode(2);
                self.num_active += 1;
                ctx.host.traffic_spawned(veh.id);
                debug_assert!(veh.lane.is_some(), "pVehicle->GetLane() != NULL");
            }
        }
        world.lane_mut(lane).num_vehicles += 1;
        debug_assert!(self.num_active <= MAX_TRAFFIC_VEHICLES, "m_dNumActiveVehicles <= MAX_TRAFFIC_VEHICLES");
    }

    /// whether a car is far enough from every player to remove (only "far" proximity counts).
    /// VsTrafficManager.cpp 0x0003F7F0
    fn should_remove(world: &VsAIWorld, scene: &AiScene, p: VsVector4) -> bool {
        if closest_player_proximity(scene, world.time, &p) == 2 {
            return true;
        }
        for pl in &scene.players {
            if vs_distance_squared(&pl.position, &p) > 10000.0 {
                return false;
            }
            let mut d = p;
            d -= pl.position;
            d.normalise();
            if vs_dot3(&d, &pl.forward) < 0.0 {
                return false;
            }
        }
        false
    }

    /// VsTrafficManager.cpp 0x0003FA20
    fn remove_far(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, scene: &AiScene, police: Option<&mut VsPoliceManager>) {
        for i in 0..self.vehicles.len() {
            if self.vehicles[i].base.active && Self::should_remove(world, scene, self.vehicles[i].base.position) {
                self.remove(ctx, world, i);
            }
        }
        if let Some(pm) = police {
            for j in 0..pm.vehicles.len() {
                let v = &pm.vehicles[j];
                if v.is_active() && v.mode == PoliceVehicleMode::Traffic && Self::should_remove(world, scene, v.traffic.base.position) {
                    pm.remove(ctx, world, j);
                }
            }
        }
    }

    /// a free car of `type_mask`; a triggered spawn may steal an ambient car behind the player.
    /// VsTrafficManager.cpp 0x0003FD40
    #[allow(clippy::too_many_arguments)]
    fn find_free_vehicle(
        &mut self,
        world: &mut VsAIWorld,
        scene: &AiScene,
        ctx: &mut AiContext,
        type_mask: u32,
        flags: u32,
        police: Option<&mut VsPoliceManager>,
    ) -> Option<TrafficSlot> {
        // a police car joins the traffic when the police manager wants one more
        if let Some(pm) = police
            && (flags & 2) == 0
            && (type_mask & 2) == 0
            && pm.allow_spawn
            && pm.num_active < pm.max_active
            && world.time > pm.last_spawn + pm.spawn_interval
            && let Some(i) = pm.vehicles.iter().position(|v| !v.is_active())
        {
            return Some(TrafficSlot::Police(i));
        }
        for (i, v) in self.vehicles.iter().enumerate() {
            if !v.base.active && (type_mask & v.get_type()) != 0 {
                return Some(TrafficSlot::Own(i));
            }
        }
        if (flags & 2) != 0 && (type_mask & 2) == 0 {
            let p0 = scene.players.first().map_or(VsVector4::default(), |p| p.position);
            for i in 0..self.vehicles.len() {
                if (self.vehicles[i].behaviour & 1) != 0 {
                    let mut d = self.vehicles[i].base.position - p0;
                    d.normalise();
                    // original quirk: dots the player's position, not its heading
                    if p0.dot3(&d) < 0.0 && vs_distance_squared(&p0, &self.vehicles[i].base.position) > 400.0 {
                        self.remove(ctx, world, i);
                        return Some(TrafficSlot::Own(i));
                    }
                }
            }
            log::warn!("** Couldn't find free vehicle for Triggering!!! **");
        }
        None
    }

    /// everything off the road, lane counts cleared.
    /// VsTrafficManager.cpp 0x000400A0
    fn remove_all(&mut self, world: &mut VsAIWorld, mut ctx: Option<&mut AiContext>) {
        for i in 0..self.vehicles.len() {
            if self.vehicles[i].base.active {
                match ctx.as_deref_mut() {
                    Some(c) => self.remove(c, world, i),
                    None => {
                        self.vehicles[i].deactivate(world, None);
                        self.num_active = self.num_active.wrapping_sub(1);
                    }
                }
            }
        }
        self.num_active = 0;
        let n = world.road_sections(RoadSectionSet::Traffic).len();
        for s in 0..n {
            let lanes = world.road_sections(RoadSectionSet::Traffic)[s].get_num_lanes() as usize;
            for l in 0..lanes {
                world.lane_mut(LaneRef { set: RoadSectionSet::Traffic, section: s, lane: l }).num_vehicles = 0;
            }
        }
    }

    /// VsTrafficManager.cpp 0x000401B0
    fn remove(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, i: usize) {
        let v = &mut self.vehicles[i];
        match v.base.physics {
            Some(h) => v.deactivate(world, Some(ctx.physics.vehicle(h))),
            None => v.deactivate(world, None),
        }
        self.num_active = self.num_active.wrapping_sub(1);
    }

    /// a car of `type_mask` onto the first lane of `section`.
    /// VsTrafficManager.cpp 0x000402A0
    fn trigger_spawn(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, scene: &AiScene, set: RoadSectionSet, section: usize, type_mask: u32, flags: u32) -> Option<usize> {
        debug_assert!(type_mask.count_ones() == 1, "CheckOnlyOneBitIsSet(type)");
        let lane = LaneRef { set, section, lane: 0 };
        // flags 2 never picks a police car, so the slot is always the manager's own
        let v = self.find_free_vehicle(world, scene, ctx, type_mask, flags, None)?;
        let (pt, sp) = self.find_spawn_point(ctx, world, scene, lane, v, false, false, None)?;
        self.spawn(ctx, world, lane, pt, v, sp, None);
        match v {
            TrafficSlot::Own(i) => Some(i),
            TrafficSlot::Police(_) => None,
        }
    }

    /// a fire engine from the set-C spline holding lane `lane_id`; it stops at the end.
    /// VsTrafficManager.cpp 0x000404E0
    pub fn trigger_fire_engine(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, scene: &AiScene, lane_id: u32) -> Option<usize> {
        let Some(s) = world.find_section_by_lane_id(RoadSectionSet::SetC, lane_id) else {
            log::error!("** ERROR! TRIGGER FIRE ENGINE - Could not find matching Spline for Traffic Trigger");
            return None;
        };
        match self.trigger_spawn(ctx, world, scene, RoadSectionSet::SetC, s, 2, 2) {
            Some(v) => {
                let veh = &mut self.vehicles[v];
                veh.end_mode = 1;
                veh.behaviour = 2;
                veh.lane_offset = 0.0;
                Some(v)
            }
            None => {
                log::error!("** ERROR! TRIGGER FIRE ENGINE - Couldn't find free vehicle to spawn on trigger spline");
                None
            }
        }
    }

    /// an ambient car from the set-B spline holding lane `lane_id`.
    /// VsTrafficManager.cpp 0x00040590
    pub fn trigger_traffic(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, scene: &AiScene, lane_id: u32) -> Option<usize> {
        let Some(s) = world.find_section_by_lane_id(RoadSectionSet::SetB, lane_id) else {
            log::error!("** ERROR! TRIGGER TRAFFIC - Could not find matching Spline for Traffic Trigger");
            return None;
        };
        match self.trigger_spawn(ctx, world, scene, RoadSectionSet::SetB, s, 1, 2) {
            Some(v) => {
                let veh = &mut self.vehicles[v];
                veh.end_mode = 2;
                veh.behaviour = 2;
                veh.lane_offset = 0.0;
                Some(v)
            }
            None => {
                log::error!("** ERROR! TRIGGER TRAFFIC - Couldn't find free vehicle to spawn on trigger spline");
                None
            }
        }
    }
}

/// the player nearest `p`.
/// VsTrafficManager.cpp 0x0003EEE0
fn closest_player<'s>(scene: &'s AiScene, p: &VsVector4) -> Option<&'s super::vs_ai_race_vehicle::AiVehicleView> {
    let mut best = f32::MAX;
    let mut found = None;
    for pl in &scene.players {
        let d = (pl.position - *p).length_squared();
        if best > d {
            best = d;
            found = Some(pl);
        }
    }
    found
}
