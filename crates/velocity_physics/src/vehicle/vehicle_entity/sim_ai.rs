//! What the AI reaches on a physics car (`VsAIVehicle` -> `VehicleEntity` / its `CFastCar`), and the
//! collision messages the physics frame posts. Plain Rust: the `velocity_ai` traits are implemented on top
//! of these by whoever links both crates.

use super::cx::Ptr;
use super::sim::{make_cx, PhysicsWorld, RAY_NORMAL, RAY_OBJECTS, RAY_CARS, RAY_TRACK};
use crate::tr_prelude::*;

/// The controls block the AI drives a car with (`VehicleEntity +0x9C`, VsAIVehicle.cpp 0x0002C7B0); read
/// by `VehicleEntity__1CB470` when the car is not human-driven.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VehicleControlsRaw {
    pub steer: f32,     // +0x00
    pub unk_04: f32,    // +0x04
    pub throttle: f32,  // +0x08
    pub brake: f32,     // +0x0C
    pub handbrake: f32, // +0x10
    pub nitrous: bool,  // +0x14
    pub horn: bool,     // +0x15 (1CB470 zeroes the throttle while it is set)
    pub unk_18: bool,   // +0x18
    pub unk_19: u16,    // +0x19
    pub unk_1b: bool,   // +0x1B
    pub unk_1c: bool,   // +0x1C
}

/// `VehicleEntity +0x9C`
const CONTROLS: usize = 156;

/// A collision the physics frame reported (`MessageManager` message 0, posted by the car-vs-car pass
/// 0x001A3A30 and the car-vs-track pass 0x001A0570).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarCollision {
    /// vehicle ids; `b` = -1 for scenery, -2 for a barrier / Armco wall (0x001A7060)
    pub a: i32,
    pub b: i32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
    /// word 10: the impact strength
    pub impact: f32,
    /// +56: the contact impulse vector in world space (PhysicsManager 0x001B39B0: contact frame x impulse)
    pub impulse: [f32; 3],
    /// +72: spark scale (5.0 car-car, 8.0 track, 30.0 walls / objects)
    pub scale: f32,
    /// +76 (byte): surface material of the contact record (+1556), -1 car-car / walls
    pub material: i8,
}

impl PhysicsWorld {
    fn ve_mem(&self, id: usize) -> &crate::fastcar::mem::Mem {
        &self.slot(id).ve.m
    }

    /// who drives the car: the pad (human) or the controls block
    pub fn set_human_control(&mut self, id: usize, human: bool) {
        if let Some(v) = self.vehs.get_mut(id).and_then(|v| v.as_mut()) {
            v.human = human;
        }
    }
    pub fn human_control(&self, id: usize) -> bool {
        self.slot(id).human
    }

    /// VsAIVehicle.cpp 0x0002C7B0: the controls block
    pub fn controls(&self, id: usize) -> VehicleControlsRaw {
        let m = self.ve_mem(id);
        let o = CONTROLS;
        VehicleControlsRaw {
            steer: m.f(o),
            unk_04: m.f(o + 4),
            throttle: m.f(o + 8),
            brake: m.f(o + 12),
            handbrake: m.f(o + 16),
            nitrous: m.b(o + 20) != 0,
            horn: m.b(o + 21) != 0,
            unk_18: m.b(o + 24) != 0,
            unk_19: m.b(o + 25) as u16 | (m.b(o + 26) as u16) << 8,
            unk_1b: m.b(o + 27) != 0,
            unk_1c: m.b(o + 28) != 0,
        }
    }
    pub fn set_controls(&mut self, id: usize, c: &VehicleControlsRaw) {
        let m = &mut self.vehs[id].as_mut().expect("no such vehicle").ve.m;
        let o = CONTROLS;
        m.set_f(o, c.steer);
        m.set_f(o + 4, c.unk_04);
        m.set_f(o + 8, c.throttle);
        m.set_f(o + 12, c.brake);
        m.set_f(o + 16, c.handbrake);
        m.set_b(o + 20, c.nitrous as u8);
        m.set_b(o + 21, c.horn as u8);
        m.set_b(o + 24, c.unk_18 as u8);
        m.set_b(o + 25, c.unk_19 as u8);
        m.set_b(o + 26, (c.unk_19 >> 8) as u8);
        m.set_b(o + 27, c.unk_1b as u8);
        m.set_b(o + 28, c.unk_1c as u8);
    }

    /// VehicleEntity.cpp 0x001BB230: orientation as a VS quaternion (the conjugate of the body's)
    pub fn vs_orientation(&mut self, id: usize) -> [f32; 4] {
        let state = self.ve_mem(id).i(4756);
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let v = Ptr::ve(id);
        c.push_frame(80);
        let (m, q) = (c.lptr(4), c.lptr(48));
        if state == 1 {
            f_1bb100(&mut c, v, m);
            f_39d0d0(&mut c, q, m);
        } else {
            f_36a140(&mut c, Ptr::car(id), q);
        }
        let r = [0.0 - c.f(q), 0.0 - c.f(q.add(4)), 0.0 - c.f(q.add(8)), c.f(q.add(12))];
        c.pop_frame();
        r
    }

    /// VehicleEntity.cpp 0x001BB200: row `k` of the body matrix (0 forward, 1 up, 2 right)
    pub fn body_row(&self, id: usize, k: usize) -> [f32; 4] {
        let m = self.ve_mem(id);
        std::array::from_fn(|i| m.f(4788 + 16 * k + 4 * i))
    }
    /// `VehicleEntity +0x14`: unsigned speed
    pub fn ai_speed(&self, id: usize) -> f32 {
        self.ve_mem(id).f(20)
    }
    /// VsAIRaceVehicle.cpp 0x00024870: velocity along the body's x row
    pub fn forward_speed(&self, id: usize) -> f32 {
        let car = &self.slot(id).car.m;
        let v = car.v3(crate::fastcar::rigid_body::off::VELOCITY);
        let r = car.v3(crate::fastcar::rigid_body::off::ROTATION);
        ((r[0] as f64 * v[0] as f64) + (r[1] as f64 * v[1] as f64) + (r[2] as f64 * v[2] as f64)) as f32
    }
    /// `body +0x80`: the body's up axis
    pub fn body_up(&self, id: usize) -> [f32; 3] {
        self.slot(id).car.m.v3(128)
    }
    /// `dword_6D4D98`, the audio's top of the rev range that 0x001B6F10 scales into (VsAudioEvents::
    /// StartPlayerEngine 0x0004C3A0 sets 6000; zero in the xbe image, which turns the scale upside down)
    pub fn set_audio_max_rpm(&mut self, rpm: i32) {
        self.glob.w.insert(0x006D_4D98, rpm as u32);
    }
    /// ZoneRender `flt_737AB0` +24 + 4·pad (pads 0, 1): the zone FX level (graphics' `ZoneFx`, 0..2) that
    /// 0x00154FC0 halves; while non-zero VsConfigFileRead 0x001B82A0 clears the pad's boost flag (+20). Mirror it
    /// before each step
    pub fn set_zone_fx_level(&mut self, pad: usize, level: f32) {
        if pad < 2 {
            self.glob.w.insert(0x0073_7AB0 + 24 + 4 * pad as i32, level.to_bits());
        }
    }
    /// the jump cam's `ReplayCamera_Manager` calls (0x001665E0(1) / 0x001665A0 from VehicleEntity 0x001CCAD0 /
    /// 0x001CF0D0); the hook runs inside `step` on the physics thread, so it must not lock this world: it reads the
    /// car position (0x001BA270) and the camera ray (0x00167680) through its `JumpCamWorld` argument. None = no
    /// camera (`CanSee` false, the jump cam never starts)
    pub fn set_jump_cam_hook(&mut self, hook: Option<std::sync::Arc<std::sync::Mutex<super::cx::JumpCamFn>>>) {
        self.glob.jump_cam = super::cx::JumpCamHook(hook);
    }
    /// `byte_75B278`, the "Jump Cam" video option (0x001CCAD0 only; 0x001CF0D0 does not check it)
    pub fn set_jump_cam_option(&mut self, on: bool) {
        let w = self.glob.word(0x0075_B278);
        self.glob.w.insert(0x0075_B278, (w & !0xFF) | on as u32);
    }
    /// `dword_749110` ("Enable Cameras For Review"; 2 = jump cam), the physics copy the jump cam writes and
    /// 0x001C5800 / 0x001CF0D0 read: sync it with the replay manager's before and after `step`
    pub fn review_cameras(&self) -> i32 {
        self.glob.word(0x0074_9110) as i32
    }
    pub fn set_review_cameras(&mut self, v: i32) {
        self.glob.w.insert(0x0074_9110, v as u32);
    }
    /// (`dword_749118` countdown, `dword_74911C` start) = flight time - `dword_6889FC` when the jump cam starts;
    /// InGameUpdate 0x000B8329 counts 749118 down (core) and 0x001C5800 / 0x001CF0D0 read both (slow motion)
    pub fn review_timer(&self) -> (f32, f32) {
        let g = |a| f32::from_bits(self.glob.word(a));
        (g(0x0074_9118), g(0x0074_911C))
    }
    pub fn set_review_timer(&mut self, countdown: f32) {
        self.glob.w.insert(0x0074_9118, countdown.to_bits());
    }
    /// `dword_68580C`, the game update rate (24.0) that 0x001C5800 lerps down during the jump cam; read by core's
    /// clock (VsCommonMain 0x0016E17C, FontHelper 0x000B38AC), reset to 24.0 by InGameUpdate 0x000B836B
    pub fn jump_cam_rate(&self) -> f32 {
        f32::from_bits(self.glob.word(0x0068_580C))
    }
    /// InGameUpdate 0x000B836B (countdown end) / FrontEndInit 0x000B5627 / InGameInit 0x000B6099 write 24.0 here
    pub fn set_jump_cam_rate(&mut self, rate: f32) {
        self.glob.w.insert(0x0068_580C, rate.to_bits());
    }
    /// CRT `rand` state (0x003DE030) the translated code draws from: 0x001A0530, 0x001CF070 (VsRandUnit),
    /// 0x001CF090, 0x001DFBB0, all inside `step`; hand the game's `CrtRand::state` in before a step, take it back after
    pub fn rand_state(&self) -> u32 {
        self.glob.rand_state
    }
    pub fn set_rand_state(&mut self, state: u32) {
        self.glob.rand_state = state;
    }
    /// VsConfigFileRead.cpp 0x001B6F10 — the rpm the HUD (COSDInGameScreen::Update 0x000F9D72), the engine
    /// audio (VsVehicleAudio::UpdateRpm 0x00052401), CarRender 0x0011ABD5 and 0x001BA7A3 read; smooths VE+4068
    /// on every call, so call it where the original does
    pub fn engine_rpm(&mut self, id: usize) -> i32 {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_1b6f10(&mut c, Ptr::ve(id)) as i32
    }
    /// VsConfigFileRead.cpp 0x001B74A0
    pub fn ai_gear(&mut self, id: usize) -> i32 {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_1b74a0(&mut c, Ptr::ve(id))
    }
    /// `body 0x00364BE0`: transmission state (4 = driving forward)
    pub fn drive_state(&mut self, id: usize) -> i32 {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_364be0(&mut c, Ptr::car(id))
    }
    /// `body 0x00364B20`: reverse selected
    pub fn is_reversing(&mut self, id: usize) -> bool {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_364b20(&mut c, Ptr::car(id)) != 0
    }
    /// `body +0xB3C`: all wheels off the ground
    pub fn is_airborne(&self, id: usize) -> bool {
        self.slot(id).car.m.i(2876) != 0
    }
    /// `VehicleEntity +0x58`: nitrous burning
    pub fn nos_burning(&self, id: usize) -> bool {
        self.ve_mem(id).b(88) != 0
    }
    /// VsAIRaceVehicle.cpp 0x000248F0 (VehicleEntity.h inline: m_NosRemaining +0x60, in frames)
    pub fn nos_remaining(&self, id: usize) -> f32 {
        self.ve_mem(id).f(96)
    }
    /// `VehicleEntity +0x64`: m_NosCapacity (frames)
    pub fn nos_capacity(&self, id: usize) -> f32 {
        self.ve_mem(id).f(100)
    }
    /// VsAIRaceVehicle.cpp 0x00024720 (inline; x87 fdiv, the "m_NosRemaining<=m_NosCapacity" assert is not kept)
    pub fn nos_ratio(&self, id: usize) -> f64 {
        let m = self.ve_mem(id);
        m.f(96) as f64 / m.f(100) as f64
    }
    /// `VehicleEntity +0x68`: m_ZoneRemaining (frames)
    pub fn zone_remaining(&self, id: usize) -> f32 {
        self.ve_mem(id).f(104)
    }
    /// `VehicleEntity +0x6C`: m_ZoneCapacity (frames)
    pub fn zone_capacity(&self, id: usize) -> f32 {
        self.ve_mem(id).f(108)
    }
    /// Speedo.cpp 0x000F78D0 (inline; x87 fdiv, the "m_ZoneRemaining<=m_ZoneCapacity" assert is not kept)
    pub fn zone_ratio(&self, id: usize) -> f64 {
        let m = self.ve_mem(id);
        m.f(104) as f64 / m.f(108) as f64
    }
    /// Speedo.cpp 0x000F7970 (`VehicleEntity +0x90` byte, returned via fild: 0.0 / 1.0)
    pub fn has_turbo(&self, id: usize) -> f64 {
        self.ve_mem(id).b(144) as f64
    }
    /// Speedo.cpp 0x000F79A0 (`VehicleEntity +0x94`, written by PhysicsCarInput 0x001BA770)
    pub fn turbo_ratio(&self, id: usize) -> f64 {
        self.ve_mem(id).f(148) as f64
    }
    /// `VehicleEntity +0x1294 == 1 && +0x11F4 < 10`
    pub fn upside_down_exempt(&self, id: usize) -> bool {
        let m = self.ve_mem(id);
        m.i(4756) == 1 && m.i(4596) < 10
    }
    /// `VehicleEntity +0x13DC`: who drives the car (0 player, 1 ai, 2 traffic, ..., 6 none)
    pub fn control_mode(&self, id: usize) -> u32 {
        self.ve_mem(id).i(5084) as u32
    }
    pub fn set_control_mode(&mut self, id: usize, m: u32) {
        self.vehs[id].as_mut().expect("no such vehicle").ve.m.set_i(5084, m as i32);
    }
    /// `VehicleEntity +0x1310`: grip multiplier
    pub fn set_grip_scale(&mut self, id: usize, s: f32) {
        self.vehs[id].as_mut().expect("no such vehicle").ve.m.set_f(4880, s);
    }
    /// VehicleEntity.cpp 0x001BAEA0: the car whose slipstream this one is in (`+0x54`, -1 none)
    pub fn slipstream_source(&self, id: usize) -> Option<usize> {
        let i = self.ve_mem(id).i(84);
        (i != -1).then_some(i as usize)
    }

    /// VsAIVehicle.cpp 0x0002BDA0, the body part: zeroed state (car vtbl +0x14), orientation (conjugate of the
    /// VS quaternion, 0x0036A370), position (0x0036B130), linear (0x0036B530) and zero angular (0x0036B730)
    /// velocity, commit (vtbl +0x20); then `VehicleEntity::Activate` 0x001C5710
    pub fn teleport(&mut self, id: usize, pos: [f32; 3], q: [f32; 4], vel: [f32; 3]) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let car = Ptr::car(id);
        c.push_frame(88);
        c.memset(c.lptr(0), 0, 88);
        let st = c.lptr(0);
        super::ov::car_vcall(&mut c, 20, car, &[Varg::P(st)]);
        c.pop_frame();
        let b = |x: f32| x.to_bits() as i32;
        f_36a370(&mut c, car, b(q[3]), b(0.0 - q[0]), b(0.0 - q[1]), b(0.0 - q[2]));
        f_36b130(&mut c, car, b(pos[0]), b(pos[1]), b(pos[2]));
        f_36b530(&mut c, car, b(vel[0]), b(vel[1]), b(vel[2]));
        f_36b730(&mut c, car, 0, 0, 0);
        super::ov::car_vcall(&mut c, 32, car, &[]);
        drop(c);
        self.activate(id);
    }
    /// VehicleEntity.cpp 0x001C5710 (of the `SyncModels` 0x001C1FF0 it calls, the physics part `sync_model`)
    pub fn activate(&mut self, id: usize) {
        let m = &mut self.vehs[id].as_mut().expect("no such vehicle").ve.m;
        if m.b(28) == 0 {
            m.set_b(28, 1);
            self.sync_model(id);
        }
    }
    /// VehicleEntity.cpp 0x001C5750
    pub fn deactivate(&mut self, id: usize) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_1c5750(&mut c, Ptr::ve(id));
    }
    pub fn is_active(&self, id: usize) -> bool {
        self.ve_mem(id).b(28) != 0
    }

    /// `body 0x00361FF0`: Ackermann steering angle on the front wheels
    pub fn set_steer_angle(&mut self, id: usize, a: f32) {
        self.vehs[id].as_mut().expect("no such vehicle").car.set_steering(a);
    }
    /// the car's steering angle (`CFastCar +0x75C`)
    pub fn steer_angle(&self, id: usize) -> f32 {
        self.slot(id).car.m.f(crate::fastcar::car::off::STEERING)
    }
    /// `body 0x003717D0`: the steering angle that points the front wheels at `target`
    pub fn steer_angle_to(&mut self, id: usize, target: [f32; 3]) -> f32 {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        c.push_frame(16);
        let t = c.lptr(0);
        c.set_v3(t, target);
        c.set_f(t.add(12), 1.0);
        let a = f_3717d0(&mut c, Ptr::car(id), t);
        c.pop_frame();
        a
    }
    /// `body 0x0036B530`
    pub fn set_linear_velocity(&mut self, id: usize, v: [f32; 3]) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let b = |x: f32| x.to_bits() as i32;
        f_36b530(&mut c, Ptr::car(id), b(v[0]), b(v[1]), b(v[2]));
    }
    /// `body 0x0036B730`
    pub fn set_angular_velocity(&mut self, id: usize, v: [f32; 3]) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let b = |x: f32| x.to_bits() as i32;
        f_36b730(&mut c, Ptr::car(id), b(v[0]), b(v[1]), b(v[2]));
    }
    /// `body 0x0036C3E0`: angular impulse
    pub fn apply_angular_impulse(&mut self, id: usize, v: [f32; 3]) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        c.push_frame(16);
        let p = c.lptr(0);
        c.set_v3(p, v);
        c.set_f(p.add(12), 1.0);
        f_36c3e0(&mut c, Ptr::car(id), p);
        c.pop_frame();
    }
    /// VehicleEntity.cpp 0x001D11D0: tyre puncture
    pub fn puncture(&mut self, id: usize, a: f32, b: f32) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_1d11d0(&mut c, Ptr::ve(id), a.to_bits() as i32, b.to_bits() as i32);
    }

    /// VsAITrafficVehicle.cpp 0x0002B800, the physics part: clear forces and velocities, unlock the wheels,
    /// zeroed car state, `VehicleEntity__1CB420`
    pub fn reset_dynamics(&mut self, id: usize) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let (v, car) = (Ptr::ve(id), Ptr::car(id));
        f_1d1120(&mut c, v);
        f_1cc380(&mut c, v);
        f_36b530(&mut c, car, 0, 0, 0);
        f_36b730(&mut c, car, 0, 0, 0);
        f_35dcc0(&mut c, car, 0);
        f_36b9d0(&mut c, car);
        f_36b880(&mut c, car);
        f_36ba40(&mut c, car);
        f_36c230(&mut c, car);
        f_36c1c0(&mut c, car);
        f_36c2a0(&mut c, car);
        c.push_frame(88);
        c.memset(c.lptr(0), 0, 88);
        let st = c.lptr(0);
        super::ov::car_vcall(&mut c, 20, car, &[Varg::P(st)]);
        c.pop_frame();
        f_1cb420(&mut c, v);
    }
    /// `body 0x0035DCC0`: lock the wheels (the grid hold) or release them
    pub fn set_wheel_lock(&mut self, id: usize, on: bool) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_35dcc0(&mut c, Ptr::car(id), on as i32);
    }
    /// `VehicleEntity +0x134C`: launch control
    pub fn set_launch_control(&mut self, id: usize, on: bool) {
        self.vehs[id].as_mut().expect("no such vehicle").ve.m.set_b(4940, on as u8);
    }
    /// `VehicleEntity +0x5C4` = body +0x4D4, the four wheel-locked flags (VsAIRaceVehicle.cpp 0x000192C0, the
    /// post-race stop); VE offsets 240.. are the CFastCar, so this goes to the car's memory
    pub fn lock_wheels(&mut self, id: usize, v: i32) {
        let m = &mut self.vehs[id].as_mut().expect("no such vehicle").car.m;
        for k in 0..4 {
            m.set_i(crate::fastcar::car::off::WHEEL_LOCKED + 4 * k, v);
        }
    }
    /// `VehicleEntity +0xDC9`, `+0xDCA` (VsAIRaceVehicle.cpp 0x00024550): auto up / down shift allowed
    pub fn shift_flags(&self, id: usize) -> (bool, bool) {
        // VE +0xDC9/+0xDCA = body +0xCD9/+0xCDA
        let m = &self.slot(id).car.m;
        (m.b(3289) != 0, m.b(3290) != 0)
    }
    /// `body +0xBDC` (VsAIRaceVehicle.cpp 0x00024530): number of gears
    pub fn gear_count(&self, id: usize) -> i32 {
        self.slot(id).car.m.i(3036)
    }
    /// `VehicleEntity +0xC90` = body +0xBA0: the gear the AI shifts by hand
    pub fn shift(&mut self, id: usize, delta: i32) {
        let m = &mut self.vehs[id].as_mut().expect("no such vehicle").car.m;
        m.set_i(2976, m.i(2976).wrapping_add(delta));
    }
    /// local config +0x6C, +0x10C, +0x128 (VsAIRaceVehicle.cpp 0x0001B5F0 supercar launch)
    pub fn launch_params(&mut self, id: usize) -> [f32; 3] {
        let c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let lc = c.ptr_at(Ptr::ve(id).add(3552));
        [c.f(lc.add(108)), c.f(lc.add(268)), c.f(lc.add(296))]
    }
    pub fn set_launch_params(&mut self, id: usize, p: [f32; 3]) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let lc = c.ptr_at(Ptr::ve(id).add(3552));
        c.set_f(lc.add(108), p[0]);
        c.set_f(lc.add(268), p[1]);
        c.set_f(lc.add(296), p[2]);
    }
    /// `VehicleEntity +0x810` minus `body 0x0035FB40(0)` (VsAIRaceVehicle.cpp respawn)
    pub fn ride_height(&self, id: usize) -> f32 {
        // VE +0x810 = body +0x720 (front-left wheel position y)
        self.slot(id).car.m.f(1824) - self.slot(id).car.m.f(1804)
    }

    /// MaterialProperties.cpp 0x0019D350: `PhysicsManager::RayCast` from `from` to `to` against the track (and
    /// objects and cars with `everything`); fraction and normal of the hit
    pub fn line_check(&mut self, from: [f32; 3], to: [f32; 3], everything: bool, ignore_car: i32) -> Option<(f32, [f32; 3])> {
        let flags = if everything { RAY_TRACK | RAY_OBJECTS | RAY_CARS | RAY_NORMAL } else { RAY_TRACK | RAY_NORMAL };
        self.ray_cast(from, to, flags, ignore_car).map(|h| (h.t, h.normal))
    }

    /// the collision messages posted since the last call (message 0 of `MessageManager__CA2C0`)
    pub fn take_collisions(&mut self) -> Vec<CarCollision> {
        let msgs = std::mem::take(&mut self.glob.messages);
        msgs.into_iter()
            .filter(|(ty, _)| *ty == 0)
            .map(|(_, w)| {
                let f = |k: usize| f32::from_bits(w[k] as u32);
                CarCollision { a: w[0], b: w[1], point: [f(2), f(3), f(4)], normal: [f(6), f(7), f(8)], impact: f(10), impulse: [f(14), f(15), f(16)], scale: f(18), material: w[19] as i8 }
            })
            .collect()
    }
}

/// The physics globals the trailblazer (TrailBlazer.cpp 0x000C4870 / 0x000C4E50 / 0x000C4FF0) saves, drives
/// and restores: `::GC` +0x84, +0x90, +0xD4, +0xD0, +0xCC and `dword_686B58`. `None` = left as it is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrailBlazerGlobals {
    pub zone_velocity: Option<f32>,       // GC->visual_zone_velocity +0x84
    pub zone_sensitivities: Option<f32>,  // GC->zone_sensitivities_multiplier +0x90
    pub friction_sa: Option<f32>,         // GC->SS_friction_limited_SA_multiplier +0xD4
    pub steer_bias_steering: Option<f32>, // GC->SS_steering_bias_when_steering +0xD0
    pub steer_bias: Option<f32>,          // GC->SS_steering_bias +0xCC
    pub g_686b58: Option<f32>,            // dword_686B58 (zone speed cap of PhysicsManager 0x0019F920)
}

impl PhysicsWorld {
    /// TrailBlazer.cpp 0x000C4870: the stores into `::GC` / `dword_686B58` (the translated code reads both the
    /// struct and the GC image, so both are written); same order as 0x000C4E50's restore
    pub fn set_trail_blazer_globals(&mut self, g: &TrailBlazerGlobals) {
        if let Some(v) = g.zone_velocity {
            self.gc.visual_zone_velocity = v;
            self.gcm.set_f(0x84, v);
        }
        if let Some(v) = g.zone_sensitivities {
            self.gc.zone_sensitivities_multiplier = v;
            self.gcm.set_f(0x90, v);
        }
        if let Some(v) = g.g_686b58 {
            self.glob.w.insert(0x0068_6B58, v.to_bits());
        }
        if let Some(v) = g.friction_sa {
            self.gc.SS_friction_limited_SA_multiplier = v;
            self.gcm.set_f(0xD4, v);
        }
        if let Some(v) = g.steer_bias_steering {
            self.gc.SS_steering_bias_when_steering = v;
            self.gcm.set_f(0xD0, v);
        }
        if let Some(v) = g.steer_bias {
            self.gc.SS_steering_bias = v;
            self.gcm.set_f(0xCC, v);
        }
    }
    /// `::GC` +0x84, +0x90, +0xD4, +0xD0, +0xCC and `dword_686B58` (the values 0x000C4870 saves)
    pub fn trail_blazer_globals(&mut self) -> [f32; 6] {
        let c = super::cx::Cx::new(&self.gc, &self.gcm, &mut self.glob);
        let g = |o| c.f(Ptr::gc(o));
        [g(0x84), g(0x90), g(0xD4), g(0xD0), g(0xCC), c.f(Ptr::glob(0x0068_6B58))]
    }
    /// TrailBlazer.cpp 0x000C4870 / 0x000C4E50: `VehicleEntity +0x10` (trailblazer on; read by 0x001D1850 and
    /// VehicleManager 0x001DE050)
    pub fn set_trail_blazer_flag(&mut self, id: usize, on: bool) {
        self.vehs[id].as_mut().expect("no such vehicle").ve.m.set_b(16, on as u8);
    }
    pub fn trail_blazer_flag(&self, id: usize) -> bool {
        self.ve_mem(id).b(16) != 0
    }
    /// VehicleEntity.cpp 0x001CC2E0 (`C::VehicleEntity`, called by TrailBlazer 0x000C4E50): zone state reset,
    /// LC +148 = +272, `dword_688B70` (physics time scale) = 1.0, LC +8 = GC +172
    pub fn trail_blazer_reset(&mut self, id: usize) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_1cc2e0(&mut c, Ptr::ve(id));
    }
    /// VsConfigFileRead.cpp 0x001B74A0 on vehicle `id`'s VE: steps the held gear (+4052, timer +4048, shift
    /// state +4056/+4060, previous +4064) towards the requested one (+3216 = car +2976) and returns it (+100 while
    /// a shift is in progress); VsAudioEvents::UpdatePlayerEngine 0x0004C44E calls it once per update
    pub fn shift_gear(&mut self, id: usize) -> i32 {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_1b74a0(&mut c, Ptr::ve(id))
    }

    /// VehicleEntity.cpp 0x001D1270 start boost (core: ReleaseCars at Go). `gauge` = OSD launch gauge
    /// (OSD3dInterface +2936, read through `unk_6FC098`); sets `launch_result` and kicks the car forward
    pub fn launch_boost(&mut self, id: usize, gauge: f32) {
        self.glob.w.insert(0x006F_C098 + 2936, gauge.to_bits());
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        f_1d1270(&mut c, Ptr::ve(id));
    }

    /// `VehicleEntity +0x1338`: launch result of 0x001D1270 (1..4; read by ScoreManagerChecks 0x0016CD50, only
    /// cleared by the PhysicsCarInput reset 0x001B9940)
    pub fn launch_result(&self, id: usize) -> i32 {
        self.ve_mem(id).i(0x1338)
    }
    /// `VehicleEntity +0x133C`: gear-shift result of 0x001D1480 (1 early/late, 2 good, 3 perfect; read by
    /// ScoreManagerChecks 0x0016CD70)
    pub fn shift_result(&self, id: usize) -> i32 {
        self.ve_mem(id).i(0x133C)
    }
    /// ScoreManager.cpp 0x0016AFE0 (from 0x00169E30 once the shift has been scored): `+0x133C = 0`
    pub fn clear_shift_result(&mut self, id: usize) {
        self.vehs[id].as_mut().expect("no such vehicle").ve.m.set_i(0x133C, 0);
    }
    /// PhysicsManager.cpp 0x0019E080 traffic loop: `VehicleEntity +0x13D8` = the traffic manager index
    pub fn set_traffic_index(&mut self, id: usize, i: i32) {
        self.vehs[id].as_mut().expect("no such vehicle").ve.m.set_i(0x13D8, i);
    }
    /// PhysicsManager.cpp 0x0019E080 police loop (after control mode 1 and VsAIPoliceVehicle 0x000177E0): VE+3220
    /// (car +2980) x2, LC +264 x2, +92 x2, +80 x1.5, +76 x1.5 (f32 stores)
    pub fn police_handling(&mut self, id: usize) {
        let mut c = make_cx(&self.gc, &self.gcm, &mut self.glob, self.track.as_ref(), &self.walls, &mut self.vehs, id);
        let ve = Ptr::ve(id);
        let p = ve.add(3220);
        let t = c.f(p) * 2.0;
        c.set_f(p, t);
        let lc = c.ptr_at(ve.add(3552));
        for (o, k) in [(264, 2.0f32), (92, 2.0), (80, 1.5), (76, 1.5)] {
            let t = c.f(lc.add(o)) * k;
            c.set_f(lc.add(o), t);
        }
    }
}
