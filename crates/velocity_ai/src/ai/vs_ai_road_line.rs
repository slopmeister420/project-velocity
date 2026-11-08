//! A polyline of `VsVector4` points whose `w` is the distance along the line
//! (src/Ai/VsAIRoadLine.cpp, VsAIRoadLine.h).
//!
//! In the data files a road line is a 16-byte header followed by `num_points` x 16-byte points.

use super::vs_ai_tools::VsAITools;
use super::vs_math::{VsVector4, vs_distance, vs_distance_squared};

/// on-disk header, `m_pHeader` (16 bytes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VsAIRoadLineHeader {
    pub kind: u32,       // +0x0 2 or 10 in the shipped data
    pub circular: u8,    // +0x4
    pub flags: [u8; 3],  // +0x5
    pub num_points: u32, // +0x8
    pub id: u32,         // +0xC lane id for traffic lanes
}

/// `VsAIRoadLine` (32 bytes): `+0x00 m_pHeader, +0x04 m_pPoints, +0x08 m_ftLength, +0x0C flag, +0x10 scratch point`.
#[derive(Debug, Clone, Default)]
pub struct VsAIRoadLine {
    pub header: VsAIRoadLineHeader,
    pub points: Vec<VsVector4>,
    pub length: f32,
    pub flag_0c: bool,
    /// interpolated result of `get_point_at_distance`.
    pub point: VsVector4,
}

fn rd32(d: &[u8], p: usize) -> u32 {
    u32::from_le_bytes(d[p..p + 4].try_into().unwrap())
}

fn rdf(d: &[u8], p: usize) -> f32 {
    f32::from_bits(rd32(d, p))
}

impl VsAIRoadLine {
    /// reads header + points at `*p`, advancing it (the in-place pointer setup of the loaders).
    pub fn read(d: &[u8], p: &mut usize) -> Option<Self> {
        let h = d.get(*p..*p + 16)?;
        let header = VsAIRoadLineHeader {
            kind: rd32(h, 0),
            circular: h[4],
            flags: [h[5], h[6], h[7]],
            num_points: rd32(h, 8),
            id: rd32(h, 12),
        };
        *p += 16;
        let n = header.num_points as usize;
        let body = d.get(*p..*p + 16 * n)?;
        let points = body
            .chunks_exact(16)
            .map(|c| VsVector4::new(rdf(c, 0), rdf(c, 4), rdf(c, 8), rdf(c, 12)))
            .collect();
        *p += 16 * n;
        Some(Self { header, points, ..Default::default() })
    }

    /// VsAIRaceVehicle.cpp 0x00024CD0
    pub fn get_num_points(&self) -> u32 {
        self.points.len() as u32
    }
    /// VsAIRaceVehicle.cpp 0x00024360
    pub fn is_circular(&self) -> bool {
        self.header.circular != 0
    }
    /// VsAIRaceVehicle.cpp 0x000246E0
    pub fn get_length(&self) -> f32 {
        self.length
    }
    /// VsAIWorld.cpp 0x00037DA0
    pub fn get_id(&self) -> u32 {
        self.header.id
    }
    /// VsAIRaceVehicle.cpp 0x00024D00
    pub fn get_point(&self, idx: u32) -> &VsVector4 {
        debug_assert!(idx < self.get_num_points(), "dIdx < GetNumPoints()");
        &self.points[idx as usize]
    }

    /// interpolated point `dist` along the line, searching from `start_idx`; writes the segment index.
    /// VsAIRoadLine.cpp 0x00025920
    pub fn get_point_at_distance(&mut self, dist: f32, start_idx: u32, out_idx: &mut u32) -> VsVector4 {
        if self.points.is_empty() {
            return self.point;
        }
        debug_assert!(dist >= 0.0, "ftDist >= 0.f");
        let n = self.get_num_points();
        let mut d = dist;
        let mut s = start_idx;
        if self.is_circular() {
            if d > self.length {
                d -= self.length;
                s = 0;
            }
        } else if d > self.length {
            *out_idx = n.wrapping_sub(2);
            return self.points[(n - 1) as usize];
        }
        if self.points[s as usize].w > d {
            s = 0;
        }
        let mut p0 = VsVector4::default();
        let mut p1 = VsVector4::default();
        let mut found = false;
        let mut i = s;
        while i < n {
            if d == self.points[i as usize].w {
                *out_idx = i;
                return self.points[i as usize];
            }
            if i >= n - 1 {
                debug_assert!(self.is_circular(), "IsCircular()");
                p0 = self.points[i as usize];
                p1 = self.points[0];
                *out_idx = i;
                found = true;
                break;
            }
            if self.points[(i + 1) as usize].w > d {
                p0 = self.points[i as usize];
                p1 = self.points[(i + 1) as usize];
                *out_idx = i;
                found = true;
                break;
            }
            i += 1;
        }
        debug_assert!(found, "bFound");
        let o = *out_idx as usize;
        let seg_d = d - self.points[o].w;
        let seg_len = if self.is_circular() && *out_idx == n - 1 {
            self.length - self.points[o].w
        } else {
            self.points[o + 1].w - self.points[o].w
        };
        let t = seg_d / seg_len;
        self.point = p0 + (p1 - p0) * t;
        self.point
    }

    /// distance along the line of `pos` projected onto segment `idx`.
    /// VsAIRoadLine.cpp 0x00025E00
    pub fn get_distance_along_line(&self, pos: &VsVector4, idx: u32) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        debug_assert!(idx < self.get_num_points(), "dStartIdx >= 0 && dStartIdx < GetNumPoints()");
        if idx != 0 || VsAITools::line_parameter(&self.points[0], &self.points[1], pos) > 0.0 {
            let a = self.points[idx as usize];
            let b = if idx == self.get_num_points() - 1 {
                if !self.is_circular() {
                    return self.points[idx as usize].w;
                }
                self.points[0]
            } else {
                self.points[(idx + 1) as usize]
            };
            let d = VsAITools::distance_along_line(&a, &b, pos);
            return self.points[idx as usize].w + d;
        }
        0.0
    }

    /// wraps a distance into `[0, length]` on circular lines.
    /// VsAIRoadLine.cpp 0x00025FD0
    pub fn wrap_distance(&self, d: f32) -> f32 {
        if d < 0.0 {
            if self.is_circular() {
                return self.length + d;
            }
            0.0
        } else if self.length < d {
            if self.is_circular() {
                return d - self.length;
            }
            self.length
        } else {
            d
        }
    }

    /// walks from `start` to the segment whose line parameter for `pos` is in [0,1].
    /// VsAIRoadLine.cpp 0x00026060
    pub fn find_segment(&self, pos: &VsVector4, start: u32) -> u32 {
        if self.points.is_empty() {
            return 0;
        }
        let n = self.get_num_points();
        let mut idx = start;
        let mut t = -1.0f32;
        let mut iters = 0u32;
        let mut passed_fwd = false;
        let mut passed_back = false;
        while (!passed_fwd || !passed_back) && (t < 0.0 || t > 1.0) {
            let b = if idx == n - 1 {
                if !self.is_circular() {
                    return idx - 1;
                }
                self.points[0]
            } else {
                self.points[(idx + 1) as usize]
            };
            let a = self.points[idx as usize];
            t = VsAITools::line_parameter(&a, &b, pos);
            if t <= 1.0 {
                if t < 0.0 {
                    passed_back = true;
                    if idx != 0 {
                        idx -= 1;
                    } else {
                        if !self.is_circular() {
                            return 0;
                        }
                        idx = n - 1;
                    }
                }
            } else {
                passed_fwd = true;
                if idx >= n - 1 {
                    if !self.is_circular() {
                        return n - 2;
                    }
                    idx = 0;
                } else {
                    idx += 1;
                }
            }
            iters += 1;
            debug_assert!(iters < 1000, "dNumIterations < 1000");
            if iters >= 1000 {
                break;
            }
        }
        idx
    }

    /// segment start of the point nearest `pos`.
    /// VsAIRoadLine.cpp 0x000262D0
    pub fn find_closest_segment(&self, pos: &VsVector4) -> u32 {
        let mut best = 1_000_000.0f32;
        let mut bi = 0u32;
        for (i, p) in self.points.iter().enumerate() {
            let d = vs_distance_squared(pos, p);
            if best > d {
                best = d;
                bi = i as u32;
            }
        }
        if bi == 0 {
            return 0;
        }
        if VsAITools::line_parameter(&self.points[(bi - 1) as usize], &self.points[bi as usize], pos) < 1.0 {
            return bi - 1;
        }
        bi
    }

    /// end points of segment `idx`.
    /// VsAIRoadLine.cpp 0x000263C0
    pub fn get_segment(&self, idx: u32) -> (VsVector4, VsVector4) {
        let n = self.get_num_points();
        debug_assert!(idx <= n - 1);
        if idx == n - 1 {
            if self.is_circular() {
                (self.points[idx as usize], self.points[0])
            } else {
                (self.points[(idx - 1) as usize], self.points[idx as usize])
            }
        } else {
            (self.points[idx as usize], self.points[(idx + 1) as usize])
        }
    }

    /// VsAIRoadLine.cpp 0x00026550
    pub fn calc_length(&mut self) {
        debug_assert!(!self.points.is_empty(), "m_pPoints && GetNumPoints() > 0");
        let n = self.points.len();
        if self.is_circular() {
            let w = self.points[n - 1].w;
            self.length = vs_distance(&self.points[n - 1], &self.points[0]) + w;
        } else {
            self.length = self.points[n - 1].w;
        }
    }

    /// recomputes the cumulative distance in every point's w.
    /// VsAIRoadLine.cpp 0x00026660
    pub fn calc_distances(&mut self) {
        debug_assert!(!self.points.is_empty(), "m_pPoints && GetNumPoints() > 0");
        for i in 0..self.points.len() {
            if i != 0 {
                let d = self.points[i] - self.points[i - 1];
                let w = self.points[i - 1].w;
                self.points[i].w = d.length() + w;
            } else {
                self.points[0].w = 0.0;
            }
        }
    }

    /// moves the first point onto the line `a`-`b` (the start line) and recomputes distances.
    /// VsAIRoadLine.cpp 0x000267E0
    pub fn set_start_point(&mut self, a: &VsVector4, b: &VsVector4) {
        let p = VsAITools::project_onto_line(a, b, &self.points[0]);
        self.points[0] = p;
        self.calc_distances();
        self.calc_length();
    }

    /// rotates the point list so `start` becomes index 0.
    /// VsAIRoadLine.cpp 0x000269A0
    pub fn rotate(&mut self, start: u32) {
        let n = self.points.len();
        let tmp: Vec<VsVector4> = (0..n)
            .map(|i| {
                let mut j = start as usize + i;
                if j >= n {
                    j -= n;
                }
                self.points[j]
            })
            .collect();
        self.points = tmp;
    }
}
