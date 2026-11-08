//! src\Compartment\CompartmentManager.cpp — sector streaming of `F3CompartmentManager` (the indexedUpdate
//! specialisation): a fixed number of slots, each holding one `Sector_<n>_high.cpt`, refilled one sector per
//! update from a priority built around the cameras (distance, frustum, heading, raceline look-ahead).
//! `Sector_<n>_low.cpt` is never loaded (the packs ship 64-128 byte stubs and the index marks them empty), so
//! residency of the high sectors is the whole LOD scheme.

use bevy::math::{Vec3, Vec4};

/// `MEM_POOL_SIZE` (CompartmentManager.cpp 0x00087800 assert, initialiser 0x00087010)
pub const MEM_POOL_SIZE: u32 = 20_480_000;
/// slot cap (`min(128, pool / slot size)`, 0x000875B0)
pub const MAX_SLOTS: usize = 128;

/// one 36-byte `index.idx` entry; entry 0 is common.cpt, then `{high, low}` per sector
#[derive(Clone, Copy, Debug, Default)]
pub struct IndexEntry {
    pub unk00: u32,         // +0x00
    pub size: u32,          // +0x04 file size
    pub centre: [f32; 3],   // +0x0C x, y, z (z stored negated)
    pub half: [f32; 3],     // +0x18 x, z, y half extents (in that order)
}

/// `index.idx` (`'_IDX'`, version 2, count = sectors + 1, entries from +0x0C)
#[derive(Clone, Debug)]
pub struct CompartmentIndex {
    pub count: u32,
    pub entries: Vec<IndexEntry>,
}

impl CompartmentIndex {
    /// CompartmentSys.cpp 0x002E6CC0 (LoadIndex: entries at file + 12)
    pub fn parse(b: &[u8]) -> Option<Self> {
        let rd = |o: usize| b.get(o..o + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap()));
        // the manager's minimum version is 2 (every live index); Tokyo's version-1 index is refused
        if rd(0)? != 0x5844_495F || rd(4)? < 2 {
            return None;
        }
        let count = rd(8)?;
        let mut entries = Vec::new();
        let mut o = 12;
        while o + 36 <= b.len() {
            let f = |k: usize| f32::from_bits(rd(o + k).unwrap());
            entries.push(IndexEntry { unk00: rd(o)?, size: rd(o + 4)?, centre: [f(12), f(16), f(20)], half: [f(24), f(28), f(32)] });
            o += 36;
        }
        Some(CompartmentIndex { count, entries })
    }

    /// sectors the manager walks (`m_compartmentCount - 1`)
    pub fn sector_count(&self) -> usize {
        self.count.saturating_sub(1) as usize
    }

    /// the high entry of sector `n` (`index + 36 * (2n + 1)`)
    pub fn sector(&self, n: usize) -> IndexEntry {
        self.entries.get(2 * n + 1).copied().unwrap_or_default()
    }

    /// box corners as 0x000897C0 builds them: x/z from the centre ± half, y = centre ± half y
    pub fn sector_box(&self, n: usize) -> (Vec3, Vec3) {
        let e = self.sector(n);
        let (cx, cy, cz) = (e.centre[0], e.centre[1], e.centre[2]);
        let (hx, hz, hy) = (e.half[0], e.half[1], e.half[2]);
        let min = Vec3::new(cx - hx, cy - hy, (0.0 - cz) - hz);
        let max = Vec3::new(hx + cx, cy + hy, (0.0 - cz) + hz);
        (min, max)
    }
}

/// what 0x000897C0 reads per camera (the camera's logic frame, plus its player's race driver)
#[derive(Clone, Debug, Default)]
pub struct StreamView {
    /// camera position (`VsBaseCamera` +0xD4)
    pub position: Vec3,
    /// camera direction (`VsBaseCamera` +0x118)
    pub forward: Vec3,
    /// camera planes (`VsBaseCamera` +0x128, 6 x {normal, d}, inside = n.p + d >= 0); none = never culled
    pub planes: Option<[Vec4; 6]>,
    /// the player's driver on a race line: its lateral offset from the raceline (`VsAIRaceVehicle` +0x2AC, read by
    /// `VsAIRaceVehicle::24A60`; the original weights by it although the weight reads like a speed) and the raceline
    /// points 40, 80, ... metres ahead (up to 9, while < 450 m; `VsAIRaceVehicle::214B0`)
    pub raceline: Option<(f32, Vec<Vec3>)>,
}

/// 0x0005E0A0: true when the box lies entirely outside one of the planes (p-vertex test)
pub fn box_outside(planes: &[Vec4; 6], min: Vec3, max: Vec3) -> bool {
    planes.iter().any(|p| {
        let v = Vec3::new(
            if p.x >= 0.0 { max.x } else { min.x },
            if p.y >= 0.0 { max.y } else { min.y },
            if p.z >= 0.0 { max.z } else { min.z },
        );
        p.truncate().dot(v) + p.w < 0.0
    })
}

/// 0x00089640: squared xz distance from a point to a box (y ignored)
pub fn dist_sq_xz(p: Vec3, min: Vec3, max: Vec3) -> f32 {
    let mut d = Vec3::ZERO;
    if p.x > max.x {
        d.x = p.x - max.x;
    } else if min.x > p.x {
        d.x = p.x - min.x;
    }
    if p.z > max.z {
        d.z = p.z - max.z;
    } else if min.z > p.z {
        d.z = p.z - min.z;
    }
    d.length_squared()
}

/// sector priorities (lower = wanted more) and the wanted set, 0x000897C0. `review` = "Enable Cameras For
/// Review" (`dword_749110`, skips the frustum/heading terms). Each priority carries its sector number in the
/// low byte of the float, as the original stores it before sorting.
pub fn sector_priorities(index: &CompartmentIndex, views: &[StreamView], review: bool) -> Option<[f32; MAX_SLOTS]> {
    let n = index.sector_count().min(MAX_SLOTS);
    let mut pri = [0f32; MAX_SLOTS];
    if views.is_empty() {
        return None;
    }
    for (cam, v) in views.iter().enumerate() {
        let mut ahead = vec![v.position];
        let mut weight = 1.0f32;
        if let Some((lateral, pts)) = &v.raceline {
            let mut s = lateral.abs();
            if s > 16.0 {
                s -= 16.0;
                weight = ((44.0 - s) / 44.0).clamp(0.0, 1.0);
            }
            // the reset of the point count when the first point is > sqrt(1.3) * 40 m away is a no-op
            ahead.extend(pts.iter().take(9));
        }
        for k in 0..n {
            let (min, max) = index.sector_box(k);
            let mut d = dist_sq_xz(v.position, min, max);
            if !review {
                if v.planes.as_ref().is_some_and(|p| box_outside(p, min, max)) {
                    d *= 20.0;
                } else {
                    let e = index.sector(k);
                    let to = Vec3::new(e.centre[0] - v.position.x, e.centre[1] - v.position.y, (-e.centre[2]) - v.position.z);
                    let to = if to.length() <= 1e-6 { Vec3::ZERO } else { to.normalize() };
                    let dot = to.dot(v.forward);
                    if dot > 0.01 {
                        d *= (1.0 - dot) * 0.3 + 0.7;
                    }
                }
            }
            for (j, p) in ahead.iter().enumerate().skip(1) {
                let f = dist_sq_xz(*p, min, max) / 900.0;
                if f < 1.0 {
                    let w = ((j + 1) as f32 * 0.15).min(1.0);
                    let g = (1.0 - w) * f + w;
                    d *= (1.0 - g) * (1.0 - weight) + g;
                }
            }
            if cam == 0 || pri[k] > d {
                pri[k] = d;
            }
            pri[k] = f32::from_bits((pri[k].to_bits() & 0xFFFF_FF00) | k as u32);
        }
    }
    Some(pri)
}

/// sectors sorted by priority (qsort 0x003E2B80 with CompartmentManager 0x000895D0), the first `slots` wanted
pub fn wanted(pri: &[f32; MAX_SLOTS], sectors: usize, slots: usize) -> u128 {
    let mut order: Vec<f32> = pri[..sectors.min(MAX_SLOTS)].to_vec();
    // insertion sort, as the CRT's
    for i in 1..order.len() {
        let mut j = i;
        while j > 0 && order[j - 1] > order[j] {
            order.swap(j - 1, j);
            j -= 1;
        }
    }
    order.iter().take(slots).fold(0u128, |w, f| w | 1u128 << (f.to_bits() & 0xFF))
}

/// what one update decided
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamAction {
    Unload(usize),
    Load(usize),
}

/// the slot pool (manager +0x54 slots, +0x1054 resident bitfield, +0x1078 slot count)
#[derive(Clone, Debug, Default)]
pub struct CompartmentStreamer {
    /// sector held by each slot (-1 free)
    pub slots: Vec<i32>,
    pub resident: u128,
    pub sectors: usize,
}

impl CompartmentStreamer {
    /// 0x000875B0: slot size = largest sector + 64, slots = min(128, free pool / size, index count);
    /// `resident_bytes` = common.cpt + Instance.cpt (+ uInstance.cpt), taken from the pool first
    pub fn new(index: &CompartmentIndex, resident_bytes: u32) -> Self {
        let pool = MEM_POOL_SIZE.saturating_sub(resident_bytes);
        let mut slot = 1u32;
        for n in 0..index.sector_count() {
            slot = slot.max(index.sector(n).size + 64);
        }
        let count = ((pool / slot) as usize).min(MAX_SLOTS).min(index.count as usize);
        CompartmentStreamer { slots: vec![-1; count], resident: 0, sectors: index.sector_count().min(MAX_SLOTS) }
    }

    /// 0x0008A270 (indexedUpdate): with every slot busy, unload the resident sector that is not wanted and has
    /// the largest priority; then load the wanted sector that is not resident with the smallest priority into a
    /// free slot. Loads complete at once here (the original streams them over a few frames).
    pub fn update(&mut self, pri: Option<&[f32; MAX_SLOTS]>) -> Vec<StreamAction> {
        let mut out = Vec::new();
        let Some(pri) = pri else { return out };
        let want = wanted(pri, self.sectors, self.slots.len());
        if want == 0 || want == self.resident {
            return out;
        }
        if self.slots.iter().all(|s| *s != -1) {
            let mut best: Option<usize> = None;
            let mut bv = f32::from_bits(0xFF7F_FFFF);
            for k in 0..MAX_SLOTS {
                if self.resident >> k & 1 != 0 && want >> k & 1 == 0 && pri[k] > bv {
                    best = Some(k);
                    bv = pri[k];
                }
            }
            let Some(k) = best else {
                log::debug!("Something not right in Compartment System");
                return out;
            };
            for s in self.slots.iter_mut() {
                if *s == k as i32 {
                    *s = -1;
                    self.resident &= !(1u128 << k);
                    out.push(StreamAction::Unload(k));
                }
            }
        }
        let mut best: Option<usize> = None;
        let mut bv = f32::from_bits(0x7F7F_FFFF);
        for k in 0..MAX_SLOTS {
            if want >> k & 1 != 0 && self.resident >> k & 1 == 0 && bv > pri[k] {
                best = Some(k);
                bv = pri[k];
            }
        }
        let Some(k) = best else {
            log::debug!("Something not right in Compartment System - unable to find a comparment to Load");
            return out;
        };
        if let Some(s) = self.slots.iter_mut().find(|s| **s == -1) {
            *s = k as i32;
            self.resident |= 1u128 << k;
            out.push(StreamAction::Load(k));
        }
        out
    }

    /// run updates until the resident set stops changing (the loading screen's job; bounded by the slot count)
    pub fn fill(&mut self, pri: Option<&[f32; MAX_SLOTS]>) -> u128 {
        for _ in 0..2 * MAX_SLOTS {
            if self.update(pri).is_empty() {
                break;
            }
        }
        self.resident
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compartment_sys::Compartment;
    use crate::renderer::renderer_types::Model;

    /// every track index: sector boxes (index z negated) contain their high model's bounding sphere centre,
    /// and the streamer settles on a set of the slot count.
    #[test]
    fn index_matches_sectors() {
        let mut tracks = 0;
        for p in crate::test_util::all_vcds() {
            if !p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("CPTS.VCD")) {
                continue;
            }
            let Ok(v) = visos::file::Vcd::open_host(&p, "cpts") else { continue };
            let Some(idx) = v.get("index.idx").and_then(CompartmentIndex::parse) else { continue };
            assert_eq!(idx.entries.len(), 2 * idx.sector_count() + 1, "{p:?}");
            let mut checked = 0;
            let mut misses = 0;
            for n in 0..idx.sector_count() {
                let Some(d) = v.get(&format!("Sector_{n}_high.cpt")) else { continue };
                let Ok(c) = Compartment::parse(d) else { continue };
                let (min, max) = idx.sector_box(n);
                for r in c.models() {
                    let Ok(m) = Model::parse(r.data) else { continue };
                    let s = Vec3::new(m.sphere[0], m.sphere[1], m.sphere[2]);
                    let e = Vec3::splat(1.0);
                    if !(s.cmpge(min - e).all() && s.cmple(max + e).all()) {
                        misses += 1;
                    }
                    checked += 1;
                }
            }
            assert!(checked > 0, "{p:?}");
            // London2 ships an index from another export (8 of 35 sectors outside their boxes)
            let stale = p.to_string_lossy().contains("London2");
            assert!(misses == 0 || stale, "{p:?}: {misses} sector models outside their index boxes");
            let resident: u32 = v.entries().filter(|e| {
                let n = e.name.to_ascii_lowercase();
                n == "common.cpt" || n == "instance.cpt" || n == "uinstance.cpt"
            }).map(|e| e.size).sum();
            let mut s = CompartmentStreamer::new(&idx, resident);
            assert!(!s.slots.is_empty());
            let (min, max) = idx.sector_box(0);
            let view = StreamView { position: (min + max) * 0.5, forward: Vec3::NEG_Z, ..Default::default() };
            let pri = sector_priorities(&idx, &[view], false);
            let set = s.fill(pri.as_ref());
            assert_eq!(set.count_ones() as usize, s.slots.len().min(idx.sector_count()), "{p:?}");
            assert!(set & 1 != 0, "{p:?}: own sector not resident");
            tracks += 1;
        }
        assert!(tracks > 0);
    }
}
