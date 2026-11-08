//! shared career team view: the ai owns `CTeamDatabase` (TeamDatabase.cpp) and mirrors it here after every change;
//! the front end (Tory's team invites, after-race "team unlocked") reads it and asks for changes with `TeamRequest`.
//! Types only; the filters are copies of `velocity_ai::team_database`'s.

use bevy::prelude::*;

/// one team as the front end reads it
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TeamView {
    /// +0x04 (TeamDatabase.cpp 0x0016F120 GetId)
    pub crc: u32,
    /// +0x08 name string crc (FEHangoutProgression.cpp 0x00290CB0)
    pub name: u32,
    /// +0x1C team car (0x00290CD0)
    pub car: u32,
    /// +0x20 invite page STATUS value (0x00290D10)
    pub status: u32,
    /// +0x28 cash bonus on joining (0x00290CF0)
    pub cash_bonus: u32,
    /// +0x3C: 1 unlocked, 2 offering invite, 0x10 joined once, 0x20 newly unlocked
    pub flags: u32,
}

impl TeamView {
    /// TeamDatabase.cpp 0x0016F1A0
    pub fn is_offering_invite(&self) -> bool {
        (self.flags & 2) == 2
    }
    /// EventDatabaseEvents.cpp 0x00093D30
    pub fn is_locked(&self) -> bool {
        (self.flags & 1) != 1
    }
    /// FEHangoutProgression.cpp 0x00290D30
    pub fn joined_once_already(&self) -> bool {
        (self.flags & 0x10) == 0x10
    }
    /// FEHangoutProgression.cpp 0x00290D90
    pub fn is_newly_unlocked(&self) -> bool {
        (self.flags & 0x20) == 0x20
    }
}

/// the ai's team list in the original list order
#[derive(Resource, Default, Clone, Debug)]
pub struct CareerTeams {
    pub teams: Vec<TeamView>,
}

impl CareerTeams {
    /// VsCommonMain.cpp 0x0016E610 — first team with that crc
    pub fn find_team(&self, crc: u32) -> Option<&TeamView> {
        self.teams.iter().find(|t| t.crc == crc)
    }

    /// VsCommonMain.cpp 0x0016E8B0 — teams offering invites other than `exclude` (the player's team)
    pub fn count_teams_offering_invites(&self, exclude: u32) -> usize {
        self.teams.iter().filter(|t| t.is_offering_invite() && exclude != t.crc).count()
    }

    /// TeamDatabase.cpp 0x0016E980 — their crcs into `cache` (list order); returns the count stored
    pub fn get_teams_offering_invites(&self, exclude: u32, cache: &mut [u32]) -> usize {
        let mut n = 0;
        for t in &self.teams {
            if t.is_offering_invite() && exclude != t.crc {
                if n >= cache.len() {
                    log::warn!("WARNING: Team id cache appears to be too small when calling CTeamDatabase::GetTeamsOfferingInvites");
                } else {
                    cache[n] = t.crc;
                    n += 1;
                }
            }
        }
        n
    }
}

/// front end → ai team changes (applied by the ai in its `Last` system)
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub enum TeamRequest {
    /// FEHangoutProgression.cpp 0x00290D60
    ClearNewlyUnlocked(u32),
    /// FEHangoutProgression.cpp 0x00290DC0
    SetJoinedOnce(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invite_filters_keep_list_order() {
        let t = |crc, flags| TeamView { crc, flags, ..Default::default() };
        let teams = CareerTeams { teams: vec![t(1, 3), t(2, 1), t(3, 2), t(1, 0)] };
        assert_eq!(teams.find_team(1).unwrap().flags, 3);
        assert_eq!(teams.count_teams_offering_invites(3), 1);
        let mut cache = [0; 1];
        assert_eq!(teams.get_teams_offering_invites(0, &mut cache), 1);
        assert_eq!(cache, [1]);
    }
}
