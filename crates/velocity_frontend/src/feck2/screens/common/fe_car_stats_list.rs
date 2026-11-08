//! The R2 car statistics list (no `__FILE__`; code 0x002894A0..0x0028B800 in the idb's FECK2ElementStaticText.cpp
//! range, 32-byte object at QuickRaceCarSelectScreen +5448 and PlayerGarageScreen +40): base model and value, the
//! installed parts by category / sub category (`CVehicleDatabase::InitPartLabels` 0x00185420 labels), the total, on
//! the common overlay's layer under a "HighlighterGraphic" box; rows outside y 150..350 are hidden and up / down
//! scroll them 5 at a time. The part label tables are `CVehicleDatabase`'s (physics' crate), copied here.

use velocity_physics::vehicle_database::vehicle_database::{CVehicleDatabase, VehicleSetup};

use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_WHITE};
use crate::feck2::feck2_object::{ElementId, LayerId, SElementDesc, ELEMENT_2D, ELEMENT_STATIC_TEXT};
use crate::render2d::Colour;
use crate::feck2::screens::common::fe_2d_overlay as overlay;

/// `GENERAL_BLANK`
const BLANK: u32 = 1580095098;
/// "InstalledPartsText"
const ROW_CRC: u32 = -631902386i32 as u32;
/// rows are shown while 150 <= y <= 350
const TOP: i32 = 150;
const BOTTOM: i32 = 350;

/// `CVehicleDatabase` +156 (`InitPartLabels` 0x00185420): the label string of each part type (`ePART_COUNT` 35)
const PART_TYPE_LABELS: [u32; 35] = [
    1818604040,
    -1802203687i32 as u32,
    1340717044,
    -248971534i32 as u32,
    -261307960i32 as u32,
    1410799840,
    -1055294654i32 as u32, // CUSTOM_PERF_ACCEL_FILTERS
    2102525851,            // CUSTOM_PERF_ACCEL_FLYWHEELS
    -514054214i32 as u32,  // CUSTOM_PERF_ACCEL_NOS
    -423225690i32 as u32,  // CUSTOM_PERF_SPEED_ENGINE
    -841616680i32 as u32,  // CUSTOM_PERF_SPEED_EXHAUST
    -238272160i32 as u32,  // CUSTOM_PERF_SPEED_TURBO
    1665177115,
    0,
    0,
    -1172363839i32 as u32,
    79852367,
    -216473935i32 as u32,  // CARCUSTOMISATION_STYL_WHEELS_STYLE
    1592117897,            // CARCUSTOMISATION_PERF_BRAKES
    1685424696,            // CUSTOM_PERF_HANDLE_TIRES
    -1143096285i32 as u32, // CUSTOM_PERF_BRAKES_DISCS
    531047780,             // CUSTOM_PERF_BRAKES_PADS
    -308859737i32 as u32,  // CARCUSTOMISATION_STYL_BODY
    414583488,             // CUSTOM_STYL_BODY_NEON
    -1177401965i32 as u32, // CARCUSTOMISATION_COLOUR_ALLOYS
    504948245,             // CARCUSTOMISATION_COLOUR_WINDOWS
    1592117897,            // CARCUSTOMISATION_PERF_BRAKES
    -1103333370i32 as u32, // CUSTOM_STYL_BODY_DECAL_BONNET
    825797604,             // CUSTOM_STYL_BODY_DECAL_SIDE
    217500367,             // CUSTOM_STYL_BODY_LOGO_SIDE
    1912807482,            // CUSTOM_STYL_BODY_LOGO_F_BUMPER
    -1963843605i32 as u32, // CUSTOM_STYL_BODY_LOGO_R_BUMPER
    975653016,             // CUSTOM_STYL_BODY_LOGO_TRIM
    664820064,             // CUSTOM_STYL_BODY_LOGO_F_WINDSCREEN
    683577315,             // CUSTOM_STYL_BODY_LOGO_R_WINDSCREEN
];
/// `CVehicleDatabase` +296: category labels (`ePARTCATEGORY_COUNT` 4)
const CATEGORY_LABELS: [u32; 4] = [
    BLANK,
    347540702, // CARCUSTOMISATION_CAT_PERFORMANCE
    499851349, // CARCUSTOMISATION_CAT_STYLING
    539264011, // CUSTOM_STYL_BODY_DECAL
];
/// `CVehicleDatabase` +312: sub category labels (`ePARTSUBCATEGORY_COUNT` 11)
const SUBCATEGORY_LABELS: [u32; 11] = [
    BLANK,
    729661107,             // CARCUSTOMISATION_PERF_ACCELERATION
    920967216,             // CARCUSTOMISATION_PERF_TOPSPEED
    -805620158i32 as u32,  // CARCUSTOMISATION_PERF_HANDLING
    1592117897,            // CARCUSTOMISATION_PERF_BRAKES
    -14578247i32 as u32,   // CARCUSTOMISATION_CAT_COLOUR
    539264011,             // CUSTOM_STYL_BODY_DECAL
    217500367,             // CUSTOM_STYL_BODY_LOGO_SIDE
    1912807482,            // CUSTOM_STYL_BODY_LOGO_F_BUMPER
    -1963843605i32 as u32, // CUSTOM_STYL_BODY_LOGO_R_BUMPER
    975653016,             // CUSTOM_STYL_BODY_LOGO_TRIM
];
/// `dword_687850` / `dword_6878E0`: category and sub category of each part type
const PART_CATEGORY: [u32; 35] = [2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 2, 1, 2, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
const PART_SUBCATEGORY: [u32; 35] = [0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 0, 0, 0, 0, 0, 4, 3, 4, 4, 5, 5, 5, 5, 5, 6, 6, 7, 8, 9, 10, 6, 6];

/// VehicleDatabase.cpp 0x00185EB0 — `GetPartTypeLabel(type, sub)`: brake parts (18) take their sub category's label
pub fn part_type_label(ty: usize, sub: usize) -> u32 {
    debug_assert!(ty < 35, "Part type is out of range.");
    if ty == 18 && sub != 0 {
        PART_TYPE_LABELS[19 + sub]
    } else {
        PART_TYPE_LABELS[ty]
    }
}

/// VehicleDatabase.cpp 0x00186230 — `GetPartCetegories(part)`: (category, sub category) of a part type
pub fn part_categories(ty: usize) -> (u32, u32) {
    (PART_CATEGORY[ty], PART_SUBCATEGORY[ty])
}

/// VehicleDatabase.cpp 0x00186420 — the brake colour's name (setup +0x84): blue / red / yellow pads, else 0
pub fn brake_colour_label(setup: &VehicleSetup) -> u32 {
    let c = setup.unk_084;
    // BRAKES00n_BLUE / _RED / _YELLOW
    if [1849748705, -388719025i32 as u32, 596135402].contains(&c) {
        return -716299548i32 as u32; // CARCUSTOMISATION_COLOUR_3
    }
    if [-1933689287i32 as u32, -887208727i32 as u32, -159486631i32 as u32].contains(&c) {
        return 994067400; // CARCUSTOMISATION_COLOUR_1
    }
    if [-2078530822i32 as u32, 177415449, -969333369i32 as u32].contains(&c) {
        return -1524312469i32 as u32; // CARCUSTOMISATION_COLOUR_6
    }
    0
}

/// one row (32 bytes: label, text, value or null, `char[15]` "%d" buffer at +16)
#[derive(Clone, Copy, Debug)]
pub struct StatsRow {
    pub label: ElementId,
    pub text: ElementId,
    pub value: Option<ElementId>,
}

#[derive(Default)]
pub struct CCarStatsList {
    pub rows: Vec<StatsRow>,           // +0 std::list
    pub layer: LayerId,                // +8
    pub setup: Option<VehicleSetup>,   // +12 (a pointer in the original)
    pub highlighter: Option<ElementId>, // +16
    pub top: i32,                      // +20
    pub bottom: i32,                   // +24
    pub y: f32,                        // +28 the next row's y while filling
}

impl CCarStatsList {
    /// FECK2ElementStaticText.cpp 0x002894A0 / 0x00289510 — ctor, then `Init(layer)` (top 150; the part labels of
    /// `InitPartLabels` are the tables above)
    pub fn new(layer: LayerId) -> Self {
        Self { layer, top: TOP, ..Default::default() }
    }

    /// FECK2ElementStaticText.cpp 0x00289780 — the setup listed
    pub fn set_setup(&mut self, setup: VehicleSetup) {
        self.setup = Some(setup);
    }

    /// FECK2ElementStaticText.cpp 0x00289550 — load every row's elements, then hide (`Show(false)` deletes them)
    pub fn reset(&mut self, fe: &mut Feck2) {
        for r in self.rows.clone() {
            fe.element_load(r.label);
            fe.element_load(r.text);
            if let Some(v) = r.value {
                fe.element_load(v);
            }
        }
        self.show(fe, false);
    }

    /// FECK2ElementStaticText.cpp 0x00289660 — delete every row and the highlighter
    pub fn clear(&mut self, fe: &mut Feck2) {
        for r in std::mem::take(&mut self.rows) {
            fe.delete_element(r.label);
            fe.delete_element(r.text);
            if let Some(v) = r.value {
                fe.delete_element(v);
            }
        }
        if let Some(h) = self.highlighter.take() {
            fe.delete_element(h);
        }
    }

    /// FECK2ElementStaticText.cpp 0x0028B110 / 0x0028B5E0 — a row at (50, y) / (250, y): label in `label_font` /
    /// `label_colour`, text in `text_font` / `text_colour`, the value ("%d", label colour) when `show_value`; y +=
    /// `advance`
    #[allow(clippy::too_many_arguments)]
    fn add_line(&mut self, fe: &mut Feck2, label: u32, text: u32, advance: f32, label_font: u32, label_colour: Colour, text_font: u32, text_colour: Colour, show_value: bool, value: i32) {
        let mut d = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: label_font, crc: ROW_CRC, ..Default::default() };
        d.position[0] = f32::from_bits(0x42480000);
        d.position[1] = self.y;
        d.string = label;
        let l = fe.create_element(self.layer, &d);
        fe.text(l).set_colour(label_colour);
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = text_font;
        d.crc = ROW_CRC;
        d.position[0] = f32::from_bits(0x437A0000);
        d.position[1] = self.y;
        d.string = text;
        let t = fe.create_element(self.layer, &d);
        fe.text(t).set_colour(text_colour);
        let v = if show_value {
            d.kind = ELEMENT_STATIC_TEXT;
            d.font = text_font;
            d.crc = ROW_CRC;
            d.position[0] = f32::from_bits(0x437A0000);
            d.position[1] = self.y;
            let v = fe.create_element(self.layer, &d);
            // _snprintf(+16, 15, "%d"), +31 = 0
            let s: Vec<u16> = format!("{value}").chars().take(15).collect::<String>().encode_utf16().collect();
            fe.text(v).set_text(&s);
            fe.text(v).set_colour(label_colour);
            Some(v)
        } else {
            None
        };
        self.rows.push(StatsRow { label: l, text: t, value: v });
        self.y += advance;
    }

    /// FECK2ElementStaticText.cpp 0x0028B3A0 — fitted decals per decal sub category: (logos side, front bumper, rear
    /// bumper, trim, the others)
    fn decal_counts(setup: &VehicleSetup) -> (u32, u32, u32, u32, u32) {
        let n = |s: &[Option<usize>]| s.iter().filter(|d| d.is_some()).count() as u32;
        let other = n(&[setup.decal_0, setup.decal_1, setup.decal_2_4, setup.decal_2_5]);
        (n(&setup.decals_3), n(&setup.decals_3_1), n(&setup.decals_3_2), n(&setup.decals_3_3), other)
    }

    /// FECK2ElementStaticText.cpp 0x0028A480 — the setup's parts of category `cat` / sub category `sub`
    fn installed(&mut self, fe: &mut Feck2, db: &CVehicleDatabase, cat: u32, sub: u32, prices: bool) {
        let Some(setup) = self.setup.clone() else { return };
        let w = COLOUR_WHITE;
        let sixteen = f32::from_bits(0x41800000);
        for p in setup.fitted_bodyparts.iter().flatten() {
            let part = db.part(*p);
            if part_categories(p.part_type) == (cat, sub) {
                self.add_line(fe, part_type_label(p.part_type, 0), part.hash, sixteen, 1, w, 1, w, prices, part.cost as i32);
            }
        }
        if cat == 1 && sub == 4 {
            for i in 0..2 {
                if let Some(b) = setup.brake_parts[i].and_then(|b| db.brake_parts.get(b)) {
                    self.add_line(fe, part_type_label(18, i + 1), b.hash, sixteen, 1, w, 1, w, prices, b.cost as i32);
                }
            }
        }
        if cat == 1 && sub == 3 {
            // the original reads the tyre without a null check
            if let Some(t) = setup.tyre.and_then(|t| db.tyres.get(t)) {
                self.add_line(fe, part_type_label(19, 0), t.hash, sixteen, 1, w, 1, w, prices, t.cost as i32);
            }
        }
        if cat == 2 && sub == 5 {
            self.add_line(fe, part_type_label(22, 0), setup.colours[0].hash, sixteen, 1, w, 1, w, false, 0);
            for (i, ty) in [(1, 23), (2, 24), (3, 25)] {
                if setup.colours[i].hash != 0 {
                    self.add_line(fe, part_type_label(ty, 0), setup.colours[i].hash, sixteen, 1, w, 1, w, false, 0);
                }
            }
            if setup.unk_084 != 0 {
                self.add_line(fe, part_type_label(26, 0), brake_colour_label(&setup), sixteen, 1, w, 1, w, false, 0);
            }
        }
        let decal = |i: Option<usize>| i.and_then(|i| db.decals.get(i)).map(|d| (d.hash, d.unk_0c as u16 as i16 as i32));
        if cat == 2 && sub == 6 {
            for (d, ty) in [(setup.decal_0, 27), (setup.decal_1, 28), (setup.decal_2_4, 33), (setup.decal_2_5, 34)] {
                if let Some((hash, cost)) = decal(d) {
                    self.add_line(fe, part_type_label(ty, 0), hash, sixteen, 1, w, 1, w, prices, cost);
                }
            }
        }
        let list: &[Option<usize>] = match (cat, sub) {
            (2, 7) => &setup.decals_3,
            (2, 8) => &setup.decals_3_1,
            (2, 9) => &setup.decals_3_2,
            (2, 10) => &setup.decals_3_3,
            _ => &[],
        };
        for d in list {
            // the decal's own name is the label here, the text is blank
            if let Some((hash, cost)) = decal(*d) {
                self.add_line(fe, hash, BLANK, sixteen, 1, w, 1, w, prices, cost);
            }
        }
    }

    /// FECK2ElementStaticText.cpp 0x002897A0 — rebuild the list from the setup (with the value / total lines and
    /// prices when `prices`), lay the texts out right of their labels and make the highlighter box
    pub fn fill(&mut self, fe: &mut Feck2, db: &mut CVehicleDatabase, prices: bool) {
        self.clear(fe);
        let w = COLOUR_WHITE;
        let orange = visos_colour(128, 255, 110, 0);
        let twenty = f32::from_bits(0x41A00000);
        let sixteen = f32::from_bits(0x41800000);
        let cost = self.setup.as_ref().and_then(|s| s.vehicle).map_or(0, |v| db.vehicles[v].unk_10 as i32);
        if prices {
            self.add_line(fe, 1079643652, BLANK, twenty, 2, orange, 1, w, false, 0); // CARSELECTION_CAR_BASEMODEL
            self.add_line(fe, -1473252213i32 as u32, BLANK, sixteen, 1, w, 1, w, true, cost); // CARSELECTION_VALUE
            self.y += 20.0;
        }
        self.add_line(fe, CATEGORY_LABELS[1], BLANK, twenty, 2, orange, 1, w, false, 0);
        for sub in 1..=4 {
            self.add_line(fe, SUBCATEGORY_LABELS[sub], BLANK, sixteen, 1, orange, 1, w, false, 0);
            self.installed(fe, db, 1, sub as u32, prices);
        }
        self.y += 20.0;
        self.add_line(fe, CATEGORY_LABELS[2], BLANK, twenty, 2, orange, 1, w, false, 0);
        self.installed(fe, db, 2, 0, prices);
        self.y += 20.0;
        self.add_line(fe, SUBCATEGORY_LABELS[5], BLANK, twenty, 2, orange, 1, w, false, 0);
        self.installed(fe, db, 2, 5, prices);
        let (side, front, rear, trim, other) = self.setup.as_ref().map_or((0, 0, 0, 0, 0), Self::decal_counts);
        if side + front + rear + trim + other != 0 {
            self.y += 20.0;
            self.add_line(fe, CATEGORY_LABELS[3], BLANK, twenty, 2, orange, 1, w, false, 0);
            for (n, sub) in [(side, 7), (front, 8), (rear, 9), (trim, 10), (other, 6)] {
                if n != 0 {
                    self.add_line(fe, SUBCATEGORY_LABELS[sub], BLANK, sixteen, 1, orange, 1, w, false, 0);
                    self.installed(fe, db, 2, sub as u32, prices);
                }
            }
        }
        if prices {
            self.y += 20.0;
            let parts = self.setup.as_ref().map_or(0, |s| crate::feck2::screens::garage::setup_parts_cost(db, s, false));
            // CARSELECTION_TOTAL_NO_COLON
            self.add_line(fe, 1031491977, BLANK, sixteen, 2, orange, 2, orange, true, cost + parts);
        }
        let mut d = SElementDesc { kind: ELEMENT_2D, crc: 1180424525, ..Default::default() }; // "HighlighterGraphic"
        d.texture = "FE_track_stats_box".into();
        d.vcd = "\\FE\\FE.vcd".into();
        d.position[0] = f32::from_bits(0xC1F00000);
        d.position[1] = f32::from_bits(0x43110000);
        d.f76 = f32::from_bits(0x42800000);
        let h = fe.create_element(self.layer, &d);
        self.highlighter = Some(h);
        for r in self.rows.clone() {
            fe.element_load(r.label);
            fe.element_load(r.text);
            let lx = fe.el(r.label).base().position[0];
            let ty = fe.el(r.text).base().position[1];
            let lw = fe.text_width(r.label);
            let x = (lw as i64 as f64 + lx as f64 + 10.0) as f32;
            fe.el(r.text).base_mut().set_position(x, ty, 0.0);
            if let Some(v) = r.value {
                fe.element_load(v);
                let vw = fe.text_width(v);
                let x = ((lx + 200.0) as f64 - vw as i64 as f64) as f32;
                fe.el(v).base_mut().set_position(x, ty, 0.0);
            }
        }
        fe.element_load(h);
        if prices {
            // 0x0028B5C0 / 0x00213BA0: the box 30 wider for the value column
            let e = fe.e2d(h);
            e.width = (e.width as i64 as f64 + 30.0) as u32;
        }
    }

    /// FECK2ElementStaticText.cpp 0x0028ACF0 — fill from the top; the bottom is where the last row ended
    pub fn refill(&mut self, fe: &mut Feck2, db: &mut CVehicleDatabase, prices: bool) {
        self.y = self.top as f32;
        self.fill(fe, db, prices);
        self.bottom = self.y as i32;
    }

    /// FECK2ElementStaticText.cpp 0x0028AD40 — show: the overlay's up/down arrows at (200, 155..350), every element,
    /// scrolled back to the top; hide: everything deleted, arrows off
    pub fn show(&mut self, fe: &mut Feck2, show: bool) {
        if show {
            if let Some(mut o) = fe.overlay.take() {
                overlay::show_arrows(&mut o, fe, 1);
                overlay::position_arrows(&mut o, fe, f32::from_bits(0x43480000), f32::from_bits(0x431B0000), f32::from_bits(0x43AF0000), 1);
                overlay::set_arrows_visible(&mut o, fe, true, 1);
                fe.overlay = Some(o);
            }
            self.top = TOP;
            for r in self.rows.clone() {
                fe.el(r.label).set_visible(true);
                fe.el(r.text).set_visible(true);
                if let Some(v) = r.value {
                    fe.el(v).set_visible(true);
                }
            }
            if let Some(h) = self.highlighter {
                fe.el(h).set_visible(true);
            }
            self.scroll(fe, 0);
        } else {
            self.clear(fe);
            if let Some(mut o) = fe.overlay.take() {
                overlay::set_arrows_visible(&mut o, fe, false, 1);
                fe.overlay = Some(o);
            }
        }
    }

    /// FECK2ElementStaticText.cpp 0x0028AEC0 — scroll by 5 * `dir` while the top stays at or above 150 and the bottom
    /// at or below 350; 0 = back to the top
    pub fn scroll(&mut self, fe: &mut Feck2, dir: i32) {
        let step = if dir == 0 { TOP - self.top } else { 5 * dir };
        if (step + self.top <= TOP || dir == 0) && (step + self.bottom >= BOTTOM || dir == 0) {
            self.top += step;
            self.bottom += step;
            for r in self.rows.clone() {
                Self::move_element(fe, step, r.text);
                Self::move_element(fe, step, r.label);
                if let Some(v) = r.value {
                    Self::move_element(fe, step, v);
                }
            }
        }
    }

    /// FECK2ElementStaticText.cpp 0x0028B010 — y += `step`, visible while 150 <= y <= 350
    fn move_element(fe: &mut Feck2, step: i32, id: ElementId) {
        let mut p = fe.el(id).base().position;
        p[1] = (p[1] as f64 + step as f64) as f32;
        fe.el(id).set_visible(p[1] >= TOP as f32 && p[1] <= BOTTOM as f32);
        fe.el(id).base_mut().set_position_v(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_manager::string_manager::string_crc;

    #[test]
    fn label_tables_match_their_strings() {
        assert_eq!(part_type_label(6, 0), string_crc("CUSTOM_PERF_ACCEL_FILTERS"));
        assert_eq!(part_type_label(18, 1), string_crc("CUSTOM_PERF_BRAKES_DISCS"));
        assert_eq!(part_type_label(18, 2), string_crc("CUSTOM_PERF_BRAKES_PADS"));
        assert_eq!(part_type_label(19, 0), string_crc("CUSTOM_PERF_HANDLE_TIRES"));
        assert_eq!(part_type_label(34, 0), string_crc("CUSTOM_STYL_BODY_LOGO_R_WINDSCREEN"));
        assert_eq!(CATEGORY_LABELS[2], string_crc("CARCUSTOMISATION_CAT_STYLING"));
        assert_eq!(SUBCATEGORY_LABELS[3], string_crc("CARCUSTOMISATION_PERF_HANDLING"));
        assert_eq!(BLANK, string_crc("GENERAL_BLANK"));
        assert_eq!(ROW_CRC, string_crc("InstalledPartsText"));
        assert_eq!(1031491977, string_crc("CARSELECTION_TOTAL_NO_COLON"));
        assert_eq!(part_categories(9), (1, 2));
    }
}
