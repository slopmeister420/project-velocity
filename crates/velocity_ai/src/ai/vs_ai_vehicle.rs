//! Base of every ai-driven car: the link to the physics car, its cached state and its controls
//! (src/Ai/VsAIVehicle.cpp).
//!
//! The physics side (`VehicleEntity`, `PhysicsCarInput`) belongs to velocity_physics; the ai talks to it
//! only through [`AiPhysicsVehicle`] and fills [`VehicleControls`], mirroring the original layout.

use super::vs_math::VsVector4;

/// the input block the ai writes (`VehicleEntity +0x9C`, reset by `VsConfigFileRead.cpp 0x001B7D90`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleControls {
    pub steer: f32,       // +0x00 -1 (left) .. 1 (right)
    pub unk_04: f32,      // +0x04
    pub throttle: f32,    // +0x08 0..1
    pub brake: f32,       // +0x0C 0..1
    pub handbrake: f32,   // +0x10 0..1
    pub nitrous: bool,    // +0x14
    pub horn: bool,       // +0x15
    pub unk_18: bool,     // +0x18
    pub unk_19: u16,      // +0x19
    pub unk_1b: bool,     // +0x1B 1 after reset
    pub unk_1c: bool,     // +0x1C
}

impl Default for VehicleControls {
    /// VsConfigFileRead.cpp 0x001B7D90
    fn default() -> Self {
        Self {
            steer: 0.0,
            unk_04: 0.0,
            throttle: 0.0,
            brake: 0.0,
            handbrake: 0.0,
            nitrous: false,
            horn: false,
            unk_18: false,
            unk_19: 0,
            unk_1b: true,
            unk_1c: false,
        }
    }
}

/// what the ai reads from and does to its physics car.
pub trait AiPhysicsVehicle {
    /// rigid body position (`body vtbl +0x1C`, PhysicsCarInput.cpp 0x001BA270)
    fn position(&self) -> VsVector4;
    /// orientation quaternion (VehicleEntity.cpp 0x001BB230)
    fn orientation(&self) -> VsVector4;
    /// world forward axis (VehicleEntity.cpp 0x001BB200 with axis 0)
    fn forward(&self) -> VsVector4;
    /// local x / y / z axes in world space (VehicleEntity.cpp 0x001C65A0 / 0x001C6630 / 0x001C66C0)
    fn axis_right(&self) -> VsVector4;
    fn axis_up(&self) -> VsVector4;
    fn axis_forward(&self) -> VsVector4;
    /// rigid body linear velocity (`body +0x20`)
    fn velocity(&self) -> VsVector4;
    /// unsigned speed in m/s (`VehicleEntity +0x14`)
    fn speed(&self) -> f32;
    /// the controls block the ai drives the car with
    fn controls(&mut self) -> &mut VehicleControls;
    fn controls_ref(&self) -> &VehicleControls;
    /// exempt from the upside-down respawn (`VehicleEntity +0x1294 == 1 && +0x11F4 < 10.0`, float flight time)
    fn upside_down_exempt(&self) -> bool {
        false
    }
    /// moves the body (`body +0x0F0` set position / orientation / velocities, then commit)
    fn teleport(&mut self, pos: &VsVector4, orientation: &VsVector4, velocity: &VsVector4);
    /// VehicleEntity.cpp 0x001C5710 (re-sync after a teleport)
    fn reset_state(&mut self) {}
    /// VehicleEntity.cpp 0x001C5750 (deactivate)
    fn deactivate(&mut self) {}
    /// `body 0x00361FF0`: steering angle applied directly to the wheels
    fn set_steer_angle(&mut self, _angle: f32) {}
    /// `body 0x0036B530` / `0x0036B730`
    fn set_linear_velocity(&mut self, _v: &VsVector4) {}
    fn set_angular_velocity(&mut self, _v: &VsVector4) {}
    /// `body 0x0036C3E0`: angular impulse
    fn apply_angular_impulse(&mut self, _v: &VsVector4) {}
    /// `VehicleEntity +0x1310` grip multiplier (1 normally, mistakes lower it)
    fn set_grip_scale(&mut self, _s: f32) {}
    /// `VehicleEntity +0x13DC` who drives the car (1 ai, 2 traffic, 4 ..., 6 none)
    fn control_mode(&self) -> u32 {
        1
    }
    fn set_control_mode(&mut self, _m: u32) {}
    /// engine rpm (VsConfigFileRead.cpp 0x001B6F10)
    fn engine_rpm(&self) -> i32 {
        0
    }
    /// current gear (VsConfigFileRead.cpp 0x001B74A0)
    fn gear(&self) -> i32 {
        1
    }
    /// nitrous left / capacity (`VehicleEntity +0x64 / +0x60`)
    fn nos_remaining(&self) -> f32 {
        0.0
    }
    fn nos_capacity(&self) -> f32 {
        0.0
    }
    /// VehicleEntity.cpp 0x001D11D0 tyre puncture
    fn puncture(&mut self, _a: f32, _b: f32) {}
    /// body up axis (`body +0x80`)
    fn body_up(&self) -> VsVector4 {
        VsVector4::new(0.0, 1.0, 0.0, 0.0)
    }
    /// velocity along the body's x row (VsAIRaceVehicle.cpp 0x00024870)
    fn forward_speed(&self) -> f32 {
        self.speed()
    }
    /// transmission state (`body 0x00364BE0`; 4 = driving forward)
    fn drive_state(&self) -> i32 {
        4
    }
    /// reverse selected (`body 0x00364B20`)
    fn is_reversing(&self) -> bool {
        false
    }
    /// all wheels off the ground (`body +0xB3C`)
    fn is_airborne(&self) -> bool {
        false
    }
    /// nitrous currently burning (`VehicleEntity +0x58`)
    fn nos_burning(&self) -> bool {
        false
    }
    /// clears forces and velocities before a respawn (VsAITrafficVehicle.cpp 0x0002B800)
    fn reset_dynamics(&mut self) {}
    /// `body 0x0035DCC0`: lets the ai steer the wheels directly (1) or through the controls (0)
    fn set_direct_steer(&mut self, _on: bool) {}
    /// launch control flag (`VehicleEntity +0x134C`)
    fn set_launch_control(&mut self, _on: bool) {}
    /// the four wheel-locked flags (`VehicleEntity +0x5C4` = CFastCar +0x4D4)
    fn lock_wheels(&mut self, _v: i32) {}
    /// auto up/down shift allowed (`VehicleEntity +0xDC9`, `+0xDCA`)
    fn shift_flags(&self) -> (bool, bool) {
        (false, false)
    }
    /// number of gears (`+0xBDC`)
    fn gear_count(&self) -> i32 {
        6
    }
    /// engine speed in the ai's units for shift decisions (`body 0x0035D090`)
    fn engine_speed(&self) -> f32 {
        0.0
    }
    /// gear the ai shifts manually (`VehicleEntity +0xC90`)
    fn shift(&mut self, _delta: i32) {}
    /// supercar launch hack: handling block values at +0x6C, +0x10C, +0x128
    fn launch_params(&self) -> [f32; 3] {
        [0.0; 3]
    }
    fn set_launch_params(&mut self, _p: [f32; 3]) {}
    /// car length extent from its database entry (`**(VehicleEntity +0xDE4) +0x0C`)
    fn extent(&self) -> f32 {
        2.25
    }
    /// the respawn's wheel heights: `VehicleEntity +0x810` and `body 0x0035FB40(0)` (= body +0x70C)
    fn respawn_wheel_heights(&self) -> (f32, f32) {
        (0.0, 0.0)
    }
    /// VehicleEntity.cpp 0x001D1810: the current history record (`+0xDE8` ring) gets +0x64 = 1.0
    fn reset_history_scale(&mut self) {}
    /// car vtbl +0x20 (commit after setting the body's velocities)
    fn commit_body(&mut self) {}
    /// contact normal of wheel `i` (ScoreManager's upright test reads wheel 1)
    fn wheel_contact_normal(&self, _i: usize) -> VsVector4 {
        VsVector4::new(0.0, 1.0, 0.0, 0.0)
    }
    /// the race car's score record gains (`ScoreManager +0xA0` zone, `+0xA4` nitrous) the physics car adds each
    /// frame (VehicleEntity 0x001C5800 via `VsRaceManager__3D640`); None = no record
    fn set_score_gains(&mut self, _gains: Option<(f32, f32)>) {}
    /// `VehicleEntity +0x1338` launch result (ScoreManagerChecks 0x0016CD50)
    fn launch_result(&self) -> i32 {
        0
    }
    /// `VehicleEntity +0x133C` gear-shift result (ScoreManagerChecks 0x0016CD70)
    fn shift_result(&self) -> i32 {
        0
    }
    /// ScoreManager 0x0016AFE0: `+0x133C = 0`
    fn clear_shift_result(&mut self) {}
    /// PhysicsManager 0x0019E080: `VehicleEntity +0x13D8` = traffic manager index
    fn set_traffic_index(&mut self, _i: i32) {}
    /// PhysicsManager 0x0019E080 police handling scaling (once, after control mode 1)
    fn police_handling(&mut self) {}
}

/// `VsAIVehicle`, the first 0x100 bytes of every race/traffic vehicle.
#[derive(Debug, Clone)]
pub struct VsAIVehicle {
    pub handling: Option<usize>,     // +0x000 m_pHandling (index into VsAIWorld::handlings)
    pub flag_004: bool,              // +0x004
    pub flag_00c: bool,              // +0x00C
    pub unk_010: i32,                // +0x010
    pub flag_014: bool,              // +0x014
    pub unk_01c: i32,                // +0x01C
    pub active: bool,                // +0x020
    pub flag_021: bool,              // +0x021
    pub physics: Option<usize>,      // +0x024 m_pPhysicsVehicle (handle), +0x028 m_pInputController
    pub own_controls: VehicleControls, // +0x02C
    pub velocity: VsVector4,         // +0x080
    pub speed: f32,                  // +0x090 signed: negative when moving backwards
    pub prev_speed: f32,             // +0x094
    pub position: VsVector4,         // +0x098
    pub spawn_position: VsVector4,   // +0x0A8
    pub orientation: VsVector4,      // +0x0B8
    pub forward: VsVector4,          // +0x0C8
    pub debug_selected: bool,        // +0x0E0
    pub damage: f32,                 // +0x0E4
    pub max_damage: f32,             // +0x0E8
}

impl Default for VsAIVehicle {
    /// VsAITrafficVehicle.cpp 0x0002B6A0
    fn default() -> Self {
        Self {
            handling: None,
            flag_004: false,
            flag_00c: false,
            unk_010: 0,
            flag_014: false,
            unk_01c: 0,
            active: false,
            flag_021: false,
            physics: None,
            own_controls: VehicleControls::default(),
            velocity: VsVector4::default(),
            speed: 0.0,
            prev_speed: 0.0,
            position: VsVector4::default(),
            spawn_position: VsVector4::default(),
            orientation: VsVector4::default(),
            forward: VsVector4::default(),
            debug_selected: false,
            damage: 0.0,
            max_damage: 100.0,
        }
    }
}

impl VsAIVehicle {
    /// VsAITrafficVehicle.cpp 0x0002B750
    pub fn reset_base(&mut self) {
        self.flag_004 = false;
        self.flag_00c = false;
        self.unk_010 = 0;
        self.flag_014 = false;
        self.unk_01c = 0;
        self.damage = 0.0;
        self.max_damage = 100.0;
    }

    /// VsAIVehicle.cpp 0x0002BD00
    pub fn set_physics_vehicle(&mut self, handle: usize) {
        self.physics = Some(handle);
    }

    /// places the car and activates it.
    /// VsAIVehicle.cpp 0x0002BDA0
    pub fn teleport(&mut self, veh: &mut dyn AiPhysicsVehicle, pos: &VsVector4, orientation: &VsVector4, velocity: &VsVector4) {
        debug_assert!(self.physics.is_some(), "m_pPhysicsVehicle");
        self.spawn_position = *pos;
        self.position = *pos;
        self.orientation = *orientation;
        self.velocity = *velocity;
        veh.teleport(pos, orientation, velocity);
        veh.reset_state();
        self.active = true;
        self.flag_021 = true;
    }

    /// VsAIVehicle.cpp 0x0002C0F0
    pub fn deactivate(&mut self, veh: Option<&mut dyn AiPhysicsVehicle>) {
        if let Some(v) = veh {
            v.deactivate();
            v.set_control_mode(6);
            // VsAIVehicle.cpp 0x0002C180
            *v.controls() = VehicleControls::default();
        }
        self.active = false;
    }

    /// steer angle towards `target` in the xz plane, clamped to +-pi/8.
    /// VsAIVehicle.cpp 0x0002C1B0
    pub fn get_steer_towards(&self, target: &VsVector4, factor: f32) -> f32 {
        let mut fwd = self.forward;
        let mut to = *target;
        to -= self.position;
        to.y = 0.0;
        to.normalise();
        fwd.y = 0.0;
        let c = fwd.cross(&to);
        let mut a = c.length() * factor;
        let lim = 0.392_699_1f32;
        if a <= lim {
            if 0.0 - lim > a {
                a = 0.0 - lim;
            }
        } else {
            a = lim;
        }
        if c.y <= 0.0 {
            if c.y >= 0.0 { 0.0 } else { a }
        } else {
            -a
        }
    }

    /// accumulates damage; true when it just crossed `max_damage`.
    /// VsAIVehicle.cpp 0x0002C420
    pub fn apply_damage(&mut self, damage: f32) -> bool {
        debug_assert!(damage >= 0.0, "damage >= 0.0f");
        let was_below = self.max_damage > self.damage;
        self.damage += damage;
        was_below && self.max_damage <= self.damage
    }

    /// VsAIVehicle.cpp 0x0002C550
    pub fn set_max_damage(&mut self, d: f32) {
        self.max_damage = d;
    }
}

