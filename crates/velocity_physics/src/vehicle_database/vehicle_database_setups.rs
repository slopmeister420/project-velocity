//! src/VehicleDatabase/VehicleDatabaseSetups.cpp — predefined setups: lookups and applying them to a
//! setup instance (the stock car every race starts from).

use super::vehicle_database::{CVehicleDatabase, NUM_BODYPART_TYPES, PART_NOS, PartRef, VehicleSetup, string_hash};

/// setup flag: the setup names a wheel.
pub const SETUP_FLAG_WHEEL: u32 = 4;
/// setup flag: hidden unless asked for.
pub const SETUP_FLAG_HIDDEN: u32 = 0x80;

impl CVehicleDatabase {
    /// VehicleDatabaseSetups.cpp 0x00189F00
    pub fn apply_predef_setup(&self, setup: &mut VehicleSetup, predef: usize) {
        let p = &self.setups[predef];
        setup.vehicle = self.find_vehicle(p.vehicle_hash);
        debug_assert!(setup.vehicle.is_some(), "pVehicleSetup->pBaseVehicleType");
        for i in 0..NUM_BODYPART_TYPES {
            if p.fit_bodypart[i] != 0 {
                setup.fitted_bodyparts[i] = p.setup_bodyparts[i];
            }
        }
        if p.flags & SETUP_FLAG_WHEEL != 0 {
            setup.wheel = self.find_wheel(p.wheel_hash);
            if setup.wheel.is_none() {
                log::warn!("Missing wheel information from vehicle Setup []");
            }
        }
    }

    /// VehicleDatabaseSetups.cpp 0x0018A210 — stock setup of a vehicle: its two predefined setups,
    /// the second NOS part, default colour, default brakes and medium tyres, no decals.
    pub fn setup_default_vehicle(&self, setup: &mut VehicleSetup, vehicle_hash: u32) -> bool {
        let vehicle = self.find_vehicle(vehicle_hash);
        if vehicle.is_none() {
            log::error!("Error: VEHICLE NOT FOUND .... Check case!!!");
        }
        debug_assert!(vehicle.is_some(), "pVehicle != NULL");
        let Some(vi) = vehicle else { return false };
        let v = &self.vehicles[vi];
        let Some(s0) = v.setup else {
            debug_assert!(false, "VehicleDatabaseSetups.cpp 202");
            return false;
        };
        setup.vehicle = Some(vi);
        for i in 0..NUM_BODYPART_TYPES {
            setup.fitted_bodyparts[i] = None;
        }
        self.apply_predef_setup(setup, s0);
        // the original dereferences a null setup pointer here when the second setup is missing
        if let Some(s1) = v.setup_2 {
            self.apply_predef_setup(setup, s1);
        }
        // the set of the front bumper; crashes in the original when no bumper is fitted
        if let Some(p) = setup.fitted_bodyparts[0] {
            let set_hash = self.bodypart_sets[p.set].hash;
            if let Some(si) = self.find_bodypart_set(set_hash) {
                let set = &self.bodypart_sets[si];
                if !set.parts[PART_NOS].is_empty() && set.part_counts[PART_NOS] >= 2 {
                    setup.fitted_bodyparts[PART_NOS] = Some(PartRef { set: si, part_type: PART_NOS, index: 1 });
                }
            }
        }
        let d = &self.vehicles[vi].data;
        if d.num_colours != 0 {
            for c in d.colours.iter().take(d.num_colours as usize) {
                if c.is_default == 1 {
                    self.set_setup_colour(setup, c.colour);
                    break;
                }
            }
        } else {
            let c = self.get_vehicle_colour(None, 0);
            self.set_setup_colour(setup, c);
        }
        for i in 1..4 {
            setup.colours[i].hash = 0;
            setup.colours[i].brand = 0;
        }
        setup.brake_parts[0] = self.find_brake_part(string_hash("BRAKE_DISC_DEFAULT"));
        setup.brake_parts[1] = self.find_brake_part(string_hash("BRAKE_PAD_DEFAULT"));
        setup.tyre = self.find_tyre(string_hash("TYRE_VS_MEDIUM"));
        setup.decal_0 = None;
        setup.decal_1 = None;
        for i in 0..10 {
            setup.decals_3[i] = None;
            setup.decals_3_flags[i] = 1;
        }
        for i in 0..2 {
            setup.decals_3_1[i] = None;
            setup.decals_3_1_flags[i] = 1;
            setup.decals_3_2[i] = None;
            setup.decals_3_2_flags[i] = 1;
            setup.decals_3_3[i] = None;
            setup.decals_3_3_flags[i] = 1;
        }
        setup.decal_2_4 = None;
        setup.decal_2_4_flags = 1;
        setup.decal_2_5 = None;
        setup.decal_2_5_flags = 1;
        true
    }

    /// VehicleDatabaseSetups.cpp 0x0018A6C0
    pub fn find_setup(&self, hash: u32) -> Option<usize> {
        self.setups.iter().position(|s| s.hash == hash)
    }

    /// VehicleDatabaseSetups.cpp 0x0018A770
    pub fn find_setup_by_name(&self, name: &str) -> Option<usize> {
        self.find_setup(string_hash(name))
    }

    fn setup_matches(s: &super::vehicle_database_setup_bin_loading::Setup, vehicle: u32, mask: u32, hidden: bool) -> bool {
        s.vehicle_hash == vehicle && (mask & s.flags) != 0 && (hidden || s.flags & SETUP_FLAG_HIDDEN == 0)
    }

    /// VehicleDatabaseSetups.cpp 0x0018A980 — `GetSetupCost(predef, nullptr)`: the predef's parts not flagged 8
    /// (SaveFileData 0x00106460) plus its wheel; the team discount branch (a2 != null) is not used by any port
    pub fn get_setup_cost(&self, predef: usize) -> i32 {
        let p = &self.setups[predef];
        let mut cost = 0i32;
        for i in 0..NUM_BODYPART_TYPES {
            if p.fit_bodypart[i] != 0 {
                if let Some(r) = p.setup_bodyparts[i] {
                    let part = self.part(r);
                    if part.flags & 8 != 8 {
                        cost = cost.wrapping_add(part.cost as i32);
                    }
                }
            }
        }
        if let Some(w) = self.find_wheel(p.wheel_hash) {
            cost = cost.wrapping_add(self.wheels[w].cost as i32);
        }
        cost
    }

    /// VehicleDatabaseSetups.cpp 0x0018A7B0
    pub fn get_nth_setup(&self, vehicle: u32, n: u32, mask: u32, hidden: bool) -> Option<usize> {
        self.setups.iter().enumerate().filter(|(_, s)| Self::setup_matches(s, vehicle, mask, hidden)).nth(n as usize).map(|x| x.0)
    }

    /// VehicleDatabaseSetups.cpp 0x0018A8A0
    pub fn count_setups(&self, vehicle: u32, mask: u32, hidden: bool) -> u32 {
        self.setups.iter().filter(|s| Self::setup_matches(s, vehicle, mask, hidden)).count() as u32
    }
}
