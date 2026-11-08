//! src\Feck2\FECK2Elements\FECK2Element2DLine.cpp — `CElement2DLine` (0xA8 bytes, vtable 0x0065A234: Init slot 11
//! 0x00293410, Render slot 12 0x002936B0): a polyline of up to five 2D points drawn by GeomTool.

use crate::feck2::feck2_object::{CElementBase, SElementDesc};
use crate::render2d::{Colour, DrawList, Quad, WHITE};

/// `MAX_POINTS_PER_FECK_2D_LINE`
pub const MAX_POINTS: usize = 5;

#[derive(Clone, Debug)]
pub struct CElement2DLine {
    pub base: CElementBase,
    pub points: [[f32; 2]; MAX_POINTS], // +108
    pub f148: u32,                      // +148
    pub f152: u32,                      // +152
    pub count: u32,                     // +156
    pub colour: Colour,                 // +160
    pub colour2: Colour,                // +164
}

impl Default for CElement2DLine {
    fn default() -> Self {
        Self { base: CElementBase::default(), points: [[0.0; 2]; MAX_POINTS], f148: 0, f152: 0, count: 0, colour: WHITE, colour2: WHITE }
    }
}

impl CElement2DLine {
    /// FECK2Element2DLine.cpp 0x00293410 — `Init`
    pub fn init(&mut self, d: &SElementDesc) {
        self.base.init(d);
        self.base.position = d.position;
        self.base.target = self.base.position;
        self.count = 0;
        self.f148 = 0;
        self.f152 = 0;
        self.colour = d.colour;
        self.colour2 = d.colour;
    }

    /// FECK2Element2DLine.cpp 0x00293590 — `AddPoint`
    pub fn add_point(&mut self, x: f32, y: f32) {
        if self.count as usize >= MAX_POINTS {
            log::error!("assert m_PointsCount<MAX_POINTS_PER_FECK_2D_LINE (FECK2Element2DLine.cpp 79)");
            return;
        }
        self.points[self.count as usize] = [x, y];
        self.count += 1;
    }

    /// FECK2Element2DLine.cpp 0x00293670 — `ClearPoints`
    pub fn clear_points(&mut self) {
        self.count = 0;
    }

    /// FECK2Element2DLine.cpp 0x002936B0 — `Render`
    pub fn render(&self, out: &mut DrawList) {
        draw_line(&self.points[..self.count as usize], 1.0, self.colour, out);
    }
}

fn normalise(v: [f32; 2]) -> [f32; 2] {
    let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if l > 0.0 { [v[0] / l, v[1] / l] } else { v }
}

/// `VsAITools 0x000293D0` cross(d, (0,0,1)) in the xy plane
fn side(d: [f32; 2]) -> [f32; 2] {
    [d[1], -d[0]]
}

/// GeomTool.cpp 0x003C6CF0 — a polyline `width` either side of the points, mitred joins; one quad per segment
/// (0x003C6C50, texture `dword_7B5714`, here untextured).
pub fn draw_line(p: &[[f32; 2]], width: f32, colour: Colour, out: &mut DrawList) {
    let n = p.len();
    if n < 2 {
        log::error!("assert \"Need at least 2 points to draw line\" (GeomTool.cpp 297)");
        return;
    }
    let mut dir = normalise([p[1][0] - p[0][0], p[1][1] - p[0][1]]); // v51
    let mut nrm = normalise(side(dir)); // v43
    let mut next_dir = [0.0f32; 2]; // v44
    let mut next_nrm = [0.0f32; 2]; // v45
    let mut l = [nrm[0] * width + p[0][0], nrm[1] * width + p[0][1]]; // v47 / v48
    let mut r = [p[0][0] - nrm[0] * width, p[0][1] - nrm[1] * width]; // v49 / v50
    for i in 0..n - 1 {
        let t = if i >= n - 2 {
            0.0
        } else {
            next_dir = normalise([p[i + 2][0] - p[i + 1][0], p[i + 2][1] - p[i + 1][1]]);
            next_nrm = normalise(side(next_dir));
            // the next segment's left edge as a plane (0x00143060), hit from this segment's left edge (0x002C3E10)
            let o = [nrm[0] * width + p[i + 1][0], nrm[1] * width + p[i + 1][1]]; // v38
            let q = [next_nrm[0] * width + p[i + 1][0], next_nrm[1] * width + p[i + 1][1]]; // v36
            let w = -(next_nrm[0] * q[0] + next_nrm[1] * q[1]);
            let denom = dir[0] * next_nrm[0] + dir[1] * next_nrm[1];
            if denom.abs() < 0.000001 {
                0.0
            } else {
                -(next_nrm[0] * o[0] + next_nrm[1] * o[1] + w) / denom
            }
        };
        let (pl, pr) = (l, r);
        l = [dir[0] * t + (nrm[0] * width + p[i + 1][0]), dir[1] * t + (nrm[1] * width + p[i + 1][1])];
        r = [(p[i + 1][0] - nrm[0] * width) - dir[0] * t, (p[i + 1][1] - nrm[1] * width) - dir[1] * t];
        out.push(Quad { texture: 0, pos: [pl, l, pr, r], uv: [[0.0; 2]; 4], colour: [colour; 4] });
        nrm = next_nrm;
        dir = next_dir;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mitred_corner() {
        // the loading screen's placeholder line 1 at (284, 196): (10,10) (40,10) (40,40)
        let mut out = DrawList::default();
        draw_line(&[[294.0, 206.0], [324.0, 206.0], [324.0, 236.0]], 1.0, WHITE, &mut out);
        assert_eq!(out.quads.len(), 2);
        // first segment runs right, its left side is y - 1; the mitre meets the second segment's edge at x 323
        assert_eq!(out.quads[0].pos[0], [294.0, 205.0]);
        assert_eq!(out.quads[0].pos[1], [325.0, 205.0]);
        assert_eq!(out.quads[0].pos[3], [323.0, 207.0]);
        assert_eq!(out.quads[1].pos[3], [323.0, 236.0]);
    }
}
