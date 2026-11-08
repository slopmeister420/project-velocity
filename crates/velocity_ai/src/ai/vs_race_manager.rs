//! The race: ai and player race vehicles, their drivers, lap counting and race order
//! (src/Ai/VsRaceManager.cpp, VsRaceManager.h).

use super::ai_context::AiContext;
use super::vs_ai_driver::{DriverFrame, DriverRef, VsAIDriver};
use super::vs_ai_player_tft_manager::{TftPlayer, VsAIPlayerTFTManager, tft};
use super::vs_ai_race_vehicle::*;
use super::vs_ai_tools::VsAITools;
use super::vs_ai_world::VsAIWorld;
use super::vs_math::{vs_min, VsVector4};

/// MAX_RACE_VEHICLES
pub const MAX_RACE_VEHICLES: u32 = 6;

#[derive(Debug, Clone, Default)]
pub struct VsRaceManager {
    pub player_positions: Vec<u32>,     // +0x00 RaceInfo.dPlayerPos, by player id
    pub race_finished: bool,            // +0x04 every player is done
    pub race_order: Vec<VehicleRef>,    // +0x08 RaceInfo.pVehicles, leader first
    pub unk_0c: u32,                    // +0x0C
    pub flag_10: bool,                  // +0x10
    pub flag_11: bool,                  // +0x11 10% of races
    pub unk_14: f32,                    // +0x14 10..1010
    pub unk_18: u32,                    // +0x18
    pub unk_1c: u32,                    // +0x1C
    pub ai: Vec<VsAIRaceVehicle>,       // +0x24 m_pAIRaceVehicles
    pub players: Vec<VsAIRaceVehicle>,  // +0x28 m_pPlayerVehicles
    pub drivers: Vec<VsAIDriver>,       // +0x2C m_pDrivers
    pub player_drivers: Vec<VsAIDriver>, // +0x30 m_pPlayerDrivers
    pub first_id: u32,                  // +0x44
    pub chit_chat_done: bool,           // +0x48
    /// the TFT line throttle (the static at 0x006D26E0)
    pub tft: VsAIPlayerTFTManager,
}

/// race setup the game hands over (`VsRaceManager::Init` reads it from globals).
#[derive(Debug, Clone, Default)]
pub struct RaceSetup {
    pub network: bool,
    /// players in a network game (`NetworkGame +0xD0` count)
    pub network_players: u32,
    /// opponents to race (event opponents minus police slots, when the world races opponents)
    pub num_ai: u32,
    /// local players (`dword_6F5C00`)
    pub num_players: u32,
}

impl VsRaceManager {
    /// VsRaceManager.cpp 0x0003AD10
    pub fn init(&mut self, ctx: &mut AiContext, world: &VsAIWorld, setup: &RaceSetup, next_id: &mut u32) {
        self.first_id = *next_id;
        if setup.network {
            self.ai.clear();
            let n = setup.network_players.saturating_sub(1) as usize;
            self.ai.resize_with(n, Default::default);
            self.players.resize_with(1, Default::default);
        } else {
            self.ai.resize_with(setup.num_ai as usize, Default::default);
            self.players.resize_with(setup.num_players as usize, Default::default);
        }
        let racing = (self.ai.len() + self.players.len()) as u32;
        debug_assert!(racing <= MAX_RACE_VEHICLES, "m_dNumRacingVehicles <= MAX_RACE_VEHICLES");
        self.create_vehicles(ctx, world);
        if !setup.network {
            self.create_drivers();
            self.bind_drivers();
        }
        self.reset(ctx, world);
    }

    pub fn num_racing(&self) -> u32 {
        (self.ai.len() + self.players.len()) as u32
    }

    pub fn vehicle(&self, r: VehicleRef) -> Option<&VsAIRaceVehicle> {
        match r {
            VehicleRef::Ai(i) => self.ai.get(i),
            VehicleRef::Player(i) => self.players.get(i),
            _ => None,
        }
    }
    pub fn vehicle_mut(&mut self, r: VehicleRef) -> Option<&mut VsAIRaceVehicle> {
        match r {
            VehicleRef::Ai(i) => self.ai.get_mut(i),
            VehicleRef::Player(i) => self.players.get_mut(i),
            _ => None,
        }
    }

    /// VsRaceManager.cpp 0x0003BFC0
    fn create_vehicles(&mut self, ctx: &mut AiContext, world: &VsAIWorld) {
        let np = self.players.len();
        self.player_positions = vec![1; np];
        self.race_order.clear();
        let mut id = self.first_id;
        for i in 0..self.ai.len() {
            let v = &mut self.ai[i];
            v.init(id, &world.globals, ctx.event, ctx.rand, ctx.game.player_prestige, false);
            v.is_traffic = false;
            v.flags_230 |= 1;
            v.start_pos = ctx.game.grid.get(i + np).copied().unwrap_or_default();
            v.start_orientation = ctx.game.grid_heading;
            self.race_order.push(VehicleRef::Ai(i));
            id += 1;
        }
        for j in 0..np {
            let v = &mut self.players[j];
            v.init(j as u32, &world.globals, ctx.event, ctx.rand, ctx.game.player_prestige, true);
            v.is_traffic = false;
            // original: sets the flags of the ai vehicle with the player's index
            if let Some(a) = self.ai.get_mut(j) {
                a.flags_230 |= 1;
            }
            let slot = if j != 0 { 1 } else { 0 };
            let v = &mut self.players[j];
            v.start_pos = ctx.game.grid.get(slot).copied().unwrap_or_default();
            v.start_orientation = ctx.game.grid_heading;
            self.race_order.push(VehicleRef::Player(j));
        }
    }

    /// VsRaceManager.cpp 0x0003BE20
    fn create_drivers(&mut self) {
        self.drivers = vec![VsAIDriver::default(); self.ai.len()];
        self.player_drivers = vec![VsAIDriver::default(); self.players.len()];
        let tft = self.tft.clone();
        for d in self.drivers.iter_mut().chain(self.player_drivers.iter_mut()) {
            d.init(&tft);
        }
    }

    /// VsRaceManager.cpp 0x0003C4E0
    fn bind_drivers(&mut self) {
        for i in 0..self.ai.len() {
            self.drivers[i].vehicle = VehicleRef::Ai(i);
            self.ai[i].driver = Some(DriverRef::Ai(i));
        }
        for j in 0..self.players.len() {
            self.player_drivers[j].vehicle = VehicleRef::Player(j);
            self.players[j].driver = Some(DriverRef::Player(j));
        }
    }

    /// VsRaceManager.cpp 0x0003AEA0
    pub fn reset(&mut self, ctx: &mut AiContext, world: &VsAIWorld) {
        self.unk_0c = 0;
        self.flag_10 = true;
        self.unk_18 = 1;
        self.unk_1c = 0;
        for v in self.ai.iter_mut().chain(self.players.iter_mut()) {
            v.reset(&world.globals, ctx.event, ctx.rand, ctx.game.player_prestige);
        }
        let tft = self.tft.clone();
        for d in self.drivers.iter_mut().chain(self.player_drivers.iter_mut()) {
            d.reset(&tft);
        }
        self.race_finished = false;
        if world.get_num_race_sections() != 0 {
            self.set_start_section(world, 0);
        }
        // the original reseeds the crt rand from its seed singleton here
        self.flag_11 = ctx.rand.unit() <= f32::from_bits(0x3DCC_CCCD);
        self.unk_14 = ctx.rand.unit() * 1000.0 + 10.0;
        self.chit_chat_done = false;
    }

    /// VsRaceManager.cpp 0x0003BC50
    pub(crate) fn set_start_section(&mut self, world: &VsAIWorld, si: usize) {
        let first_point = if world.sub_route_points.is_empty() { None } else { Some(0) };
        for v in self.ai.iter_mut() {
            v.race_section = Some(si);
            v.next_sub_point = first_point;
        }
        for j in 0..self.players.len() {
            let v = &mut self.players[j];
            v.race_section = Some(si);
            v.next_sub_point = first_point;
            // original: clears the ai vehicle's previous state with the player's index
            if let Some(a) = self.ai.get_mut(j) {
                a.prev_state = RaceState::Grid;
            }
            let v = &mut self.players[j];
            v.state = RaceState::Grid;
            v.raceline_seg = 0;
            v.left_seg = 0;
            v.right_seg = 0;
        }
    }

    /// all cars as other cars see them.
    pub fn build_scene(&self, traffic: &[AiVehicleView], police: &[AiVehicleView]) -> AiScene {
        AiScene {
            ai: self.ai.iter().enumerate().map(|(i, v)| v.view(VehicleRef::Ai(i))).collect(),
            players: self.players.iter().enumerate().map(|(i, v)| v.view(VehicleRef::Player(i))).collect(),
            traffic: traffic.to_vec(),
            police: police.to_vec(),
        }
    }

    fn refresh(&self, scene: &mut AiScene, r: VehicleRef) {
        match r {
            VehicleRef::Ai(i) => scene.ai[i] = self.ai[i].view(r),
            VehicleRef::Player(i) => scene.players[i] = self.players[i].view(r),
            _ => {}
        }
    }

    /// VsRaceManager.cpp 0x0003B2C0
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        ctx: &mut AiContext,
        world: &mut VsAIWorld,
        traffic: &[AiVehicleView],
        police: &[AiVehicleView],
        tft_out: &mut dyn TftPlayer,
        respawns: &mut dyn RespawnSource,
        dt: f32,
    ) {
        let time = world.time;
        let mut scene = self.build_scene(traffic, police);
        if ctx.event.mode == 6
            && self.players.first().is_some_and(|p| p.started)
            && self.players[0].state == RaceState::PlayerAssist
        {
            let mut tft = std::mem::take(&mut self.tft);
            let p = &mut self.players[0];
            let mut f = RaceFrame { ctx, world, scene: &scene, tft: &mut tft, tft_out, speech_enabled: false, num_players: 0, time };
            p.set_human_control(true, &mut f);
            p.state = RaceState::Racing;
            self.tft = tft;
        }
        if ctx.event.mode != 1 {
            self.update_lap_crossing(ctx, world, &scene, tft_out, time);
        }
        let np = self.players.len() as u32;
        let speech = ctx.game.speech_enabled;
        let refs: Vec<VehicleRef> = (0..self.ai.len()).map(VehicleRef::Ai).chain((0..self.players.len()).map(VehicleRef::Player)).collect();
        // state of every active car first
        for &r in &refs {
            if !self.vehicle(r).unwrap().base.active {
                continue;
            }
            let mut tft = std::mem::take(&mut self.tft);
            {
                let v = self.vehicle_mut(r).unwrap();
                let mut f = RaceFrame { ctx, world, scene: &scene, tft: &mut tft, tft_out, speech_enabled: speech, num_players: np, time };
                v.update_state(&mut f, false);
            }
            self.tft = tft;
            self.refresh(&mut scene, r);
        }
        // drivers of ai-controlled cars
        for i in 0..self.ai.len() {
            let Some(h) = self.ai[i].base.physics else { continue };
            if self.drivers.is_empty() || ctx.physics.vehicle_ref(h).control_mode() != 1 {
                continue;
            }
            let mut df = DriverFrame {
                time,
                race_clock: ctx.game.race_clock,
                speech_enabled: speech,
                num_players: np,
                globals: &world.globals,
                rand: ctx.rand,
                tft: &mut self.tft,
                tft_out,
            };
            self.drivers[i].update(&self.ai[i], &scene, &mut df, dt);
        }
        for &r in &refs {
            if !self.vehicle(r).unwrap().base.active {
                continue;
            }
            let mut tft = std::mem::take(&mut self.tft);
            {
                let (v, d) = match r {
                    VehicleRef::Ai(i) => (&mut self.ai[i], self.drivers.get_mut(i)),
                    VehicleRef::Player(i) => (&mut self.players[i], self.player_drivers.get_mut(i)),
                    _ => unreachable!(),
                };
                let mut f = RaceFrame { ctx, world, scene: &scene, tft: &mut tft, tft_out, speech_enabled: speech, num_players: np, time };
                v.update(&mut f, d, respawns, dt);
            }
            self.tft = tft;
            self.refresh(&mut scene, r);
        }
        if ctx.event.mode != 1 {
            self.update_race_order(world, &scene, ctx.event.mode);
        }
        // the debug text overlay of the original is not ported
        if ctx.game.race_phase == 3 && !self.chit_chat_done && ctx.game.timer_time > ctx.game.countdown_start {
            self.chit_chat(ctx, world, &scene, tft_out, time);
            self.chit_chat_done = true;
        }
    }

    /// one random talkative ai says something on the grid.
    /// VsRaceManager.cpp 0x0003B840
    fn chit_chat(&mut self, ctx: &mut AiContext, world: &VsAIWorld, scene: &AiScene, tft_out: &mut dyn TftPlayer, time: f32) {
        if self.drivers.is_empty() {
            return;
        }
        let mut cand = Vec::new();
        for i in 0..self.ai.len() {
            let v = &self.ai[i];
            let Some(h) = v.base.physics else { continue };
            if v.base.active
                && ctx.physics.vehicle_ref(h).control_mode() == 1
                && self.drivers[i].opponent.as_ref().is_some_and(|o| !o.string_lists[0].is_empty())
            {
                cand.push(i);
            }
        }
        if cand.is_empty() {
            return;
        }
        let n = cand.len();
        let hi = (n - 1) as f32;
        let r = (ctx.rand.unit() as f64 * n as f64) as f32;
        let pick = cand[vs_min(r, hi) as i32 as usize];
        let mut df = DriverFrame {
            time,
            race_clock: ctx.game.race_clock,
            speech_enabled: ctx.game.speech_enabled,
            num_players: self.players.len() as u32,
            globals: &world.globals,
            rand: ctx.rand,
            tft: &mut self.tft,
            tft_out,
        };
        let _ = scene;
        self.drivers[pick].speak(tft::CHIT_CHAT, &self.ai[pick], &mut df);
    }

    /// laps, start detection and finish times.
    /// VsRaceManager.cpp 0x0003C730
    fn update_lap_crossing(&mut self, ctx: &mut AiContext, world: &mut VsAIWorld, scene: &AiScene, tft_out: &mut dyn TftPlayer, time: f32) {
        let Some((la, lb)) = ctx.game.start_line else { return };
        // the start line (dword_6F5C24 / 6F5C34; the finish line on circuits, ProcessSplineData)
        let (sa, sb) = ctx.game.start_line_original.unwrap_or((la, lb));
        let laps = ctx.event.num_laps as u32;
        let lap_len = world.race_sections().first().map_or(0.0, |s| s.raceline.get_length());
        let order = self.race_order.clone();
        let np = self.players.len() as u32;
        for r in order {
            let Some(h) = self.vehicle(r).unwrap().base.physics else { continue };
            if !self.vehicle(r).unwrap().base.active {
                continue;
            }
            let pos = ctx.physics.vehicle_ref(h).position();
            let race_clock = ctx.game.race_clock;
            let started = self.vehicle(r).unwrap().started;
            if started {
                let c = line_side(&la, &lb, &pos);
                let v = self.vehicle_mut(r).unwrap();
                if c != 0 {
                    if c == 1 && v.line_state == 2 {
                        v.lap = v.lap.wrapping_sub(1);
                    } else if c == 2 && v.line_state == 1 {
                        if v.lap < laps {
                            v.lap += 1;
                            v.flag_0fc = false;
                            v.reset_dist();
                            set_stat_f(&mut v.stats, 2, race_clock);
                        } else {
                            v.finished = true;
                        }
                    }
                    v.line_state = c;
                } else {
                    v.line_state = 0;
                }
                if !v.finished_flag {
                    if v.finished {
                        v.finished_flag = true;
                        set_stat_f(&mut v.stats, 1, race_clock);
                    } else {
                        let est = (laps as f32 * lap_len - v.get_race_distance()) / 40.0 + race_clock;
                        set_stat_f(&mut v.stats, 1, est);
                    }
                }
                if v.finished_flag {
                    if v.human_control && !ctx.game.network {
                        let mut tft = std::mem::take(&mut self.tft);
                        {
                            let v = self.vehicle_mut(r).unwrap();
                            let mut f = RaceFrame { ctx, world, scene, tft: &mut tft, tft_out, speech_enabled: false, num_players: np, time };
                            v.set_human_control(false, &mut f);
                        }
                        self.tft = tft;
                    }
                    let v = self.vehicle_mut(r).unwrap();
                    v.flag_0fc = false;
                    v.reset_dist();
                    let best = stat_f(&v.stats, 11);
                    let last = stat_f(&v.stats, 3);
                    if best < 0.0001 || best > last {
                        if last > 0.1 {
                            v.stats[11] = v.stats[3];
                        }
                        v.stats[20] = v.stats[19];
                    }
                }
            } else if crossed_start(&sa, &sb, &pos) {
                let v = self.vehicle_mut(r).unwrap();
                v.lap = 1;
                v.started = true;
                v.stats[3] = 0;
                v.stats[2] = 0;
            }
        }
    }

    /// sorts the race order by race distance and hands out positions and neighbours.
    /// VsRaceManager.cpp 0x0003CBC0
    fn update_race_order(&mut self, world: &mut VsAIWorld, scene: &AiScene, mode: u32) {
        let n = self.race_order.len();
        if !self.ai.is_empty() || self.players.len() != 1 {
            let mut swapped = 1;
            while swapped != 0 {
                swapped = 0;
                for j in 0..n.saturating_sub(1) {
                    let r = self.race_order[j];
                    {
                        let v = self.vehicle_mut(r).unwrap();
                        if !v.base.active || !v.started {
                            v.race_distance = 0.0;
                            v.lap = 0;
                        }
                    }
                    let v = self.vehicle(r).unwrap();
                    let nx = self.vehicle(self.race_order[j + 1]).unwrap();
                    if !v.finished_flag || !v.base.active {
                        if nx.race_distance > v.race_distance || (!v.base.active && nx.base.active) {
                            self.race_order.swap(j, j + 1);
                            swapped += 1;
                        }
                    }
                }
            }
            if n == 1 {
                let r = self.race_order[0];
                let v = self.vehicle_mut(r).unwrap();
                v.vehicle_ahead = VehicleRef::None;
                v.vehicle_behind = VehicleRef::None;
            } else {
                for i in 0..n {
                    let r = self.race_order[i];
                    let a = find_vehicle_ahead(scene, r);
                    let b = find_vehicle_behind(scene, r);
                    let v = self.vehicle_mut(r).unwrap();
                    v.vehicle_ahead = a;
                    v.vehicle_behind = b;
                    if v.stats[22] & 0xFF == 0 {
                        v.stats[22] = (v.stats[22] & !0xFF) | (i as u32 + 1);
                    }
                }
            }
            for i in 0..n {
                let r = self.race_order[i];
                let v = self.vehicle_mut(r).unwrap();
                v.stats[22] = (v.stats[22] & !0xFF00) | ((i as u32 + 1) << 8);
                if v.is_player {
                    let id = v.id as usize;
                    if let Some(p) = self.player_positions.get_mut(id) {
                        *p = i as u32 + 1;
                    }
                }
            }
        } else if let Some(p) = self.player_positions.first_mut() {
            *p = 1;
        }
        if !self.race_finished {
            let mut all = true;
            for i in 0..n {
                let r = self.race_order[i];
                let v = self.vehicle_mut(r).unwrap();
                if v.is_player {
                    if mode != 5 || v.failure == FAILURE_NONE {
                        if world.globals.trigger_finish_race {
                            v.finished_flag = true;
                            all = true;
                            world.globals.trigger_finish_race = false;
                        } else {
                            all &= v.finished_flag;
                        }
                    } else {
                        all = true;
                    }
                }
            }
            self.race_finished = all;
        }
    }

    /// race restart: every car back to its grid slot.
    /// VsRaceManager.cpp 0x0003C5D0
    pub fn reset_to_grid(&mut self, f_ctx: &mut AiContext, world: &mut VsAIWorld, tft_out: &mut dyn TftPlayer, with_speed: bool) {
        let scene = self.build_scene(&[], &[]);
        let time = world.time;
        let speed = if with_speed { 40.0 } else { 0.0 };
        for i in 0..self.ai.len() {
            if !self.ai[i].base.active {
                continue;
            }
            let mut tft = std::mem::take(&mut self.tft);
            {
                let mut f = RaceFrame { ctx: f_ctx, world, scene: &scene, tft: &mut tft, tft_out, speech_enabled: false, num_players: 0, time };
                let d = self.drivers.get_mut(i);
                self.ai[i].reset_to_grid(&mut f, d, speed);
            }
            self.tft = tft;
        }
        for j in 0..self.players.len() {
            let mut tft = std::mem::take(&mut self.tft);
            {
                let mut f = RaceFrame { ctx: f_ctx, world, scene: &scene, tft: &mut tft, tft_out, speech_enabled: false, num_players: 0, time };
                let d = self.player_drivers.get_mut(j);
                self.players[j].reset_to_grid(&mut f, d, speed);
            }
            self.tft = tft;
        }
    }

    /// runs `op` on race car `r` with a frame outside the update (timeline events)
    fn with_frame(
        &mut self,
        f_ctx: &mut AiContext,
        world: &mut VsAIWorld,
        tft_out: &mut dyn TftPlayer,
        r: VehicleRef,
        op: impl FnOnce(&mut VsAIRaceVehicle, &mut RaceFrame),
    ) {
        let scene = self.build_scene(&[], &[]);
        let time = world.time;
        let mut tft = std::mem::take(&mut self.tft);
        if let Some(v) = self.vehicle_mut(r) {
            let mut f = RaceFrame { ctx: f_ctx, world, scene: &scene, tft: &mut tft, tft_out, speech_enabled: false, num_players: 0, time };
            op(v, &mut f);
        }
        self.tft = tft;
    }

    /// F3EventRespawnPlayerOnRaceline: every player car back on its raceline (0x000208D0) at `speed`.
    /// TimelineAI.cpp 0x00176D90
    pub fn respawn_players_on_raceline(&mut self, f_ctx: &mut AiContext, world: &mut VsAIWorld, tft_out: &mut dyn TftPlayer, speed: f32) {
        for i in 0..self.players.len() {
            self.with_frame(f_ctx, world, tft_out, VehicleRef::Player(i), |v, f| v.respawn_on_raceline(f, speed));
        }
    }

    /// the crash camera's end: VsAIRaceVehicle 0x000208D0 on the crashed car (its `+0x78` id) at `speed`
    /// (`flt_6850F0`). VsCrashCamera.cpp 0x00062D90
    pub fn crash_camera_respawn(&mut self, f_ctx: &mut AiContext, world: &mut VsAIWorld, tft_out: &mut dyn TftPlayer, id: u32, speed: f32) {
        let r = self.find_vehicle_by_id(id);
        if self.vehicle(r).is_some() {
            self.with_frame(f_ctx, world, tft_out, r, |v, f| v.respawn_on_raceline(f, speed));
        }
    }

    /// F3EventPlayerAIControl: with exactly one player, its human control (0x00018920).
    /// TimelineAI.cpp 0x00176F00
    pub fn player_human_control(&mut self, f_ctx: &mut AiContext, world: &mut VsAIWorld, tft_out: &mut dyn TftPlayer, human: bool) {
        if self.players.len() == 1 {
            self.with_frame(f_ctx, world, tft_out, VehicleRef::Player(0), |v, f| v.set_human_control(human, f));
        }
    }

    /// F3EventRespawn: the active players, then the active ai cars, that are inside the area trigger (`inside` bit
    /// per car id) or human driven take the respawn points in turn (`points`: position, rotation x,y,z,w; None =
    /// no such trigger, the point is still used up).
    /// TimelineAI.cpp 0x00177150
    pub fn timeline_respawn(
        &mut self,
        f_ctx: &mut AiContext,
        world: &mut VsAIWorld,
        tft_out: &mut dyn TftPlayer,
        inside: u32,
        points: &[Option<(VsVector4, VsVector4)>],
        speed: f32,
    ) {
        let mut next = 0usize;
        let refs: Vec<VehicleRef> =
            (0..self.players.len()).map(VehicleRef::Player).chain((0..self.ai.len()).map(VehicleRef::Ai)).collect();
        for r in refs {
            let Some(v) = self.vehicle(r) else { continue };
            if !v.base.active {
                continue;
            }
            // 0x00177C90: "u32ID < 32"
            let in_area = v.id < 32 && inside & (1 << v.id) != 0;
            if !(in_area || v.is_player) {
                continue;
            }
            let point = points.get(next).copied().flatten();
            next += 1;
            if let Some((p, rot)) = point {
                self.with_frame(f_ctx, world, tft_out, r, |v, f| v.timeline_respawn_at(f, &p, &rot, speed));
            }
        }
    }

    /// VsRaceManager.cpp 0x0003D640
    pub fn find_vehicle_by_id(&self, id: u32) -> VehicleRef {
        for (i, v) in self.players.iter().enumerate() {
            if v.id == id {
                return VehicleRef::Player(i);
            }
        }
        for (i, v) in self.ai.iter().enumerate() {
            if v.id == id {
                return VehicleRef::Ai(i);
            }
        }
        VehicleRef::None
    }

    /// last active car in the race order.
    /// VsRaceManager.cpp 0x0003DB70
    pub fn get_last_vehicle(&self) -> VehicleRef {
        for &r in self.race_order.iter().rev() {
            if self.vehicle(r).is_some_and(|v| v.base.active) {
                return r;
            }
        }
        debug_assert!(false);
        VehicleRef::None
    }

    /// players plus active ai cars.
    /// VsRaceManager.cpp 0x0003DCA0
    pub fn get_race_length_count(&self) -> u32 {
        self.players.len() as u32 + self.ai.iter().filter(|v| v.base.active).count() as u32
    }

    /// RestartRace (GameManager 0x000BE0F0) after VsAIWorld::Reset: every car in the order resets its score manager
    /// (0x0003DF30 = +0x1C8, 0x00168F70); the record block is shared, so the cleared fields reach `stats` too.
    /// VsRaceManager.cpp 0x0003DA60
    pub fn reset_scores(&mut self) {
        for i in 0..self.race_order.len() {
            let r = self.race_order[i];
            if let Some(v) = self.vehicle_mut(r)
                && let Some(s) = v.score.as_mut()
            {
                s.reset();
                v.stats_from_score();
            }
        }
    }

    /// the last active car in the race order (asserts "false" when there is none).
    /// VsRaceManager.cpp 0x0003DB70
    pub fn last_active_vehicle(&self) -> Option<VehicleRef> {
        let r = self.race_order.iter().rev().copied().find(|&r| self.vehicle(r).is_some_and(|v| v.base.active));
        if r.is_none() {
            log::error!("assert: false (VsRaceManager.cpp 1310)");
        }
        r
    }
}

#[cfg(test)]
#[path = "lap_test.rs"]
mod lap_test;

/// racers the ahead/behind search considers: players, ai cars, then police in race mode.
fn race_candidates(scene: &AiScene) -> impl Iterator<Item = &AiVehicleView> {
    scene.players.iter().chain(scene.ai.iter()).chain(scene.police.iter().filter(|p| !p.is_traffic))
}

/// nearest car ahead on the same section, else the rearmost one there (lapping).
/// VsRaceManager.cpp 0x0003CFC0
pub fn find_vehicle_ahead(scene: &AiScene, me: VehicleRef) -> VehicleRef {
    let Some(m) = scene.get(me) else { return VehicleRef::None };
    let mut best = -1.0f32;
    let mut found = VehicleRef::None;
    for o in race_candidates(scene) {
        if o.active && o.r != me && o.race_section == m.race_section {
            let d = o.dist_along - m.dist_along;
            if d >= 0.0 && (best == -1.0 || best > d) {
                best = d;
                found = o.r;
            }
        }
    }
    if found == VehicleRef::None {
        best = f32::from_bits(0x4B18_9680);
        for o in race_candidates(scene) {
            if o.active && o.r != me && o.race_section == m.race_section && best > o.dist_along {
                best = o.dist_along;
                found = o.r;
            }
        }
    }
    found
}

/// nearest car behind on the same section, else the frontmost one there.
/// VsRaceManager.cpp 0x0003D3E0
pub fn find_vehicle_behind(scene: &AiScene, me: VehicleRef) -> VehicleRef {
    let Some(m) = scene.get(me) else { return VehicleRef::None };
    let mut best = 1.0f32;
    let mut found = VehicleRef::None;
    for o in race_candidates(scene) {
        if o.active && o.r != me && o.race_section == m.race_section {
            let d = o.dist_along - m.dist_along;
            if d < 0.0 && (best == 1.0 || d > best) {
                best = d;
                found = o.r;
            }
        }
    }
    if found == VehicleRef::None {
        best = f32::from_bits(0xCB18_9680);
        for o in race_candidates(scene) {
            if o.active && o.r != me && o.race_section == m.race_section && o.dist_along > best {
                best = o.dist_along;
                found = o.r;
            }
        }
    }
    found
}

/// which side of the start/finish line `p` is, within 10m of it: 1 before, 2 after, 0 elsewhere.
/// VsRaceManager.cpp 0x0003D780
pub fn line_side(a: &VsVector4, b: &VsVector4, p: &VsVector4) -> u32 {
    if VsAITools::is_left_of_line(a, b, p) {
        if VsAITools::distance_to_line(a, b, p) < 10.0 {
            let t = VsAITools::line_parameter(a, b, p);
            if (0.0..=1.0).contains(&t) {
                return 1;
            }
        }
        return 0;
    }
    if VsAITools::distance_to_line(a, b, p) >= 10.0 {
        return 0;
    }
    let t = VsAITools::line_parameter(a, b, p);
    if !(0.0..=1.0).contains(&t) { 0 } else { 2 }
}

/// just past the start line.
/// VsRaceManager.cpp 0x0003D920
pub fn crossed_start(a: &VsVector4, b: &VsVector4, p: &VsVector4) -> bool {
    if VsAITools::is_left_of_line(a, b, p) || VsAITools::distance_to_line(a, b, p) >= 10.0 {
        return false;
    }
    let t = VsAITools::line_parameter(a, b, p);
    (0.0..=1.0).contains(&t)
}
