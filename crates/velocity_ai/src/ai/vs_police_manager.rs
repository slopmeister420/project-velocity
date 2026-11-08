//! The police: two cars that drive as traffic until the player speeds past or hits one, then chase,
//! escalate with the wanted level and give up when the player gets away
//! (src/Ai/VsPoliceManager.cpp; constructor/reset/shutdown and the state table sit in the VsAIWorld
//! region, 0x38790..0x389C0).

use super::ai_context::AiContext;
use super::vs_ai_driver::{DriverRef, VsAIDriver};
use super::vs_ai_player_tft_manager::{TftPlayer, VsAIPlayerTFTManager};
use super::vs_ai_police_vehicle::{PoliceVehicleMode, VsAIPoliceVehicle};
use super::vs_ai_race_vehicle::{AiScene, RaceFrame, RespawnSource, VehicleRef};
use super::vs_ai_traffic_vehicle::{TrafficEvent, TrafficView};
use super::vs_ai_world::VsAIWorld;
use super::vs_math::*;
use super::vs_race_manager::{crossed_start, find_vehicle_ahead, find_vehicle_behind, line_side};

/// ePUS police states (`+0x18`); enter/update/exit tables at +0x1C/+0x2C/+0x3C (0x000389C0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum PoliceState {
    /// everyone on traffic duty
    #[default]
    Patrol = 0,
    /// one car chasing
    Alerted = 1,
    /// wanted level 2: two cars may chase
    Chase = 2,
    /// a pursuit race: chasing from the start, no traffic spawns
    Pursuit = 3,
}

/// everything a police update reaches besides the manager.
pub struct PoliceEnv<'a, 'c> {
    pub ctx: &'a mut AiContext<'c>,
    pub world: &'a mut VsAIWorld,
    pub tft: &'a mut VsAIPlayerTFTManager,
    pub tft_out: &'a mut dyn TftPlayer,
    pub speech_enabled: bool,
    pub num_players: u32,
}

impl<'c> PoliceEnv<'_, 'c> {
    fn frame<'s>(&'s mut self, scene: &'s AiScene) -> RaceFrame<'s, 'c> {
        let time = self.world.time;
        RaceFrame {
            ctx: &mut *self.ctx,
            world: &mut *self.world,
            scene,
            tft: &mut *self.tft,
            tft_out: &mut *self.tft_out,
            speech_enabled: self.speech_enabled,
            num_players: self.num_players,
            time,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct VsPoliceManager {
    pub vehicles: Vec<VsAIPoliceVehicle>, // +0x04 count (2), +0x10 array
    pub drivers: Vec<VsAIDriver>,         // +0x14
    pub num_active: u32,                  // +0x08
    pub max_active: u32,                  // +0x0C
    pub state: PoliceState,               // +0x18
    pub first_id: u32,                    // +0x4C
    pub target: VehicleRef,               // +0x50 the car being chased
    pub alerted: Option<usize>,           // +0x54 the car that saw it first
    pub wanted: f32,                      // +0x58 wanted level
    pub spawn_interval: f32,              // +0x5C seconds between traffic spawns of police cars
    pub last_spawn: f32,                  // +0x60
    pub allow_spawn: bool,                // +0x64 the traffic manager may put police cars on the road
}

impl VsPoliceManager {
    /// VsPoliceManager.cpp 0x00038790
    pub fn new(ctx: &mut AiContext, world: &mut VsAIWorld, tft: &VsAIPlayerTFTManager, first_id: u32) -> Self {
        let mut m = Self { first_id, ..Default::default() };
        m.create_vehicles(ctx, world);
        m.create_drivers(tft);
        m.bind_drivers();
        m.reset(ctx, world, tft);
        m
    }

    /// VsPoliceManager.cpp 0x00038A50
    fn create_vehicles(&mut self, ctx: &mut AiContext, world: &VsAIWorld) {
        let n = 2u32;
        debug_assert!(n > 0, "m_dNumVehicles > 0");
        let g = world.globals.clone();
        let mut id = self.first_id;
        for _ in 0..n {
            let mut v = VsAIPoliceVehicle::new(id, ctx, &g);
            v.traffic.is_traffic = true;
            v.traffic.behaviour = 1;
            v.traffic.police = true;
            v.race.is_traffic = false;
            v.race.flags_230 |= 2;
            self.vehicles.push(v);
            id += 1;
        }
    }

    /// VsPoliceManager.cpp 0x00038C10
    fn create_drivers(&mut self, tft: &VsAIPlayerTFTManager) {
        self.drivers = vec![VsAIDriver::default(); self.vehicles.len()];
        for d in self.drivers.iter_mut() {
            d.init(tft);
        }
    }

    /// VsPoliceManager.cpp 0x00038D30
    fn bind_drivers(&mut self) {
        for i in 0..self.vehicles.len() {
            self.vehicles[i].race.driver = Some(DriverRef::Police(i));
            self.drivers[i].vehicle = VehicleRef::Police(i);
        }
    }

    /// VsPoliceManager.cpp 0x00038820
    pub fn reset(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, tft: &VsAIPlayerTFTManager) {
        let g = world.globals.clone();
        for i in 0..self.vehicles.len() {
            self.vehicles[i].reset(ctx, &g);
            self.drivers[i].reset(tft);
        }
        self.remove_all(ctx, world);
        self.set_state(ctx, world, PoliceState::Patrol);
        self.last_spawn = 0.0;
        self.wanted = 0.0;
    }

    pub fn views(&self) -> Vec<super::vs_ai_race_vehicle::AiVehicleView> {
        self.vehicles.iter().enumerate().map(|(i, v)| v.view(i)).collect()
    }
    pub fn traffic_views(&self) -> Vec<TrafficView> {
        self.vehicles.iter().map(|v| v.traffic_view()).collect()
    }

    /// a pursuit race: the first cars chase the player from the grid.
    /// VsPoliceManager.cpp 0x00038DC0
    pub fn setup_pursuit(&mut self, env: &mut PoliceEnv, scene: &mut AiScene) {
        debug_assert!(env.ctx.event.mode == 6, "g_FEGameVariables.m_pEvent->GetCurrentRace()->m_eRaceType == eRACETYPE_PURSUIT");
        debug_assert!(!env.ctx.game.network, "!g_FEGameVariables.mbNetworkRace");
        let num_players = scene.players.len();
        debug_assert!(num_players == 1, "dNumPlayers == 1");
        let mut n = (env.ctx.game.grid.len() as u32).wrapping_sub(1);
        if n > 2 {
            n = 2;
        }
        self.target = VehicleRef::Player(0);
        for i in 0..n as usize {
            {
                let v = &mut self.vehicles[i];
                v.race.start_orientation = env.ctx.game.grid_heading;
                v.race.start_pos = env.ctx.game.grid.get(i + num_players).copied().unwrap_or_default();
            }
            {
                let mut f = env.frame(scene);
                self.vehicles[i].race.reset_to_grid(&mut f, Some(&mut self.drivers[i]), 40.0);
            }
            self.vehicles[i].race.police_mode = if i != 0 { 1 } else { 0 };
            {
                let t = self.target;
                let mut f = env.frame(scene);
                self.vehicles[i].start_chase(&mut f, Some(&mut self.drivers[i]), t);
            }
            scene.police[i] = self.vehicles[i].view(i);
            self.num_active += 1;
        }
        self.set_state(env.ctx, env.world, PoliceState::Pursuit);
    }

    /// VsPoliceManager.cpp 0x00039040
    pub fn remove_all(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld) {
        for i in 0..self.vehicles.len() {
            self.remove(ctx, world, i);
        }
        self.num_active = 0;
    }

    /// VsPoliceManager.cpp 0x000390B0
    pub fn remove(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, i: usize) {
        if self.vehicles[i].is_active() {
            self.vehicles[i].deactivate(ctx, world, Some(&mut self.drivers[i]));
            self.num_active = self.num_active.wrapping_sub(1);
        }
        let r = &mut self.vehicles[i].race;
        r.stuck = false;
        r.upside_down = false;
        r.grace_until = -1.0;
    }

    /// a police car joins the traffic at a spawn point.
    /// VsPoliceManager.cpp 0x00039190
    pub fn spawn_as_traffic(&mut self, ctx: &mut AiContext, i: usize, pos: &VsVector4, orientation: &VsVector4, velocity: &VsVector4) {
        let v = &mut self.vehicles[i].traffic;
        debug_assert!(!v.base.active, "pVehicle && !pVehicle->IsActive()");
        let h = v.base.physics.expect("m_pPhysicsVehicle");
        v.base.teleport(ctx.physics.vehicle(h), pos, orientation, velocity);
        ctx.physics.vehicle(h).set_control_mode(2);
        self.num_active += 1;
    }

    /// VsPoliceManager.cpp 0x00039260
    pub fn find_by_id(&self, id: u32) -> Option<usize> {
        self.vehicles.iter().position(|v| v.race.id == id)
    }

    /// VsPoliceManager.cpp 0x00039330
    pub fn update(&mut self, env: &mut PoliceEnv, scene: &mut AiScene, traffic: &[TrafficView], respawns: &mut dyn RespawnSource, dt: f32) {
        debug_assert!(self.num_active <= self.vehicles.len() as u32, "m_dNumActiveVehicles <= m_dNumVehicles");
        self.update_state(env, scene, dt);
        match self.state {
            PoliceState::Alerted => self.update_alerted(env, scene),
            PoliceState::Chase => self.update_alerted(env, scene),
            PoliceState::Patrol | PoliceState::Pursuit => {}
        }
        self.update_vehicles(env, scene, traffic, respawns, dt);
        // the "Police Mode / Max Active / Wanted Level" debug text is not ported
    }

    /// VsPoliceManager.cpp 0x00039600
    fn update_vehicles(&mut self, env: &mut PoliceEnv, scene: &mut AiScene, traffic: &[TrafficView], respawns: &mut dyn RespawnSource, dt: f32) {
        for i in 0..self.vehicles.len() {
            if !self.vehicles[i].is_active() {
                continue;
            }
            if self.vehicles[i].mode == PoliceVehicleMode::Race {
                self.track(env, scene, i);
            }
            let ev = {
                let mut f = env.frame(scene);
                self.vehicles[i].update(&mut f, Some(&mut self.drivers[i]), traffic, respawns, dt)
            };
            if let TrafficEvent::Despawned { .. } = ev {
                self.num_active = self.num_active.wrapping_sub(1);
            }
            scene.police[i] = self.vehicles[i].view(i);
        }
    }

    /// laps and neighbours of a chasing car.
    /// VsPoliceManager.cpp 0x000396B0
    fn track(&mut self, env: &mut PoliceEnv, scene: &mut AiScene, i: usize) {
        debug_assert!(self.vehicles[i].mode == PoliceVehicleMode::Race, "pVehicle && pVehicle->GetVehicleMode() == eVT_RaceVehicle");
        let pos = self.vehicles[i].race.base.position;
        if let Some((a, b)) = env.ctx.game.start_line {
            let r = &mut self.vehicles[i].race;
            if line_side(&a, &b, &pos) == 2 && r.flag_0fc {
                r.lap += 1;
                r.flag_0fc = false;
                r.reset_dist();
            }
        }
        if self.vehicles[i].race.lap == 0
            && let Some((a, b)) = env.ctx.game.start_line_original
            && crossed_start(&a, &b, &pos)
        {
            self.vehicles[i].race.lap = 1;
        }
        {
            let mut f = env.frame(scene);
            self.vehicles[i].race.update_state(&mut f, false);
        }
        scene.police[i] = self.vehicles[i].view(i);
        for j in 0..self.vehicles.len() {
            if self.vehicles[j].is_active() && self.vehicles[j].mode == PoliceVehicleMode::Race {
                let me = VehicleRef::Police(j);
                self.vehicles[j].race.vehicle_ahead = find_vehicle_ahead(scene, me);
                self.vehicles[j].race.vehicle_behind = find_vehicle_behind(scene, me);
            }
        }
    }

    /// state transitions, then the screen flash.
    /// VsPoliceManager.cpp 0x000398A0
    fn update_state(&mut self, env: &mut PoliceEnv, scene: &mut AiScene, dt: f32) {
        match self.state {
            PoliceState::Patrol => {
                if self.alert_by_proximity(scene) || self.alert_by_collision(scene) {
                    self.set_state_env(env, scene, PoliceState::Alerted);
                }
                self.flash(env.ctx, scene, None);
            }
            PoliceState::Alerted => {
                if self.lost_target(env.ctx, scene, 200.0) {
                    self.set_state_env(env, scene, PoliceState::Patrol);
                } else if self.wanted_level(scene, dt) >= 2.0 {
                    self.set_state_env(env, scene, PoliceState::Chase);
                }
                self.flash_nearest(env.ctx, scene);
            }
            PoliceState::Chase => {
                if self.lost_target(env.ctx, scene, 300.0) {
                    self.set_state_env(env, scene, PoliceState::Patrol);
                }
                self.flash_nearest(env.ctx, scene);
            }
            PoliceState::Pursuit => self.flash_nearest(env.ctx, scene),
        }
    }

    fn set_state_env(&mut self, env: &mut PoliceEnv, scene: &mut AiScene, new: PoliceState) {
        if new == PoliceState::Alerted {
            self.enter_alerted(env, scene);
            self.state = new;
        } else {
            self.set_state(env.ctx, env.world, new);
        }
    }

    /// VsPoliceManager.cpp 0x000399C0
    fn set_state(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, new: PoliceState) {
        // every exit handler is empty (0x3A450 / 0x3A6A0 / 0x3A840 / 0x3A930)
        self.state = new;
        match new {
            PoliceState::Patrol => self.enter_patrol(ctx, world),
            PoliceState::Alerted => debug_assert!(false, "alerting needs the race frame"),
            PoliceState::Chase => self.enter_chase(),
            PoliceState::Pursuit => self.enter_pursuit(),
        }
    }

    /// grows while a chasing car is within 5m of the target along the route.
    /// VsPoliceManager.cpp 0x00039B80
    fn wanted_level(&mut self, scene: &AiScene, dt: f32) -> f32 {
        let td = scene.get(self.target).map_or(0.0, |t| t.race_distance);
        let close = self.vehicles.iter().any(|v| {
            v.is_active() && v.mode == PoliceVehicleMode::Race && vs_abs(td - v.race.race_distance) < 5.0
        });
        if close {
            self.wanted += dt * f32::from_bits(0x3DCC_CCCD);
        }
        self.wanted
    }

    /// the screen flash follows the chasing car nearest the target along the route.
    /// VsPoliceManager.cpp 0x00039C80
    fn flash_nearest(&mut self, ctx: &mut AiContext, scene: &AiScene) {
        let td = scene.get(self.target).map_or(0.0, |t| t.race_distance);
        let mut best = f32::MAX;
        let mut found = None;
        for (i, v) in self.vehicles.iter().enumerate() {
            if v.is_active() && v.mode == PoliceVehicleMode::Race {
                let d = vs_abs(td - v.race.race_distance);
                if best > d {
                    best = d;
                    found = Some(i);
                }
            }
        }
        self.flash(ctx, scene, found);
    }

    /// a traffic police car within 20m of a player doing over 30 m/s.
    /// VsPoliceManager.cpp 0x00039D80
    fn alert_by_proximity(&mut self, scene: &AiScene) -> bool {
        let Some(p0) = scene.player0() else { return false };
        for i in 0..self.vehicles.len() {
            let v = &self.vehicles[i];
            if v.is_active() && v.mode == PoliceVehicleMode::Traffic && vs_distance_squared(&p0.position, &v.position()) < 400.0 && p0.speed > 30.0 {
                self.target = p0.r;
                self.alerted = Some(i);
                return true;
            }
        }
        false
    }

    /// a traffic police car the player has hit.
    /// VsPoliceManager.cpp 0x00039E70
    fn alert_by_collision(&mut self, scene: &AiScene) -> bool {
        let Some(p0) = scene.player0() else { return false };
        for i in 0..self.vehicles.len() {
            let v = &self.vehicles[i];
            if v.is_active() && v.mode == PoliceVehicleMode::Traffic && self.drivers[i].get_collision().0 == p0.r {
                self.target = p0.r;
                self.alerted = Some(i);
                return true;
            }
        }
        false
    }

    /// no chasing car within `range` along the route, behind the target or in view.
    /// VsPoliceManager.cpp 0x00039F80
    fn lost_target(&self, ctx: &AiContext, scene: &AiScene, range: f32) -> bool {
        debug_assert!(self.target != VehicleRef::None, "m_pTargetVehicle");
        // the original reads the target's render transform (forward +0x118, position +0xD4)
        let Some(t) = scene.get(self.target) else { return true };
        for v in &self.vehicles {
            if v.is_active() && v.mode == PoliceVehicleMode::Race {
                if range > vs_abs(t.race_distance - v.race.race_distance) {
                    return false;
                }
                let d = v.race.base.position - t.position;
                if t.forward.dot3(&d) > 0.0 {
                    return false;
                }
                if v.race.base.physics.is_some_and(|h| ctx.host.in_camera_view(h)) {
                    return false;
                }
            }
        }
        true
    }

    /// the red/blue screen-edge flash of the chasing car, fading out by 64m from the player.
    /// VsPoliceManager.cpp 0x0003A1C0
    fn flash(&self, ctx: &mut AiContext, scene: &AiScene, v: Option<usize>) {
        let Some(i) = v else {
            ctx.host.police_flash(&VsVector4::new(0.0, 0.0, 0.0, 0.0));
            return;
        };
        let mut c = VsVector4::new(0.0, 0.0, 0.0, 0.0);
        let t = self.vehicles[i].siren_time;
        if t <= 0.5 {
            c.z = vs_sin(6.283_185_5 * t);
        } else {
            c.x = vs_sin((t - 0.5) * 6.283_185_5);
        }
        let p0 = scene.player0().map_or(VsVector4::default(), |p| p.position);
        let d2 = (self.vehicles[i].race.base.position - p0).length_squared();
        let fade = f32::from_bits(0x4280_0000);
        let mut a = f32::from_bits(0x3F00_0000) - d2 / (fade * fade);
        if a < 0.0 {
            a = 0.0;
        }
        c.w = a;
        ctx.host.police_flash(&c);
    }

    /// VsPoliceManager.cpp 0x0003A360
    fn enter_patrol(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld) {
        for i in 0..self.vehicles.len() {
            if self.vehicles[i].is_active() && self.vehicles[i].mode == PoliceVehicleMode::Race {
                self.remove(ctx, world, i);
            }
        }
        self.target = VehicleRef::None;
        self.alerted = None;
        self.spawn_interval = 30.0;
        self.max_active = 1;
        self.allow_spawn = true;
        self.wanted = 0.0;
    }

    /// the car that noticed the target starts the chase.
    /// VsPoliceManager.cpp 0x0003A470
    fn enter_alerted(&mut self, env: &mut PoliceEnv, scene: &mut AiScene) {
        let a = self.alerted.expect("m_pTargetVehicle != NULL && m_pAlertedVehicle != NULL");
        debug_assert!(self.target != VehicleRef::None);
        let t = self.target;
        {
            let mut f = env.frame(scene);
            self.vehicles[a].start_chase(&mut f, Some(&mut self.drivers[a]), t);
        }
        scene.police[a] = self.vehicles[a].view(a);
        self.vehicles[a].race.police_mode = 0;
        self.spawn_interval = 14.0;
        self.max_active = 1;
        self.allow_spawn = true;
        self.wanted = 1.0;
        if let Some(h) = self.vehicles[a].race.base.physics {
            env.ctx.host.camera_point_of_interest(h, 0.5);
        }
    }

    /// other police traffic within 20m of the target joins in as support.
    /// VsPoliceManager.cpp 0x0003A580 (Alerted) / 0x0003A770 (Chase)
    fn update_alerted(&mut self, env: &mut PoliceEnv, scene: &mut AiScene) {
        debug_assert!(self.target != VehicleRef::None, "m_pTargetVehicle");
        let tp = scene.get(self.target).map_or(VsVector4::default(), |t| t.position);
        for i in 0..self.vehicles.len() {
            let v = &self.vehicles[i];
            if v.is_active() && v.mode == PoliceVehicleMode::Traffic && vs_distance_squared(&tp, &v.position()) < 400.0 {
                let t = self.target;
                {
                    let mut f = env.frame(scene);
                    self.vehicles[i].start_chase(&mut f, Some(&mut self.drivers[i]), t);
                }
                scene.police[i] = self.vehicles[i].view(i);
                self.vehicles[i].race.police_mode = 1;
            }
        }
    }

    /// VsPoliceManager.cpp 0x0003A6C0
    fn enter_chase(&mut self) {
        debug_assert!(self.target != VehicleRef::None, "m_pTargetVehicle != NULL");
        self.spawn_interval = 14.0;
        self.max_active = 2;
        self.allow_spawn = true;
    }

    /// VsPoliceManager.cpp 0x0003A860
    fn enter_pursuit(&mut self) {
        debug_assert!(self.target != VehicleRef::None, "m_pTargetVehicle != NULL");
        self.spawn_interval = 14.0;
        self.max_active = 2;
        self.allow_spawn = false;
    }
}
