//! The `VsAIWorld` singleton with the managers it owns (`+0x47E0` traffic, `+0x47E4` race, `+0x47E8`
//! police) and its life cycle: Init, LoadWorldData, Reset, Update, Shutdown, the collision message
//! handler (src/Ai/VsAIWorld.cpp). The data side of VsAIWorld is `vs_ai_world::VsAIWorld`.

use std::path::Path;

use super::ai_context::{AiContext, AiHost};
use super::armco::Armco;
use super::vs_ai_globals::VsAIGlobals;
use super::vs_ai_player_tft_manager::{TftPlayer, VsAIPlayerTFTManager};
use super::vs_ai_race_vehicle::{AiVehicleView, RaceFrame, RespawnSource, VehicleRef, FAILURE_NONE};
use super::vs_ai_traffic_vehicle::TrafficView;
use super::vs_ai_world::VsAIWorld;
use super::vs_math::*;
use super::vs_police_manager::{PoliceEnv, VsPoliceManager};
use super::vs_race_manager::{RaceSetup, VsRaceManager};
use super::vs_traffic_manager::{VsTrafficManager, MAX_TRAFFIC_VEHICLES};

/// the switches `VsAIWorld::Init` takes (GameManager.cpp 0x000B5F80 "Ai World").
#[derive(Debug, Clone, Copy, Default)]
pub struct AiWorldFlags {
    /// traffic option and not a network race
    pub traffic: bool,
    /// `byte_6F70D8`: not split screen
    pub single_player: bool,
    /// (traffic or a pursuit) and not a drag race
    pub police: bool,
    /// barrier splines (`!professional`)
    pub barriers: bool,
    /// armco walls (`professional`)
    pub armco: bool,
}

impl AiWorldFlags {
    /// the flags GameManager derives from the race selection and the event.
    pub fn for_race(traffic_option: bool, network: bool, splitscreen: bool, professional: bool, race_type: u32) -> Self {
        let traffic = traffic_option && !network;
        Self {
            traffic,
            single_player: !splitscreen,
            police: (traffic || race_type == 6) && race_type != 5,
            barriers: !professional,
            armco: professional,
        }
    }
}

#[derive(Debug, Default)]
pub struct AiWorld {
    pub world: VsAIWorld,
    pub race: VsRaceManager,
    pub traffic: Option<VsTrafficManager>,
    pub police: Option<VsPoliceManager>,
    /// the armco walls (Armco.cpp statics 0x006D2440..0x006D2464)
    pub armco: Option<Armco>,
}

impl AiWorld {
    /// VsAIWorld.cpp 0x0002C960
    pub fn init(ai_dir: impl Into<std::path::PathBuf>, ctx: &mut AiContext, flags: AiWorldFlags, setup: &RaceSetup) -> Self {
        Self::init_with(ai_dir, ctx, flags, setup, |_| {})
    }

    /// `init` with the statics (tweak block 0x006D3698, TFT manager 0x006D26E0) carried over from earlier races:
    /// `statics` runs on the fresh world before anything reads them.
    pub fn init_with(
        ai_dir: impl Into<std::path::PathBuf>,
        ctx: &mut AiContext,
        flags: AiWorldFlags,
        setup: &RaceSetup,
        statics: impl FnOnce(&mut AiWorld),
    ) -> Self {
        let mut w = VsAIWorld::new(ai_dir);
        debug_assert!(!w.initialised, "!m_bInitialised");
        w.traffic_enabled = flags.traffic;
        w.flag_4767 = flags.single_player;
        w.police_enabled = flags.police;
        w.barriers_enabled = flags.barriers;
        w.armco_enabled = flags.armco;
        w.time_step = 0.0;
        w.time = 0.0;
        let mut a = AiWorld { world: w, ..Default::default() };
        statics(&mut a);
        // ids: players first, then the opponents, 14 traffic cars, the police
        let mut id = setup.num_players;
        a.race.init(ctx, &a.world, setup, &mut id);
        let mut id = ctx.event.num_opponents as u32;
        if flags.traffic {
            a.traffic = Some(VsTrafficManager::new(&mut a.world, id, ctx.host));
        }
        id += MAX_TRAFFIC_VEHICLES;
        if flags.police {
            let tft = a.race.tft.clone();
            a.police = Some(VsPoliceManager::new(ctx, &mut a.world, &tft, id));
        }
        // the collision message listeners are `on_collision`
        a.world.initialised = true;
        a
    }

    /// VsAIWorld.cpp 0x0002CC90
    pub fn reset(&mut self, ctx: &mut AiContext) {
        debug_assert!(self.world.initialised, "m_bInitialised");
        self.world.time = 0.0;
        self.race.reset(ctx, &self.world);
        if let Some(t) = self.traffic.as_mut() {
            t.reset(&mut self.world, ctx.host);
        }
        if let Some(p) = self.police.as_mut() {
            let tft = self.race.tft.clone();
            p.reset(ctx, &mut self.world, &tft);
        }
        self.world.globals.force_failure = FAILURE_NONE;
    }

    /// VsAIWorld.cpp 0x0002CD90
    pub fn shutdown(&mut self) {
        if !self.world.initialised {
            return;
        }
        self.world.initialised = false;
        self.traffic = None;
        self.police = None;
        self.armco = None;
        self.world.spline = None;
        self.world.barriers = [None, None];
        self.world.handlings.clear();
        self.world.response_curves.clear();
        self.race = VsRaceManager::default();
    }

    /// the race's AI data: splines, barriers or armco, handling, rev curves.
    /// VsAIWorld.cpp 0x000301F0
    pub fn load_world_data(&mut self, level: &Path, start_line: Option<(VsVector4, VsVector4)>) -> bool {
        log::info!("** AI - Loading World Data");
        self.world.level_path = level.to_string_lossy().into_owned();
        self.world.load_spline_data(level);
        self.world.process_spline_data(start_line);
        if self.world.barriers_enabled {
            self.world.load_barrier_data(level);
        }
        if self.world.is_armco_enabled() {
            self.armco = Armco::load(level, "armco01.ar");
        }
        if self.world.get_num_race_sections() != 0 {
            self.race.set_start_section(&self.world, 0);
        }
        self.world.load_race_vehicle_handling(&mut self.race);
        self.world.setup_startline_revs(&mut self.race);
        if let Some(t) = self.traffic.as_mut() {
            self.world.load_traffic_vehicle_handling(t);
        }
        if let Some(p) = self.police.as_mut() {
            self.world.load_police_vehicle_handling(p);
        }
        // AddTweaks (0x0002D360) registers the debug menu entries only
        log::info!("** AI - Loading Complete");
        true
    }

    /// blockers find their physics objects by id once the level objects exist.
    /// VsAIWorld.cpp 0x00031280
    pub fn link_traffic_blocker_objects(&mut self, find: impl Fn(u32) -> Option<u32>, host: &mut dyn AiHost) {
        let Some(s) = self.world.spline.as_mut() else { return };
        for b in s.traffic_blockers.iter_mut() {
            b.physics_object = find(b.physics_object_id);
            if let Some(o) = b.physics_object
                && !b.active
            {
                host.traffic_blocker_object(o, true);
            }
        }
    }

    pub fn traffic_views(&self) -> (Vec<AiVehicleView>, Vec<TrafficView>) {
        match self.traffic.as_ref() {
            Some(t) => (t.vehicles.iter().enumerate().map(|(i, v)| v.race_view(i)).collect(), t.views()),
            None => (Vec::new(), Vec::new()),
        }
    }
    pub fn police_views(&self) -> Vec<AiVehicleView> {
        self.police.as_ref().map_or(Vec::new(), |p| p.views())
    }

    /// VsAIWorld.cpp 0x00034280
    pub fn update(&mut self, ctx: &mut AiContext, tft_out: &mut dyn TftPlayer, respawns: &mut dyn RespawnSource, dt: f32) {
        // the debug vehicle's handling is copied into the tweak globals here (debug menu only)
        self.world.time_step = dt;
        self.world.time += dt;
        if self.world.get_num_race_sections() != 0 {
            let (tv, _) = self.traffic_views();
            let pv = self.police_views();
            self.race.update(ctx, &mut self.world, &tv, &pv, tft_out, respawns, dt);
        }
        if self.world.traffic_enabled
            && self.world.get_num_traffic_sections() != 0
            && let Some(t) = self.traffic.as_mut()
        {
            let pv = self.police.as_ref().map_or(Vec::new(), |p| p.views());
            let tv: Vec<AiVehicleView> = t.vehicles.iter().enumerate().map(|(i, v)| v.race_view(i)).collect();
            let scene = self.race.build_scene(&tv, &pv);
            t.update(ctx, &mut self.world, &scene, self.police.as_mut());
        }
        if self.world.police_enabled
            && let Some(p) = self.police.as_mut()
        {
            let (tv, tviews) = match self.traffic.as_ref() {
                Some(t) => (t.vehicles.iter().enumerate().map(|(i, v)| v.race_view(i)).collect(), t.views()),
                None => (Vec::new(), Vec::new()),
            };
            let pv = p.views();
            let mut scene = self.race.build_scene(&tv, &pv);
            let speech = ctx.game.speech_enabled;
            let num_players = self.race.players.len() as u32;
            let mut env = PoliceEnv {
                ctx,
                world: &mut self.world,
                tft: &mut self.race.tft,
                tft_out,
                speech_enabled: speech,
                num_players,
            };
            p.update(&mut env, &mut scene, &tviews, respawns, dt);
        }
    }

    /// grid reset at the start of a race; a pursuit also sends the police after the player.
    /// VsRaceManager.cpp 0x0003C5D0 + VsPoliceManager.cpp 0x00038DC0 (GameManager.cpp 0x000BC5F0)
    pub fn reset_to_grid(&mut self, ctx: &mut AiContext, tft_out: &mut dyn TftPlayer, pursuit: bool) {
        self.race.reset_to_grid(ctx, &mut self.world, tft_out, pursuit);
        if pursuit && let Some(p) = self.police.as_mut() {
            let (tv, _) = match self.traffic.as_ref() {
                Some(t) => (t.vehicles.iter().enumerate().map(|(i, v)| v.race_view(i)).collect::<Vec<_>>(), ()),
                None => (Vec::new(), ()),
            };
            let pv = p.views();
            let mut scene = self.race.build_scene(&tv, &pv);
            let speech = ctx.game.speech_enabled;
            let num_players = self.race.players.len() as u32;
            let mut env = PoliceEnv { ctx, world: &mut self.world, tft: &mut self.race.tft, tft_out, speech_enabled: speech, num_players };
            p.setup_pursuit(&mut env, &mut scene);
        }
    }

    /// drag round over with the player qualified (GameManager.cpp 0x000BC5F0 state 5): the car GameManager +0x14C
    /// keeps, VsRaceManager 0x0003DB70 (None when that car is human: the ELIMINATED branch).
    pub fn drag_eliminated_car(&self) -> Option<VehicleRef> {
        self.race.last_active_vehicle().filter(|r| matches!(r, VehicleRef::Ai(_)))
    }

    /// GameManager::RestartRace 0x000BE0F0: the +0x14C car leaves the drag (VsAIVehicle 0x0002C0F0).
    pub fn drag_deactivate(&mut self, ctx: &mut AiContext, eliminated: VehicleRef) {
        let Some(v) = self.race.vehicle_mut(eliminated) else { return };
        match v.base.physics {
            Some(h) => v.base.deactivate(Some(ctx.physics.vehicle(h))),
            None => v.base.deactivate(None),
        }
        log::info!("AI: drag car {} eliminated", v.id);
    }

    /// GameManager.cpp 0x000BF270: every inactive ai racer comes back (VsAIVehicle 0x0002BDA0 at VS_ZERO, identity
    /// orientation 0x002B6E50, no velocity) before RestartRace; the grid reset then places it.
    pub fn drag_reactivate(&mut self, ctx: &mut AiContext) {
        let identity = VsVector4::new(0.0, 0.0, 0.0, 1.0);
        for v in self.race.ai.iter_mut() {
            // the original's cars all exist; unbound ones (before the first bind) have no physics car yet
            if let (false, Some(h)) = (v.base.active, v.base.physics) {
                v.base.teleport(ctx.physics.vehicle(h), &VS_ZERO, &identity, &VS_ZERO);
            }
        }
    }

    /// F3EventEnableRaceSection: `+0x25` of the race section whose raceline id is `raceline` (0x00177B20).
    /// TimelineAI.cpp 0x00176B50
    pub fn enable_race_section(&mut self, raceline: u32, enable: bool) {
        match self.world.race_sections_mut().iter_mut().find(|s| s.raceline.get_id() == raceline) {
            Some(s) => s.flag_25 = enable,
            None => log::warn!("Race section {raceline} not found"),
        }
    }

    fn racer(&mut self, r: VehicleRef) -> Option<&mut super::vs_ai_race_vehicle::VsAIRaceVehicle> {
        match r {
            VehicleRef::Police(i) => self.police.as_mut().map(|p| &mut p.vehicles[i].race),
            _ => self.race.vehicle_mut(r),
        }
    }

    /// a physics collision between cars `a` and `b` (ids, -1 for scenery): both take damage and their
    /// drivers learn who hit whom.
    /// VsAIDriver.cpp 0x000160B0 (the world's message handler)
    pub fn on_collision(&mut self, ctx: &mut AiContext, tft_out: &mut dyn TftPlayer, a: i32, b: i32, impact: f32) {
        // every race car's score manager listens too, for messages naming it first (ReplayCamera_Wheel 0x00169240)
        if a >= 0 {
            let mut r = self.race.find_vehicle_by_id(a as u32);
            if r == VehicleRef::None {
                r = self.police.as_ref().and_then(|p| p.find_by_id(a as u32)).map_or(VehicleRef::None, VehicleRef::Police);
            }
            if let Some(s) = self.racer(r).and_then(|v| v.score.as_mut()) {
                s.on_collision();
            }
        }
        self.damage(ctx, tft_out, a, b, impact);
        self.damage(ctx, tft_out, b, a, impact);
        if a == -1 || b == -1 {
            return;
        }
        let mut va = self.race.find_vehicle_by_id(a as u32);
        let mut vb = self.race.find_vehicle_by_id(b as u32);
        if self.world.police_enabled {
            if va == VehicleRef::None {
                va = self.police.as_ref().and_then(|p| p.find_by_id(a as u32)).map_or(VehicleRef::None, VehicleRef::Police);
            }
            if vb == VehicleRef::None {
                vb = self.police.as_ref().and_then(|p| p.find_by_id(b as u32)).map_or(VehicleRef::None, VehicleRef::Police);
            }
        }
        if va == VehicleRef::None || vb == VehicleRef::None {
            return;
        }
        let (pa, sa, ia, vela) = {
            let v = self.racer(va).unwrap();
            (v.base.position, v.overtake_side, v.is_player, v.base.velocity)
        };
        let (pb, sb, ib, velb) = {
            let v = self.racer(vb).unwrap();
            (v.base.position, v.overtake_side, v.is_player, v.base.velocity)
        };
        // a_rammed: `a` ran into `b`
        let a_rammed = if ia || sa != 3 {
            if ib || sb != 3 {
                let mut d = pb - pa;
                d.normalise();
                let mut va_dir = vela;
                va_dir.normalise();
                let da = vs_dot3(&va_dir, &d);
                d *= -1.0;
                let mut vb_dir = velb;
                vb_dir.normalise();
                let db = vs_dot3(&vb_dir, &d);
                da > db
            } else {
                false
            }
        } else {
            true
        };
        let g: VsAIGlobals = self.world.globals.clone();
        let time = self.world.time;
        if let Some(d) = self.driver_of(va) {
            d.on_collision(vb, !a_rammed, impact, &g, time);
        }
        if let Some(d) = self.driver_of(vb) {
            d.on_collision(va, a_rammed, impact, &g, time);
        }
    }

    fn driver_of(&mut self, r: VehicleRef) -> Option<&mut super::vs_ai_driver::VsAIDriver> {
        use super::vs_ai_driver::DriverRef;
        let d = self.racer(r)?.driver?;
        match d {
            DriverRef::Ai(i) => self.race.drivers.get_mut(i),
            DriverRef::Player(i) => self.race.player_drivers.get_mut(i),
            DriverRef::Police(i) => self.police.as_mut().and_then(|p| p.drivers.get_mut(i)),
        }
    }

    /// collision damage to race car `id`.
    /// VsAIDriver.cpp 0x00016390
    fn damage(&mut self, ctx: &mut AiContext, tft_out: &mut dyn TftPlayer, id: i32, other: i32, impact: f32) {
        if id == -1 {
            return;
        }
        let r = self.race.find_vehicle_by_id(id as u32);
        if self.race.vehicle(r).is_none() {
            return;
        }
        // original bug: the "hit by the player" scale looks this same car up again, so never applies
        let _ = other;
        let (tv, _) = self.traffic_views();
        let pv = self.police_views();
        let scene = self.race.build_scene(&tv, &pv);
        let time = self.world.time;
        let speech = ctx.game.speech_enabled;
        let np = self.race.players.len() as u32;
        let mut tft: VsAIPlayerTFTManager = std::mem::take(&mut self.race.tft);
        {
            let (v, d) = match r {
                VehicleRef::Ai(i) => (&mut self.race.ai[i], self.race.drivers.get_mut(i)),
                VehicleRef::Player(i) => (&mut self.race.players[i], self.race.player_drivers.get_mut(i)),
                _ => unreachable!(),
            };
            let mut f = RaceFrame { ctx, world: &mut self.world, scene: &scene, tft: &mut tft, tft_out, speech_enabled: speech, num_players: np, time };
            v.apply_damage(&mut f, d, impact, 6);
        }
        self.race.tft = tft;
    }
}

impl VsAIWorld {
    /// police cars: race part 350Z, traffic part TRAFFIC.
    /// VsAIWorld.cpp 0x00032B40
    pub fn load_police_vehicle_handling(&mut self, pm: &mut VsPoliceManager) -> bool {
        log::info!("-- Loading Police Vehicle Handling");
        for v in pm.vehicles.iter_mut() {
            v.race.base.handling = self.load_handling("350Z");
            v.traffic.base.handling = self.load_handling("TRAFFIC");
            if let Some(h) = v.traffic.base.handling {
                self.place_handling_curves(h);
            }
        }
        true
    }
}
