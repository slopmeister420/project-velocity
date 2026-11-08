//! Team database: `TEAMS.BIN` (src/TeamDatabase/TeamDatabase.cpp).
//!
//! The game reads it from `\VEHICLES\XMLBIN.VCD`; a loose copy also ships as `AI/TEAMS.BIN`.
//! Layout: `u32 count | u32 string pool size | count x team | string pool`, where a team is
//! `crc, vehicle, (skipped), 4 x u32, flags, 2 x string pool offset, n + n car-maker sponsors,
//! n + n part-maker sponsors, f32`.

#[derive(Debug, thiserror::Error)]
pub enum TeamDbError {
    #[error("truncated team database")]
    Truncated,
}

/// one team (72 bytes at runtime, `+0x44` = next).
#[derive(Debug, Clone, Default)]
pub struct Team {
    pub crc: u32,                          // +0x04
    pub unk_08: u32,                       // +0x08 (+0x00 holds a VehicleSetup when a vehicle with the team crc exists)
    pub skipped: u32,                      // file-only word the loader steps over
    /// +0x1C the team car (FEHangoutProgression 0x00290CD0, set up by `SetupDefaultVehicle`)
    pub vehicle_crc: u32,
    pub unk_20: u32,                       // +0x20
    pub unk_24: u32,                       // +0x24
    pub unk_28: u32,                       // +0x28
    pub flags: u32,                        // +0x3C bit0 unlocked, bit1 offering invites, 0x20 newly unlocked
    pub string_0c: String,                 // +0x0C string pool entry
    pub string_14: String,                 // +0x14 string pool entry
    pub car_manufacturer_sponsors: Vec<u32>, // +0x2C count, +0x30 mpCarManufacturerSponsors
    pub part_manufacturer_sponsors: Vec<u32>, // +0x34 count, +0x38 mpPartManufacturerSponsors
    pub unk_40: f32,                       // +0x40
}

impl Team {
    /// TeamDatabase.cpp 0x0016F1A0
    pub fn is_offering_invite(&self) -> bool {
        (self.flags & 2) == 2
    }
    /// TeamDatabase.cpp 0x0016F120
    pub fn get_id(&self) -> u32 {
        self.crc
    }
    /// EventDatabaseEvents.cpp 0x00093D30
    pub fn is_locked(&self) -> bool {
        (self.flags & 1) != 1
    }
    /// EventDatabaseEvents.cpp 0x00093CA0
    pub fn unlock(&mut self) {
        self.flags |= 1;
    }
    /// EventDatabaseEvents.cpp 0x00093CD0
    pub fn set_newly_unlocked(&mut self) {
        self.flags |= 0x20;
    }
    /// EventDatabaseEvents.cpp 0x00093D00
    pub fn set_offering_invite(&mut self) {
        self.flags |= 2;
    }
    /// TeamDatabase.cpp 0x0016F140
    pub fn lock(&mut self) {
        self.flags &= !1;
    }
    /// TeamDatabase.cpp 0x0016F170
    pub fn clear_offering_invite(&mut self) {
        self.flags &= !2;
    }
    /// FEHangoutProgression.cpp 0x00290CB0 — team name string crc (+0x08)
    pub fn name(&self) -> u32 {
        self.unk_08
    }
    /// FEHangoutProgression.cpp 0x00290CD0 — team car (+0x1C)
    pub fn car(&self) -> u32 {
        self.vehicle_crc
    }
    /// FEHangoutProgression.cpp 0x00290CF0 — cash bonus on joining (+0x28, "idCashBonus")
    pub fn cash_bonus(&self) -> u32 {
        self.unk_28
    }
    /// FEHangoutProgression.cpp 0x00290D10 — invite page STATUS value (+0x20)
    pub fn status(&self) -> u32 {
        self.unk_20
    }
    /// FEHangoutProgression.cpp 0x00290D30
    pub fn joined_once_already(&self) -> bool {
        (self.flags & 0x10) == 0x10
    }
    /// FEHangoutProgression.cpp 0x00290DC0
    pub fn set_joined_once(&mut self) {
        self.flags |= 0x10;
    }
    /// FEHangoutProgression.cpp 0x00290D90
    pub fn is_newly_unlocked(&self) -> bool {
        (self.flags & 0x20) == 0x20
    }
    /// FEHangoutProgression.cpp 0x00290D60
    pub fn clear_newly_unlocked(&mut self) {
        self.flags &= !0x20;
    }
}

#[derive(Debug, Clone, Default)]
pub struct TeamDatabase {
    /// in load order; the original pushes to the head of a list, so its iteration order is reversed
    pub teams: Vec<Team>,
}

fn rd(d: &[u8], p: &mut usize) -> Result<u32, TeamDbError> {
    let v = d.get(*p..*p + 4).ok_or(TeamDbError::Truncated)?;
    *p += 4;
    Ok(u32::from_le_bytes(v.try_into().unwrap()))
}

fn pool_str(pool: &[u8], off: u32) -> String {
    let s = pool.get(off as usize..).unwrap_or(&[]);
    let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf8_lossy(&s[..end]).into_owned()
}

impl TeamDatabase {
    /// reads `TEAMS.BIN` out of `\VEHICLES\XMLBIN.VCD` (or its loose folder) in the game dir.
    /// TeamDatabase.cpp 0x0016EC30
    pub fn load() -> Option<Self> {
        let dir = visos::file::vs_file_directory::VsFileDirectoryDetails::open("\\VEHICLES\\XMLBIN.VCD").ok()?;
        let data = dir.load_file("TEAMS.BIN").ok()?;
        Self::read_from_binary(&data).ok()
    }

    /// TeamDatabase.cpp 0x0016EC30
    pub fn read_from_binary(d: &[u8]) -> Result<Self, TeamDbError> {
        let mut p = 0usize;
        let mut count = rd(d, &mut p)?;
        let pool_size = rd(d, &mut p)? as usize;
        let mut raw = Vec::new();
        while count != 0 {
            let crc = rd(d, &mut p)?;
            let unk_08 = rd(d, &mut p)?;
            let skipped = rd(d, &mut p)?;
            let vehicle_crc = rd(d, &mut p)?;
            let unk_20 = rd(d, &mut p)?;
            let unk_24 = rd(d, &mut p)?;
            let unk_28 = rd(d, &mut p)?;
            let flags = rd(d, &mut p)?;
            let off_0c = rd(d, &mut p)?;
            let off_14 = rd(d, &mut p)?;
            let n = rd(d, &mut p)?;
            let mut cars = Vec::new();
            for _ in 0..n {
                cars.push(rd(d, &mut p)?);
            }
            let n = rd(d, &mut p)?;
            let mut parts = Vec::new();
            for _ in 0..n {
                parts.push(rd(d, &mut p)?);
            }
            let unk_40 = f32::from_bits(rd(d, &mut p)?);
            raw.push((
                Team {
                    crc,
                    unk_08,
                    skipped,
                    vehicle_crc,
                    unk_20,
                    unk_24,
                    unk_28,
                    flags,
                    car_manufacturer_sponsors: cars,
                    part_manufacturer_sponsors: parts,
                    unk_40,
                    ..Default::default()
                },
                off_0c,
                off_14,
            ));
            count -= 1;
        }
        let pool = d.get(p..p + pool_size).ok_or(TeamDbError::Truncated)?;
        let mut db = Self::default();
        for (mut t, a, b) in raw {
            if db.teams.iter().any(|x| x.crc == t.crc) {
                log::warn!("WARNING: Attempted to create a team with a duplicate CRC.");
                continue;
            }
            t.string_0c = pool_str(pool, a);
            t.string_14 = pool_str(pool, b);
            db.teams.push(t);
        }
        Ok(db)
    }

    /// VsCommonMain.cpp 0x0016E610 — the team by crc
    pub fn find_team(&self, crc: u32) -> Option<&Team> {
        self.teams.iter().rev().find(|t| t.get_id() == crc)
    }

    pub fn find_team_mut(&mut self, crc: u32) -> Option<&mut Team> {
        self.teams.iter_mut().rev().find(|t| t.get_id() == crc)
    }

    /// the team unlock of EventDatabaseBinLoading.cpp 0x00091890 (a successful event with an unlock team)
    pub fn unlock_team(&mut self, crc: u32) {
        match self.find_team_mut(crc) {
            Some(t) => {
                if t.is_locked() {
                    t.unlock();
                    t.set_newly_unlocked();
                    t.set_offering_invite();
                }
            }
            None => log::warn!("WARNING: Event trying to unlock a team with an unknown CRC."),
        }
    }

    /// VsCommonMain.cpp 0x0016E790 RefreshAllTeamsLockStatus — every team locked without invites, then the teams
    /// of completed events (`CEvent+0x15 & 2`, `+0x40`) unlocked and inviting
    pub fn refresh_all_teams_lock_status(&mut self, completed_event_teams: impl IntoIterator<Item = (bool, u32)>) {
        for t in self.teams.iter_mut().rev() {
            t.clear_offering_invite();
            t.lock();
        }
        for (completed, team) in completed_event_teams {
            if completed && team != 0 {
                match self.find_team_mut(team) {
                    Some(t) => {
                        t.unlock();
                        t.set_offering_invite();
                    }
                    None => log::warn!("WARNING: RefreshAllTeamsLockStatus() encountered a bad team CRC."),
                }
            }
        }
    }

    /// VsCommonMain.cpp 0x0016E8B0 — teams offering invites other than `exclude` (the player's team)
    pub fn count_teams_offering_invites(&self, exclude: u32) -> usize {
        self.teams.iter().rev().filter(|t| t.is_offering_invite() && exclude != t.get_id()).count()
    }

    /// TeamDatabase.cpp 0x0016E980
    pub fn get_teams_offering_invites(&self, cache: &mut [u32], exclude: u32) -> usize {
        let mut n = 0;
        // original list order (most recently created first)
        for t in self.teams.iter().rev() {
            if t.is_offering_invite() && exclude != t.get_id() {
                if n >= cache.len() {
                    log::warn!("WARNING: Team id cache appears to be too small when calling CTeamDatabase::GetTeamsOfferingInvites");
                } else {
                    cache[n] = t.get_id();
                    n += 1;
                }
            }
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_game_teams_bin() {
        let Some(dir) = crate::game_dir() else { return };
        let data = std::fs::read(dir.join("AI").join("TEAMS.BIN")).unwrap();
        let db = TeamDatabase::read_from_binary(&data).unwrap();
        assert_eq!(db.teams.len(), 5);
        assert_eq!(db.teams[0].string_0c, "black");
        assert_eq!(db.teams[0].string_14, "white");
        assert_eq!(db.teams[1].string_0c, "white");
        assert_eq!(db.teams[1].string_14, "silver");
        assert!(!TeamDatabase::load().expect("TEAMS.BIN in XMLBIN.VCD").teams.is_empty());
    }

    #[test]
    fn unlock_and_refresh() {
        let Some(mut db) = TeamDatabase::load() else { return };
        let (a, b) = (db.teams[0].crc, db.teams[1].crc);
        db.refresh_all_teams_lock_status([(true, a), (false, b)]);
        assert_eq!(db.find_team(a).unwrap().flags & 3, 3);
        assert_eq!(db.find_team(b).unwrap().flags & 3, 0);
        db.unlock_team(b);
        assert_eq!(db.find_team(b).unwrap().flags & 0x23, 0x23);
        db.unlock_team(a);
        assert_eq!(db.find_team(a).unwrap().flags & 0x20, 0, "an unlocked team is left alone");
        let mut n = [0; 8];
        assert_eq!(db.get_teams_offering_invites(&mut n, 0), 2);
        assert_eq!(db.count_teams_offering_invites(a), 1);
        let t = db.find_team_mut(b).unwrap();
        assert!(t.is_newly_unlocked() && !t.joined_once_already());
        t.clear_newly_unlocked();
        t.set_joined_once();
        assert_eq!(t.flags & 0x30, 0x10);
    }
}
