//! the car info panel of the quick race car select (`+5444`, code in the FECK2ElementStaticText.cpp-labelled range
//! 0x00287A30..0x002893F0): cost / mileage / transmission lines, three 9-segment bars (acceleration, top speed,
//! handling) in a sliding "FE_statsBox" with the R2 "car stats" prompt above it. The object has two panels (split
//! screen); the quick race uses panel 0 only, so only that one is built here. The R2 stats list (+5448,
//! 0x00289550..0x0028AEC0) is not ported.

use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};

use crate::feck2::feck2_manager::{visos_colour, Feck2};
use crate::feck2::feck2_object::*;
use crate::feck2::screens::common::fe_2d_overlay as overlay;
use crate::string_manager::string_manager::string_crc;

/// `dword_6C1784` / `dword_6C1788` panel 0 origin
pub const PANEL_X: f32 = 320.0;
pub const PANEL_Y: f32 = 335.0;
/// `dword_6C1794` bar rows, `dword_6C17A0` text rows (index 3 of the first table reads the second's first entry)
pub const BAR_Y: [f32; 4] = [16.0, 30.0, 44.0, -38.0];
pub const TEXT_Y: [f32; 3] = [-38.0, -24.0, -10.0];
/// `dword_76A1D8` / `dword_76A1FC` / `dword_76A1F0` (static init 0x004308A0..)
pub const LABELS: [&str; 3] = ["CARINFO_COST_HEADING", "CARINFO_MILEAGE_HEADING", "CARINFO_TRANSMISSION_HEADING"];
pub const VALUES: [&str; 3] = ["CARINFO_COST_VALUE", "CARINFO_MILEAGE_VALUE", "SPLITSCREEN_TRANSMISSION_CAR1"];
pub const BAR_HEADS: [&str; 3] = ["CARINFO_ACCELERATION_HEADING", "CARINFO_TOPSPEED_HEADING", "CARINFO_HANDLING_HEADING"];
/// R2 nav prompt slot of the overlay
const NAV_R2: usize = 13;

#[derive(Clone, Debug, Default)]
pub struct CCarInfoPanel {
    pub labels: [ElementId; 3],      // +4
    pub values: [ElementId; 3],      // +28
    pub background: ElementId,       // +52
    pub highlight: ElementId,        // +60
    pub bar_heads: [ElementId; 3],   // +68
    pub bars: [[ElementId; 9]; 3],   // +92 acceleration, +164 top speed, +236 handling
}

impl CCarInfoPanel {
    /// FECK2ElementStaticText.cpp 0x00287C00 — create (panel 0; the idb's label for this code)
    pub fn create(fe: &mut Feck2, layer: LayerId) -> Self {
        let mut p = Self::default();
        let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, crc: string_crc("Car1Name"), ..Default::default() };
        for j in 0..3 {
            d.font = 0;
            d.position[0] = PANEL_X;
            d.position[1] = PANEL_Y + TEXT_Y[j];
            d.string = string_crc(LABELS[j]);
            p.labels[j] = fe.create_element(layer, &d);
        }
        for j in 0..3 {
            d.font = 0;
            d.position[0] = PANEL_X + 105.0;
            d.position[1] = PANEL_Y + TEXT_Y[j];
            // original: indexed without the panel stride
            d.string = string_crc(VALUES[j]);
            p.values[j] = fe.create_element(layer, &d);
        }
        for j in 0..3 {
            d.font = 0;
            d.position[0] = PANEL_X + 0.0;
            d.position[1] = PANEL_Y + BAR_Y[j];
            d.string = string_crc(BAR_HEADS[j]);
            p.bar_heads[j] = fe.create_element(layer, &d);
        }
        d.kind = ELEMENT_2D;
        d.crc = string_crc("Car1Data");
        d.vcd = "\\FE\\FE.vcd".into();
        for m in 0..9 {
            for row in 0..3 {
                d.position[0] = (PANEL_X + 105.0) + (8 * m) as f32;
                d.texture = "white".into();
                d.position[1] = PANEL_Y + BAR_Y[row];
                p.bars[row][m] = fe.create_element(layer, &d);
            }
        }
        d.kind = ELEMENT_2D;
        d.crc = string_crc("CarInfoHighlight");
        d.position[0] = PANEL_X - 22.0;
        d.position[1] = ((PANEL_Y - BAR_Y[3]) + TEXT_Y[0]) - 22.0;
        d.texture = "FE_info_highlighter".into();
        p.highlight = fe.create_element(layer, &d);
        d.crc = string_crc("CarInfoBackground");
        d.position[0] = PANEL_X - 22.0;
        d.position[1] = ((PANEL_Y - BAR_Y[3]) + TEXT_Y[0]) - 20.0;
        d.texture = "FE_statsBox".into();
        p.background = fe.create_element(layer, &d);
        p
    }

    /// FECK2ElementStaticText.cpp 0x002882C0 — load: everything faded out, bars 5x10 dark orange, then hidden
    pub fn load(&mut self, fe: &mut Feck2) {
        for j in 0..3 {
            for id in [self.labels[j], self.values[j], self.bar_heads[j]] {
                fe.element_load(id);
                fe.element_fade(id, 0, 0, 0);
            }
        }
        fe.element_load(self.background);
        fe.element_load(self.highlight);
        let c = visos_colour(128, 120, 77, 14);
        let a = ((c >> 24) as f64 * 0.7f32 as f64) as u32;
        fe.e2d(self.highlight).set_colour((c & 0x00FF_FFFF) | (a << 24), false, visos_colour(128, 128, 128, 128));
        for row in &self.bars {
            for &b in row {
                fe.element_load(b);
                fe.e2d(b).set_colour(c & 0x00FF_FFFF, false, visos_colour(128, 128, 128, 128));
                fe.e2d(b).set_size(5, 10);
                fe.element_fade(b, 0, 0, 0);
            }
        }
        self.show(fe, false);
        self.set_visible(fe, false);
    }

    /// FECK2ElementStaticText.cpp 0x00288B90 — visibility of the box, highlight, bar headings and bars
    pub fn set_visible(&mut self, fe: &mut Feck2, v: bool) {
        fe.el(self.background).set_visible(v);
        fe.el(self.highlight).set_visible(v);
        for j in 0..3 {
            fe.el(self.bar_heads[j]).set_visible(v);
            for &b in &self.bars[j] {
                fe.el(b).set_visible(v);
            }
        }
    }

    /// FECK2ElementStaticText.cpp 0x00288840 / 0x00288950 / 0x00288A70 — light `9 * v` segments orange, the rest grey
    fn set_bar(&mut self, fe: &mut Feck2, row: usize, v: f32) {
        let mut n = (9.0 * v.clamp(0.0, 1.0)) as i32;
        let mut c = visos_colour(128, 255, 110, 0);
        for &b in &self.bars[row] {
            if n == 0 {
                c = visos_colour(128, 64, 64, 64);
            }
            fe.e2d(b).set_colour(c, false, visos_colour(128, 128, 128, 128));
            n -= 1;
        }
    }

    /// FECK2ElementStaticText.cpp 0x00289220 — fill from a setup: bars, cost and mileage
    pub fn fill(&mut self, fe: &mut Feck2, db: &mut CVehicleDatabase, setup: &VehicleSetup) {
        let Some(v) = setup.vehicle else { return };
        // VehicleDatabasePhysics 0x00189500: 6 - 6 * 0.5 * ((top speed + acceleration modifiers) * 0.5), in st0
        let base = 6.0f32;
        let half = base * 0.5;
        let m = ((setup.get_top_speed_modifier(db) as f64 + setup.get_acceleration_modifier(db) as f64) as f32) * 0.5;
        let accel = (base as f64 - (half * m) as f64) as f32;
        self.set_bar(fe, 0, (accel - 5.0) / 5.0);
        let handling = setup.get_handling_modifier(db);
        self.set_bar(fe, 2, handling);
        let top = setup.get_top_speed(db) as f32;
        self.set_bar(fe, 1, (top - 120.0) / 100.0);
        let cost = db.vehicles[v].unk_10 as i32 + db.get_bodyparts_cost(setup, false);
        let s = fe.text(self.values[0]).string.unwrap_or(0);
        fe.strings.set_argument(s, string_crc("dCost"), cost as u32).ok();
        let mileage = setup.mileage as i64 as u32;
        let s = fe.text(self.values[1]).string.unwrap_or(0);
        fe.strings.set_argument(s, string_crc("dMileage"), mileage).ok();
    }

    /// FECK2ElementStaticText.cpp 0x002893F0 — the transmission line
    pub fn set_transmission(&mut self, fe: &mut Feck2, automatic: bool) {
        let s = if automatic { "SPLITSCREEN_TRANSMISSION_AUTO" } else { "SPLITSCREEN_TRANSMISSION_MANUAL" };
        fe.text(self.values[2]).set_string(string_crc(s));
    }

    /// FECK2ElementStaticText.cpp 0x00288C90 — (panel 0) the R2 prompt above the box, then 0x00288DC0 slides the box in / out
    pub fn show(&mut self, fe: &mut Feck2, show: bool) {
        if let Some(mut o) = fe.overlay.take() {
            overlay::show_nav(&mut o, fe, NAV_R2, show);
            overlay::position_nav(&mut o, fe, NAV_R2, PANEL_X, (PANEL_Y + TEXT_Y[0]) - 29.0);
            fe.overlay = Some(o);
        }
        self.slide(fe, show);
    }

    /// FECK2ElementStaticText.cpp 0x00288DC0
    fn slide(&mut self, fe: &mut Feck2, show: bool) {
        let dist = 250.0f32;
        let (alpha, time, delay) = if show { (100.0f32, 30.0f32, 30.0f32) } else { (0.0, 30.0, 0.0) };
        if show {
            let x = (PANEL_X - 22.0) + dist;
            let y = ((PANEL_Y - BAR_Y[0]) + -24.0) - 20.0;
            fe.element_set_position(self.background, x, y, 0.0);
            fe.el(self.background).base_mut().move_by_x((0.0 - dist) as i32, time as i32, 0);
            fe.element_set_position(self.highlight, x + 5.0, y - 4.0, 0.0);
            fe.el(self.highlight).base_mut().move_by_x((0.0 - dist) as i32, time as i32, 0);
            self.set_visible(fe, true);
        } else {
            fe.el(self.background).base_mut().move_by_x(dist as i32, 30, 0);
            fe.el(self.highlight).base_mut().move_by_x(dist as i32, time as i32, 0);
        }
        fe.element_fade(self.background, alpha as i32, time as i32, 0);
        fe.element_fade(self.highlight, (alpha * 0.7f32) as i32, time as i32, 0);
        for j in 0..3 {
            for id in [self.labels[j], self.values[j], self.bar_heads[j]] {
                fe.element_fade(id, alpha as i32, time as i32, delay as i32);
            }
        }
        for row in &self.bars {
            for &b in row {
                fe.element_fade(b, alpha as i32, time as i32, delay as i32);
            }
        }
    }
}
