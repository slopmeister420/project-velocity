//! src\Feck2\Screens\Common\FE3DVehicles.cpp — the cars on a foyer's vehicle spots (`dword_768978`, 2580-byte
//! slots): `<foyer>\Vehicles.txt` (loader 0x002595E0, FEAlloyEditor_Basic.cpp-labelled), which car each spot
//! shows (0x00259D20), its visibility (0x0025E570) and colour (0x0025E210 / 0x0025E250), the showroom
//! turntable (0x0025B090) and rotation (0x0025DB50) and the door state (0x0025D9A0 / 0x0025BF80). The bevy side
//! (`foyer.rs`) spawns one `SpawnCarRender` per visible slot from the slot's setup and turns its root.
//!
//! Spot kinds: PLAYERCAR (the garage's selected car), PLAYER1CAR / PLAYER2CAR (the quick race selection
//! `dword_6F70DC`, garage cars for tier 8), CUSTOMCAR (the customisation screen's car) and CARSLOT (the garage
//! screen's car) copy garage setups; SHOWROOM_SINGLE is the car showroom's car (`dword_6F90CC`); RANDOMCAR /
//! NETWORK_CAR need screens that are not ported.

use visos::file::vs_file_directory::VsFileDirectoryDetails;
use visos::handy::vs_text_file::VsTextFile;
use velocity_physics::vehicle_database::vehicle_database::{string_hash, CVehicleDatabase, VehicleSetup};

use crate::feck2::screens::garage::GarageVars;

/// spot kinds (slot +2380)
pub const SPOT_PLAYERCAR: u32 = 1;
pub const SPOT_PLAYER1CAR: u32 = 2;
pub const SPOT_PLAYER2CAR: u32 = 3;
pub const SPOT_CUSTOMCAR: u32 = 4;
pub const SPOT_CARSLOT: u32 = 5;
pub const SPOT_SHOWROOM_SINGLE: u32 = 6;
pub const SPOT_RANDOMCAR: u32 = 7;
pub const SPOT_NETWORK_CAR: u32 = 8;

#[derive(Clone, Debug, Default)]
pub struct FoyerVehicle {
    pub pos: [f32; 3],                // +0 (y raised by half the wheel's +0x0C when shown)
    pub cam_pos: [f32; 3],            // +16
    pub cam_look_at: [f32; 3],        // +32
    pub cam_up: [f32; 3],             // +48
    pub name: String,                 // +736 (other spot names are vehicle names)
    pub rotation: f32,                // +1248 degrees about y
    pub base_rotation: f32,           // +1252 the Vehicles.txt rotation
    /// +1256 front wheel steer in degrees (0x0025E040; the wheel matrices 0x0025B4D0 / 0x0025BA40 turn f_l by -steer,
    /// the mirrored f_r by +steer — drawn once graphics exposes the wheel parts, request in graphics.md)
    pub steer: f32,
    pub ground_y: f32,                // +1260
    pub kind: u32,                    // +2380
    pub garage: bool,                 // +2448 (`byte_6F90D0`: tier 8 selection)
    pub setup: Option<VehicleSetup>,  // +1824 the setup on show
    pub door_t: [f32; 8],             // +2384 opening of the door parts (0x0025BF80: 4 doors, 2 boots, bonnet, t_boot)
    pub doors_open: bool,             // +2576 wanted state of the doors
    pub loaded: bool,                 // +2578
    pub visible: bool,                // +2579
    pub bounce: f32,                  // +1264 wheel change bounce (0x0025E7D0): the car rises, the wheels stay put
    pub lift: f32,                    // half the wheel's +0x0C (the setup's wheel record, +1940)
}

/// "Wheel change bounce time / height / oscillations" `dword_6C1510` / `flt_6C1514` / `dword_6C1518`
pub static BOUNCE_TIME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3F00_0000);
pub static BOUNCE_HEIGHT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3BA3_D70A);
pub static BOUNCE_OSCILLATIONS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(2);

#[derive(Clone, Debug, Default)]
pub struct FE3DVehicles {
    pub foyer: String,                // +15564
    pub slots: Vec<FoyerVehicle>,
    pub count: usize,                 // +15552 slots shown by `show`
    pub garage: bool,                 // +15560 copy of `byte_6F90D0`
    pub garage_index: usize,          // +15724 garage slot shown on CARSLOT (0x00246040)
    pub door_anim: bool,              // +15694 the door parts are animated (0x00259580)
    pub doors_allowed: bool,          // +15695 (0x00259580; also switches the turntable)
    pub turntable: bool,              // +15698 slot 0 turns (0x0025D800)
    pub showroom: bool,               // +15699 (0x00220210, set by the car showroom's Enter; no reader in F3_D's FE)
    pub spin_hold: i32,               // +15704 frames before the turntable spins on after the stick let go
    pub door_window: (f32, f32),      // +15708 / +15712 the door move's start / end time (0x0025D9A0)
    pub bounce_on: bool,              // +15716 a wheel change bounce runs
    pub bounce_start: f32,            // +15720 its start time (front end stopwatch)
    /// door sounds called by `update_door` this frame (true = PlayCarOpen, false = PlayCarClose), sent by the bevy side
    pub door_sounds: Vec<bool>,
    /// CarRender 0x00116BC0 per decal kind 0..5 (chassis logos, f / r bumper, skirt, front / rear screen): the `logoNN`
    /// slots the shown car's models have. `None` until graphics reports them (docs/graphics.md): the setup's table sizes
    pub decal_slots: Option<[u32; 6]>,
}

/// the setup's decal table sizes per kind (`decals_3` 10, the bumper / skirt tables 2, the two screen slots 1)
const DECAL_TABLE_SLOTS: [u32; 6] = [10, 2, 2, 2, 1, 1];

/// turntable: frames the stick's turn holds off the spin (`dword_6C151C` 150.0) and the frames over which it eases
/// back in (`dword_6C1520` 50.0)
const SPIN_HOLD: f32 = f32::from_bits(0x43160000);
const SPIN_EASE: f32 = f32::from_bits(0x42480000);

impl FE3DVehicles {
    /// FE3DVehicles.cpp 0x0025E720 — decal slots of `kind` on the shown car (CarRender 0x00116BC0)
    pub fn decal_slot_count(&self, kind: usize) -> i32 {
        self.decal_slots.unwrap_or(DECAL_TABLE_SLOTS).get(kind).copied().unwrap_or(0) as i32
    }

    /// FEAlloyEditor_Basic.cpp 0x00259540 — foyer the vehicle file is read from
    pub fn set_foyer(&mut self, foyer: &str) {
        self.foyer = foyer.to_string();
    }

    /// FEAlloyEditor_Basic.cpp 0x002595E0 — `LoadVehicleFile("<name>")` (`<name>.txt` in the foyer)
    pub fn load(&mut self, file: &str, garage: bool) {
        self.garage = garage;
        self.slots.clear();
        self.count = 0;
        let Ok(dir) = VsFileDirectoryDetails::open(&format!("\\Foyers\\{}", self.foyer)) else { return };
        let mut t = VsTextFile::default();
        if !t.load(Some(&dir), &format!("{file}.txt")) {
            return;
        }
        let n = t.read_s32().map_or(0, |(_, v)| v);
        let v3 = |t: &mut VsTextFile| t.read_f32n::<3>().map_or([0.0; 3], |(_, v)| v);
        for _ in 0..n.max(0) {
            let name = t.read_string().unwrap_or_default();
            let pos = v3(&mut t);
            let rotation = t.read_f32().map_or(0.0, |(_, v)| v);
            let cam_pos = v3(&mut t);
            let cam_look_at = v3(&mut t);
            let cam_up = v3(&mut t);
            let kind = match name.as_str() {
                "PLAYERCAR" => SPOT_PLAYERCAR,
                "CUSTOMCAR" => SPOT_CUSTOMCAR,
                "PLAYER1CAR" => SPOT_PLAYER1CAR,
                "PLAYER2CAR" => SPOT_PLAYER2CAR,
                "CARSLOT" => SPOT_CARSLOT,
                "SHOWROOM_SINGLE" => SPOT_SHOWROOM_SINGLE,
                "RANDOMCAR" => SPOT_RANDOMCAR,
                "NETWORK_CAR" => SPOT_NETWORK_CAR,
                _ => 0,
            };
            self.slots.push(FoyerVehicle {
                pos,
                cam_pos,
                cam_look_at,
                cam_up,
                name,
                rotation,
                base_rotation: rotation,
                ground_y: pos[1],
                kind,
                garage,
                ..Default::default()
            });
        }
        self.count = self.slots.len();
    }

    /// FEAlloyEditor_Basic.cpp 0x00259D20 — `Show(slot or all)`: build each spot's setup and put the car up.
    /// `selected` = `dword_6F70DC` per player; garage spots copy the garage slot's setup (GameVars 0x000C1F90):
    /// PLAYERCAR `dword_6F71C4`, CUSTOMCAR `dword_6F90C4`, CARSLOT this object's +15724.
    pub fn show(&mut self, slot: Option<usize>, db: &CVehicleDatabase, selected: [u32; 2], garage: &[u32], garage_selected: usize, vars: &GarageVars) {
        let range = match slot {
            Some(s) => s..s + 1,
            None => 0..self.count,
        };
        let garage_index = self.garage_index;
        for i in range {
            let Some(v) = self.slots.get_mut(i) else { continue };
            let from_garage = |idx: usize| -> Option<VehicleSetup> {
                vars.setup(idx as u32).cloned().or_else(|| {
                    // no garage setups (no vehicle database at init): default setup of the hash
                    let hash = garage.get(idx).copied().or_else(|| fallback_car(db))?;
                    let mut s = VehicleSetup::new();
                    db.setup_default_vehicle(&mut s, hash);
                    Some(s)
                })
            };
            let default_of = |hash: u32| -> VehicleSetup {
                let mut s = VehicleSetup::new();
                db.setup_default_vehicle(&mut s, hash);
                s
            };
            let setup = match v.kind {
                SPOT_PLAYERCAR => from_garage(garage_selected),
                SPOT_PLAYER1CAR | SPOT_PLAYER2CAR => {
                    let p = (v.kind - SPOT_PLAYER1CAR) as usize;
                    if v.garage { from_garage(garage_selected) } else { Some(default_of(selected[p])) }
                }
                SPOT_CUSTOMCAR => from_garage(vars.custom_slot as usize),
                SPOT_CARSLOT => from_garage(garage_index),
                SPOT_NETWORK_CAR => None,
                // the car showroom's car (`dword_6F90CC`)
                // (0 before the hangout's showroom entry set it: the original asserts in SetupDefaultVehicle)
                SPOT_SHOWROOM_SINGLE => match db.find_vehicle(vars.showroom_vehicle) {
                    Some(_) => Some(default_of(vars.showroom_vehicle)),
                    None => {
                        log::error!("Error: VEHICLE NOT FOUND .... Check case!!! ({:08x})", vars.showroom_vehicle);
                        None
                    }
                },
                // RANDOMCAR (GameVars 0x000C22D0): its screens are not ported
                SPOT_RANDOMCAR => None,
                _ => Some(default_of(string_hash(&v.name))),
            };
            let Some(setup) = setup.filter(|s| s.vehicle.is_some()) else {
                log::warn!("VEHICLE NOT FOUND - {}", v.name);
                break;
            };
            // the car sits half its wheel's +0x0C above the spot
            let lift = setup.wheel.and_then(|w| db.wheels.get(w)).map_or(0.0, |w| w.unk_0c / 2.0);
            v.lift = lift;
            v.pos[1] = v.ground_y + lift;
            v.setup = Some(setup);
            v.loaded = true;
            v.visible = true;
        }
    }

    /// FE3DVehicles.cpp 0x0025E570 — `SetVisible(slot, show)`
    pub fn set_visible(&mut self, slot: usize, show: bool) {
        if let Some(v) = self.slots.get_mut(slot) {
            v.visible = show;
        }
    }

    /// FE3DVehicles.cpp 0x0025E210 — body colour of the slot's setup (`VehicleDatabaseColours` 0x00188060);
    /// 0x0025E250 then reapplies the paint (the bevy side follows the setup).
    pub fn set_colour(&mut self, slot: usize, db: &CVehicleDatabase, colour: Option<usize>) {
        if let Some(s) = self.slots.get_mut(slot).and_then(|v| v.setup.as_mut()) {
            db.set_setup_colour(s, colour);
        }
    }

    /// FEAlloyEditor_Basic.cpp 0x00259580 — doors allowed (+15695, which also switches the turntable) and the door
    /// parts animated (+15694)
    pub fn set_doors_allowed(&mut self, allowed: bool) {
        self.doors_allowed = allowed;
        self.set_turntable(allowed);
        self.door_anim = true;
    }

    /// FE3DVehicles.cpp 0x0025D800 — turntable on slot 0
    pub fn set_turntable(&mut self, on: bool) {
        self.turntable = on;
    }

    /// FE3DVehicles.cpp 0x0025D8F0 — a slot's rotation (0 past the slot count)
    pub fn rotation(&self, slot: usize) -> f32 {
        if slot <= self.count {
            self.slots.get(slot).map_or(0.0, |v| v.rotation)
        } else {
            0.0
        }
    }

    /// FE3DVehicles.cpp 0x0025D940 — every door part of a slot at `t` at once
    pub fn set_doors(&mut self, slot: usize, t: f32) {
        if let Some(v) = self.slots.get_mut(slot) {
            v.door_t = [t; 8];
        }
    }

    /// FE3DVehicles.cpp 0x0025D9A0 — open / close a slot's doors: the 2 s window restarts from where the doors are
    /// (`now` = the front end stopwatch, CFrontendManager +8)
    pub fn open_doors(&mut self, slot: usize, open: bool, now: f32) {
        let Some(v) = self.slots.get_mut(slot) else { return };
        if open == v.doors_open {
            return;
        }
        v.doors_open = open;
        let (start, end) = self.door_window;
        let mut t = 0.0f32;
        // VsFle(now, end, 1e-6)
        if now <= end + f32::from_bits(897988541) {
            t = 1.0 - (now - start) / (end - start);
            debug_assert!((-1e-6..=1.0 + 1e-6).contains(&t), "VsFge(t, 0.0f) && VsFle(t, 1.0f)");
        }
        self.door_window.0 = now - 2.0 * t;
        self.door_window.1 = self.door_window.0 + 2.0;
    }

    /// FE3DVehicles.cpp 0x0025BF80 (the door-state part; the part matrix needs graphics' door models) — part `k`'s
    /// opening eases along the window towards the wanted state: `t = (sin(pi s - pi/2) + 1) / 2`
    pub fn update_door(&mut self, slot: usize, k: usize, now: f32) {
        let (start, end) = self.door_window;
        let Some(v) = self.slots.get_mut(slot) else { return };
        let s = ((now - start) / (end - start)).clamp(0.0, 1.0);
        let eased = (((s * std::f32::consts::PI) - std::f32::consts::FRAC_PI_2).sin() + 1.0) * 0.5;
        // VsAudio::PlayCarOpen 0x0004AB00 (0x0025C1CE: moved this call and t in 0..=0.2) / PlayCarClose 0x0004AC30
        // (0x0025C3A6: the closing step reached 0), once per door part step
        let mut moved = false;
        if v.doors_open {
            if v.door_t[k] < 1.0 {
                v.door_t[k] = eased;
                moved = true;
            }
            if moved && v.door_t[k] >= 0.0 && v.door_t[k] <= 0.2 {
                self.door_sounds.push(true);
            }
            if v.door_t[k] >= 1.0 {
                v.door_t[k] = 1.0;
            }
        } else {
            if v.door_t[k] > 0.0 {
                v.door_t[k] = 1.0 - eased;
                moved = true;
            }
            if v.door_t[k] <= 0.0 {
                if moved {
                    self.door_sounds.push(false);
                }
                v.door_t[k] = 0.0;
            }
        }
    }

    /// FE3DVehicles.cpp 0x0025EAB0 — a wheel change starts the bounce (from 0x0025CF40 and 0x0025CB20 part 12)
    pub fn start_wheel_bounce(&mut self, now: f32) {
        self.bounce_on = true;
        self.bounce_start = now;
    }

    /// FE3DVehicles.cpp 0x0025E7D0 — heights of the loaded slots: ground + lift + the decaying wheel change bounce
    /// `sin(2 pi osc t) * height / (trunc(t * osc) + 1)`, t = elapsed / bounce time (ends past 1)
    pub fn update_heights(&mut self, now: f32) {
        use std::sync::atomic::Ordering::Relaxed;
        let time = f32::from_bits(BOUNCE_TIME.load(Relaxed));
        let height = f32::from_bits(BOUNCE_HEIGHT.load(Relaxed));
        let osc = BOUNCE_OSCILLATIONS.load(Relaxed);
        let (on, start) = (self.bounce_on, self.bounce_start);
        let mut ended = false;
        for v in self.slots.iter_mut().filter(|v| v.setup.is_some()) {
            if on {
                let mut t = (now - start) / time;
                if t > 1.0 {
                    ended = true;
                    t = 1.0;
                    v.bounce = 0.0;
                }
                let n = ((t / (1.0 / osc as f32)) as i32 + 1) as f32;
                let r = osc as f32 * t;
                v.bounce = (2.0 * std::f32::consts::PI * r).sin() * height * (1.0 / n);
            }
            v.pos[1] = (v.ground_y + v.lift) + v.bounce;
        }
        if ended {
            self.bounce_on = false;
        }
    }

    /// FE3DVehicles.cpp 0x0025BF80 (per frame, the state part for every part of every loaded slot)
    pub fn step_doors(&mut self, now: f32) {
        if !self.door_anim {
            return;
        }
        for slot in 0..self.slots.len() {
            if self.slots[slot].loaded {
                for k in 0..8 {
                    self.update_door(slot, k, now);
                }
            }
        }
    }

    /// FE3DVehicles.cpp 0x0025E040 — `SetSteer(slot, degrees)`: wrapped once into 0..360 (> 360 and < 0 only), then
    /// the matrices again (0x0025DB50 with the current rotation) when the car is loaded
    pub fn set_steer(&mut self, slot: usize, mut a: f32, now: f32) {
        if slot as i64 > self.count as i64 - 1 {
            return;
        }
        if a > 360.0 {
            a -= 360.0;
        } else if a < 0.0 {
            a += 360.0;
        }
        let Some(v) = self.slots.get_mut(slot) else { return };
        v.steer = a;
        if v.loaded {
            let r = v.rotation;
            self.set_rotation(slot, r, now);
        }
    }

    /// FE3DVehicles.cpp 0x0025DB50 — `SetRotation(slot, degrees)`: wrapped once into 0..360; the car's world
    /// matrix follows (`foyer.rs` sets the root transform), with the door parts stepped when they are animated
    pub fn set_rotation(&mut self, slot: usize, mut a: f32, now: f32) {
        if slot + 1 > self.count {
            return;
        }
        if a > 360.0 {
            a -= 360.0;
        } else if a < 0.0 {
            a += 360.0;
        }
        if !self.slots.get(slot).is_some_and(|v| v.loaded) {
            return;
        }
        self.slots[slot].rotation = a;
        if self.door_anim {
            // six door parts (-1 part ids), then the bonnet (2 -> 6) and the top boot (4 -> 7)
            for k in 0..8 {
                self.update_door(slot, k, now);
            }
        }
    }

    /// FEAlloyEditor_Basic.cpp 0x0025B090 (the turntable part; loading is synchronous here) — slot 0 turns by
    /// the cube of the `CameraRotateHoriz` stick times 2; let go, it waits `SPIN_HOLD` frames and eases back into
    /// 0.2° a frame over `SPIN_EASE` frames. One step per front end frame, like the original.
    pub fn update_turntable(&mut self, stick: f32, now: f32) {
        if !self.turntable || !self.slots.first().is_some_and(|v| v.loaded) {
            return;
        }
        let mut a = self.rotation(0);
        if stick == 0.0 {
            if self.spin_hold > 0 {
                self.spin_hold -= 1;
            }
            if self.spin_hold != 0 {
                if SPIN_EASE > self.spin_hold as f32 {
                    a -= ((SPIN_EASE - self.spin_hold as f32) / SPIN_EASE) * 0.2;
                }
            } else {
                a -= 0.2;
            }
        } else {
            self.spin_hold = SPIN_HOLD as i32;
            a += stick * stick * stick * 2.0;
        }
        self.set_rotation(0, a, now);
    }
}

/// stand-in for the profile garage's first car: the first available LA tier-1 car
fn fallback_car(db: &CVehicleDatabase) -> Option<u32> {
    db.vehicles.iter().find(|v| v.path.to_ascii_lowercase().contains("\\la\\") && v.data.tier == 1).map(|v| v.hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_car() -> FE3DVehicles {
        let mut v = FE3DVehicles { count: 1, ..Default::default() };
        v.slots.push(FoyerVehicle { loaded: true, rotation: 10.0, base_rotation: 10.0, ..Default::default() });
        v
    }

    #[test]
    fn turntable_holds_then_eases_back_in() {
        let mut v = one_car();
        v.set_turntable(true);
        // no stick, no hold: 0.2° a frame
        v.update_turntable(0.0, 0.0);
        assert!((v.rotation(0) - 9.8).abs() < 1e-5);
        // the stick turns by its cube * 2 and holds the spin for 150 frames
        v.update_turntable(0.5, 0.0);
        assert!((v.rotation(0) - 10.05).abs() < 1e-4);
        for _ in 0..100 {
            v.update_turntable(0.0, 0.0);
        }
        // hold 150 -> 50 left: no spin yet
        assert!((v.rotation(0) - 10.05).abs() < 1e-4);
        v.update_turntable(0.0, 0.0);
        // 49 left: (50 - 49) / 50 * 0.2
        assert!((v.rotation(0) - (10.05 - 0.004)).abs() < 1e-4);
        // wraps once into 0..360
        v.set_rotation(0, -5.0, 0.0);
        assert_eq!(v.rotation(0), 355.0);
    }

    #[test]
    fn wheel_change_bounce_decays_and_ends() {
        let mut v = one_car();
        v.slots[0].setup = Some(VehicleSetup::new());
        v.slots[0].ground_y = 1.0;
        v.start_wheel_bounce(3.0);
        // t = 0.125: sin(pi / 2) * 0.005 / 1
        v.update_heights(3.0625);
        assert!((v.slots[0].bounce - 0.005).abs() < 1e-6 && (v.slots[0].pos[1] - 1.005).abs() < 1e-5);
        // t = 0.875: osc t = 1.75, n = 2: sin(3.5 pi) = -1 -> -0.0025
        v.update_heights(3.4375);
        assert!((v.slots[0].bounce + 0.0025).abs() < 1e-6);
        v.update_heights(4.0);
        assert!(!v.bounce_on && v.slots[0].bounce == 0.0 || v.slots[0].bounce.abs() < 1e-5);
        v.update_heights(4.1);
        assert!(v.slots[0].pos[1] == 1.0 && !v.bounce_on);
    }

    #[test]
    fn doors_ease_over_two_seconds() {
        let mut v = one_car();
        v.door_anim = true;
        v.open_doors(0, true, 10.0);
        assert_eq!(v.door_window, (10.0, 12.0));
        v.set_rotation(0, 0.0, 11.0);
        assert!((v.slots[0].door_t[0] - 0.5).abs() < 1e-5);
        v.set_rotation(0, 0.0, 13.0);
        assert_eq!(v.slots[0].door_t[0], 1.0);
        // closing half way through an opening starts the window where the doors are
        v.slots[0].door_t = [0.0; 8];
        v.open_doors(0, false, 12.5);
        assert_eq!(v.door_window, (12.5, 14.5));
        // t = 1 - (13 - 12.5) / 2 = 0.75 -> the window starts 1.5 s back
        v.open_doors(0, true, 13.0);
        assert_eq!(v.door_window, (11.5, 13.5));
    }
}
