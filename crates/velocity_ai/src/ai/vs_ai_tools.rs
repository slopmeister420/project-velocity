//! Geometry helpers shared by the ai (src/Ai/VsAITools.cpp).
//!
//! The debug-draw helpers of this file (0x00028770 player-control overlay, 0x00028D00 response-curve
//! plot) only emit 2D lines through the font manager and are not ported.

use super::vs_math::{VsVector4, vs_acos, vs_distance, vs_dot3, vs_vec_ne};

/// collision queries the tools make against the physics world (`PhysicsManager` in the original).
pub trait AiCollisionQuery {
    /// fraction along `start`->`end` of the first hit.
    /// PhysicsManager 0x0019D350 (`MaterialProperties__19D350` in the idb)
    fn line_check(&self, start: &VsVector4, end: &VsVector4) -> Option<f32>;
    /// fraction and surface normal of the first hit against objects in `mask`.
    /// PhysicsManager 0x0019D3E0
    fn line_check_normal(&self, start: &VsVector4, end: &VsVector4, mask: u32) -> Option<(f32, VsVector4)>;
}

pub struct VsAITools;

impl VsAITools {
    /// perpendicular distance from `p` to the infinite line `a`-`b`.
    /// VsAITools.cpp 0x00027B20
    pub fn distance_to_line(a: &VsVector4, b: &VsVector4, p: &VsVector4) -> f32 {
        let proj = Self::project_onto_line(a, b, p);
        vs_distance(p, &proj)
    }

    /// distance from `a` to the projection of `p` on the line `a`-`b` (unsigned).
    /// VsAITools.cpp 0x00027C90
    pub fn distance_along_line(a: &VsVector4, b: &VsVector4, p: &VsVector4) -> f32 {
        let proj = Self::project_onto_line(a, b, p);
        vs_distance(a, &proj)
    }

    /// projection of `p` on the infinite line `a`-`b`; w is `a`'s.
    /// VsAITools.cpp 0x00027E00
    pub fn project_onto_line(a: &VsVector4, b: &VsVector4, p: &VsVector4) -> VsVector4 {
        debug_assert!(vs_vec_ne(a, b), "VecA != VecB");
        let mut ab = *b;
        ab -= *a;
        let mut ap = *p;
        ap -= *a;
        let num = vs_dot3(&ap, &ab);
        let den = vs_dot3(&ab, &ab);
        let t = num / den;
        let mut out = *a;
        ab *= t;
        out += ab;
        out
    }

    /// t of `p` projected on `a`-`b` (0 at a, 1 at b).
    /// VsAITools.cpp 0x00027F70
    pub fn line_parameter(a: &VsVector4, b: &VsVector4, p: &VsVector4) -> f32 {
        debug_assert!(vs_vec_ne(a, b), "VecA != VecB");
        let mut ab = *b;
        ab -= *a;
        let mut ap = *p;
        ap -= *a;
        let num = vs_dot3(&ap, &ab);
        let den = vs_dot3(&ab, &ab);
        num / den
    }

    /// xz-plane side test of `c` against `a`->`b`.
    /// VsAITools.cpp 0x00028090
    pub fn is_left_of_line(a: &VsVector4, b: &VsVector4, c: &VsVector4) -> bool {
        let dz = a.z - c.z;
        let p = (b.x - a.x) * dz;
        let dx = a.x - c.x;
        let v = p - (b.z - a.z) * dx;
        v <= 0.0
    }

    /// xz intersection of segments `a`-`b` and `c`-`d`; the hit point interpolates `a`-`b`.
    /// VsAITools.cpp 0x00028150
    pub fn intersect_lines_2d(a: &VsVector4, b: &VsVector4, c: &VsVector4, d: &VsVector4) -> Option<VsVector4> {
        let r = (d.x - c.x) * (a.z - c.z) - (d.z - c.z) * (a.x - c.x);
        let den = (d.z - c.z) * (b.x - a.x) - (d.x - c.x) * (b.z - a.z);
        let ua = r / den;
        if ua > 1.0 || ua < 0.0 {
            return None;
        }
        let s = (b.x - a.x) * (a.z - c.z) - (b.z - a.z) * (a.x - c.x);
        let ub = s / den;
        if ub > 1.0 || ub < 0.0 {
            return None;
        }
        Some(*a + (*b - *a) * ua)
    }

    /// world hit point on `start`->`end`.
    /// VsAITools.cpp 0x00028380
    pub fn line_check(q: &dyn AiCollisionQuery, start: &VsVector4, end: &VsVector4) -> Option<VsVector4> {
        let frac = q.line_check(start, end)?;
        let mut d = *end;
        d -= *start;
        d *= frac;
        let mut out = *start;
        out += d;
        Some(out)
    }

    /// world hit point, normal (w = 1) and fraction on `start`->`end` against barrier objects (mask 17).
    /// VsAITools.cpp 0x00028460
    pub fn line_check_normal(
        q: &dyn AiCollisionQuery,
        start: &VsVector4,
        end: &VsVector4,
    ) -> Option<(VsVector4, VsVector4, f32)> {
        let (frac, n) = q.line_check_normal(start, end, 17)?;
        let mut d = *end;
        d -= *start;
        d *= frac;
        let mut out = *start;
        out += d;
        Some((out, VsVector4::new(n.x, n.y, n.z, 1.0), frac))
    }

    /// angle from `a` to `b` (unit vectors) in [0, 2pi), sign from the cross product's y.
    /// VsAITools.cpp 0x00028580
    pub fn angle_between(a: &VsVector4, b: &VsVector4) -> f32 {
        let d = a.dot3(b);
        let ang = vs_acos(d);
        let c = a.cross(b);
        if c.y > 0.0 { ang } else { 6.283_185_5 - ang }
    }
}
