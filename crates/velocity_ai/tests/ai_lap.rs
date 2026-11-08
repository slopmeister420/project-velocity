//! The ported AI drives the real physics cars (translated FastCar / VehicleEntity on the track collision)
//! round a circuit: grid from the race's eventdef, cars bound like PhysicsManager 0x0019E080, the AI's
//! update before every physics frame (GameManager InGameUpdate order), collisions fed back to the AI.



use std::sync::{Arc, Mutex};

use velocity_ai::physics_ai::PhysicsAi;
use velocity_ai::ai::ai_context::{AiContext, AiEventInfo, AiGameState, NullHost};
use velocity_ai::ai::ai_world::{AiWorld, AiWorldFlags};
use velocity_ai::ai::vs_ai_player_tft_manager::NullTft;
use velocity_ai::ai::vs_ai_race_vehicle::{NoRespawns, RaceState, VehicleRef};
use velocity_ai::ai::vs_math::{vs_quat_axis_angle, VsRand, VsVector4};
use velocity_ai::ai::vs_race_manager::RaceSetup;
use velocity_ai::opponent_database::OpponentDatabase;
use velocity_game::game_data_loading::game_data_loading::CEventDefinition;
use velocity_physics::c_config::load_global_config_with_fallback;
use velocity_physics::plugin::SharedPhysicsWorld;
use velocity_physics::vehicle::vehicle_entity::cx::TrackWorld;
use velocity_physics::vehicle::vehicle_entity::sim::{Explosion, PhysicsWorld, SpawnParams};
use velocity_physics::vehicle::vehicle_entity::LocalConfigInputs;
use velocity_physics::vehicle_database::string_hash;
use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};

const DT: f32 = 1.0 / 24.0;
/// a quick race's temp event names its opponents AL, BUD, JULIO, ROBYN, JAVIER (0x00093630)
const OPPONENTS: [&str; 5] = ["AL", "BUD", "JULIO", "ROBYN", "JAVIER"];

fn v4(p: [f32; 3]) -> VsVector4 {
    VsVector4::new(p[0], p[1], p[2], 1.0)
}

/// xz distance from `p` to a polyline
fn polyline_distance(pts: &[VsVector4], p: &VsVector4) -> (f32, VsVector4) {
    let mut best = (f32::MAX, *p);
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        let (dx, dz) = (b.x - a.x, b.z - a.z);
        let l = dx * dx + dz * dz;
        let t = if l > 0.0 { (((p.x - a.x) * dx + (p.z - a.z) * dz) / l).clamp(0.0, 1.0) } else { 0.0 };
        let q = VsVector4::new(a.x + dx * t, p.y, a.z + dz * t, 1.0);
        let d = ((q.x - p.x).powi(2) + (q.z - p.z).powi(2)).sqrt();
        if d < best.0 {
            best = (d, q);
        }
    }
    best
}

/// how far outside the road edges `p` is (<= 0 on the road)
fn distance_off_road(w: &AiWorld, p: &VsVector4) -> f32 {
    let mut best = f32::MAX;
    for s in w.world.race_sections() {
        let (d, q) = polyline_distance(&s.raceline.points, p);
        if d - 40.0 > best {
            continue;
        }
        let half = polyline_distance(&s.left.points, &q).0.max(polyline_distance(&s.right.points, &q).0);
        best = best.min(d - half);
    }
    best
}

/// what each car did: lap time, race distance when it first left the road (> 6 m past the edge lines)
struct CarResult {
    lap: Option<f32>,
    first_off: Option<f32>,
    finite: bool,
}

/// per-frame test hook after the physics step: (time, ai world, physics); true ends the run
type Hook<'a> = &'a mut dyn FnMut(f32, &mut AiWorld, &SharedPhysicsWorld) -> bool;

fn run_lap(track: &str, race: &str, num_ai: usize) -> Vec<CarResult> {
    run_lap_with(track, race, num_ai, 400.0, &mut |_, _, _| false)
}

fn run_lap_with(track: &str, race: &str, num_ai: usize, max_t: f32, hook: Hook) -> Vec<CarResult> {
    let Some(game) = velocity_ai::game_dir() else { return Vec::new() };
    let race_dir = format!(r"\Tracks\{track}\{race}\");
    let def = CEventDefinition::load(&race_dir, 1).expect("eventdef");
    let gc = load_global_config_with_fallback(0).expect("Main.cfg");
    let shared: SharedPhysicsWorld = Arc::new(Mutex::new(Some(PhysicsWorld::new(gc, Some(TrackWorld::load(track).expect("collision"))))));

    // the race inputs from the eventdef (GameDataLoading.cpp 0x000A2EF0, as velocity_ai's plugin builds them)
    let heading = vs_quat_axis_angle(((0.0 - def.start_direction) + 90.0) * f32::from_bits(0x3C8E_FA35), &VsVector4::new(0.0, 1.0, 0.0, 1.0));
    let game_state = AiGameState {
        race_phase: 4,
        grid: def.start_positions.iter().map(|&p| v4(p)).collect(),
        grid_heading: heading,
        start_line_original: Some((v4(def.start_line[0]), v4(def.start_line[1]))),
        start_line: Some((v4(def.finish_line[0]), v4(def.finish_line[1]))),
        finish_point: v4(def.unknown_line_point),
        ..Default::default()
    };
    let n = 1 + num_ai;
    assert!(game_state.grid.len() >= n, "{track}: {} grid slots", game_state.grid.len());

    // PhysicsManager 0x0019E080: the player (vehicle 0) then the opponents, on their grid slots. The cars are
    // built from setups like the game's (VehicleEntity 0x001CABF0): the player in a stock 350Z, each opponent in
    // its tier-1 car (OpponentDatabase 0x000E72C0); every shipped setup has tyre softness >= 0.5, the bare
    // default config (0) is a harder tyre than any car the game creates.
    let vdb = CVehicleDatabase::load().expect("vehicle database");
    let db = OpponentDatabase::load().expect("opponent database");
    for i in 0..n {
        let mut inputs = if i == 0 {
            let mut s = VehicleSetup::new();
            assert!(vdb.setup_default_vehicle(&mut s, string_hash("350Z")));
            LocalConfigInputs::from_setup(&vdb, &s)
        } else {
            let o = db.get_opponent_by_name(OPPONENTS[i - 1]).expect("opponent");
            let (s, _) = db.build_vehicle_setup(&vdb, o.name_crc, 0).expect("opponent car");
            LocalConfigInputs::from_setup(&vdb, &s)
        };
        inputs.is_player = i == 0;
        let p = game_state.grid[i];
        let id = shared.lock().unwrap().as_mut().unwrap().spawn(&SpawnParams {
            inputs,
            position: [p.x, p.y + 0.5, p.z],
            orientation: [heading.x, heading.y, heading.z, heading.w],
            slot: if i == 0 { 0 } else { -1 },
        });
        assert_eq!(id, i);
        // control mode (VE+0x13DC): 0 player, 1 race opponent
        shared.lock().unwrap().as_mut().unwrap().set_control_mode(id, if i == 0 { 0 } else { 1 });
    }
    let mut phys = PhysicsAi::new(shared.clone());
    phys.sync_cars();

    let mut host = NullHost;
    let mut rand = VsRand::default();
    let mut tft = NullTft;
    let event = AiEventInfo { mode: 3, num_laps: def.num_laps.max(1) as u8, num_opponents: num_ai as u8, ..Default::default() };
    let setup = RaceSetup { num_ai: num_ai as u32, num_players: 1, ..Default::default() };
    let flags = AiWorldFlags::for_race(false, false, false, false, event.mode);
    let mut aiw = {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        AiWorld::init(game.join("AI"), &mut ctx, flags, &setup)
    };
    // the player's car is driven by the AI too ("PlayerAIControl")
    aiw.world.globals.player_ai_control = true;
    // the debug menu's "AI/Difficulty Scale" (0..1, default 0.6): LAPDIFF=x for experiments
    if let Some(d) = std::env::var("LAPDIFF").ok().and_then(|v| v.parse().ok()) {
        aiw.world.globals.difficulty_scale = d;
    }
    let level = velocity_ai::resolve(&game, &race_dir);
    assert!(aiw.load_world_data(&level, game_state.start_line), "{track} ai data");
    // the barrier walls the race load builds (BarrierRender 0x00108030); LAPWALLS=0 leaves them out
    if std::env::var("LAPWALLS").map_or(true, |v| v != "0")
        && let Some(m) = velocity_ai::barrier_render::build_collision_mesh(&aiw.world)
    {
        eprintln!("{track}: barrier walls, {} faces", m.faces.len());
        shared.lock().unwrap().as_mut().unwrap().set_barrier_walls(TrackWorld::from_walls(&m.vertices, &m.faces));
    }
    {
        let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
        aiw.race.players[0].start_pos = ctx.game.grid[0];
        aiw.race.players[0].start_orientation = ctx.game.grid_heading;
        aiw.race.players[0].base.set_physics_vehicle(0);
        aiw.race.players[0].set_player(true, &aiw.world.globals, 0);
        for i in 0..num_ai {
            aiw.race.ai[i].start_pos = ctx.game.grid[1 + i];
            aiw.race.ai[i].start_orientation = ctx.game.grid_heading;
            aiw.race.ai[i].base.set_physics_vehicle(1 + i);
            let name = OPPONENTS[i];
            aiw.race.drivers[i].set_opponent(db.get_opponent_by_name(name).expect("opponent").clone());
        }
        let refs: Vec<VehicleRef> = [VehicleRef::Player(0)].into_iter().chain((0..num_ai).map(VehicleRef::Ai)).collect();
        for r in refs {
            let v = aiw.race.vehicle_mut(r).unwrap();
            let (sp, so, h) = (v.start_pos, v.start_orientation, v.base.physics.unwrap());
            v.base.teleport(ctx.physics.vehicle(h), &sp, &so, &VsVector4::default());
        }
        aiw.reset_to_grid(&mut ctx, &mut tft, false);
    }

    let refs: Vec<VehicleRef> = [VehicleRef::Player(0)].into_iter().chain((0..num_ai).map(VehicleRef::Ai)).collect();
    let mut game_state = game_state;
    let mut t = 0.0f32;
    let mut lap_time = vec![None; n];
    let mut worst_off = vec![0.0f32; n];
    let mut off_dist = vec![0.0f32; n];
    let mut last_pos: Vec<Option<[f32; 3]>> = vec![None; n];
    let mut recover = vec![0u32; n];
    let mut lowest = vec![f32::MAX; n];
    let mut collisions = 0usize;
    let mut wall_hits = 0usize;
    let mut top_speed = vec![0.0f32; n];
    let mut first_off = vec![None; n];
    let mut finite = vec![true; n];
    let dbg = std::env::var("LAPDBG").is_ok();
    while t < max_t {
        t += DT;
        game_state.timer_time = t;
        game_state.race_clock = t;
        {
            let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
            aiw.update(&mut ctx, &mut tft, &mut NoRespawns, DT);
        }
        // VehicleManager__1DD9F0: the AI's human-control flag picks pad or controls block
        {
            let mut g = shared.lock().unwrap();
            let w = g.as_mut().unwrap();
            w.set_human_control(0, aiw.race.players[0].human_control);
        }
        phys.flush_controls();
        let hits = {
            let mut g = shared.lock().unwrap();
            let w = g.as_mut().unwrap();
            w.step(DT, &[]);
            w.take_collisions()
        };
        collisions += hits.len();
        wall_hits += hits.iter().filter(|c| c.b == -2).count();
        for c in hits {
            let mut ctx = AiContext { physics: &mut phys, host: &mut host, game: &game_state, event: &event, rand: &mut rand };
            aiw.on_collision(&mut ctx, &mut tft, c.a, c.b, c.impact);
        }
        if hook(t, &mut aiw, &shared) {
            break;
        }
        for (k, &r) in refs.iter().enumerate() {
            let v = aiw.race.vehicle(r).unwrap();
            let (pos, spd) = {
                let mut g = shared.lock().unwrap();
                let w = g.as_mut().unwrap();
                (w.position(k), w.ai_speed(k))
            };
            top_speed[k] = top_speed[k].max(spd);
            finite[k] &= pos.iter().all(|x| x.is_finite());
            lowest[k] = lowest[k].min(pos[1]);
            if (t / DT).round() as u32 % 6 == 0 {
                let off = distance_off_road(&aiw, &v4(pos));
                worst_off[k] = worst_off[k].max(off);
                if let Some(l) = last_pos[k]
                    && off > 0.0
                {
                    off_dist[k] += ((pos[0] - l[0]).powi(2) + (pos[2] - l[2]).powi(2)).sqrt();
                }
                last_pos[k] = Some(pos);
                if off > 6.0 && first_off[k].is_none() {
                    first_off[k] = Some(v.race_distance);
                }
            }
            if v.state == RaceState::Recover {
                recover[k] += 1;
            }
            if v.lap >= 2 && lap_time[k].is_none() {
                lap_time[k] = Some(t);
            }
            if k == 0
                && let Some((t0, t1)) = std::env::var("LAPTRACE").ok().and_then(|s| {
                    let (a, b) = s.split_once(',')?;
                    Some((a.parse::<f32>().ok()?, b.parse::<f32>().ok()?))
                })
                && t >= t0
                && t <= t1
            {
                let mut g = shared.lock().unwrap();
                let w = g.as_mut().unwrap();
                let fw = w.body_row(k, 0);
                let av = w.angular_velocity(k);
                let vel = w.velocity(k);
                let c = w.controls(k);
                eprintln!(
                    "tr t {t:.3} pos {:.1},{:.1} spd {:.1} v {:.1},{:.1} fwd {:.2},{:.2} yaw {:.2} ai_steer {:.3} car_steer {:.3} ct {:.3} la0 {:.1},{:.1} thr {:.2} brk {:.2} gear {} off {:.1}",
                    pos[0], pos[2], spd, vel[0], vel[2], fw[0], fw[2], av[1], v.steer_angle, w.steer_angle(k), v.corner_tightness,
                    v.look_ahead[0].pos.x, v.look_ahead[0].pos.z, c.throttle, c.brake, w.ai_gear(k), distance_off_road(&aiw, &v4(pos))
                );
            }
            if dbg && (k == 0 || std::env::var("LAPALL").is_ok()) && ((t / DT).round() as u32 % 48 == 0) {
                let c = phys_controls(&shared, k);
                eprintln!(
                    "t {t:.2} {r:?} fail {} dmg {:.0} ct {:.3} st {:?} pos {:.0},{:.1},{:.0} spd {:.1} thr {:.2} brk {:.2} steer {:.2} dist {:.0} lap {} off {:.1}",
                    v.failure, v.base.damage, v.corner_tightness, v.state, pos[0], pos[1], pos[2], spd, c.throttle, c.brake, c.steer, v.race_distance, v.lap, distance_off_road(&aiw, &v4(pos))
                );
            }
        }
        if lap_time.iter().all(|l| l.is_some()) {
            break;
        }
    }
    let lap_len = aiw.world.race_sections()[0].raceline.get_length();
    for (k, &r) in refs.iter().enumerate() {
        let v = aiw.race.vehicle(r).unwrap();
        eprintln!(
            "{track} {race} {r:?}: lap {:?}s, lap len {lap_len:.0}m, race dist {:.0}, top {:.1} m/s, worst off-road {:.2}m, driven off-road {:.0}m, recover frames {}, lowest y {:.1}",
            lap_time[k], v.race_distance, top_speed[k], worst_off[k], off_dist[k], recover[k], lowest[k]
        );
    }
    eprintln!("{track} {race}: {collisions} collision messages, {wall_hits} with walls");
    (0..n).map(|k| CarResult { lap: lap_time[k], first_off: first_off[k], finite: finite[k] }).collect()
}

fn phys_controls(w: &SharedPhysicsWorld, id: usize) -> velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw {
    w.lock().unwrap().as_ref().unwrap().controls(id)
}

/// the player's car under AI control ("PlayerAIControl", bPLAYER config) in a stock 350Z: takes the first
/// S-bend (~1100 m) at 54 m/s, completes the lap. The AI never brakes for corners, as in the original: corner
/// braking (0x0001C1C0) needs |corner tightness| >= skill * 0.5 + "Difficulty Scale" (0.6), and the tightness is
/// the cube of two steer angles clamped to +-pi/8 (0x0002C1B0), so at most 0.4845.
#[test]
fn ai_drives_physics_car_london_circuit() {
    let r = run_lap("London", r"Circuit\Race01", 0);
    let Some(p) = r.first() else { return };
    assert!(p.finite, "NaN in the physics");
    assert!(p.first_off.is_none_or(|d| d > 1300.0), "player left the road at {:?}m", p.first_off);
    assert!(p.lap.is_some(), "player did not complete the lap");
}

/// where and when a car came back from a respawn
#[derive(Debug, Default, Clone, Copy)]
struct Respawned {
    at: f32,
    raceline_off: f32,
    up_y: f32,
}

/// xz distance from `p` to the nearest raceline
fn raceline_distance(w: &AiWorld, p: &VsVector4) -> f32 {
    w.world.race_sections().iter().map(|s| polyline_distance(&s.raceline.points, p).0).fold(f32::MAX, f32::min)
}

/// runs London Circuit Race01 with one opponent; from `t0` `disturb(physics, first)` messes with the opponent
/// (physics vehicle 1) every frame until it respawns: 0x000208D0 sets the grace time (+0x320) to AI time + 2, which
/// nothing else does while the car cannot drive off (0x000205A0 needs > 5 m/s). Then the car is left alone for 10 s;
/// returns the respawn and the speed it reached afterwards.
fn opponent_respawn(t0: f32, disturb: &mut dyn FnMut(&mut PhysicsWorld, bool)) -> Option<(Respawned, f32)> {
    velocity_ai::game_dir()?;
    let mut grace0: Option<f32> = None;
    let mut got: Option<Respawned> = None;
    let mut after = 0.0f32;
    let mut first = true;
    let mut last_state = String::new();
    run_lap_with("London", r"Circuit\Race01", 1, t0 + 40.0, &mut |t, aiw, shared| {
        let mut g = shared.lock().unwrap();
        let w = g.as_mut().unwrap();
        let p = w.position(1);
        if t < t0 {
            return false;
        }
        let v = aiw.race.vehicle(VehicleRef::Ai(0)).unwrap();
        if let Some(r) = got {
            after = after.max(w.ai_speed(1));
            return t > r.at + 10.0;
        }
        last_state = format!("t {t:.2} state {:?} stuck {} upside {} grace {:.2} pos {p:?}", v.state, v.stuck, v.upside_down, v.grace_until);
        match grace0 {
            Some(g0) if v.grace_until != g0 => {
                got = Some(Respawned { at: t, raceline_off: raceline_distance(aiw, &v4(p)), up_y: w.body_up(1)[1] });
                eprintln!("respawn at {t:.2}s pos {p:?} state {:?} grace {:.2}", v.state, v.grace_until);
                return false;
            }
            None => grace0 = Some(v.grace_until),
            _ => {}
        }
        disturb(w, first);
        first = false;
        false
    });
    if got.is_none() {
        eprintln!("no respawn; last: {last_state}");
    }
    got.map(|r| (r, after))
}

/// an opponent held still (velocities zeroed after every physics frame) is stuck: 0x00020490 marks it once
/// |speed| < 1.5 (stuck time), Recover after 2 s, and 0x000206A0 respawns it 9 s after the stuck time
/// (0x000208D0: on its raceline point, facing along the raceline), after which it drives on.
#[test]
fn stuck_opponent_respawns_on_raceline() {
    let t0 = 20.0;
    let Some((r, after)) = opponent_respawn(t0, &mut |w, _| {
        w.set_linear_velocity(1, [0.0; 3]);
        w.set_angular_velocity(1, [0.0; 3]);
    }) else {
        assert!(velocity_ai::game_dir().is_none(), "the held opponent never respawned");
        return;
    };
    eprintln!("stuck respawn {r:?}, top speed after {after:.1}");
    assert!(r.at - t0 > 9.0 && r.at - t0 < 9.5, "respawned {:.2}s after the hold", r.at - t0);
    assert!(r.raceline_off < 0.5, "respawned {:.2}m off the raceline", r.raceline_off);
    assert!(r.up_y > 0.95, "respawned tilted (up y {:.3})", r.up_y);
    assert!(after > 10.0, "did not drive on after the respawn ({after:.1} m/s)");
}

/// an opponent turned onto its roof: 0x00020700 sees body up . +y <= 0.3 and respawns it after 3 s on its
/// raceline point, upright.
#[test]
fn upside_down_opponent_respawns_on_raceline() {
    let t0 = 20.0;
    let roof = vs_quat_axis_angle(std::f32::consts::PI, &VsVector4::new(0.0, 0.0, 1.0, 1.0));
    let Some((r, after)) = opponent_respawn(t0, &mut |w, first| {
        if first {
            let p = w.position(1);
            w.teleport(1, [p[0], p[1] + 1.5, p[2]], [roof.x, roof.y, roof.z, roof.w], [0.0; 3]);
        }
    }) else {
        assert!(velocity_ai::game_dir().is_none(), "the flipped opponent never respawned");
        return;
    };
    eprintln!("upside-down respawn {r:?}, top speed after {after:.1}");
    assert!(r.at - t0 > 3.0 && r.at - t0 < 3.5, "respawned {:.2}s after the flip", r.at - t0);
    assert!(r.raceline_off < 0.5, "respawned {:.2}m off the raceline", r.raceline_off);
    assert!(r.up_y > 0.95, "respawned tilted (up y {:.3})", r.up_y);
    assert!(after > 10.0, "did not drive on after the respawn ({after:.1} m/s)");
}

/// a timeline explosion (PhysicsManager 0x0019FD80) beside and below the opponent: the ai has no explosion code of
/// its own (TimelineExplosion 0x0017ACD0 reaches no ai function; VE+4876 = 96 from 0x001D1170 is not read by the
/// ai), so only the recover paths (0x00020700 upside down 3 s, 0x00020490/0x000206A0 stuck 9 s) react. A side push
/// (case 1) leaves the car driving on the route. A blast under the car (case 2) throws it ~50 m up onto a ledge
/// 7 m above the road: upside-down clears whenever the body is briefly upright mid-air, and on the ledge the
/// recover loop (back out to 5 m/s, leave, stuck again) resets the stuck time, so the 9 s respawn never fires
/// (original logic, see "Recover loop"); any respawn must land on the raceline, upright.
#[test]
fn exploded_opponent_recovers() {
    if velocity_ai::game_dir().is_none() {
        return;
    }
    // (side offset, height offset, vehicle force, angular force, must drive on)
    for (dx, dy, force, angular, drive_on) in [(3.0f32, -1.0f32, 40.0f32, 4.0f32, true), (0.5, -1.5, 60.0, 20.0, false)] {
        exploded_opponent_case(dx, dy, force, angular, drive_on);
    }
}

fn exploded_opponent_case(dx: f32, dy: f32, force: f32, angular: f32, drive_on: bool) {
    let t0 = 20.0;
    let mut done = false;
    let mut grace0: Option<f32> = None;
    let mut respawned: Option<Respawned> = None;
    let mut kick = 0.0f32;
    let mut end = (0.0f32, 0.0f32, 0.0f32);
    run_lap_with("London", r"Circuit\Race01", 1, t0 + 30.0, &mut |t, aiw, shared| {
        let mut g = shared.lock().unwrap();
        let w = g.as_mut().unwrap();
        let p = w.position(1);
        let v = aiw.race.vehicle(VehicleRef::Ai(0)).unwrap();
        if t < t0 {
            return false;
        }
        if !done {
            done = true;
            grace0 = Some(v.grace_until);
            let v0 = w.velocity(1);
            let e = Explosion {
                position: [p[0] + dx, p[1] + dy, p[2]],
                blast_radius: 20.0,
                force_objects: 8.0,
                angular_force_objects: 1.0,
                force_vehicles: force,
                angular_force_vehicles: angular,
                sector_angle: 0.0,
                ignore_vehicle: -1,
            };
            assert!(w.explosion(&e) >= 1, "explosion pushed nothing");
            let v1 = w.velocity(1);
            kick = ((v1[0] - v0[0]).powi(2) + (v1[1] - v0[1]).powi(2) + (v1[2] - v0[2]).powi(2)).sqrt();
            return false;
        }
        // 0x000208D0 sets the grace time to AI time + 2; leaving Recover (0x000205A0) sets + 4
        if respawned.is_none() && grace0.is_some_and(|g0| v.grace_until != g0) && v.grace_until - t < 3.0 {
            respawned = Some(Respawned { at: t, raceline_off: raceline_distance(aiw, &v4(p)), up_y: w.body_up(1)[1] });
        }
        grace0 = Some(v.grace_until);
        end = (t, raceline_distance(aiw, &v4(p)), w.ai_speed(1).abs());
        if std::env::var("LAPDBG").is_ok() && (t * 24.0).round() as i32 % 12 == 0 {
            eprintln!(
                "t {t:.1} state {:?} fail {} stuck {} upside {} grace {:.2} up {:.2} spd {:.1} pos {p:?} off {:.1}",
                v.state, v.failure, v.stuck, v.upside_down, v.grace_until, w.body_up(1)[1], w.ai_speed(1), end.1
            );
        }
        false
    });
    eprintln!("blast ({dx}, {dy}) force {force} angular {angular}: explosion kick {kick:.1} m/s, respawn {respawned:?}, end t {:.1} raceline {:.1} m speed {:.1}", end.0, end.1, end.2);
    assert!(kick > 5.0, "the explosion barely moved the car ({kick:.2} m/s)");
    assert!(end.1.is_finite() && end.2.is_finite(), "NaN after the explosion");
    if let Some(r) = respawned {
        assert!(r.raceline_off < 0.5 && r.up_y > 0.95, "bad respawn {r:?}");
    }
    if drive_on {
        assert!(end.1 < 15.0, "{:.1} m off the raceline at the end", end.1);
        assert!(end.2 > 10.0, "not driving at the end ({:.1} m/s)", end.2);
    }
}

/// three opponents (AI config) and the AI-driven player: no blow-ups, and the opponents complete laps.
#[test]
fn ai_pack_drives_physics_cars_london_circuit() {
    let r = run_lap("London", r"Circuit\Race01", 3);
    if r.is_empty() {
        return;
    }
    assert!(r.iter().all(|c| c.finite), "NaN in the physics");
    let laps = r[1..].iter().filter(|c| c.lap.is_some()).count();
    assert!(laps == 3, "only {laps} of 3 opponents completed a lap");
}
