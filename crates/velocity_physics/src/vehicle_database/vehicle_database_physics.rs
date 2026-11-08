//! src/VehicleDatabase/VehicleDatabasePhysics.cpp — performance modifiers of a setup instance, from
//! the upgrade levels of its fitted bodyparts, brake parts and tyre (read by `VehicleEntity::Create`).
//! levels are part +0x21 (bodypart) / +0x19 (brake part); all modifiers are clamped to 0..1.

use super::vehicle_database::{
    CVehicleDatabase, PART_AIRFILTER, PART_ENGINE, PART_EXHAUST, PART_FLYWHEEL, PART_NOS, PART_SUSPENSION, PART_TURBO,
    VehicleSetup,
};
use crate::fastcar::vsmath::clamp as vs_clamp;

/// `dword_688A74` — weight of engine+turbo in the acceleration modifier (data, 0.7f).
pub const ACCELERATION_ENGINE_WEIGHT: f32 = f32::from_bits(0x3F33_3333);
/// `dword_688A78` — weight of engine+turbo in the top speed modifier (data, 0.7f).
pub const TOP_SPEED_ENGINE_WEIGHT: f32 = f32::from_bits(0x3F33_3333);

/// the six values `VehicleEntity::Create` takes from a setup instance.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PerformanceModifiers {
    pub acceleration: f32,  // -> vehicle +112
    pub top_speed: f32,     // -> vehicle +116
    pub nos: f32,           // -> vehicle +100 (GetNosModifier returns in st0; Create stores it)
    pub tire_softness: f32, // -> vehicle +124
    pub body_roll: f32,     // -> vehicle +128
    pub brake: f32,         // -> vehicle +120
}

impl PerformanceModifiers {
    /// acceleration, top speed, tire softness, body roll, brake.
    pub fn as_array(&self) -> [f32; 5] {
        [self.acceleration, self.top_speed, self.tire_softness, self.body_roll, self.brake]
    }
}

impl VehicleSetup {
    /// VehicleDatabasePhysics.cpp 0x001895C0
    pub fn get_bodypart_level(&self, db: &CVehicleDatabase, part_type: usize) -> u32 {
        match self.fitted_bodyparts[part_type] {
            Some(p) => db.part(p).level as u32,
            None => 0,
        }
    }

    /// VehicleDatabasePhysics.cpp 0x00189600
    pub fn get_brake_part_level(&self, db: &CVehicleDatabase, slot: usize) -> u32 {
        match self.brake_parts[slot] {
            Some(b) => db.brake_parts[b].level as u32,
            None => 0,
        }
    }

    /// VehicleDatabasePhysics.cpp 0x00189640
    pub fn get_acceleration_modifier(&self, db: &CVehicleDatabase) -> f32 {
        // fild/fadd/fstp, then sse
        let a = self.get_bodypart_level(db, PART_ENGINE) as f32;
        let mut engine = (self.get_bodypart_level(db, PART_TURBO) as f64 + a as f64) as f32;
        engine /= 10.0f32;
        // the "Handling.Performance" tweak edits the global
        let weight = crate::physics_tweaks::global_f32(0x0068_8A74, ACCELERATION_ENGINE_WEIGHT);
        engine *= weight;
        let b = self.get_bodypart_level(db, PART_AIRFILTER) as f32;
        let mut intake = (self.get_bodypart_level(db, PART_FLYWHEEL) as f64 + b as f64) as f32;
        intake /= 10.0f32;
        intake = (1.0f32 - weight) * intake;
        vs_clamp(engine + intake, 0.0, 1.0)
    }

    /// VehicleDatabasePhysics.cpp 0x00189750
    pub fn get_top_speed_modifier(&self, db: &CVehicleDatabase) -> f32 {
        let a = self.get_bodypart_level(db, PART_ENGINE) as f32;
        let mut engine = (self.get_bodypart_level(db, PART_TURBO) as f64 + a as f64) as f32;
        engine /= 10.0f32;
        let weight = crate::physics_tweaks::global_f32(0x0068_8A78, TOP_SPEED_ENGINE_WEIGHT);
        engine *= weight;
        let mut exhaust = self.get_bodypart_level(db, PART_EXHAUST) as f32;
        exhaust = vs_clamp(exhaust, 0.0, 5.0);
        exhaust /= 5.0f32;
        exhaust = (1.0f32 - weight) * exhaust;
        vs_clamp(engine + exhaust, 0.0, 1.0)
    }

    /// VehicleDatabasePhysics.cpp 0x00189850
    pub fn get_brake_modifier(&self, db: &CVehicleDatabase) -> f32 {
        let a = self.get_brake_part_level(db, 0) as f32;
        let mut v = (self.get_brake_part_level(db, 1) as f64 + a as f64) as f32;
        v /= 10.0f32;
        vs_clamp(v, 0.0, 1.0)
    }

    /// VehicleDatabasePhysics.cpp 0x001898E0 — tyre grip class plus suspension level.
    pub fn get_tire_softness_modifier(&self, db: &CVehicleDatabase) -> f32 {
        let mut v = 0.0f32;
        match self.tyre {
            Some(t) => match db.tyres[t].grip {
                0 => v = 0.9f32,
                1 => v = 0.7f32,
                2 => v = 0.5f32,
                3 => v = 0.3f32,
                4 => v = 0.1f32,
                // the original asserts (line 146) and keeps 0.0
                _ => debug_assert!(false, "VehicleDatabasePhysics.cpp 146"),
            },
            None => v = 0.5f32,
        }
        // x87: (level - 2.0f) * -0.05f + v, one store
        let lvl = self.get_bodypart_level(db, PART_SUSPENSION) as f64;
        let v = ((lvl - 2.0) * (-0.05f32) as f64 + v as f64) as f32;
        vs_clamp(v, 0.0, 1.0)
    }

    /// VehicleDatabasePhysics.cpp 0x00189A50
    pub fn get_body_roll_modifier(&self, db: &CVehicleDatabase) -> f32 {
        let v = (self.get_bodypart_level(db, PART_SUSPENSION) as f64 / 4.0) as f32;
        vs_clamp(v, 0.0, 1.0)
    }

    /// VehicleDatabasePhysics.cpp 0x00189AB0 — returned in st0 (level * 5 * 24).
    pub fn get_nos_modifier(&self, db: &CVehicleDatabase) -> f32 {
        (self.get_bodypart_level(db, PART_NOS) as f64 * 5.0 * 24.0) as f32
    }

    /// VehicleDatabasePhysics.cpp 0x00189420 — base top speed (vehicle stats +0) scaled by up to 25%.
    pub fn get_top_speed(&self, db: &CVehicleDatabase) -> u32 {
        let base = db.vehicles[self.vehicle.expect("setup vehicle")].stats.values[0] as i64 as f32;
        let q = base * 0.25f32;
        let t = (self.get_top_speed_modifier(db) as f64 * q as f64) as f32;
        (base as f64 + t as f64) as i64 as u32
    }

    /// VehicleDatabasePhysics.cpp 0x00189490 — the same base value scaled by the acceleration modifier.
    pub fn get_acceleration(&self, db: &CVehicleDatabase) -> u32 {
        let base = db.vehicles[self.vehicle.expect("setup vehicle")].stats.values[0] as i64 as f32;
        let q = base * 0.25f32;
        let t = (self.get_acceleration_modifier(db) as f64 * q as f64) as f32;
        (base as f64 + t as f64) as i64 as u32
    }

    /// VehicleDatabasePhysics.cpp 0x00189590
    pub fn get_handling_modifier(&self, db: &CVehicleDatabase) -> f32 {
        self.get_tire_softness_modifier(db)
    }

    /// the getters in the order `VehicleEntity::Create` (0x001CABF0) calls them.
    pub fn performance_modifiers(&self, db: &CVehicleDatabase) -> PerformanceModifiers {
        PerformanceModifiers {
            acceleration: self.get_acceleration_modifier(db),
            top_speed: self.get_top_speed_modifier(db),
            nos: self.get_nos_modifier(db),
            tire_softness: self.get_tire_softness_modifier(db),
            body_roll: self.get_body_roll_modifier(db),
            brake: self.get_brake_modifier(db),
        }
    }

    /// name hash physics reads as `***a6` (vehicle entry +0).
    pub fn vehicle_hash(&self, db: &CVehicleDatabase) -> Option<u32> {
        self.vehicle.map(|v| db.vehicles[v].hash)
    }
}
