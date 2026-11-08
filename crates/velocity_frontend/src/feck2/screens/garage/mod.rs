//! src\Feck2\Screens\Garage\ — the player garage (use / sell a car) and the car customisation screen, plus the
//! garage part of the player variables they work on.

pub mod fe_car_customisation_screen;
pub mod fe_pink_slip_screen;
pub mod fe_player_garage_screen;

use velocity_physics::vehicle_database::vehicle_database::{string_hash, CVehicleDatabase, VehicleSetup};

/// garage slots in the player variables (`dword_6F76A4`, 139 dwords each)
pub const GARAGE_SLOTS: usize = 6;

/// the player variables the garage screens use (`g_FEGameVariables`, GameVars.cpp). The car hashes and the
/// current slot are `Feck2::quick_race.garage` / `garage_selected` (`dword_6F71A8` / `dword_6F71C4`).
#[derive(Clone, Debug, Default)]
pub struct GarageVars {
    pub cash: i32,                  // dword_6F719C
    pub prestige: i32,              // dword_6F71A0
    pub current_vehicle: u32,       // dword_6F71C0
    pub setups: Vec<VehicleSetup>,  // dword_6F76A4, one per slot
    pub custom_slot: u32,           // dword_6F90C4 garage slot on the customisation screen
    pub custom_vehicle: u32,        // dword_6F90C8 its vehicle hash
    pub vehicle_flags: u32,         // byte_6F90D0 (8 = vehicles come from the garage)
    pub showroom_vehicle: u32,      // dword_6F90CC the car showroom's car (FE3DVehicles SHOWROOM_SINGLE)
}

impl GarageVars {
    /// GameVars.cpp 0x000C2350 — `IsVehicleInGarage(player, hash)`
    pub fn in_garage(garage: &[u32], hash: u32) -> bool {
        garage.contains(&hash)
    }

    /// GameVars.cpp 0x000C20F0 — `AddGarageVehicle(hash, setup)`: at most 6 cars; returns the slot it went to
    /// (the count before, also when full)
    pub fn add_garage_vehicle(&mut self, garage: &mut Vec<u32>, hash: u32, setup: VehicleSetup) -> u32 {
        let n = garage.len();
        if n < GARAGE_SLOTS {
            garage.push(hash);
            if self.setups.len() <= n {
                self.setups.resize_with(n + 1, VehicleSetup::new);
            }
            self.setups[n] = setup;
        }
        n as u32
    }
}

impl GarageVars {
    /// GamePlayer.cpp 0x000C1930 — the garage part of `SetInitialVariables`: 250000 cash, 300ZX / RX8 / NSX.
    pub fn set_initial_variables(&mut self, garage: &mut Vec<u32>, selected: &mut u32) {
        self.cash = 0x3D090;
        self.prestige = 0;
        *garage = vec![string_hash("300ZX"), string_hash("RX8"), string_hash("NSX")];
        *selected = 0;
        self.current_vehicle = garage[0];
        self.custom_slot = 0;
        self.custom_vehicle = string_hash("350Z");
        self.vehicle_flags = 0;
    }

    /// FECK2Manager.cpp 0x002022F0 (from `GameManager::InitDatabases`) — a default setup per garage car, the free
    /// slots up to 6 get a 350Z.
    pub fn init_setups(&mut self, db: &CVehicleDatabase, garage: &mut [u32]) {
        self.setups = (0..GARAGE_SLOTS)
            .map(|i| {
                let mut s = VehicleSetup::new();
                let hash = garage.get(i).copied().unwrap_or_else(|| string_hash("350Z"));
                db.setup_default_vehicle(&mut s, hash);
                s
            })
            .collect();
    }

    /// GameVars.cpp 0x000C1F90 — the setup of a garage slot
    pub fn setup(&self, slot: u32) -> Option<&VehicleSetup> {
        self.setups.get(slot as usize)
    }
}

/// run `f` on the common overlay
pub(crate) fn ov<R>(fe: &mut crate::feck2::feck2_manager::Feck2, f: impl FnOnce(&mut crate::feck2::screens::common::fe_2d_overlay::CFE2DOverlay, &mut crate::feck2::feck2_manager::Feck2) -> R) -> Option<R> {
    let mut o = fe.overlay.take()?;
    let r = f(&mut o, fe);
    fe.overlay = Some(o);
    Some(r)
}

/// VehicleDatabase.cpp 0x00185140 — what the setup's parts cost: bodyparts (+0x22), brake parts (+0x08), tyre
/// (+0x0A) and decals (+0x0C, low word); `mark` clears the bought flags of the bodyparts, brake parts and tyre.
pub fn setup_parts_cost(db: &mut CVehicleDatabase, setup: &VehicleSetup, mark: bool) -> i32 {
    let mut v = 0i32;
    for p in setup.fitted_bodyparts.iter().flatten() {
        let part = &mut db.bodypart_sets[p.set].parts[p.part_type][p.index];
        v += part.cost as i32;
        if mark {
            part.flags &= !8; // VehicleDatabase.cpp 0x00186870
        }
    }
    for b in setup.brake_parts.iter().flatten() {
        v += db.brake_parts[*b].cost as i32;
        if mark {
            db.brake_parts[*b].flags &= !2;
        }
    }
    if let Some(t) = setup.tyre {
        v += db.tyres[t].cost as i32;
        if mark {
            db.tyres[t].fitted = 0;
        }
    }
    let decal = |d: Option<usize>| d.map_or(0, |d| db.decals[d].unk_0c as u16 as i16 as i32);
    v += decal(setup.decal_0) + decal(setup.decal_1) + decal(setup.decal_2_4) + decal(setup.decal_2_5);
    for d in setup.decals_3 {
        v += decal(d);
    }
    for d in setup.decals_3_1.iter().chain(&setup.decals_3_2).chain(&setup.decals_3_3) {
        v += decal(*d);
    }
    v
}
