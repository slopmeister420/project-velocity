//! The `velocity_ai` traits on the physics world: AI handle = physics vehicle id. Every call goes straight to
//! the shared world, except the controls block, which the AI writes through `controls()` and
//! [`PhysicsAi::flush_controls`] copies into the cars (VE+0x9C) before the physics frame.
//! glue (not an original file), moved from velocity_physics tests/ai_support.


use crate::ai::ai_context::AiPhysicsWorld;
use crate::ai::vs_ai_tools::AiCollisionQuery;
use crate::ai::vs_ai_vehicle::{AiPhysicsVehicle, VehicleControls};
use crate::ai::vs_math::{VsRand, VsVector4};
use velocity_physics::vehicle::vehicle_entity::sim::FrictionSpot;
use velocity_physics::plugin::SharedPhysicsWorld;
use velocity_physics::vehicle::vehicle_entity::cx::{Cx, Ptr};
use velocity_physics::vehicle::vehicle_entity::sim::PhysicsWorld;
use velocity_physics::vehicle::vehicle_entity::sim_ai::VehicleControlsRaw;

fn v3(v: &VsVector4) -> [f32; 3] {
    [v.x, v.y, v.z]
}
fn v4(a: [f32; 3], w: f32) -> VsVector4 {
    VsVector4::new(a[0], a[1], a[2], w)
}
fn row(r: [f32; 4]) -> VsVector4 {
    VsVector4::new(r[0], r[1], r[2], 1.0)
}

pub struct CarProxy {
    world: SharedPhysicsWorld,
    id: usize,
    /// copy of VE+0x9C, reloaded by `load_controls`: the original AI writes the car's block in place
    controls: VehicleControls,
    /// `controls()` was taken since the last load: only these blocks are written back
    dirty: bool,
}

impl CarProxy {
    fn w<R>(&self, f: impl FnOnce(&mut PhysicsWorld) -> R) -> R {
        let mut g = self.world.lock().unwrap_or_else(|e| e.into_inner());
        f(g.as_mut().expect("physics world"))
    }
}

pub fn to_raw(c: &VehicleControls) -> VehicleControlsRaw {
    VehicleControlsRaw {
        steer: c.steer,
        unk_04: c.unk_04,
        throttle: c.throttle,
        brake: c.brake,
        handbrake: c.handbrake,
        nitrous: c.nitrous,
        horn: c.horn,
        unk_18: c.unk_18,
        unk_19: c.unk_19,
        unk_1b: c.unk_1b,
        unk_1c: c.unk_1c,
    }
}

pub fn from_raw(c: &VehicleControlsRaw) -> VehicleControls {
    VehicleControls {
        steer: c.steer,
        unk_04: c.unk_04,
        throttle: c.throttle,
        brake: c.brake,
        handbrake: c.handbrake,
        nitrous: c.nitrous,
        horn: c.horn,
        unk_18: c.unk_18,
        unk_19: c.unk_19,
        unk_1b: c.unk_1b,
        unk_1c: c.unk_1c,
    }
}

impl AiPhysicsVehicle for CarProxy {
    fn position(&self) -> VsVector4 {
        v4(self.w(|w| w.position(self.id)), 1.0)
    }
    fn orientation(&self) -> VsVector4 {
        let q = self.w(|w| w.vs_orientation(self.id));
        VsVector4::new(q[0], q[1], q[2], q[3])
    }
    fn forward(&self) -> VsVector4 {
        let r = self.w(|w| w.body_row(self.id, 0));
        VsVector4::new(r[0], r[1], r[2], r[3])
    }
    fn axis_right(&self) -> VsVector4 {
        row(self.w(|w| w.body_row(self.id, 2)))
    }
    fn axis_up(&self) -> VsVector4 {
        row(self.w(|w| w.body_row(self.id, 1)))
    }
    fn axis_forward(&self) -> VsVector4 {
        row(self.w(|w| w.body_row(self.id, 0)))
    }
    fn velocity(&self) -> VsVector4 {
        v4(self.w(|w| w.velocity(self.id)), 0.0)
    }
    fn speed(&self) -> f32 {
        self.w(|w| w.ai_speed(self.id))
    }
    fn controls(&mut self) -> &mut VehicleControls {
        self.dirty = true;
        &mut self.controls
    }
    fn controls_ref(&self) -> &VehicleControls {
        &self.controls
    }
    fn upside_down_exempt(&self) -> bool {
        // VsAIRaceVehicle.cpp 0x00020721: `cmp dword [ve+1294h], 1` then `comiss 10.0, [ve+11F4h]` (float flight
        // time); physics' `upside_down_exempt` compares +0x11F4 as an int (asked in docs/physics.md)
        self.w(|w| {
            let m = &w.slot(self.id).ve.m;
            m.i(4756) == 1 && m.f(4596) < 10.0
        })
    }
    fn teleport(&mut self, pos: &VsVector4, orientation: &VsVector4, velocity: &VsVector4) {
        let q = [orientation.x, orientation.y, orientation.z, orientation.w];
        self.w(|w| w.teleport(self.id, v3(pos), q, v3(velocity)));
    }
    fn reset_state(&mut self) {
        self.w(|w| w.activate(self.id));
    }
    fn deactivate(&mut self) {
        self.w(|w| w.deactivate(self.id));
    }
    fn set_steer_angle(&mut self, angle: f32) {
        self.w(|w| w.set_steer_angle(self.id, angle));
    }
    fn set_linear_velocity(&mut self, v: &VsVector4) {
        self.w(|w| w.set_linear_velocity(self.id, v3(v)));
    }
    fn set_angular_velocity(&mut self, v: &VsVector4) {
        self.w(|w| w.set_angular_velocity(self.id, v3(v)));
    }
    fn apply_angular_impulse(&mut self, v: &VsVector4) {
        self.w(|w| w.apply_angular_impulse(self.id, v3(v)));
    }
    fn set_grip_scale(&mut self, s: f32) {
        self.w(|w| w.set_grip_scale(self.id, s));
    }
    fn control_mode(&self) -> u32 {
        self.w(|w| w.control_mode(self.id))
    }
    fn set_control_mode(&mut self, m: u32) {
        self.w(|w| w.set_control_mode(self.id, m));
    }
    fn engine_rpm(&self) -> i32 {
        self.w(|w| w.engine_rpm(self.id))
    }
    fn gear(&self) -> i32 {
        self.w(|w| w.ai_gear(self.id))
    }
    fn nos_remaining(&self) -> f32 {
        self.w(|w| w.nos_remaining(self.id))
    }
    fn nos_capacity(&self) -> f32 {
        self.w(|w| w.nos_capacity(self.id))
    }
    fn puncture(&mut self, a: f32, b: f32) {
        self.w(|w| w.puncture(self.id, a, b));
    }
    fn body_up(&self) -> VsVector4 {
        v4(self.w(|w| w.body_up(self.id)), 0.0)
    }
    fn forward_speed(&self) -> f32 {
        self.w(|w| w.forward_speed(self.id))
    }
    fn drive_state(&self) -> i32 {
        self.w(|w| w.drive_state(self.id))
    }
    fn is_reversing(&self) -> bool {
        self.w(|w| w.is_reversing(self.id))
    }
    fn is_airborne(&self) -> bool {
        self.w(|w| w.is_airborne(self.id))
    }
    fn nos_burning(&self) -> bool {
        self.w(|w| w.nos_burning(self.id))
    }
    fn reset_dynamics(&mut self) {
        self.w(|w| w.reset_dynamics(self.id));
    }
    fn set_direct_steer(&mut self, on: bool) {
        self.w(|w| w.set_wheel_lock(self.id, on));
    }
    fn set_launch_control(&mut self, on: bool) {
        self.w(|w| w.set_launch_control(self.id, on));
    }
    fn lock_wheels(&mut self, v: i32) {
        self.w(|w| w.lock_wheels(self.id, v));
    }
    fn shift_flags(&self) -> (bool, bool) {
        self.w(|w| w.shift_flags(self.id))
    }
    fn gear_count(&self) -> i32 {
        self.w(|w| w.gear_count(self.id))
    }
    fn engine_speed(&self) -> f32 {
        self.w(|w| w.engine_speed(self.id))
    }
    fn shift(&mut self, delta: i32) {
        self.w(|w| w.shift(self.id, delta));
    }
    fn launch_params(&self) -> [f32; 3] {
        self.w(|w| w.launch_params(self.id))
    }
    fn set_launch_params(&mut self, p: [f32; 3]) {
        self.w(|w| w.set_launch_params(self.id, p));
    }
    fn respawn_wheel_heights(&self) -> (f32, f32) {
        // VE +0x810 = body +0x720; 0x0035FB40(0) = body float[451] = +0x70C
        self.w(|w| {
            let m = &w.slot(self.id).car.m;
            (m.f(1824), m.f(1804))
        })
    }
    fn reset_history_scale(&mut self) {
        // VehicleEntity.cpp 0x001D1810: VsConfigFileRead 0x001B6E80(VE+0xDE8, 0) = ring base + 208 * head
        // (VE+0xDEC; the `0 > count` clamp never applies), record +0x64 = 1.0
        self.w(|w| {
            let ve = &mut w.vehs[self.id].as_mut().expect("no such vehicle").ve;
            let head = ve.m.i(3564);
            ve.hist.set_f((208 * head + 100) as usize, 1.0);
        })
    }
    fn commit_body(&mut self) {
        // car vtbl +0x20 (0x00362930, ov::car_vcall slot 32)
        use velocity_physics::fastcar::rigid_body::RigidBody;
        self.w(|w| w.vehs[self.id].as_mut().expect("no such vehicle").car.update_derived());
    }
    fn wheel_contact_normal(&self, i: usize) -> VsVector4 {
        v4(self.w(|w| w.wheel(self.id, i).contact_normal), 0.0)
    }
    fn set_score_gains(&mut self, gains: Option<(f32, f32)>) {
        self.w(|w| w.set_score_gains(self.id, gains));
    }
    fn launch_result(&self) -> i32 {
        self.w(|w| w.launch_result(self.id))
    }
    fn shift_result(&self) -> i32 {
        self.w(|w| w.shift_result(self.id))
    }
    fn clear_shift_result(&mut self) {
        self.w(|w| w.clear_shift_result(self.id));
    }
    fn set_traffic_index(&mut self, i: i32) {
        self.w(|w| w.set_traffic_index(self.id, i));
    }
    fn police_handling(&mut self) {
        self.w(|w| w.police_handling(self.id));
    }
}

/// all physics cars by AI handle (= vehicle id)
pub struct PhysicsAi {
    world: SharedPhysicsWorld,
    cars: Vec<CarProxy>,
}

impl PhysicsAi {
    pub fn new(world: SharedPhysicsWorld) -> Self {
        Self { world, cars: Vec::new() }
    }
    fn w<R>(&self, f: impl FnOnce(&mut PhysicsWorld) -> R) -> R {
        let mut g = self.world.lock().unwrap_or_else(|e| e.into_inner());
        f(g.as_mut().expect("physics world"))
    }
    /// proxies for every vehicle id up to the world's last
    pub fn sync_cars(&mut self) {
        let n = self.w(|w| w.vehicle_ids().last().map_or(0, |i| i + 1));
        while self.cars.len() < n {
            let id = self.cars.len();
            self.cars.push(CarProxy { world: self.world.clone(), id, controls: VehicleControls::default(), dirty: false });
        }
    }
    /// every car's VE+0x9C into its proxy (the original AI reads and writes the block in place, so the pad
    /// smoothing state 0x001B82A0 keeps in a human car's block must not be replaced by a stale copy)
    pub fn load_controls(&mut self) {
        self.sync_cars();
        let ids = self.w(|w| w.vehicle_ids());
        for c in &mut self.cars {
            c.dirty = false;
            if ids.contains(&c.id) {
                let raw = c.w(|w| w.controls(c.id));
                c.controls = from_raw(&raw);
            }
        }
    }
    /// the blocks the AI took through `controls()` back into their cars (VE+0x9C), before the physics frame
    pub fn flush_controls(&mut self) {
        let raw: Vec<(usize, VehicleControlsRaw)> =
            self.cars.iter_mut().filter_map(|c| std::mem::take(&mut c.dirty).then(|| (c.id, to_raw(&c.controls)))).collect();
        self.w(|w| {
            for (id, c) in &raw {
                if w.vehicle_ids().contains(id) {
                    w.set_controls(*id, c);
                }
            }
        });
    }
}

impl AiCollisionQuery for PhysicsAi {
    /// MaterialProperties.cpp 0x0019D350 with `a5 = 0`, no car ignored
    fn line_check(&self, start: &VsVector4, end: &VsVector4) -> Option<f32> {
        self.w(|w| w.line_check(v3(start), v3(end), false, -1)).map(|h| h.0)
    }
    /// PhysicsManager 0x0019D3E0 with the caller's flags
    fn line_check_normal(&self, start: &VsVector4, end: &VsVector4, mask: u32) -> Option<(f32, VsVector4)> {
        self.w(|w| w.ray_cast(v3(start), v3(end), mask as i32, -1)).map(|h| (h.t, v4(h.normal, 0.0)))
    }
}

impl AiPhysicsWorld for PhysicsAi {
    fn vehicle(&mut self, handle: usize) -> &mut dyn AiPhysicsVehicle {
        self.sync_cars();
        &mut self.cars[handle]
    }
    fn vehicle_ref(&self, handle: usize) -> &dyn AiPhysicsVehicle {
        assert!(handle < self.cars.len(), "physics vehicle {handle} not synced");
        &self.cars[handle]
    }
    fn slipstream_source(&self, handle: usize) -> Option<usize> {
        self.w(|w| w.slipstream_source(handle))
    }
    fn steer_angle_to(&self, handle: usize, target: &VsVector4) -> f32 {
        self.w(|w| w.steer_angle_to(handle, v3(target)))
    }
    fn ready(&self) -> bool {
        self.world.lock().unwrap_or_else(|e| e.into_inner()).is_some()
    }
    fn has_vehicle(&self, handle: usize) -> bool {
        self.ready() && self.w(|w| w.vehicle_ids().contains(&handle))
    }
    /// `::GC` +0x84, +0x90, +0xD4, +0xD0, +0xCC and `dword_686B58` as the physics code holds them
    fn trail_blazer_globals(&self) -> [f32; 6] {
        if !self.ready() {
            return [0.0; 6];
        }
        self.w(|w| {
            let cx = Cx::new(&w.gc, &w.gcm, &mut w.glob);
            [
                cx.f(Ptr::gc(0x84)),
                cx.f(Ptr::gc(0x90)),
                cx.f(Ptr::gc(0xD4)),
                cx.f(Ptr::gc(0xD0)),
                cx.f(Ptr::gc(0xCC)),
                cx.f(Ptr::glob(0x0068_6B58)),
            ]
        })
    }
    /// PhysicsObjectManager 0x001E18E0 on the physics world, drawing from the ai's (shared) rand stream
    fn friction_spot(&mut self, pos: &VsVector4, friction: f32, radius: f32, race_distance: f32, ignore_car: i32, expiry: f32, rand: &mut VsRand) -> bool {
        if !self.ready() {
            return false;
        }
        let spot = FrictionSpot { position: v3(pos), friction_high: friction, radius, race_distance, ignore_car, expiry };
        self.w(|w| {
            w.set_rand_state(rand.seed);
            let hit = w.friction_spot(&spot);
            rand.seed = w.rand_state();
            hit
        })
    }
    fn end_update(&mut self, human_control: &[(usize, bool)]) {
        if !self.ready() {
            return;
        }
        self.sync_cars();
        self.w(|w| {
            for &(id, h) in human_control {
                if w.vehicle_ids().contains(&id) {
                    w.set_human_control(id, h);
                }
            }
        });
        self.flush_controls();
    }
    fn begin_access(&mut self) {
        if self.ready() {
            self.load_controls();
        }
    }
    fn end_access(&mut self) {
        if self.ready() {
            self.flush_controls();
        }
    }
}
