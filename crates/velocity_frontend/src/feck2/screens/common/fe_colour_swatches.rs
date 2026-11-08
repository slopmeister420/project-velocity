//! src\Feck2\Screens\Hangout\FEHangoutProgression.cpp (0x00290DF0..0x00291CA0, 180-byte object) — the colour
//! swatch grid the car select opens with option 1 (`mpColourSwatches`): 5 rows of 6 "white" squares (16x16, 20
//! apart) tinted with the colour records, a 16x4 highlighter under the current square and the colour's name.
//! Two modes: the vehicle's own colours (+168 set, quick race and the customisation's manufacturer paint) and a
//! colour kind's brand (+172 kind, +176 brand index: the customisation's gloss / metallic / pearl, neon, alloys,
//! windows).

use velocity_physics::vehicle_database::vehicle_database::CVehicleDatabase;

use crate::feck2::feck2_manager::Feck2;
use crate::feck2::feck2_object::*;
use crate::string_manager::string_manager::string_crc;

pub const MAX_SWATCHES: usize = 30;

#[derive(Clone, Debug, Default)]
pub struct CColourSwatches {
    pub layer: LayerId,                    // +0
    pub vehicle: Option<usize>,            // +4 (setup's vehicle)
    pub swatches: [ElementId; MAX_SWATCHES], // +8
    pub highlighter: ElementId,            // +128
    pub name: ElementId,                   // +132
    pub x: i32,                            // +136
    pub y: i32,                            // +140
    pub shown: bool,                       // +144
    pub count: u32,                        // +148
    pub rows: u32,                         // +152
    pub last_row: u32,                     // +156 entries on the last row
    pub row: i32,                          // +160
    pub col: i32,                          // +164
    pub use_vehicle: bool,                 // +168
    pub kind: u32,                         // +172 colour kind (brand mode)
    pub brand_index: u32,                  // +176 nth brand of that kind
}

/// the record's body colours as D3DCOLORs (`VISOS_ColourXBOX::Set` at database init, record +0x34 / +0x38)
fn record_colours(db: &CVehicleDatabase, i: usize) -> (u32, u32) {
    let c = &db.colours[i];
    let k = visos_render::car_palette::CarColour::from_record(c.unk_0c, c.colour_a, c.colour_b);
    (k.colour_a, k.colour_b)
}

impl CColourSwatches {
    /// FEHangoutProgression.cpp 0x00290E90 / 0x00290DF0 / 0x00290F20 — `Init(layer, x, y)` (+ ctor, create)
    pub fn init(fe: &mut Feck2, layer: LayerId, x: i32, y: i32) -> Self {
        let mut s = Self { layer, x: x - 60, y, ..Default::default() };
        let mut d = SElementDesc::default();
        for row in 0..5 {
            d.position[1] = (20 * (row + 1) + s.y) as u32 as f32;
            for col in 0..6 {
                d.kind = ELEMENT_2D;
                d.crc = string_crc("CarColour");
                d.position[0] = (20 * col + s.x) as u32 as f32;
                d.texture = "white".into();
                s.swatches[(col + 6 * row) as usize] = fe.create_element(layer, &d);
            }
        }
        d.kind = ELEMENT_2D;
        d.crc = string_crc("ColourSwatchHighighter");
        d.position[0] = s.x as u32 as f32;
        d.position[1] = s.y as u32 as f32 + 20.0 - 4.0;
        d.texture = "white".into();
        d.vcd = "\\FE\\FECustom.vcd".into();
        s.highlighter = fe.create_element(layer, &d);
        d.kind = ELEMENT_STATIC_TEXT;
        d.font = 0;
        d.crc = string_crc("ColourNameText");
        d.position[0] = s.x as u32 as f32 + 60.0;
        d.position[1] = s.y as u32 as f32;
        d.string = 0;
        s.name = fe.create_element(layer, &d);
        s
    }

    /// FEHangoutProgression.cpp 0x00291A70 — load and size everything, hidden
    pub fn load(&mut self, fe: &mut Feck2) {
        for &e in &self.swatches {
            fe.element_load(e);
            fe.e2d(e).set_size(16, 16);
            fe.el(e).set_visible(false);
        }
        fe.element_load(self.highlighter);
        fe.e2d(self.highlighter).set_size(16, 4);
        fe.el(self.highlighter).set_visible(false);
        fe.element_load(self.name);
        fe.el(self.name).set_visible(false);
        fe.text(self.name).set_justification(2);
    }

    /// FEHangoutProgression.cpp 0x00291BA0
    pub fn set_vehicle(&mut self, vehicle: Option<usize>) {
        self.vehicle = vehicle;
    }

    /// FEHangoutProgression.cpp 0x00291240 — show / hide (only the used swatches show)
    pub fn show(&mut self, fe: &mut Feck2, show: bool) {
        self.shown = show;
        for i in 0..MAX_SWATCHES {
            fe.el(self.swatches[i]).set_visible(show && (i as u32) < self.count);
        }
        fe.el(self.highlighter).set_visible(show);
        fe.el(self.name).set_visible(show);
        if show {
            self.row = 0;
            self.col = 0;
        }
    }

    /// FEHangoutProgression.cpp 0x00291320 — `Fill(useVehicle, kind, brandIndex)`: tint the squares
    pub fn fill(&mut self, fe: &mut Feck2, use_vehicle: bool, kind: u32, brand_index: u32) {
        self.count = 0;
        self.use_vehicle = use_vehicle;
        self.kind = kind;
        self.brand_index = brand_index;
        let Some(db) = fe.vehicles.take() else { return };
        let brand = db.get_nth_colour_brand(kind, brand_index, false);
        if use_vehicle {
            self.count = self.vehicle.map_or(0, |v| db.vehicles[v].data.num_colours as u32);
        } else {
            self.count += db.count_matching_colour_brand(brand, kind);
        }
        if self.count > MAX_SWATCHES as u32 {
            log::warn!("WARNING... Too many colours for this vehicle!");
            self.count = MAX_SWATCHES as u32;
        }
        if self.count != 0 {
            self.rows = self.count / 6;
            self.last_row = self.count % 6;
            if self.last_row != 0 {
                self.rows += 1;
            } else {
                self.last_row = self.count.min(6);
            }
            if use_vehicle {
                for i in 0..self.count {
                    if let Some(c) = db.get_vehicle_colour(self.vehicle, i as i16) {
                        let (a, b) = record_colours(&db, c);
                        fe.e2d(self.swatches[i as usize]).set_colour(a, true, b);
                    }
                }
            } else {
                let mut shown = 0;
                for n in 0..db.count_matching_colour_brand(brand, kind) {
                    if shown >= self.count {
                        continue;
                    }
                    if let Some(c) = db.get_nth_matching_colour_brand(brand, kind, n) {
                        let (a, b) = record_colours(&db, c);
                        if db.colours[c].unk_0c == 5 {
                            // see-through: alpha (byte +0x3C / 255)^2, the colour halved
                            let t = (db.colours[c].unk_3c & 0xFF) as f32 / 255.0;
                            let t = t * t;
                            let alpha = (t * 128.0) as u8 as u32;
                            let k = crate::feck2::feck2_manager::visos_colour(alpha, ((a >> 16) & 0xFF) / 2, ((a >> 8) & 0xFF) / 2, (a & 0xFF) / 2);
                            fe.e2d(self.swatches[shown as usize]).set_colour(k, true, k);
                        } else {
                            fe.e2d(self.swatches[shown as usize]).set_colour(a, true, b);
                        }
                    }
                    shown += 1;
                }
            }
        } else {
            // no colours: the first six of the global table
            self.rows = 1;
            self.last_row = 6;
            self.count = 6;
            for i in 0..6 {
                if let Some(c) = db.get_vehicle_colour(None, i) {
                    let (a, b) = record_colours(&db, c);
                    fe.e2d(self.swatches[i as usize]).set_colour(a, true, b);
                }
            }
        }
        fe.vehicles = Some(db);
    }

    /// FEHangoutProgression.cpp 0x00291760 — row up / down (wraps; clamps the column on the last row)
    pub fn move_row(&mut self, fe: &mut Feck2, d: i32) {
        self.row += d;
        if self.row < 0 {
            self.row = self.rows as i32 - 1;
        }
        if self.row > self.rows as i32 - 1 {
            self.row = 0;
        }
        if self.row == self.rows as i32 - 1 && self.col >= self.last_row as i32 {
            self.col = self.last_row as i32 - 1;
        }
        if d != 0 {
            self.place(fe);
        }
    }

    /// FEHangoutProgression.cpp 0x00291830 — column left / right (wraps within the row)
    pub fn move_col(&mut self, fe: &mut Feck2, d: i32) {
        let n = if self.row == self.rows as i32 - 1 { self.last_row as i32 } else { 6 };
        self.col += d;
        if self.col < 0 {
            self.col = n - 1;
        }
        if self.col > n - 1 {
            self.col = 0;
        }
        if d != 0 {
            self.place(fe);
        }
    }

    /// FEHangoutProgression.cpp 0x002918E0 — highlighter under the square, the colour's name
    fn place(&mut self, fe: &mut Feck2) {
        let x = (20 * self.col + self.x) as u32 as f32;
        let y = (self.y + 20 * (self.row + 1) - 4) as u32 as f32;
        fe.element_set_position(self.highlighter, x, y, 0.0);
        if let Some(c) = self.current(fe) {
            let h = fe.vehicles.as_ref().map_or(0, |db| db.colours[c].hash);
            fe.text(self.name).set_string(h);
        }
    }

    /// FEHangoutProgression.cpp 0x00291BC0 — the colour under the highlighter (and its name)
    pub fn current(&mut self, fe: &mut Feck2) -> Option<usize> {
        let i = self.col + 6 * self.row;
        let c = fe.vehicles.as_ref().and_then(|db| {
            if self.use_vehicle {
                db.get_vehicle_colour(self.vehicle, i as i16)
            } else {
                let brand = db.get_nth_colour_brand(self.kind, self.brand_index, false);
                db.get_nth_matching_colour_brand(brand, self.kind, i as u32)
            }
        });
        if let Some(c) = c {
            let h = fe.vehicles.as_ref().map_or(0, |db| db.colours[c].hash);
            fe.text(self.name).set_string(h);
        }
        c
    }

    /// FEHangoutProgression.cpp 0x00291CA0 — start on the setup's colour (matched by id and brand)
    pub fn select(&mut self, fe: &mut Feck2, hash: u32, brand: u32) {
        let mut found = 0u32;
        self.row = 0;
        self.col = 0;
        if let Some(db) = fe.vehicles.as_ref() {
            if self.use_vehicle {
                for i in 0..self.count {
                    if let Some(c) = db.get_vehicle_colour(self.vehicle, i as i16) {
                        if db.colours[c].brand == brand && db.colours[c].hash == hash {
                            found = i;
                            break;
                        }
                    }
                }
            } else {
                let b = db.get_nth_colour_brand(self.kind, self.brand_index, false);
                for j in 0..db.count_matching_colour_brand(b, self.kind) {
                    if self.count != 0 {
                        if let Some(c) = db.get_nth_matching_colour_brand(b, self.kind, j) {
                            if db.colours[c].brand == brand && db.colours[c].hash == hash {
                                found = j;
                                break;
                            }
                        }
                    }
                }
            }
        }
        self.row = (found / 6) as i32;
        self.col = (found % 6) as i32;
        self.move_row(fe, 0);
        self.move_col(fe, 0);
        self.place(fe);
    }
}
