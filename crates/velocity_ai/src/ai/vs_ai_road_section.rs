//! Road-network element types (src/Ai/VsAIRoadSection.h, VsAIRaceSection in VsAIRaceVehicle.h).
//!
//! Pointers of the original become indices into the owning `VsAIWorld` arrays.

use super::vs_ai_road_line::VsAIRoadLine;
use super::vs_math::VsVector4;

/// which `VsAIWorld` road-section array a lane lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoadSectionSet {
    /// `+0x4780 m_pTrafficSections`, header count `+0x14`
    Traffic = 0,
    /// `+0x4784`, header count `+0x18`; searched before traffic lanes by `FindLane`
    SetB = 1,
    /// `+0x4788`, header count `+0x1C`
    SetC = 2,
    /// `+0x478C m_pTrailBlazerSections`, header count `+0x28`
    TrailBlazer = 3,
}

/// lane address inside the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LaneRef {
    pub set: RoadSectionSet,
    pub section: usize,
    pub lane: usize,
}

/// one lane (100 bytes): a road line plus traffic bookkeeping.
#[derive(Debug, Clone, Default)]
pub struct VsAILane {
    pub line: VsAIRoadLine,       // +0x00
    pub index: u32,               // +0x20 running lane number across all sections
    pub unk_24: u32,              // +0x24
    pub speed: f32,               // +0x28 80 at init
    pub num_vehicles: u32,        // +0x2C traffic cars on the lane
    pub unk_30: u32,              // +0x30 3 at init
    pub bbox_min: VsVector4,      // +0x34 xz min, y = 0 (traffic lanes only)
    pub bbox_max: VsVector4,      // +0x44 xz max, y = 0
    /// +0x54 race section the lane crosses (ambush timing); no ai code sets it, so it stays null
    pub race_section: Option<usize>,
    pub race_section_dist: f32,   // +0x58 distance along that section's raceline, -1 at init
    /// `+0x5C m_dNumBlockers`, `+0x60` (blocker index, distance along the lane), ascending distance
    pub blockers: Vec<(usize, f32)>,
}

impl VsAILane {
    /// VsAIRoadLine.cpp 0x00026C60
    pub fn new(line: VsAIRoadLine) -> Self {
        Self { line, race_section_dist: -1.0, ..Default::default() }
    }
}

/// a road section (12 bytes): `+0x04 m_pTrafficSectionHeader` (16 bytes, lane count first), `+0x08` lanes.
#[derive(Debug, Clone, Default)]
pub struct VsAIRoadSection {
    pub header: [u32; 4],
    pub lanes: Vec<VsAILane>,
}

impl VsAIRoadSection {
    /// VsAIRoadSection.h 0x00037AF0
    pub fn get_num_lanes(&self) -> u32 {
        self.header[0]
    }
}

/// branch/join between two race sections (36 bytes).
#[derive(Debug, Clone, Default)]
pub struct VsAISubRoutePoint {
    pub from: usize,          // +0x00 section whose raceline the point is on (branch) / the joining section (join)
    pub to: usize,            // +0x04
    pub next: Option<usize>,  // +0x08 next sub-route point of `from`, sorted by distance
    pub join: bool,           // +0x0C 1 = "Join", 0 = "Branch"
    pub dist: f32,            // +0x10 distance along the connecting raceline
    pub pos: VsVector4,       // +0x14
}

/// a race route section (40 bytes): raceline plus left/right road edges.
#[derive(Debug, Clone, Default)]
pub struct VsAIRaceSection {
    pub index: u32,                  // +0x00
    pub raceline: VsAIRoadLine,      // +0x04
    pub left: VsAIRoadLine,          // +0x08
    pub right: VsAIRoadLine,         // +0x0C
    pub branch_from: Option<usize>,  // +0x10 section this one branches off
    pub sub_route_points: Vec<usize>, // +0x14 count, +0x18 array
    pub track_start: f32,            // +0x1C distance on the main route where this section starts
    pub track_end: f32,              // +0x20
    pub tracked: bool,               // +0x24
    pub flag_25: bool,               // +0x25 enabled: set at construction, F3EventEnableRaceSection (0x00177B20)
}

impl VsAIRaceSection {
    /// VsAIRoadLine.cpp 0x00026E40
    pub fn new() -> Self {
        Self { flag_25: true, track_start: -1.0, track_end: -1.0, ..Default::default() }
    }
    /// VsAIRaceVehicle.cpp 0x000241E0
    pub fn get_raceline(&self) -> &VsAIRoadLine {
        &self.raceline
    }
    /// VsAIRaceVehicle.cpp 0x00023AE0
    pub fn get_sub_route_point(&self, idx: u32) -> Option<usize> {
        if self.sub_route_points.is_empty() {
            return None;
        }
        debug_assert!((idx as usize) < self.sub_route_points.len(), "dIdx >= 0 && dIdx < m_dNumSubRoutePoints");
        self.sub_route_points.get(idx as usize).copied()
    }
}

/// a traffic blocker (16 bytes): `+0x00` file header, `+0x04` lanes, `+0x08` physics object, `+0x0C` flag.
#[derive(Debug, Clone, Default)]
pub struct VsAITrafficBlocker {
    pub pos: VsVector4,          // header +0x00
    pub id: u32,                 // header +0x10
    pub flag: u32,               // header +0x14
    pub physics_object_id: u32,  // header +0x1C
    pub lane_ids: Vec<u32>,      // header +0x18 count, then ids
    pub lanes: Vec<Option<LaneRef>>, // resolved `ppLanes`
    pub physics_object: Option<u32>, // +0x08 resolved object handle
    pub active: bool,            // +0x0C header flag != 0
}

/// a traffic light (32 bytes): `+0x00` file record, `+0x04..+0x10` four blockers, `+0x14` current pair,
/// `+0x18` state, `+0x1C` state start time.
#[derive(Debug, Clone, Default)]
pub struct VsAITrafficLightData {
    pub id: u32,                       // record +0x00
    pub cycle_time: f32,               // record +0x04
    pub blocker_ids: [u32; 4],         // record +0x08
    pub record_tail: [u32; 2],         // record +0x18
    pub blockers: [Option<usize>; 4],  // resolved
    /// `+0x14` blocker pair currently timing: 0 = blockers 0/1, 1 = blockers 2/3
    pub current_pair: Option<usize>,
    pub state: u32,                    // +0x18 0 timing the cycle, 1 all-red clearance
    pub start_time: f32,               // +0x1C -1 = not started
}
