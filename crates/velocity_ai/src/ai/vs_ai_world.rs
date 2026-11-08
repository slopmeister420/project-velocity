//! The ai world: road network, race route graph, handling and the race/traffic/police managers
//! (src/Ai/VsAIWorld.cpp).
//!
//! Track data (`Tracks/<area>/<mode>/<race>/`):
//! - `AISpline.spl`: `"AISPLINES" | u8 version (2) | pad | u32 counts[7] @0x10` then race sections
//!   (3 road lines each: raceline, left, right), road sections of the four sets (16-byte header with the
//!   lane count, then lanes as road lines), traffic blockers (32-byte header + lane ids, padded to 4),
//!   traffic lights (32 bytes). Counts: race sections, traffic sections, set B, set C, lights,
//!   blockers, trail blazer sections.
//! - `barier01.br` (arrow barriers), `noentr01.br` (no-entry barriers): `"SPL0" | u32 count | 8 pad`
//!   then `count` road lines.

use super::vs_race_manager::VsRaceManager;
use std::path::{Path, PathBuf};

use super::vs_ai_globals::VsAIGlobals;
use super::vs_ai_response_curve::VsAIResponseCurve;
use super::vs_ai_road_line::VsAIRoadLine;
use super::vs_ai_road_section::*;
use super::vs_ai_tools::VsAITools;
use super::vs_math::VsVector4;
use crate::crc::string_crc;
use crate::text_file::VsTextFile;

pub const AI_SPLINE_VERSION: u8 = 2;

#[derive(Debug, thiserror::Error)]
pub enum SplineError {
    #[error("*** AI Data mismatch in AISpline.spl - Version is {0}, expected 2")]
    Version(u8),
    #[error("truncated ai spline data")]
    Truncated,
}

fn rd32(d: &[u8], p: usize) -> Result<u32, SplineError> {
    d.get(p..p + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or(SplineError::Truncated)
}

/// counts in the `AISpline.spl` header (`m_pSplineHeader`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VsAISplineHeader {
    pub version: u8,
    pub num_race_sections: u32,      // +0x10
    pub num_traffic_sections: u32,   // +0x14
    pub num_sections_b: u32,         // +0x18
    pub num_sections_c: u32,         // +0x1C
    pub num_traffic_lights: u32,     // +0x20
    pub num_traffic_blockers: u32,   // +0x24
    pub num_trail_blazer_sections: u32, // +0x28
}

/// parsed `AISpline.spl`.
#[derive(Debug, Clone, Default)]
pub struct VsAISplineData {
    pub header: VsAISplineHeader,
    pub race_sections: Vec<VsAIRaceSection>,
    pub road_sections: [Vec<VsAIRoadSection>; 4],
    pub traffic_blockers: Vec<VsAITrafficBlocker>,
    pub traffic_lights: Vec<VsAITrafficLightData>,
    /// bytes consumed, for the positive control
    pub size: usize,
}

impl VsAISplineData {
    /// the in-place pointer walk of `SetupSplineData`, minus the lookups that need the whole world.
    /// VsAIWorld.cpp 0x00030760
    pub fn parse(d: &[u8]) -> Result<Self, SplineError> {
        let version = *d.get(9).ok_or(SplineError::Truncated)?;
        if version != AI_SPLINE_VERSION {
            log::error!("*** AI Data mismatch in AISpline.spl - Version is {version}, expected 2");
            return Err(SplineError::Version(version));
        }
        let header = VsAISplineHeader {
            version,
            num_race_sections: rd32(d, 0x10)?,
            num_traffic_sections: rd32(d, 0x14)?,
            num_sections_b: rd32(d, 0x18)?,
            num_sections_c: rd32(d, 0x1C)?,
            num_traffic_lights: rd32(d, 0x20)?,
            num_traffic_blockers: rd32(d, 0x24)?,
            num_trail_blazer_sections: rd32(d, 0x28)?,
        };
        let mut p = 48usize;
        let mut out = Self { header, ..Default::default() };
        for _ in 0..header.num_race_sections {
            let mut s = VsAIRaceSection::new();
            s.raceline = VsAIRoadLine::read(d, &mut p).ok_or(SplineError::Truncated)?;
            s.left = VsAIRoadLine::read(d, &mut p).ok_or(SplineError::Truncated)?;
            s.right = VsAIRoadLine::read(d, &mut p).ok_or(SplineError::Truncated)?;
            out.race_sections.push(s);
        }
        let counts = [
            header.num_traffic_sections,
            header.num_sections_b,
            header.num_sections_c,
            header.num_trail_blazer_sections,
        ];
        for (set, &n) in counts.iter().enumerate() {
            out.road_sections[set] = Self::setup_road_sections(d, &mut p, n)?;
        }
        for _ in 0..header.num_traffic_blockers {
            let h = d.get(p..p + 32).ok_or(SplineError::Truncated)?;
            let f = |o: usize| f32::from_le_bytes(h[o..o + 4].try_into().unwrap());
            let w = |o: usize| u32::from_le_bytes(h[o..o + 4].try_into().unwrap());
            let mut b = VsAITrafficBlocker {
                pos: VsVector4::new(f(0), f(4), f(8), f(12)),
                id: w(16),
                flag: w(20),
                physics_object_id: w(28),
                ..Default::default()
            };
            p += 32;
            let n = w(24);
            for _ in 0..n {
                b.lane_ids.push(rd32(d, p)?);
                p += 4;
            }
            b.active = b.flag != 0;
            // pad the id list to a multiple of 4 entries
            let r = n % 4;
            let mut pad = 4i32 - r as i32;
            if r == 0 {
                pad -= 4;
            }
            p += 4 * pad as usize;
            out.traffic_blockers.push(b);
        }
        for _ in 0..header.num_traffic_lights {
            let w = |o: usize| rd32(d, p + o);
            let l = VsAITrafficLightData {
                id: w(0)?,
                cycle_time: f32::from_bits(w(4)?),
                blocker_ids: [w(8)?, w(12)?, w(16)?, w(20)?],
                record_tail: [w(24)?, w(28)?],
                blockers: [None; 4],
                ..Default::default()
            };
            p += 32;
            out.traffic_lights.push(l);
        }
        out.size = p;
        Ok(out)
    }

    /// VsAIWorld.cpp 0x00030580
    fn setup_road_sections(d: &[u8], p: &mut usize, n: u32) -> Result<Vec<VsAIRoadSection>, SplineError> {
        let mut v = Vec::new();
        for _ in 0..n {
            let header = [rd32(d, *p)?, rd32(d, *p + 4)?, rd32(d, *p + 8)?, rd32(d, *p + 12)?];
            *p += 16;
            let mut s = VsAIRoadSection { header, lanes: Vec::new() };
            for _ in 0..s.get_num_lanes() {
                let line = VsAIRoadLine::read(d, p).ok_or(SplineError::Truncated)?;
                s.lanes.push(VsAILane::new(line));
            }
            v.push(s);
        }
        Ok(v)
    }
}

/// a barrier file (`barier01.br` / `noentr01.br`).
#[derive(Debug, Clone, Default)]
pub struct VsAIBarriers {
    pub lines: Vec<VsAIRoadLine>,
    pub size: usize,
}

impl VsAIBarriers {
    /// VsAIWorld.cpp 0x00033FC0
    pub fn parse(d: &[u8]) -> Option<Self> {
        // VsAIWorld.cpp 0x000341F0: 16-byte header, count at +4
        let count = u32::from_le_bytes(d.get(4..8)?.try_into().unwrap());
        let mut p = 16usize;
        let mut lines = Vec::new();
        for _ in 0..count {
            lines.push(VsAIRoadLine::read(d, &mut p)?);
        }
        Some(Self { lines, size: p })
    }
}

/// per-car ai handling (52 bytes), `AI/Handling/<car>.txt`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VsAIHandling {
    pub crc: u32,                     // +0x00 crc of the requested car name
    pub speed_ratio_rc: usize,        // +0x04 index into `VsAIWorld::response_curves`
    pub steering_rc: usize,           // +0x08
    pub throttle_rc: usize,           // +0x0C
    pub brake_rc: usize,              // +0x10
    pub sp3_rc: usize,                // +0x14 meant to be RC_sp3, see `load_handling`
    pub max_speed: f32,               // +0x18
    pub look_ahead_min: f32,          // +0x1C
    pub look_ahead_max: f32,          // +0x20
    pub steering_factor: f32,         // +0x24
    pub throttle_factor: f32,         // +0x28
    pub brake_factor: f32,            // +0x2C
    pub steer_damping_factor: f32,    // +0x30
}

/// rev curve names (0x00684E84, count 0x00684E80)
pub const STARTLINE_REVS: [&str; 3] = ["Revs1", "Revs2", "Revs3"];

/// the ai world data (`VsAIWorld`, 0x4820 bytes). Managers are owned separately.
#[derive(Debug, Default)]
pub struct VsAIWorld {
    pub globals: VsAIGlobals,             // 0x006D3698 tweak block
    pub debug_vehicle_id: u32,            // +0x4604
    pub flag_4608: bool,                  // +0x4608
    pub level_path: String,               // +0x4664
    pub initialised: bool,                // +0x4764
    pub traffic_enabled: bool,            // +0x4765
    pub police_enabled: bool,             // +0x4766
    pub flag_4767: bool,                  // +0x4767
    pub barriers_enabled: bool,           // +0x4768
    pub armco_enabled: bool,              // +0x4769
    pub spline: Option<VsAISplineData>,   // +0x476C header, +0x4770 data
    pub barriers: [Option<VsAIBarriers>; 2], // +0x4774 arrow, +0x4778 no-entry (data); +0x47E8 lines
    pub sub_route_points: Vec<VsAISubRoutePoint>, // +0x47A0 points, +0x47D4 count
    pub untracked_sections: u32,          // +0x47CC
    pub prev_untracked_sections: u32,     // +0x47D0
    pub time_step: f32,                   // +0x47D8
    pub time: f32,                        // +0x47DC
    pub handlings: Vec<VsAIHandling>,     // +0x47A4 VsArray<VsAIHandling*>
    pub response_curves: Vec<VsAIResponseCurve>, // +0x47B8 VsArray<VsAIResponseCurve*>
    /// `\AI` directory used for handling and response curves
    pub ai_dir: PathBuf,
    /// 0x00684E74 spin impulse of a puncture; its sign flips on some failures
    pub spin_impulse: f32,
}

impl VsAIWorld {
    /// VsAIVehicle.cpp 0x0002C7D0
    pub fn new(ai_dir: impl Into<PathBuf>) -> Self {
        Self { ai_dir: ai_dir.into(), spin_impulse: f32::from_bits(0x469C_4000), ..Default::default() }
    }

    fn spline(&self) -> Option<&VsAISplineData> {
        self.spline.as_ref()
    }

    /// VsAIRaceVehicle.cpp 0x00024330
    pub fn get_num_race_sections(&self) -> u32 {
        self.spline().map_or(0, |s| s.header.num_race_sections)
    }
    /// VsAIWorld.cpp 0x00037B80
    pub fn get_num_traffic_sections(&self) -> u32 {
        self.spline().map_or(0, |s| s.header.num_traffic_sections)
    }
    /// VsAIWorld.cpp 0x00037BB0
    pub fn get_num_sections_b(&self) -> u32 {
        self.spline().map_or(0, |s| s.header.num_sections_b)
    }
    /// VsAIWorld.cpp 0x00037BE0
    pub fn get_num_sections_c(&self) -> u32 {
        self.spline().map_or(0, |s| s.header.num_sections_c)
    }
    /// VsAIWorld.cpp 0x00037C10
    pub fn get_num_trail_blazer_sections(&self) -> u32 {
        self.spline().map_or(0, |s| s.header.num_trail_blazer_sections)
    }
    /// VsAIWorld.cpp 0x00037C40
    pub fn get_num_traffic_lights(&self) -> u32 {
        self.spline().map_or(0, |s| s.header.num_traffic_lights)
    }
    /// VsAIWorld.cpp 0x000377C0
    pub fn get_num_traffic_blockers(&self) -> u32 {
        self.spline().map_or(0, |s| s.header.num_traffic_blockers)
    }
    /// VsAIWorld.cpp 0x000377F0
    pub fn is_armco_enabled(&self) -> bool {
        self.armco_enabled
    }

    /// VsAIWorld.cpp 0x00034980
    pub fn get_race_section(&self, idx: u32) -> Option<&VsAIRaceSection> {
        let s = self.spline()?;
        debug_assert!(idx < self.get_num_race_sections(), "dIdx<GetNumRaceSections()");
        s.race_sections.get(idx as usize)
    }
    pub fn race_sections(&self) -> &[VsAIRaceSection] {
        self.spline().map_or(&[], |s| &s.race_sections)
    }
    pub fn race_sections_mut(&mut self) -> &mut [VsAIRaceSection] {
        self.spline.as_mut().map_or(&mut [], |s| &mut s.race_sections)
    }
    /// VsAIWorld.cpp 0x00034A30
    pub fn get_traffic_section(&self, idx: u32) -> Option<&VsAIRoadSection> {
        self.spline()?.road_sections[RoadSectionSet::Traffic as usize].get(idx as usize)
    }
    /// VsAIWorld.cpp 0x00034C40
    pub fn get_trail_blazer_section(&self, idx: u32) -> Option<&VsAIRoadSection> {
        self.spline()?.road_sections[RoadSectionSet::TrailBlazer as usize].get(idx as usize)
    }
    /// VsAIWorld.cpp 0x00034F70 / 0x00034E80: the trailblazer section with a lane of id `id`; the trigger then
    /// follows the section's first lane line (`section+8`)
    pub fn find_trail_blazer_line_mut(&mut self, id: u32) -> Option<&mut VsAIRoadLine> {
        let n = self.get_num_trail_blazer_sections() as usize;
        let secs = &mut self.spline.as_mut()?.road_sections[RoadSectionSet::TrailBlazer as usize];
        let i = secs.iter().take(n).position(|s| s.lanes.iter().take(s.get_num_lanes() as usize).any(|l| l.line.get_id() == id))?;
        secs[i].lanes.first_mut().map(|l| &mut l.line)
    }
    pub fn road_sections(&self, set: RoadSectionSet) -> &[VsAIRoadSection] {
        self.spline().map_or(&[], |s| &s.road_sections[set as usize])
    }
    pub fn lane(&self, r: LaneRef) -> &VsAILane {
        &self.spline.as_ref().unwrap().road_sections[r.set as usize][r.section].lanes[r.lane]
    }
    pub fn lane_mut(&mut self, r: LaneRef) -> &mut VsAILane {
        &mut self.spline.as_mut().unwrap().road_sections[r.set as usize][r.section].lanes[r.lane]
    }
    /// VsAIWorld.cpp 0x00034AE0
    pub fn get_traffic_light(&self, idx: u32) -> Option<&VsAITrafficLightData> {
        self.spline()?.traffic_lights.get(idx as usize)
    }
    /// VsAIWorld.cpp 0x00034B90
    pub fn get_traffic_blocker(&self, idx: u32) -> Option<&VsAITrafficBlocker> {
        self.spline()?.traffic_blockers.get(idx as usize)
    }

    /// lane with header id `id`, set B first, then traffic sections.
    /// VsAIWorld.cpp 0x00034FB0
    pub fn find_lane(&self, id: u32) -> Option<LaneRef> {
        let s = self.spline()?;
        if s.road_sections[1].is_empty() && s.road_sections[0].is_empty() {
            return None;
        }
        for set in [RoadSectionSet::SetB, RoadSectionSet::Traffic] {
            for (si, sec) in s.road_sections[set as usize].iter().enumerate() {
                for (li, lane) in sec.lanes.iter().enumerate() {
                    if lane.line.get_id() == id {
                        return Some(LaneRef { set, section: si, lane: li });
                    }
                }
            }
        }
        None
    }

    /// section of `set` holding a lane with id `id`.
    /// VsAIWorld.cpp 0x00034E80 (trailblazer) / 0x00034DB0 (set B, TriggerTraffic)
    pub fn find_section_by_lane_id(&self, set: RoadSectionSet, id: u32) -> Option<usize> {
        let s = self.spline()?;
        s.road_sections[set as usize]
            .iter()
            .position(|sec| sec.lanes.iter().any(|l| l.line.get_id() == id))
    }

    /// VsAIWorld.cpp 0x00035100
    pub fn find_traffic_blocker(&self, id: u32) -> Option<usize> {
        self.spline()?.traffic_blockers.iter().position(|b| b.id == id)
    }

    /// flips the blocker's file flag when it still matches the runtime flag.
    /// VsAIWorld.cpp 0x00034CF0
    pub fn toggle_traffic_blocker(&mut self, id: u32) {
        let idx = self.find_traffic_blocker(id);
        debug_assert!(idx.is_some(), "pBlocker != NULL");
        let Some(idx) = idx else { return };
        let b = &mut self.spline.as_mut().unwrap().traffic_blockers[idx];
        if (b.flag != 0) == b.active {
            b.flag = (b.flag == 0) as u32;
        }
    }

    /// reads `<level_path>\AISpline.spl` and sets it up.
    /// VsAIWorld.cpp 0x00030390
    pub fn load_spline_data(&mut self, level_path: &Path) -> bool {
        log::info!("-- Loading AI Spline Data");
        debug_assert!(self.spline.is_none(), "m_pSplineData == NULL && \"Spline data already allocated\"");
        let ok = match std::fs::read(crate::resolve(level_path, "AISpline.spl")) {
            Ok(d) => {
                self.spline = VsAISplineData::parse(&d).ok();
                self.setup_spline_data();
                true
            }
            Err(_) => {
                log::warn!("** No AI Spline data found **");
                false
            }
        };
        log::info!("-- Loading AI Spline Data - Done!");
        ok
    }

    /// resolves blocker lanes and light blockers once everything is in memory.
    /// VsAIWorld.cpp 0x00030760
    pub fn setup_spline_data(&mut self) {
        let Some(s) = self.spline.as_ref() else { return };
        let mut lanes = Vec::new();
        for b in &s.traffic_blockers {
            let v: Vec<Option<LaneRef>> = b.lane_ids.iter().map(|&id| self.find_lane(id)).collect();
            for l in &v {
                if l.is_none() {
                    log::error!("** Error, couldn't find matching lane for traffic blocker");
                }
            }
            lanes.push(v);
        }
        let mut light_blockers = Vec::new();
        for l in &s.traffic_lights {
            let r = l.blocker_ids.map(|id| self.find_traffic_blocker(id));
            if r[0].is_none() || r[2].is_none() {
                log::warn!("** AI Traffic Lights - Missing blockers, lights will not function correctly!!! See Andy T **");
            }
            light_blockers.push(r);
        }
        let s = self.spline.as_mut().unwrap();
        for (b, v) in s.traffic_blockers.iter_mut().zip(lanes) {
            b.lanes = v;
            b.active = b.flag != 0;
        }
        for (l, r) in s.traffic_lights.iter_mut().zip(light_blockers) {
            l.blockers = r;
        }
    }

    /// lane numbering, lengths and blocker lists of one road section.
    /// VsAIWorld.cpp 0x00030F90
    fn init_road_section(&mut self, set: RoadSectionSet, section: usize, lane_counter: &mut u32) {
        let n = self.road_sections(set)[section].lanes.len();
        for li in 0..n {
            let r = LaneRef { set, section, lane: li };
            let lane = self.lane_mut(r);
            lane.speed = f32::from_bits(0x42A0_0000);
            lane.num_vehicles = 0;
            lane.unk_30 = 3;
            lane.index = *lane_counter;
            *lane_counter += 1;
            lane.line.calc_length();
            self.setup_lane_blockers(r);
        }
    }

    /// derived data for the race route and the road sections.
    /// `start_line` is the start/finish line (`dword_6F5C44`/`dword_6F5C54`) a circular route is snapped to.
    /// VsAIWorld.cpp 0x00031040
    pub fn process_spline_data(&mut self, start_line: Option<(VsVector4, VsVector4)>) {
        let n = self.get_num_race_sections();
        if n != 0 {
            for (i, s) in self.race_sections_mut().iter_mut().enumerate() {
                s.raceline.calc_length();
                s.left.calc_length();
                s.right.calc_length();
                s.index = i as u32;
            }
            if self.race_sections()[0].raceline.is_circular()
                && let Some((a, b)) = start_line
            {
                self.race_sections_mut()[0].raceline.set_start_point(&a, &b);
            }
            if n <= 1 {
                self.race_sections_mut()[0].tracked = true;
            } else {
                self.process_race_sections();
            }
        }
        let mut lane_counter = 0u32;
        for i in 0..self.get_num_traffic_sections() as usize {
            self.init_road_section(RoadSectionSet::Traffic, i, &mut lane_counter);
        }
        self.calc_lane_bounds();
        for i in 0..self.get_num_sections_b() as usize {
            self.init_road_section(RoadSectionSet::SetB, i, &mut lane_counter);
        }
        for i in 0..self.get_num_sections_c() as usize {
            self.init_road_section(RoadSectionSet::SetC, i, &mut lane_counter);
        }
        for i in 0..self.get_num_trail_blazer_sections() as usize {
            self.init_road_section(RoadSectionSet::TrailBlazer, i, &mut lane_counter);
        }
    }

    /// test probe: the (section, end, closest raw distance) that ProcessRaceSections finds no section for.
    #[cfg(test)]
    pub fn unmatched_section_ends(&mut self, start_line: Option<(VsVector4, VsVector4)>) -> Vec<(usize, u32, f32)> {
        let n = self.get_num_race_sections() as usize;
        let mut out = Vec::new();
        if n <= 1 {
            return out;
        }
        for s in self.race_sections_mut().iter_mut() {
            s.raceline.calc_length();
        }
        if self.race_sections()[0].raceline.is_circular()
            && let Some((a, b)) = start_line
        {
            self.race_sections_mut()[0].raceline.set_start_point(&a, &b);
        }
        for i in 1..n {
            for end in 0..2u32 {
                let rl = &self.race_sections()[i].raceline;
                let pos = if end != 0 { *rl.get_point(rl.get_num_points() - 1) } else { *rl.get_point(0) };
                let mut found = false;
                let mut raw = f32::MAX;
                for k in (0..n).filter(|&k| k != i) {
                    let rl = &self.race_sections()[k].raceline;
                    let seg = rl.find_closest_segment(&pos);
                    let (a, b) = rl.get_segment(seg);
                    let t = VsAITools::line_parameter(&a, &b, &pos);
                    raw = raw.min(VsAITools::distance_to_line(&a, &b, &pos));
                    found |= (-0.000001..=1.000001).contains(&t);
                }
                if !found {
                    out.push((i, end, raw));
                }
            }
        }
        out
    }

    /// builds the branch/join points between race sections and the tracking distances.
    /// VsAIWorld.cpp 0x00031390
    pub fn process_race_sections(&mut self) {
        log::info!("-- Processing RaceSections");
        debug_assert!(self.sub_route_points.is_empty(), "m_pSubRoutePoints == NULL");
        let n = self.get_num_race_sections() as usize;
        let num_points = 2 * n - 2;
        let mut points = vec![VsAISubRoutePoint::default(); num_points];
        let mut counter = 0usize;
        self.race_sections_mut()[0].branch_from = None;
        for i in 1..n {
            for end in 0..2 {
                let pos = {
                    let rl = &self.race_sections()[i].raceline;
                    if end != 0 { *rl.get_point(rl.get_num_points() - 1) } else { *rl.get_point(0) }
                };
                let mut closest = None;
                let mut closest_seg = 0u32;
                let mut best = f32::from_bits(0x7F7F_FFFF);
                for k in 0..n {
                    if k == i {
                        continue;
                    }
                    let rl = &self.race_sections()[k].raceline;
                    let seg = rl.find_closest_segment(&pos);
                    let (a, b) = rl.get_segment(seg);
                    let d = VsAITools::distance_to_line(&a, &b, &pos);
                    let t = VsAITools::line_parameter(&a, &b, &pos);
                    if best > d && t >= -0.000001 && t <= 1.000001 {
                        best = d;
                        closest = Some(k);
                        closest_seg = seg;
                    }
                }
                // London Circuit Race09: section 1 branches at section 0's first point, which the start snap moves
                // along the line, so the branch lies just before segment 0 (t < 0). The original asserts
                // "pClosestSection != NULL" and then reads through NULL; the port takes the nearest line instead.
                if closest.is_none() {
                    log::warn!("ai: race section {i} end {end} lies on no race section segment; using the nearest");
                    for k in (0..n).filter(|&k| k != i) {
                        let rl = &self.race_sections()[k].raceline;
                        let seg = rl.find_closest_segment(&pos);
                        let (a, b) = rl.get_segment(seg);
                        let d = VsAITools::distance_to_line(&a, &b, &pos);
                        if best > d {
                            best = d;
                            closest = Some(k);
                            closest_seg = seg;
                        }
                    }
                }
                let Some(c) = closest else { continue };
                let pt = &mut points[counter];
                if end != 0 {
                    pt.from = i;
                    pt.to = c;
                    pt.join = true;
                } else {
                    pt.from = c;
                    pt.to = i;
                    pt.join = false;
                }
                if end == 0 {
                    self.race_sections_mut()[i].branch_from = Some(c);
                }
                points[counter].dist = self.race_sections()[c].raceline.get_distance_along_line(&pos, closest_seg);
                points[counter].pos = pos;
                counter += 1;
            }
        }
        for si in 0..n {
            let mut list: Vec<usize> = (0..num_points).filter(|&p| points[p].from == si).collect();
            // bubble sort by distance, as the original
            let mut swapped = 1;
            while swapped != 0 {
                swapped = 0;
                for j in 0..list.len().saturating_sub(1) {
                    if points[list[j]].dist > points[list[j + 1]].dist {
                        list.swap(j, j + 1);
                        swapped += 1;
                    }
                }
            }
            let circular = self.race_sections()[si].raceline.is_circular();
            if !circular && !list.is_empty() {
                // the join goes last on an open route
                let last = list[list.len() - 1];
                for j in 0..list.len() - 1 {
                    if points[list[j]].join {
                        let jp = list[j];
                        list[j] = last;
                        let l = list.len();
                        list[l - 1] = jp;
                        break;
                    }
                }
            }
            for j in 0..list.len() {
                points[list[j]].next = if j == list.len() - 1 {
                    if circular { Some(list[0]) } else { None }
                } else {
                    Some(list[j + 1])
                };
            }
            self.race_sections_mut()[si].sub_route_points = list;
        }
        self.sub_route_points = points;
        self.untracked_sections = n as u32 - 1;
        self.prev_untracked_sections = n as u32;
        log::info!("Calculating tracking distances for RaceSections...");
        self.race_sections_mut()[0].tracked = true;
        while self.untracked_sections != self.prev_untracked_sections {
            self.prev_untracked_sections = self.untracked_sections;
            self.calc_tracking_distances(0, 0);
        }
        if self.untracked_sections != 0 {
            log::warn!(
                "*** WARNING - Found {} RaceSections that won't have race position tracking - Position tracking will be disabled for vehicles on this route ***",
                self.untracked_sections
            );
        }
        log::info!("-- Processing RaceSections - Done!");
    }

    /// maps a section's start/end onto main-route distance, recursing along its sub-route points.
    /// VsAIWorld.cpp 0x00031EB0
    fn calc_tracking_distances(&mut self, si: usize, depth: u32) {
        // the original has no guard; a branch-of-a-branch loop would recurse forever
        if depth > 64 {
            log::error!("calc_tracking_distances: route loop at section {si}");
            return;
        }
        let first = 0usize;
        let (has_parent, tracked) = {
            let s = &self.race_sections()[si];
            (s.branch_from.is_some(), s.tracked)
        };
        if has_parent && !tracked {
            for pi in 0..self.sub_route_points.len() {
                let p = self.sub_route_points[pi].clone();
                if p.join || p.to != si {
                    if p.from == si {
                        if p.to == first {
                            self.race_sections_mut()[si].track_end = p.dist;
                        } else if self.race_sections()[p.to].tracked {
                            let t = &self.race_sections()[p.to];
                            let span = t.track_end - t.track_start;
                            let frac = p.dist / t.raceline.get_length();
                            let v = span * frac + t.track_start;
                            self.race_sections_mut()[si].track_end = v;
                        }
                    }
                } else if p.from == first {
                    self.race_sections_mut()[si].track_start = p.dist;
                } else if self.race_sections()[p.from].tracked {
                    let f = &self.race_sections()[p.from];
                    let span = f.track_end - f.track_start;
                    let frac = p.dist / f.raceline.get_length();
                    let v = span * frac + f.track_start;
                    self.race_sections_mut()[si].track_start = v;
                }
            }
        }
        let s = &self.race_sections()[si];
        if !s.tracked && s.track_start >= 0.0 && s.track_end >= 0.0 {
            self.race_sections_mut()[si].tracked = true;
            self.untracked_sections -= 1;
            log::info!("Tracking calculated for Race Section {}", si + 1);
        }
        let list = self.race_sections()[si].sub_route_points.clone();
        for p in list {
            let to = self.sub_route_points[p].to;
            if to != first {
                self.calc_tracking_distances(to, depth + 1);
            }
        }
    }

    /// xz bounding boxes of the traffic lanes.
    /// VsAIWorld.cpp 0x00032160
    fn calc_lane_bounds(&mut self) {
        let Some(s) = self.spline.as_mut() else { return };
        for sec in s.road_sections[RoadSectionSet::Traffic as usize].iter_mut() {
            for lane in sec.lanes.iter_mut() {
                let p0 = *lane.line.get_point(0);
                let (mut min_x, mut min_z, mut max_x, mut max_z) = (p0.x, p0.z, p0.x, p0.z);
                for p in &lane.line.points {
                    if min_x > p.x {
                        min_x = p.x;
                    }
                    if p.x > max_x {
                        max_x = p.x;
                    }
                    if min_z > p.z {
                        min_z = p.z;
                    }
                    if p.z > max_z {
                        max_z = p.z;
                    }
                }
                lane.bbox_min.x = min_x;
                lane.bbox_min.y = 0.0;
                lane.bbox_min.z = min_z;
                lane.bbox_max.x = max_x;
                lane.bbox_max.y = 0.0;
                lane.bbox_max.z = max_z;
            }
        }
    }

    /// the blockers on a lane with their distance along it, nearest first.
    /// VsAIWorld.cpp 0x00032420
    fn setup_lane_blockers(&mut self, r: LaneRef) {
        let Some(s) = self.spline.as_ref() else { return };
        let mut list = Vec::new();
        for (bi, b) in s.traffic_blockers.iter().enumerate() {
            for l in &b.lanes {
                if *l == Some(r) {
                    let line = &self.lane(r).line;
                    let seg = line.find_closest_segment(&b.pos);
                    list.push((bi, line.get_distance_along_line(&b.pos, seg)));
                }
            }
        }
        let mut swapped = 1;
        while swapped > 0 {
            swapped = 0;
            for j in 0..list.len().saturating_sub(1) {
                if list[j].1 - list[j + 1].1 > 0.0 {
                    list.swap(j, j + 1);
                    swapped += 1;
                }
            }
        }
        self.lane_mut(r).blockers = list;
    }

    /// arrow and no-entry barriers of the level.
    /// VsAIWorld.cpp 0x00033CA0
    pub fn load_barrier_data(&mut self, level_path: &Path) -> bool {
        log::info!("-- Loading Barrier Data");
        let mut ok = true;
        for (i, name) in ["barier01.br", "noentr01.br"].iter().enumerate() {
            match std::fs::read(crate::resolve(level_path, name)) {
                Ok(d) => self.barriers[i] = VsAIBarriers::parse(&d),
                Err(_) => {
                    log::warn!(
                        "{}",
                        if i == 0 { "** No Arrow Barrier data found **" } else { "** No No-Entry Barrier data found **" }
                    );
                    ok = false;
                }
            }
        }
        log::info!("-- Loading Barrier Data - Done!");
        ok
    }

    /// VsAIWorld.cpp 0x00034130
    pub fn get_barrier(&self, kind: u32, idx: i32) -> Option<&VsAIRoadLine> {
        if kind <= 1
            && let Some(b) = &self.barriers[kind as usize]
            && idx >= 0
        {
            return b.lines.get(idx as usize);
        }
        None
    }
    /// VsAIWorld.cpp 0x000341A0
    pub fn get_num_barriers(&self, kind: u32) -> u32 {
        if kind <= 1 && let Some(b) = &self.barriers[kind as usize] {
            return b.lines.len() as u32;
        }
        0
    }

    /// VsAIWorld.cpp 0x00033790
    pub fn find_response_curve(&self, name: &str) -> Option<usize> {
        let crc = string_crc(name);
        self.response_curves.iter().position(|rc| rc.crc == crc)
    }

    /// VsAIWorld.cpp 0x000338C0
    pub fn find_handling(&self, name: &str) -> Option<usize> {
        let crc = string_crc(name);
        self.handlings.iter().position(|h| h.crc == crc)
    }

    fn get_or_load_curve(&mut self, name: &str, file: &str) -> usize {
        if let Some(i) = self.find_response_curve(name) {
            return i;
        }
        let mut rc = VsAIResponseCurve::new();
        rc.load(&self.ai_dir, file);
        self.response_curves.push(rc);
        self.response_curves.len() - 1
    }

    /// loads (once) the handling of car `name`, falling back to 350Z; returns its index.
    /// VsAIWorld.cpp 0x00032F10
    pub fn load_handling(&mut self, name: &str) -> Option<usize> {
        if let Some(i) = self.find_handling(name) {
            log::info!("Handling already loaded for - {name}");
            return Some(i);
        }
        let dir = crate::resolve(&self.ai_dir, "Handling");
        let mut crc = 0;
        let mut f = match VsTextFile::load(&crate::resolve(&dir, &format!("{name}.txt"))) {
            Ok(f) => {
                log::info!("Loaded handling for - {name}");
                crc = string_crc(name);
                f
            }
            Err(_) => {
                log::error!("Cannot find AI handling file for {name}, attempting to load 350Z handling");
                match VsTextFile::load(&crate::resolve(&dir, "350Z.txt")) {
                    Ok(f) => {
                        log::info!("Loaded handling for - 350Z");
                        crc = string_crc("350Z");
                        f
                    }
                    Err(_) => {
                        log::error!("\n ** AI_ASSERT - Cannot find 350Z handling file. **");
                        VsTextFile::default()
                    }
                }
            }
        };
        let _vehicle = f.read_string();
        let mut h = VsAIHandling { crc, ..Default::default() };
        let speed = f.read_string().unwrap_or_default();
        h.speed_ratio_rc = self.get_or_load_curve(&speed, &speed);
        let steer = f.read_string().unwrap_or_default();
        h.steering_rc = self.get_or_load_curve(&steer, &steer);
        let throttle = f.read_string().unwrap_or_default();
        h.throttle_rc = self.get_or_load_curve(&throttle, &throttle);
        let brake = f.read_string().unwrap_or_default();
        h.brake_rc = self.get_or_load_curve(&brake, &brake);
        // original bug: looks up "RC_sp3" but loads the brake curve file under that slot
        h.sp3_rc = self.get_or_load_curve("RC_sp3", &brake);
        let mut num = || f.read_single_number().unwrap_or(0.0);
        h.max_speed = num();
        h.look_ahead_min = num();
        h.look_ahead_max = num();
        h.steering_factor = num();
        h.throttle_factor = num();
        h.brake_factor = num();
        h.steer_damping_factor = num();
        self.handlings.push(h);
        Some(self.handlings.len() - 1)
    }

    /// debug-draw spots of a handling's curves (`+0x34`/`+0x38` of each curve).
    pub(crate) fn place_handling_curves(&mut self, h: usize) {
        let hd = self.handlings[h].clone();
        for (rc, pos) in [
            (hd.speed_ratio_rc, (50, 350)),
            (hd.steering_rc, (50, 450)),
            (hd.throttle_rc, (250, 350)),
            (hd.brake_rc, (250, 450)),
            (hd.sp3_rc, (350, 300)),
        ] {
            self.response_curves[rc].draw_pos = pos;
        }
    }

    /// every ai and player race car drives with the 350Z handling.
    /// VsAIWorld.cpp 0x000327A0
    pub fn load_race_vehicle_handling(&mut self, rm: &mut VsRaceManager) -> bool {
        log::info!("-- Loading Race Vehicle Handling");
        for v in rm.ai.iter_mut().chain(rm.players.iter_mut()) {
            v.base.handling = self.load_handling("350Z");
            if let Some(h) = v.base.handling {
                self.place_handling_curves(h);
            }
        }
        true
    }

    /// traffic cars drive with the TRAFFIC handling, when traffic is on.
    /// VsAIWorld.cpp 0x00032D70
    pub fn load_traffic_vehicle_handling(&mut self, tm: &mut super::vs_traffic_manager::VsTrafficManager) -> bool {
        if self.traffic_enabled {
            log::info!("-- Loading Traffic Vehicle Handling");
            for v in tm.vehicles.iter_mut() {
                v.base.handling = self.load_handling("TRAFFIC");
                if let Some(h) = v.base.handling {
                    self.place_handling_curves(h);
                }
            }
        }
        true
    }

    /// ai cars cycle through the three rev curves.
    /// VsAIWorld.cpp 0x00032A90
    pub fn setup_startline_revs(&self, rm: &mut VsRaceManager) -> bool {
        for (i, v) in rm.ai.iter_mut().enumerate() {
            v.startline_revs.load(&self.ai_dir, STARTLINE_REVS[i % STARTLINE_REVS.len()]);
            if let Some(rc) = &mut v.startline_revs.rc {
                rc.wrap = true;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spline_files() -> Vec<PathBuf> {
        let Some(dir) = crate::game_dir() else { return vec![] };
        let mut out = Vec::new();
        let mut stack = vec![dir.join("Tracks")];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("AISpline.spl")) {
                    out.push(p);
                }
            }
        }
        out.sort();
        out
    }

    #[test]
    fn parse_all_ai_splines() {
        let files = spline_files();
        if files.is_empty() {
            return;
        }
        assert_eq!(files.len(), 59);
        for f in &files {
            let d = std::fs::read(f).unwrap();
            let s = VsAISplineData::parse(&d).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
            assert_eq!(s.size, d.len(), "{}", f.display());
            assert!(s.header.num_race_sections >= 1, "{}", f.display());
        }
    }

    #[test]
    fn process_all_ai_worlds() {
        for f in spline_files() {
            let mut w = VsAIWorld::new(crate::game_dir().unwrap().join("AI"));
            assert!(w.load_spline_data(f.parent().unwrap()));
            w.process_spline_data(None);
            let n = w.get_num_race_sections() as usize;
            if n > 1 {
                assert_eq!(w.sub_route_points.len(), 2 * n - 2);
            }
            for s in w.race_sections() {
                assert!(s.raceline.get_length() > 0.0, "{}", f.display());
            }
            if w.untracked_sections != 0 {
                eprintln!("{}: {} untracked race sections", f.display(), w.untracked_sections);
            }
            for set in [RoadSectionSet::Traffic, RoadSectionSet::SetB, RoadSectionSet::SetC, RoadSectionSet::TrailBlazer] {
                for sec in w.road_sections(set) {
                    assert_eq!(sec.lanes.len() as u32, sec.get_num_lanes());
                }
            }
        }
    }

    #[test]
    fn parse_all_barriers() {
        let Some(dir) = crate::game_dir() else { return };
        let mut n = 0;
        let mut stack = vec![dir.join("Tracks")];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("br")) {
                    let d = std::fs::read(&p).unwrap();
                    assert_eq!(&d[..4], b"SPL0");
                    let b = VsAIBarriers::parse(&d).unwrap_or_else(|| panic!("{}", p.display()));
                    assert_eq!(b.size, d.len(), "{}", p.display());
                    n += 1;
                }
            }
        }
        assert!(n > 0);
    }

    #[test]
    fn load_all_handling() {
        let Some(dir) = crate::game_dir() else { return };
        let mut w = VsAIWorld::new(dir.join("AI"));
        for e in std::fs::read_dir(dir.join("AI").join("Handling")).unwrap() {
            let p = e.unwrap().path();
            let name = p.file_stem().unwrap().to_string_lossy().into_owned();
            let i = w.load_handling(&name).unwrap();
            let h = &w.handlings[i];
            assert_eq!(h.crc, string_crc(&name));
            assert!(h.max_speed > 0.0 && h.look_ahead_max >= h.look_ahead_min, "{name}");
        }
        let i = w.find_handling("350Z").unwrap();
        let h = w.handlings[i].clone();
        assert_eq!(h.max_speed, 60.0);
        assert_eq!(h.steer_damping_factor, 40.0);
        assert_eq!(w.response_curves[h.steering_rc].name, "RC_ISq");
        // the RC_sp3 slot really holds a copy of the brake curve
        assert_eq!(w.response_curves[h.sp3_rc].name, "RC_sp1");
    }
}

#[cfg(test)]
mod debug_tests {
    use super::*;
    #[test]
    #[ignore]
    fn dump_route_graph() {
        let dir = crate::game_dir().unwrap();
        let p = std::env::var("AI_TRACK").unwrap_or("Tracks/LACoast/Circuit/Race01".into());
        let mut w = VsAIWorld::new(dir.join("AI"));
        w.load_spline_data(&dir.join(p));
        w.process_spline_data(None);
        for s in w.race_sections() {
            eprintln!(
                "sec {} len {:.1} circ {} from {:?} track {:.1}..{:.1} {} pts {:?} p0 {:?}",
                s.index, s.raceline.length, s.raceline.is_circular(), s.branch_from, s.track_start, s.track_end, s.tracked,
                s.sub_route_points, s.raceline.points[0]
            );
        }
        for (i, p) in w.sub_route_points.iter().enumerate() {
            eprintln!("pt {i}: {} -> {} join {} dist {:.1} next {:?}", p.from, p.to, p.join, p.dist, p.next);
        }
    }
}
