//! glue: the opponents' and teams' cars PhysicsManager 0x0019E080 asks for (`RaceCarSource`), from this crate's
//! opponent and team databases (game flow cannot see them).

use velocity_game::game_mode::race_cars::RaceCarSource;
use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};

use crate::opponent_database::OpponentDatabase;
use crate::team_database::TeamDatabase;

pub struct AiRaceCarSource {
    pub opponents: Option<OpponentDatabase>,
    pub teams: Option<TeamDatabase>,
}

impl AiRaceCarSource {
    pub fn load() -> Self {
        Self { opponents: OpponentDatabase::load(), teams: TeamDatabase::load() }
    }
}

impl RaceCarSource for AiRaceCarSource {
    fn opponent_car(&self, db: &CVehicleDatabase, crc: u32, index: usize) -> Option<(VehicleSetup, String)> {
        self.opponents.as_ref()?.build_vehicle_setup(db, crc, index)
    }

    /// VsCommonMain 0x0016E610 finds the opponent's team (`+0x24`); the team's `+0x00` holds the stock setup of
    /// the vehicle named by the team crc when one exists (TeamDatabase 0x0016EC30: FindVehicle(+0x04) then
    /// SetupDefaultVehicle(+0x04), else null) — PhysicsManager 0x0019E45C reads `+0x00`, not the `+0x1C` team car
    fn team_car(&self, db: &CVehicleDatabase, crc: u32) -> Option<VehicleSetup> {
        let o = self.opponents.as_ref()?.get_opponent_by_crc(crc)?;
        let t = self.teams.as_ref()?.find_team(o.group)?;
        db.find_vehicle(t.crc)?;
        let mut s = VehicleSetup::new();
        db.setup_default_vehicle(&mut s, t.crc).then_some(s)
    }

    /// FEHangoutProgression 0x0028F430: the team by crc (VsCommonMain 0x0016E610), its car `+0x1C`
    /// (FEHangoutProgression 0x00290CD0) through `CVehicleDatabase::SetupDefaultVehicle`
    fn team_car_by_team(&self, db: &CVehicleDatabase, team: u32) -> Option<VehicleSetup> {
        let Some(t) = self.teams.as_ref()?.find_team(team) else {
            log::error!("assert: Player team CRC appears invalid.");
            return None;
        };
        let mut s = VehicleSetup::new();
        if !db.setup_default_vehicle(&mut s, t.vehicle_crc) {
            log::error!("assert: Problem getting team car information. ({:08x})", t.vehicle_crc);
            return None;
        }
        Some(s)
    }

    /// PhysicsManager 0x0019E080 "Police Car setup": opponent `n % count` of the group with flag 2
    /// (OpponentDatabase 0x000E7B50 / 0x000E7C20), its first car
    fn police_car(&self, db: &CVehicleDatabase, group: u32, n: usize) -> Option<(VehicleSetup, String)> {
        let o = self.opponents.as_ref()?;
        let count = o.get_opponent_count(group, 2);
        if count <= 0 {
            log::error!("assert: dPoliceOpponentCount > 0");
            return None;
        }
        let p = o.get_opponent((n as i16) % count, group, 2)?;
        o.build_vehicle_setup(db, p.name_crc, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_team_has_a_car() {
        let (Ok(vdb), s) = (CVehicleDatabase::load(), AiRaceCarSource::load()) else { return };
        let Some(teams) = s.teams.as_ref() else { return };
        for t in &teams.teams {
            assert!(s.team_car_by_team(&vdb, t.crc).is_some(), "team {:08x} car {:08x}", t.crc, t.vehicle_crc);
        }
    }
}
