//! headless lap: ai cars on a kinematic stand-in vehicle drive a circuit's raceline.

use super::*;
use crate::ai::ai_context::{AiEventInfo, AiGameState, AiPhysicsWorld, NullHost};
use crate::ai::vs_ai_player_tft_manager::NullTft;
use crate::ai::vs_ai_tools::AiCollisionQuery;
use crate::ai::vs_ai_vehicle::{AiPhysicsVehicle, VehicleControls};
use crate::ai::vs_math::*;
use crate::ai::vs_ai_road_section::RoadSectionSet;
use crate::ai::ai_world::{AiWorld, AiWorldFlags};
use crate::ai::vs_traffic_manager::{MAX_TRAFFIC_VEHICLES, VsTrafficManager};
use crate::opponent_database::OpponentDatabase;

const WHEELBASE: f32 = 2.7;
const ACCEL: f32 = 7.0;
const BRAKE: f32 = 11.0;
const TOP_SPEED: f32 = 60.0;
/// lateral grip (m/s^2). the default tweaks never take the corner-braking branch
/// (|corner tightness| <= (2*pi/8)^3 = 0.4845 < 0.5*skill + 0.6), so race ai corners flat out and relies on
/// the car; with 1g the stand-in leaves the road at the first 90 degree bend.
const GRIP: f32 = 200.0;

/// a bicycle-model car with no inertia beyond speed.
#[derive(Default)]
struct KinematicCar {
    pos: VsVector4,
    fwd: VsVector4,
    speed: f32,
    steer: f32,
    controls: VehicleControls,
}

impl KinematicCar {
    /// body forward is +X through the orientation matrix (`vs_quat_rotate`).
    fn quat(&self) -> VsVector4 {
        vs_quat_axis_angle(self.fwd.z.atan2(self.fwd.x), &VS_UNIT_Y)
    }
    fn step(&mut self, dt: f32, ground: &[VsVector4]) {
        let c = &self.controls;
        let a = c.throttle * ACCEL * (1.0 - self.speed / TOP_SPEED) - c.brake * BRAKE - c.handbrake * BRAKE;
        self.speed = (self.speed + a * dt).max(0.0);
        let mut k = self.steer.tan() / WHEELBASE;
        if self.speed > 1.0 {
            let lim = GRIP / (self.speed * self.speed);
            k = k.clamp(-lim, lim);
        }
        let phi = -k * self.speed * dt;
        let (s, co) = phi.sin_cos();
        let f = self.fwd;
        self.fwd = VsVector4::xyz(f.x * co + f.z * s, 0.0, -f.x * s + f.z * co);
        self.pos.x += self.fwd.x * self.speed * dt;
        self.pos.z += self.fwd.z * self.speed * dt;
        self.pos.y = ground_height(ground, &self.pos);
    }
}

fn ground_height(pts: &[VsVector4], p: &VsVector4) -> f32 {
    let mut best = (f32::MAX, 0.0);
    for q in pts {
        let d = (q.x - p.x).powi(2) + (q.z - p.z).powi(2);
        if d < best.0 {
            best = (d, q.y);
        }
    }
    best.1
}

impl AiPhysicsVehicle for KinematicCar {
    fn position(&self) -> VsVector4 {
        self.pos
    }
    fn orientation(&self) -> VsVector4 {
        self.quat()
    }
    fn forward(&self) -> VsVector4 {
        self.fwd
    }
    fn axis_right(&self) -> VsVector4 {
        VsVector4::xyz(-self.fwd.z, 0.0, self.fwd.x)
    }
    fn axis_up(&self) -> VsVector4 {
        VS_UNIT_Y
    }
    fn axis_forward(&self) -> VsVector4 {
        self.fwd
    }
    fn velocity(&self) -> VsVector4 {
        self.fwd * self.speed
    }
    fn speed(&self) -> f32 {
        self.speed
    }
    fn controls(&mut self) -> &mut VehicleControls {
        &mut self.controls
    }
    fn controls_ref(&self) -> &VehicleControls {
        &self.controls
    }
    fn teleport(&mut self, pos: &VsVector4, orientation: &VsVector4, velocity: &VsVector4) {
        self.pos = *pos;
        let mut f = vs_quat_rotate(orientation, &VS_UNIT_X);
        f.y = 0.0;
        f.normalise();
        self.fwd = f;
        self.speed = velocity.length();
    }
    fn set_steer_angle(&mut self, angle: f32) {
        self.steer = angle;
    }
    fn set_linear_velocity(&mut self, v: &VsVector4) {
        self.speed = v.length();
    }
    fn control_mode(&self) -> u32 {
        1
    }
}

struct World {
    cars: Vec<KinematicCar>,
}

impl AiCollisionQuery for World {
    /// ground everywhere, nothing else: vertical rays hit halfway
    fn line_check(&self, s: &VsVector4, e: &VsVector4) -> Option<f32> {
        let h = (s.x - e.x).abs() + (s.z - e.z).abs();
        (s.y - e.y > h).then_some(0.5)
    }
    fn line_check_normal(&self, s: &VsVector4, e: &VsVector4, _mask: u32) -> Option<(f32, VsVector4)> {
        self.line_check(s, e).map(|t| (t, VS_UNIT_Y))
    }
}

impl AiPhysicsWorld for World {
    fn vehicle(&mut self, h: usize) -> &mut dyn AiPhysicsVehicle {
        &mut self.cars[h]
    }
    fn vehicle_ref(&self, h: usize) -> &dyn AiPhysicsVehicle {
        &self.cars[h]
    }
    fn steer_angle_to(&self, h: usize, t: &VsVector4) -> f32 {
        let c = &self.cars[h];
        let mut d = *t - c.pos;
        d.y = 0.0;
        d.normalise();
        let cr = c.fwd.cross(&d);
        -cr.y.asin()
    }
}

/// start line across the road at the raceline start, oriented so the grid is on its left.
fn start_line(w: &VsAIWorld) -> (VsVector4, VsVector4, VsVector4, VsVector4) {
    let s = &w.race_sections()[0];
    let p0 = *s.raceline.get_point(0);
    let p1 = *s.raceline.get_point(1);
    let mut dir = p1 - p0;
    dir.y = 0.0;
    dir.normalise();
    let side = VsVector4::xyz(-dir.z, 0.0, dir.x);
    let half = 15.0;
    let mut a = p0 + side * half;
    let mut b = p0 - side * half;
    a.w = 1.0;
    b.w = 1.0;
    let behind = p0 - dir * 5.0;
    if !VsAITools::is_left_of_line(&a, &b, &behind) {
        std::mem::swap(&mut a, &mut b);
    }
    (a, b, p0, dir)
}

/// xz distance from `p` to a polyline, and the closest point on it.
fn polyline_distance(pts: &[VsVector4], p: &VsVector4) -> (f32, VsVector4) {
    let mut best = (f32::MAX, *p);
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        let (dx, dz) = (b.x - a.x, b.z - a.z);
        let l = dx * dx + dz * dz;
        let t = if l > 0.0 { (((p.x - a.x) * dx + (p.z - a.z) * dz) / l).clamp(0.0, 1.0) } else { 0.0 };
        let q = VsVector4::xyz(a.x + dx * t, p.y, a.z + dz * t);
        let d = ((q.x - p.x).powi(2) + (q.z - p.z).powi(2)).sqrt();
        if d < best.0 {
            best = (d, q);
        }
    }
    best
}

/// how far outside the road `p` is (<= 0 on it): distance to the nearest raceline beyond the wider
/// side of the road at that point, over every race section.
fn distance_off_road(w: &VsAIWorld, p: &VsVector4) -> f32 {
    let mut best = f32::MAX;
    for s in w.race_sections() {
        let (d, q) = polyline_distance(&s.raceline.points, p);
        if d - 40.0 > best {
            continue;
        }
        let half = polyline_distance(&s.left.points, &q).0.max(polyline_distance(&s.right.points, &q).0);
        best = best.min(d - half);
    }
    best
}

/// what the traffic did over a run.
#[derive(Debug, Default)]
struct TrafficStats {
    spawns: u32,
    max_active: u32,
    light_switches: u32,
    worst_lane_dist: f32,
    red_runs: u32,
    frames: u32,
    prev_active: Vec<bool>,
    prev_dist: Vec<f32>,
    prev_flags: Vec<u32>,
}

impl TrafficStats {
    fn sample(&mut self, tm: &VsTrafficManager, w: &VsAIWorld, phys: &World) {
        self.frames += 1;
        let blockers = &w.spline.as_ref().unwrap().traffic_blockers;
        let flags: Vec<u32> = blockers.iter().map(|b| b.flag).collect();
        if !self.prev_flags.is_empty() {
            self.light_switches += flags.iter().zip(&self.prev_flags).filter(|(a, b)| a != b).count() as u32;
        }
        self.prev_active.resize(tm.vehicles.len(), false);
        self.prev_dist.resize(tm.vehicles.len(), 0.0);
        self.max_active = self.max_active.max(tm.num_active);
        for (i, v) in tm.vehicles.iter().enumerate() {
            if v.base.active && !self.prev_active[i] {
                self.spawns += 1;
            } else if v.base.active && self.frames % 10 == 0 {
                let pos = phys.cars[v.base.physics.unwrap()].pos;
                let d = polyline_distance(&w.lane(v.lane.unwrap()).line.points, &pos).0;
                self.worst_lane_dist = self.worst_lane_dist.max(d);
            }
            if v.base.active && self.prev_active[i] {
                // a blocker passed while red (it was red on the frame before too)
                for &(b, bd) in &w.lane(v.lane.unwrap()).blockers {
                    if self.prev_dist[i] < bd && v.dist_along >= bd && flags[b] != 0 && self.prev_flags.get(b).is_some_and(|&f| f != 0) {
                        self.red_runs += 1;
                    }
                }
            }
            self.prev_active[i] = v.base.active;
            self.prev_dist[i] = v.dist_along;
        }
        self.prev_flags = flags;
    }
}

/// one ai-driven player (the "player ai control" tweak) plus `num_ai` opponents.
fn run_lap(track: &str, num_ai: usize, traffic: bool) {
    run_lap_for(track, num_ai, traffic, 3, 600.0, true);
}

/// `strict` checks laps, excursions and traffic; otherwise only that nothing panics within `max_t`.
fn run_lap_for(track: &str, num_ai: usize, traffic: bool, mode: u32, max_t: f32, strict: bool) {
    let Some(game) = crate::game_dir() else { return };
    let level = game.join("Tracks").join(track);
    let (la, lb, p0, dir) = {
        let mut probe = VsAIWorld::new(game.join("AI"));
        assert!(probe.load_spline_data(&level), "{track}");
        probe.process_spline_data(None);
        start_line(&probe)
    };
    let n = num_ai + 1;
    let n_traffic = if traffic { MAX_TRAFFIC_VEHICLES as usize + 2 } else { 0 };
    // handles: ai cars 0..num_ai, the player, then 14 traffic cars and 2 police cars
    let mut phys = World { cars: (0..n + n_traffic).map(|_| KinematicCar::default()).collect() };
    let mut host = NullHost;
    let mut rand = VsRand::default();
    let mut tft = NullTft;
    let db = OpponentDatabase::load().expect("opponent database");
    let mut game_state = AiGameState { race_phase: 4, start_line: Some((la, lb)), start_line_original: Some((la, lb)), ..Default::default() };
    // circuit
    let event = AiEventInfo { mode, num_laps: 3, num_opponents: num_ai as u8, ..Default::default() };
    let setup = RaceSetup { num_ai: num_ai as u32, num_players: 1, ..Default::default() };
    let flags = AiWorldFlags::for_race(traffic, false, false, false, event.mode);
    let mut aiw = {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        AiWorld::init(game.join("AI"), &mut ctx, flags, &setup)
    };
    aiw.world.globals.player_ai_control = true;
    assert!(aiw.load_world_data(&level, Some((la, lb))));
    let world = &aiw.world;
    let mut ground: Vec<VsVector4> = world.race_sections().iter().flat_map(|s| s.raceline.points.clone()).collect();
    for s in world.road_sections(RoadSectionSet::Traffic) {
        for l in &s.lanes {
            ground.extend(l.line.points.iter().copied());
        }
    }
    let lap_len = world.race_sections()[0].raceline.get_length();

    // staggered grid behind the line; slot 0 is the player's
    let side = VsVector4::xyz(-dir.z, 0.0, dir.x);
    game_state.grid = (0..n)
        .map(|i| {
            let mut p = p0 - dir * (10.0 + 8.0 * i as f32) + side * if i % 2 == 0 { 2.5 } else { -2.5 };
            p.y = ground_height(&ground, &p);
            p.w = 1.0;
            p
        })
        .collect();
    game_state.grid_heading = vs_quat_axis_angle(dir.z.atan2(dir.x), &VS_UNIT_Y);
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        // the grid comes from the event definition, which VsRaceManager reads at init
        for (i, v) in aiw.race.ai.iter_mut().enumerate() {
            v.start_pos = ctx.game.grid[i + 1];
            v.start_orientation = ctx.game.grid_heading;
        }
        aiw.race.players[0].start_pos = ctx.game.grid[0];
        aiw.race.players[0].start_orientation = ctx.game.grid_heading;
        // what PhysicsManager.cpp 0x0019E080 does when it builds the race
        let prestige = ctx.game.player_prestige;
        aiw.race.players[0].base.set_physics_vehicle(num_ai);
        aiw.race.players[0].set_player(true, &aiw.world.globals, prestige);
        for i in 0..num_ai {
            aiw.race.ai[i].base.set_physics_vehicle(i);
            aiw.race.drivers[i].set_opponent(db.opponents[i].clone());
        }
        if let Some(tm) = aiw.traffic.as_mut() {
            for (i, v) in tm.vehicles.iter_mut().enumerate() {
                v.base.set_physics_vehicle(n + i);
            }
        }
        if let Some(pm) = aiw.police.as_mut() {
            for (i, v) in pm.vehicles.iter_mut().enumerate() {
                v.set_physics_vehicle(n + MAX_TRAFFIC_VEHICLES as usize + i);
            }
        }
        // the game spawns the cars before the grid reset (GameManager); that activates them
        for r in (0..num_ai).map(VehicleRef::Ai).chain([VehicleRef::Player(0)]) {
            let v = aiw.race.vehicle_mut(r).unwrap();
            let (sp, so, h) = (v.start_pos, v.start_orientation, v.base.physics.unwrap());
            v.base.teleport(ctx.physics.vehicle(h), &sp, &so, &VS_ZERO);
        }
        aiw.reset_to_grid(&mut ctx, &mut tft, false);
    }
    let mut traffic_stats = TrafficStats::default();
    let mut police_spawned = false;

    let refs: Vec<VehicleRef> = (0..num_ai).map(VehicleRef::Ai).chain([VehicleRef::Player(0)]).collect();
    let dt = 1.0 / 60.0;
    let mut t = 0.0f32;
    let mut worst_excursion = vec![0.0f32; n];
    let mut recover_frames = vec![0u32; n];
    let mut lap_time = vec![None; n];
    while t < max_t {
        t += dt;
        game_state.timer_time = t;
        game_state.race_clock = t;
        {
            let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
            aiw.update(&mut ctx, &mut tft, &mut NoRespawns, dt);
        }
        if let Some(tm) = aiw.traffic.as_ref() {
            traffic_stats.sample(tm, &aiw.world, &phys);
        }
        if let Some(pm) = aiw.police.as_ref() {
            police_spawned |= pm.vehicles.iter().any(|v| v.is_active());
        }
        let rm = &aiw.race;
        let world = &aiw.world;
        for c in phys.cars.iter_mut() {
            c.step(dt, &ground);
        }
        for (k, &r) in refs.iter().enumerate() {
            let v = rm.vehicle(r).unwrap();
            if (t * 60.0).round() as u32 % 10 == 0 {
                let ex = distance_off_road(&world, &v.base.position);
                worst_excursion[k] = worst_excursion[k].max(ex);
            }
            if v.state == RaceState::Recover {
                recover_frames[k] += 1;
            }
            if v.lap >= 2 && lap_time[k].is_none() {
                lap_time[k] = Some(t);
            }
        }
        if std::env::var("LAPDBG").is_ok() && t < 20.0 && (t * 60.0).round() as u32 % 30 == 0 {
            let v = &rm.players[0];
            let c = &phys.cars[num_ai];
            let la = &v.look_ahead;
            eprintln!(
                "t {t:.1} st {:?} p {:.1},{:.1} f {:.2},{:.2} spd {:.1} steer {:.3} la0 {:.1},{:.1} la1 {:.1},{:.1} la2 {:.1},{:.1} ct {:.4} thr {:.2} brk {:.2} el {:.1} er {:.1} w {:.1} seg {} along {:.0}",
                v.state, c.pos.x, c.pos.z, c.fwd.x, c.fwd.z, c.speed, c.steer, la[0].pos.x, la[0].pos.z, la[1].pos.x, la[1].pos.z, la[2].pos.x, la[2].pos.z,
                v.corner_tightness, c.controls.throttle, c.controls.brake, v.edge_left_sq.sqrt(), v.edge_right_sq.sqrt(), v.road_width_sq.sqrt(), v.raceline_seg, v.dist_along
            );
        }
        if std::env::var("LAPDBG").is_ok() && (t * 60.0).round() as u32 % 1200 == 0 {
            let v = &rm.players[0];
            let c = &phys.cars[num_ai];
            eprintln!(
                "t {t:.0} state {:?} pos {:.0},{:.0},{:.0} spd {:.1} thr {:.2} brk {:.2} steer {:.3} dist {:.0} lap {}",
                v.state, c.pos.x, c.pos.y, c.pos.z, c.speed, c.controls.throttle, c.controls.brake, c.steer, v.race_distance, v.lap
            );
        }
        if lap_time.iter().all(|l| l.is_some()) {
            break;
        }
    }
    let rm = &aiw.race;
    for (k, &r) in refs.iter().enumerate() {
        let v = rm.vehicle(r).unwrap();
        eprintln!(
            "{track} {r:?}: lap {:?}s, lap len {lap_len:.0}m, race dist {:.0}, worst off-road {:.2}m, recover frames {}, pos {}",
            lap_time[k],
            v.race_distance,
            worst_excursion[k],
            recover_frames[k],
            rm.race_order.iter().position(|x| *x == r).unwrap() + 1
        );
    }
    if !strict {
        return;
    }
    if traffic {
        eprintln!("{track} traffic: {traffic_stats:?}");
        assert!(traffic_stats.spawns > 0, "{track}: no traffic spawned");
        assert!(traffic_stats.light_switches > 0, "{track}: lights never switched");
        assert!(traffic_stats.worst_lane_dist < 4.0, "{track}: traffic left its lane by {}m", traffic_stats.worst_lane_dist);
        assert_eq!(traffic_stats.red_runs, 0, "{track}: traffic ran a red blocker");
        assert!(police_spawned, "{track}: no police car joined the traffic");
    }
    for (k, &r) in refs.iter().enumerate() {
        assert!(lap_time[k].is_some(), "{track} {r:?} never completed a lap");
        // pure pursuit to a 10-20m lookahead cuts corner apexes past the edge line by a few metres
        assert!(worst_excursion[k] < 6.0, "{track} {r:?} left the road by {}m", worst_excursion[k]);
    }
}

#[test]
fn ai_laps_london_circuit() {
    run_lap("London/Circuit/Race01", 0, false);
}

#[test]
fn ai_pack_laps_london_circuit() {
    run_lap("London/Circuit/Race01", 3, false);
}

#[test]
fn ai_pack_laps_la_downtown_circuit_with_traffic() {
    run_lap("LADown/Circuit/Race01", 3, true);
}

/// every race's AISpline processed with its eventdef finish line, as the plugin does.
#[test]
fn every_race_processes_with_its_finish_line() {
    use velocity_game::game_data_loading::game_data_loading::CEventDefinition;
    let Some(game) = crate::game_dir() else { return };
    let mut bad = Vec::new();
    for area in std::fs::read_dir(game.join("Tracks")).unwrap().flatten() {
        for ty in std::fs::read_dir(area.path()).into_iter().flatten().flatten() {
            for race in std::fs::read_dir(ty.path()).into_iter().flatten().flatten() {
                let ed = race.path().join("eventdef.txt");
                let Ok(bytes) = std::fs::read(&ed) else { continue };
                let mut w = VsAIWorld::new(game.join("AI"));
                if !w.load_spline_data(&race.path()) {
                    continue;
                }
                let d = CEventDefinition::parse(&mut visos::handy::vs_text_file::VsTextFile::from_bytes("eventdef.txt", bytes), 1);
                let l = d.finish_line;
                let line = (l[0] != l[1]).then(|| {
                    (VsVector4::new(l[0][0], l[0][1], l[0][2], 1.0), VsVector4::new(l[1][0], l[1][1], l[1][2], 1.0))
                });
                let name = race.path().strip_prefix(game.join("Tracks")).unwrap().to_string_lossy().replace('\\', "/");
                let missing = w.unmatched_section_ends(line);
                if !missing.is_empty() {
                    eprintln!("{name}: unmatched section ends {missing:?}");
                    bad.push(name);
                }
            }
        }
    }
    // the one shipped race whose data trips the original's "pClosestSection != NULL" (port: nearest line)
    assert_eq!(bad, ["London/Circuit/Race09"]);
    let mut w = VsAIWorld::new(game.join("AI"));
    assert!(w.load_spline_data(&game.join("Tracks/London/Circuit/Race09")));
    let ed = std::fs::read(game.join("Tracks/London/Circuit/Race09/eventdef.txt")).unwrap();
    let l = CEventDefinition::parse(&mut visos::handy::vs_text_file::VsTextFile::from_bytes("eventdef.txt", ed), 1).finish_line;
    w.process_spline_data(Some((VsVector4::new(l[0][0], l[0][1], l[0][2], 1.0), VsVector4::new(l[1][0], l[1][1], l[1][2], 1.0))));
    assert_eq!(w.race_sections()[1].branch_from, Some(0));
}

/// every race in the game with opponents and traffic: no ai panic in the first 90 s.
#[test]
#[ignore]
fn ai_sweep_all_races() {
    let Some(game) = crate::game_dir() else { return };
    let mut races = Vec::new();
    for area in std::fs::read_dir(game.join("Tracks")).unwrap().flatten() {
        for ty in std::fs::read_dir(area.path()).into_iter().flatten().flatten() {
            for race in std::fs::read_dir(ty.path()).into_iter().flatten().flatten() {
                let has = std::fs::read_dir(race.path())
                    .into_iter()
                    .flatten()
                    .flatten()
                    .any(|f| f.file_name().to_string_lossy().eq_ignore_ascii_case("aispline.spl"));
                if has {
                    let rel = race.path().strip_prefix(game.join("Tracks")).unwrap().to_string_lossy().replace('\\', "/");
                    races.push((rel, ty.file_name().to_string_lossy().to_ascii_lowercase()));
                }
            }
        }
    }
    races.sort();
    let only = std::env::var("SWEEP").ok();
    let mut failed = Vec::new();
    for (race, ty) in &races {
        if only.as_ref().is_some_and(|o| !race.to_ascii_lowercase().contains(&o.to_ascii_lowercase())) {
            continue;
        }
        let mode = match ty.as_str() { "drag" => 5, "sprint" => 2, _ => 3 };
        let r = std::panic::catch_unwind(|| run_lap_for(race, 3, mode != 5, mode, 90.0, false));
        eprintln!("sweep {race}: {}", if r.is_ok() { "ok" } else { "PANIC" });
        if r.is_err() {
            failed.push(race.clone());
        }
    }
    assert!(failed.is_empty(), "ai panics in {failed:?}");
}

/// drag rounds (GameManager 0x000BC5F0 / RestartRace 0x000BE0F0 / 0x000BF270): a round on LADown Drag Race01 with
/// three opponents finishes, the last active car (0x0003DB70) is put out by the next restart, sits out round 2
/// (reset only touches active cars, it stays last in the order), and the drag restart brings it back.
#[test]
fn drag_rounds_put_the_last_car_out() {
    let Some(game) = crate::game_dir() else { return };
    let track = "LADown/Drag/Race01";
    let level = game.join("Tracks").join(track);
    // the race's own eventdef (grid, start line, finish = lap line), as the plugin's race_state
    let def = {
        let bytes = std::fs::read(level.join("eventdef.txt")).expect("eventdef");
        let mut f = visos::handy::vs_text_file::VsTextFile::from_bytes("eventdef.txt", bytes);
        velocity_game::game_data_loading::game_data_loading::CEventDefinition::parse(&mut f, 1)
    };
    let v4 = |p: [f32; 3]| VsVector4::new(p[0], p[1], p[2], 1.0);
    let (la, lb) = (v4(def.start_line[0]), v4(def.start_line[1]));
    let finish = (v4(def.finish_line[0]), v4(def.finish_line[1]));
    let num_ai = 3;
    let n = num_ai + 1;
    assert!(def.start_positions.len() >= n);
    let mut phys = World { cars: (0..n).map(|_| KinematicCar::default()).collect() };
    let mut host = NullHost;
    let mut rand = VsRand::default();
    let mut tft = NullTft;
    let db = OpponentDatabase::load().expect("opponent database");
    let mut game_state = AiGameState { race_phase: 4, start_line: Some(finish), start_line_original: Some((la, lb)), ..Default::default() };
    let event = AiEventInfo { mode: 5, num_laps: 1, num_opponents: n as u8, ..Default::default() };
    let setup = RaceSetup { num_ai: num_ai as u32, num_players: 1, ..Default::default() };
    let flags = AiWorldFlags::for_race(false, false, false, false, event.mode);
    assert!(!flags.police);
    let mut aiw = {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        AiWorld::init(game.join("AI"), &mut ctx, flags, &setup)
    };
    aiw.world.globals.player_ai_control = true;
    assert!(aiw.load_world_data(&level, Some(finish)));
    let ground: Vec<VsVector4> = aiw.world.race_sections().iter().flat_map(|s| s.raceline.points.clone()).collect();
    game_state.grid = def.start_positions.iter().map(|&p| v4(p)).collect();
    game_state.grid_heading = vs_quat_axis_angle(((0.0 - def.start_direction) + 90.0) * f32::from_bits(0x3C8E_FA35), &VS_UNIT_Y);
    let refs: Vec<VehicleRef> = (0..num_ai).map(VehicleRef::Ai).chain([VehicleRef::Player(0)]).collect();
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        for (i, v) in aiw.race.ai.iter_mut().enumerate() {
            v.start_pos = ctx.game.grid[i + 1];
            v.start_orientation = ctx.game.grid_heading;
        }
        aiw.race.players[0].start_pos = ctx.game.grid[0];
        aiw.race.players[0].start_orientation = ctx.game.grid_heading;
        aiw.race.players[0].base.set_physics_vehicle(num_ai);
        aiw.race.players[0].set_player(true, &aiw.world.globals, 0);
        for i in 0..num_ai {
            aiw.race.ai[i].base.set_physics_vehicle(i);
            aiw.race.drivers[i].set_opponent(db.opponents[i].clone());
        }
        for &r in &refs {
            let v = aiw.race.vehicle_mut(r).unwrap();
            let (sp, so, h) = (v.start_pos, v.start_orientation, v.base.physics.unwrap());
            v.base.teleport(ctx.physics.vehicle(h), &sp, &so, &VS_ZERO);
        }
        aiw.reset_to_grid(&mut ctx, &mut tft, false);
    }
    let dt = 1.0 / 24.0;
    let clock = std::cell::Cell::new(0.0f32);
    let run = |aiw: &mut AiWorld, phys: &mut World, game_state: &mut AiGameState, rand: &mut VsRand, secs: f32| {
        let end = clock.get() + secs;
        while clock.get() < end && !aiw.race.race_finished {
            let t = clock.get() + dt;
            clock.set(t);
            game_state.timer_time = t;
            game_state.race_clock = t;
            let mut ctx = AiContext { physics: &mut *phys, host: &mut NullHost, game: &*game_state, event: &event, rand: &mut *rand };
            aiw.update(&mut ctx, &mut NullTft, &mut NoRespawns, dt);
            for c in phys.cars.iter_mut() {
                c.step(dt, &ground);
            }
        }
    };
    run(&mut aiw, &mut phys, &mut game_state, &mut rand, 180.0);
    for &r in &refs {
        let v = aiw.race.vehicle(r).unwrap();
        eprintln!("{r:?}: t {:.1} dist {:.0} lap {} state {:?} fin {} active {}", clock.get(), v.race_distance, v.lap, v.state, v.finished_flag, v.base.active);
    }
    assert!(aiw.race.race_finished, "drag round 1 never finished");
    let last = aiw.race.last_active_vehicle().unwrap();
    let out = aiw.drag_eliminated_car();
    assert_eq!(out.is_none(), matches!(last, VehicleRef::Player(_)));
    // the player is ai driven here and may come last: put the last ai car out then (what the test is about)
    let out = out.unwrap_or_else(|| *aiw.race.race_order.iter().rev().find(|r| matches!(r, VehicleRef::Ai(_))).unwrap());
    eprintln!("drag round 1 at {:.1} s: order {:?}, out {out:?}", clock.get(), aiw.race.race_order);
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        aiw.reset(&mut ctx);
        aiw.drag_deactivate(&mut ctx, out);
        aiw.reset_to_grid(&mut ctx, &mut tft, false);
    }
    assert!(!aiw.race.race_finished);
    assert!(!aiw.race.vehicle(out).unwrap().base.active);
    assert_eq!(aiw.race.get_race_length_count(), n as u32 - 1);
    run(&mut aiw, &mut phys, &mut game_state, &mut rand, 20.0);
    assert!(!aiw.race.vehicle(out).unwrap().base.active);
    assert_eq!(*aiw.race.race_order.last().unwrap(), out, "the car that is out leads the order");
    assert!(refs.iter().filter(|&&r| r != out).all(|&r| aiw.race.vehicle(r).unwrap().race_distance > 50.0), "round 2 did not start");
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        aiw.drag_reactivate(&mut ctx);
        aiw.reset(&mut ctx);
        aiw.reset_to_grid(&mut ctx, &mut tft, false);
    }
    assert_eq!(aiw.race.get_race_length_count(), n as u32);
    let v = aiw.race.vehicle(out).unwrap();
    assert!(vs_distance(&v.base.position, &v.start_pos) < 0.5, "reactivated car not on its grid slot");
}

/// a pursuit: two police cars leave the grid chasing a scripted human player along the raceline.
#[test]
fn police_pursuit_follows_player() {
    let Some(game) = crate::game_dir() else { return };
    let track = "London/Circuit/Race01";
    let level = game.join("Tracks").join(track);
    let (la, lb, p0, dir) = {
        let mut probe = VsAIWorld::new(game.join("AI"));
        assert!(probe.load_spline_data(&level));
        probe.process_spline_data(None);
        start_line(&probe)
    };
    // player, then the two police cars
    let mut phys = World { cars: (0..3).map(|_| KinematicCar::default()).collect() };
    let mut host = NullHost;
    let mut rand = VsRand::default();
    let mut tft = NullTft;
    let mut game_state = AiGameState { race_phase: 4, start_line: Some((la, lb)), start_line_original: Some((la, lb)), ..Default::default() };
    let event = AiEventInfo { mode: 6, num_laps: 1, num_opponents: 0, ..Default::default() };
    let setup = RaceSetup { num_ai: 0, num_players: 1, ..Default::default() };
    let flags = AiWorldFlags::for_race(false, false, false, false, event.mode);
    assert!(flags.police && !flags.traffic);
    let mut aiw = {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        AiWorld::init(game.join("AI"), &mut ctx, flags, &setup)
    };
    assert!(aiw.load_world_data(&level, Some((la, lb))));
    let ground: Vec<VsVector4> = aiw.world.race_sections().iter().flat_map(|s| s.raceline.points.clone()).collect();
    let side = VsVector4::xyz(-dir.z, 0.0, dir.x);
    game_state.grid = (0..3)
        .map(|i| {
            let mut p = p0 - dir * (10.0 + 8.0 * i as f32) + side * if i % 2 == 0 { 2.5 } else { -2.5 };
            p.y = ground_height(&ground, &p);
            p.w = 1.0;
            p
        })
        .collect();
    game_state.grid_heading = vs_quat_axis_angle(dir.z.atan2(dir.x), &VS_UNIT_Y);
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        let p = &mut aiw.race.players[0];
        p.start_pos = ctx.game.grid[0];
        p.start_orientation = ctx.game.grid_heading;
        p.base.set_physics_vehicle(0);
        p.set_player(true, &aiw.world.globals, 0);
        for (i, v) in aiw.police.as_mut().unwrap().vehicles.iter_mut().enumerate() {
            v.set_physics_vehicle(1 + i);
        }
        let (sp, so) = (aiw.race.players[0].start_pos, aiw.race.players[0].start_orientation);
        aiw.race.players[0].base.teleport(ctx.physics.vehicle(0), &sp, &so, &VS_ZERO);
        aiw.reset_to_grid(&mut ctx, &mut tft, true);
    }
    let pm = aiw.police.as_ref().unwrap();
    assert_eq!(pm.state, crate::ai::vs_police_manager::PoliceState::Pursuit);
    assert!(pm.vehicles.iter().all(|v| v.is_active() && v.race.state == RaceState::Chase));

    let mut raceline = aiw.world.race_sections()[0].raceline.clone();
    let dt = 1.0 / 60.0;
    let mut t = 0.0f32;
    let mut worst_gap = 0.0f32;
    let mut worst_off = 0.0f32;
    while t < 60.0 {
        t += dt;
        game_state.timer_time = t;
        game_state.race_clock = t;
        {
            let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
            aiw.update(&mut ctx, &mut tft, &mut NoRespawns, dt);
        }
        // the "human": steer at the raceline 25m ahead, hold 30 m/s
        {
            let c = &mut phys.cars[0];
            let seg = raceline.find_closest_segment(&c.pos);
            let d = raceline.wrap_distance(raceline.get_distance_along_line(&c.pos, seg) + 25.0);
            let mut out = 0;
            let target = raceline.get_point_at_distance(d, seg, &mut out);
            let mut to = target - c.pos;
            to.y = 0.0;
            to.normalise();
            let cr = c.fwd.cross(&to);
            c.steer = (-cr.y.asin()).clamp(-0.39, 0.39);
            c.controls.throttle = if c.speed < 20.0 { 1.0 } else { 0.0 };
            c.controls.brake = if c.speed > 25.0 { 1.0 } else { 0.0 };
        }
        for c in phys.cars.iter_mut() {
            c.step(dt, &ground);
        }
        if std::env::var("LAPDBG").is_ok() && (t * 60.0).round() as u32 % 120 == 0 {
            let pm = aiw.police.as_ref().unwrap();
            let p = &aiw.race.players[0];
            eprintln!(
                "t {t:.0} player d {:.0} pos {:.0},{:.0} sec {:?} spd {:.1} hc {} st {:?} | police {:?}",
                p.race_distance, phys.cars[0].pos.x, phys.cars[0].pos.z, p.race_section, phys.cars[0].speed, p.human_control, p.state,
                pm.vehicles.iter().map(|v| (v.race.race_distance as i32, phys.cars[v.race.base.physics.unwrap()].speed as i32, v.race.state, v.race.speed_match, v.race.target_speed as i32)).collect::<Vec<_>>()
            );
        }
        if t > 10.0 {
            let pm = aiw.police.as_ref().unwrap();
            let pd = aiw.race.players[0].race_distance;
            for v in &pm.vehicles {
                worst_gap = worst_gap.max(vs_abs(pd - v.race.race_distance));
                if (t * 60.0).round() as u32 % 10 == 0 {
                    worst_off = worst_off.max(distance_off_road(&aiw.world, &v.race.base.position));
                }
            }
        }
    }
    let pm = aiw.police.as_ref().unwrap();
    eprintln!(
        "pursuit: state {:?}, worst gap to player {worst_gap:.0}m, worst off-road {worst_off:.2}m, player dist {:.0}",
        pm.state, aiw.race.players[0].race_distance
    );
    assert!(pm.vehicles.iter().all(|v| v.is_active() && v.race.state == RaceState::Chase));
    assert!(worst_gap < 200.0, "police lost the player by {worst_gap}m");
    assert!(worst_off < 6.0, "police left the road by {worst_off}m");
}

/// LADown Circuit Race02's first trailblazer trigger (spline 0x3E7ED980, 8 m): a player car started on it and
/// moved along the line at 35 m/s finishes it, with the danger record on in between
#[test]
fn trail_blazer_follows_ladown_spline() {
    use crate::ai::vs_ai_race_vehicle::{TrailLine, VsAIRaceVehicle};
    use velocity_game::game_mode::trail_blazer::{TrailEvent, TrailSpline};
    let Some(game) = crate::game_dir() else { return };
    let mut w = VsAIWorld::new(game.join("AI"));
    assert!(w.load_spline_data(&game.join("Tracks").join("LADown/Circuit/Race02")));
    w.process_spline_data(None);
    assert!(w.find_trail_blazer_line_mut(0xDEAD_BEEF).is_none());
    let mut v = VsAIRaceVehicle::default();
    v.set_player(true, &w.globals.clone(), 0);
    let start = *w.find_trail_blazer_line_mut(0x3E7E_D980).expect("trailblazer spline").get_point(0);
    v.base.position = start;
    v.base.speed = 35.0;
    assert!(v.start_trail_blazer(&mut w, 0x3E7E_D980, 8.0, 0, 0, false, [1.0; 6]));
    assert!(v.trail_events.contains(&TrailEvent::DangerEnabled(true)));
    let mut d = 0.0;
    let mut on = false;
    for _ in 0..2000 {
        let line = w.find_trail_blazer_line_mut(0x3E7E_D980).unwrap();
        let s = TrailLine(std::cell::RefCell::new(line));
        d = (d + 35.0 / 24.0f32).min(s.length());
        let p = s.point_at(d, 0);
        let q = s.point_at((d + 1.0).min(s.length()), 0);
        v.base.position = VsVector4::new(p.x, p.y, p.z, 1.0);
        let f = (q - p).normalize_or_zero();
        v.base.forward = VsVector4::new(f.x, f.y, f.z, 0.0);
        let car = velocity_game::game_mode::trail_blazer::TrailCar { position: p, speed: 35.0, heading: f };
        let tb = v.trail_blazer.as_mut().unwrap();
        tb.update(1.0 / 24.0, Some(&s), &car, 0.0, &mut v.trail_events);
        on |= v.trail_events.iter().any(|e| matches!(e, TrailEvent::RenderParams { on: true, .. }));
        if !tb.active() {
            break;
        }
    }
    let tb = v.trail_blazer.as_ref().unwrap();
    assert!(on && !tb.active() && tb.success, "trailblazer at {d} m");
}

/// records FlareRender 0x001270E0 calls; camera 0 at `cam`
struct FlashHost {
    cam: (VsVector4, VsVector4),
    flashes: Vec<crate::ai::ai_context::AiFlashRecord>,
}

impl crate::ai::ai_context::AiHost for FlashHost {
    fn camera(&self, player: usize) -> Option<(VsVector4, VsVector4)> {
        (player == 0).then_some(self.cam)
    }
    fn flare_flash(&mut self, rec: &crate::ai::ai_context::AiFlashRecord) {
        self.flashes.push(*rec);
    }
}

/// a two-human race (front-end split screen: `byte_6F70D8` = 1, so VsRaceManager 0x0003AD10 still seats
/// competitors - 2 ai cars): both players get race positions and progress; then a forced NOS explosion
/// (failure 1, 0x00019740) on the ai cars flashes towards player 0's camera.
#[test]
fn split_screen_two_players_and_nos_flash() {
    let Some(game) = crate::game_dir() else { return };
    let track = "London/Circuit/Race01";
    let level = game.join("Tracks").join(track);
    let (la, lb, p0, dir) = {
        let mut probe = VsAIWorld::new(game.join("AI"));
        assert!(probe.load_spline_data(&level), "{track}");
        probe.process_spline_data(None);
        start_line(&probe)
    };
    let (num_players, num_ai) = (2usize, 2usize);
    let n = num_players + num_ai;
    // physics ids: players 0..2, then the ai cars (PhysicsManager 0x0019E080 order)
    let mut phys = World { cars: (0..n).map(|_| KinematicCar::default()).collect() };
    let mut host = NullHost;
    let mut rand = VsRand::default();
    let mut tft = NullTft;
    let db = OpponentDatabase::load().expect("opponent database");
    let mut game_state = AiGameState { race_phase: 4, start_line: Some((la, lb)), start_line_original: Some((la, lb)), ..Default::default() };
    let event = AiEventInfo { mode: 3, num_laps: 3, num_opponents: n as u8, ..Default::default() };
    let setup = RaceSetup { num_ai: num_ai as u32, num_players: num_players as u32, ..Default::default() };
    let flags = AiWorldFlags::for_race(false, false, false, false, event.mode);
    let mut aiw = {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        AiWorld::init(game.join("AI"), &mut ctx, flags, &setup)
    };
    assert_eq!((aiw.race.players.len(), aiw.race.ai.len()), (num_players, num_ai));
    assert_eq!(aiw.race.player_positions.len(), num_players);
    aiw.world.globals.player_ai_control = true;
    assert!(aiw.load_world_data(&level, Some((la, lb))));
    let ground: Vec<VsVector4> = aiw.world.race_sections().iter().flat_map(|s| s.raceline.points.clone()).collect();
    let side = VsVector4::xyz(-dir.z, 0.0, dir.x);
    game_state.grid = (0..n)
        .map(|i| {
            let mut p = p0 - dir * (10.0 + 8.0 * i as f32) + side * if i % 2 == 0 { 2.5 } else { -2.5 };
            p.y = ground_height(&ground, &p);
            p.w = 1.0;
            p
        })
        .collect();
    game_state.grid_heading = vs_quat_axis_angle(dir.z.atan2(dir.x), &VS_UNIT_Y);
    let refs: Vec<VehicleRef> = (0..num_players).map(VehicleRef::Player).chain((0..num_ai).map(VehicleRef::Ai)).collect();
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        for (k, &r) in refs.iter().enumerate() {
            let v = aiw.race.vehicle_mut(r).unwrap();
            v.start_pos = ctx.game.grid[k];
            v.start_orientation = ctx.game.grid_heading;
            v.base.set_physics_vehicle(k);
        }
        for i in 0..num_players {
            aiw.race.players[i].set_player(true, &aiw.world.globals, 0);
        }
        for i in 0..num_ai {
            aiw.race.drivers[i].set_opponent(db.opponents[i].clone());
        }
        for &r in &refs {
            let v = aiw.race.vehicle_mut(r).unwrap();
            let (sp, so, h) = (v.start_pos, v.start_orientation, v.base.physics.unwrap());
            v.base.teleport(ctx.physics.vehicle(h), &sp, &so, &VS_ZERO);
        }
        aiw.reset_to_grid(&mut ctx, &mut tft, false);
    }
    let dt = 1.0 / 24.0;
    let mut t = 0.0f32;
    while t < 40.0 {
        t += dt;
        game_state.timer_time = t;
        game_state.race_clock = t;
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        aiw.update(&mut ctx, &mut tft, &mut NoRespawns, dt);
        for c in phys.cars.iter_mut() {
            c.step(dt, &ground);
        }
    }
    let pos = aiw.race.player_positions.clone();
    eprintln!("split screen at {t:.0} s: player positions {pos:?}, order {:?}", aiw.race.race_order);
    for &r in &refs {
        let v = aiw.race.vehicle(r).unwrap();
        eprintln!("  {r:?}: dist {:.0} state {:?}", v.race_distance, v.state);
        assert!(v.race_distance > 300.0, "{r:?} did not progress");
    }
    assert!(pos.iter().all(|&p| (1..=n as u32).contains(&p)) && pos[0] != pos[1], "player positions {pos:?}");
    // camera 0 = 20 m behind the last car, looking along player 0's heading
    let p0v = aiw.race.players[0].base.position;
    let fwd = aiw.race.players[0].base.forward;
    let last = aiw.race.vehicle(*aiw.race.race_order.last().unwrap()).unwrap().base.position;
    let mut fh = FlashHost { cam: (last - fwd * 20.0, fwd), flashes: Vec::new() };
    aiw.world.globals.force_failure = 1;
    t += dt;
    game_state.timer_time = t;
    game_state.race_clock = t;
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut fh, game: &game_state, event: &event, rand: &mut rand };
        aiw.update(&mut ctx, &mut tft, &mut NoRespawns, dt);
    }
    assert!(aiw.race.ai.iter().all(|v| v.failure == 1));
    assert!(aiw.race.players.iter().all(|v| v.failure != 1), "players take no failure 1");
    // expected per ai car in front of the camera: k = clamp(0.5 / (|p0 - car| * 0.05)), a = clamp(k (d + 0.5) 0.666)
    let mut expect = 0;
    for v in &aiw.race.ai {
        let mut to = v.base.position - fh.cam.0;
        to.normalise();
        let d = vs_dot3(&fh.cam.1, &to);
        let k = vs_clamp(0.5 / ((p0v - v.base.position).length() * 0.05), 0.0, 1.0);
        if d > 0.0 && vs_clamp(k * (d + 0.5) * f32::from_bits(0x3F2A_7EFA), 0.0, 1.0) > 0.0 {
            expect += 1;
        }
    }
    eprintln!("flashes {:?}", fh.flashes);
    assert!(expect > 0);
    assert_eq!(fh.flashes.len(), expect);
    for f in &fh.flashes {
        assert!(f.intensity > 0.0 && f.intensity <= 1.0 && f.size <= 0.5 && f.fall == f32::from_bits(0x3D4C_CCCD) && !f.screen);
    }
}
