//! A police car: one physics car driven either as a race car chasing its target or as an innocent
//! looking traffic car (src/Ai/VsAIPoliceVehicle.cpp; the constructor, reset and shutdown sit in the
//! VsAIPlayerTFTManager region, 0x17420..0x17530).

use super::vs_ai_driver::VsAIDriver;
use super::vs_ai_race_vehicle::{AiVehicleView, RaceFrame, RaceState, RespawnSource, VehicleRef, VsAIRaceVehicle};
use super::vs_ai_traffic_vehicle::{TrafficEvent, TrafficFrame, TrafficView, VsAITrafficVehicle};
use super::vs_math::VsVector4;

/// `+0x08` eVT vehicle mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum PoliceVehicleMode {
    /// chasing as a race car
    Race = 0,
    /// driving as traffic
    #[default]
    Traffic = 1,
    /// parked (never used by the code)
    Idle = 2,
}

#[derive(Debug, Clone, Default)]
pub struct VsAIPoliceVehicle {
    pub race: VsAIRaceVehicle,       // +0x00 (984 bytes)
    pub traffic: VsAITrafficVehicle, // +0x04 (332 bytes)
    pub mode: PoliceVehicleMode,     // +0x08
    pub siren: bool,                 // +0x0C
    pub siren_time: f32,             // +0x10 0..1 flash phase
}

/// the two siren lights: (light, intensity, alpha, r, g, b) on and off.
const LIGHTS_ON: [(u32, u32, u8, u8, u8, u8); 2] = [(0, 0x3E4C_CCCD, 128, 0, 112, 255), (1, 0x3E4C_CCCD, 128, 255, 0, 0)];
const LIGHTS_OFF: [(u32, u32, u8, u8, u8, u8); 2] = [(0, 0x3E4C_CCCD, 128, 0, 112, 15), (1, 0x3E4C_CCCD, 128, 255, 0, 0)];

impl VsAIPoliceVehicle {
    /// VsAIPoliceVehicle.cpp 0x00017420
    pub fn new(id: u32, f: &mut super::ai_context::AiContext, g: &super::vs_ai_globals::VsAIGlobals) -> Self {
        let mut v = Self::default();
        v.race.init(id, g, f.event, f.rand, f.game.player_prestige, false);
        v.traffic.init(id);
        v.reset(f, g);
        v
    }

    /// VsAIPoliceVehicle.cpp 0x000174E0
    pub fn reset(&mut self, f: &mut super::ai_context::AiContext, g: &super::vs_ai_globals::VsAIGlobals) {
        self.race.reset(g, f.event, f.rand, f.game.player_prestige);
        self.traffic.reset();
        self.mode = PoliceVehicleMode::Traffic;
        self.siren = false;
    }

    /// on the road in either mode.
    /// VsAIPoliceVehicle.cpp 0x00017740
    pub fn is_active(&self) -> bool {
        self.race.base.active || self.traffic.base.active
    }

    /// both parts drive the same physics car.
    /// VsAIPoliceVehicle.cpp 0x000177E0
    pub fn set_physics_vehicle(&mut self, h: usize) {
        self.race.base.set_physics_vehicle(h);
        self.traffic.base.set_physics_vehicle(h);
    }

    /// VsAIPoliceVehicle.cpp 0x00017880
    pub fn position(&self) -> VsVector4 {
        if self.mode == PoliceVehicleMode::Traffic { self.traffic.base.position } else { self.race.base.position }
    }

    /// as other cars see it.
    pub fn view(&self, i: usize) -> AiVehicleView {
        let mut v = if self.mode == PoliceVehicleMode::Race { self.race.view(VehicleRef::Police(i)) } else { self.traffic.race_view(i) };
        v.r = VehicleRef::Police(i);
        v.is_police = true;
        v
    }
    pub fn traffic_view(&self) -> TrafficView {
        self.traffic.view()
    }

    /// siren lights on or off; restarts the flash phase.
    /// VsAIPoliceVehicle.cpp 0x000179B0
    fn set_siren(&mut self, on: bool, host: &mut dyn super::ai_context::AiHost) {
        self.siren = on;
        let lights = if on { LIGHTS_ON } else { LIGHTS_OFF };
        for (light, intensity, a, r, g, b) in lights {
            host.police_light(self.race.id, light, on, f32::from_bits(intensity), a, r, g, b);
        }
        self.siren_time = 0.0;
    }

    /// switch to race mode and chase `target` from where the car is.
    /// VsAIPoliceVehicle.cpp 0x00017A60
    pub fn start_chase(&mut self, f: &mut RaceFrame, drv: Option<&mut VsAIDriver>, target: VehicleRef) {
        debug_assert!(target != VehicleRef::None, "pVehicle != NULL");
        self.mode = PoliceVehicleMode::Race;
        self.race.chase_target = target;
        let t = f.scene.get(target).cloned().unwrap_or_default();
        self.race.race_section = t.race_section;
        self.race.set_state(f, drv, RaceState::Chase);
        self.race.update_state(f, true);
        self.race.lap = t.lap;
        self.race.flag_0fc = true;
        self.race.find_next_sub_route_point(f.world);
        if let Some(h) = self.race.base.physics {
            f.ctx.physics.vehicle(h).set_control_mode(1);
        }
        self.set_siren(true, f.ctx.host);
    }

    /// back to traffic duty.
    /// VsAIPoliceVehicle.cpp 0x00017BA0
    pub fn to_traffic(&mut self, ctx: &mut super::ai_context::AiContext) {
        self.mode = PoliceVehicleMode::Traffic;
        self.traffic.is_traffic = true;
        if let Some(h) = self.race.base.physics {
            ctx.physics.vehicle(h).set_control_mode(2);
        }
        self.set_siren(false, ctx.host);
    }

    /// off the road in both modes.
    /// VsAIPoliceVehicle.cpp 0x00017790
    pub fn deactivate(&mut self, ctx: &mut super::ai_context::AiContext, world: &mut super::vs_ai_world::VsAIWorld, drv: Option<&mut VsAIDriver>) {
        self.traffic.deactivate(world, None);
        match self.race.base.physics {
            Some(h) => self.race.base.deactivate(Some(ctx.physics.vehicle(h))),
            None => self.race.base.deactivate(None),
        }
        if let Some(d) = drv {
            let g = world.globals.clone();
            d.on_collision(VehicleRef::None, false, 0.0, &g, world.time);
        }
        self.to_traffic(ctx);
    }

    /// VsAIPoliceVehicle.cpp 0x000175F0
    pub fn update(
        &mut self,
        f: &mut RaceFrame,
        drv: Option<&mut VsAIDriver>,
        traffic: &[TrafficView],
        respawns: &mut dyn RespawnSource,
        dt: f32,
    ) -> TrafficEvent {
        let mut drv = drv;
        if f.ctx.event.mode == 6
            && f.scene.player0().is_some_and(|p| p.started)
            && self.race.state == RaceState::PlayerAssist
        {
            self.race.set_state(f, drv.as_deref_mut(), RaceState::Chase);
        }
        let mut ev = TrafficEvent::None;
        match self.mode {
            PoliceVehicleMode::Race => self.race.update(f, drv, respawns, dt),
            PoliceVehicleMode::Traffic => {
                let mut tf = TrafficFrame { ctx: &mut *f.ctx, world: &mut *f.world, scene: f.scene, traffic };
                ev = self.traffic.update(&mut tf);
            }
            PoliceVehicleMode::Idle => {}
        }
        if self.siren {
            self.siren_time += dt;
            if self.siren_time > 1.0 {
                self.siren_time = 0.0;
            }
        }
        ev
    }
}
