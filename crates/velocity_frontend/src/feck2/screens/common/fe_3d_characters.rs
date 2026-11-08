//! src\Feck2\Screens\Common\FE3DCharacters.cpp — the foyer's characters (`Chars.txt`, `Crowd.txt`, `Dance.txt`)
//! and their idle pickers, plus the character element (`CElementCharacter`, element kinds 6 / 7, code in the
//! FEAlloyEditor_Rim.cpp-labelled range 0x00284A30..0x00285910, vtable 0x00659188).
//!
//! The data lives in the FE3DVehicles object (`dword_768C30`): chars at +0 (2916-byte records), crowd at +17496
//! and dancers at +48696 (2080-byte records), element pointers at +61176 / +61224 / +61284, counts at
//! +61316 / +61320 / +61324. Here the element state is kept with its record; the bevy side (`foyer.rs`) spawns
//! the skinned models through visos_render.
//!
//! The elements are created hidden (0x00260710 / 0x00260B60 / 0x00260E30, also by the main menu's Enter) and only
//! the hangout's Start (FEHangoutScreen.cpp 0x00219300) and 0x002456B0 show them (Leave 0x00219920 hides them), so
//! the main menu draws no characters in the original either.
//! Shipped data: hangout and hangCE list 5 chars and `CHARACTERS : (0)` crowd / dancers.
//! The previous-anim crossfade (+116 / +132 / +2564, `dword_6C1780` = 1 s) goes to graphics as
//! `SkinnedModelBlend` (`blend`, written by `foyer.rs`). Props (`Prop`, element pointers +61200): `create_props`
//! 0x002609D0 / `place_props` 0x0025F2A0 / `drop_props` 0x00261300; shipped: Showroom's "Powerbook".

use std::sync::Arc;

use visos::file::vs_file_directory::VsFileDirectoryDetails;
use visos::handy::vs_text_file::VsTextFile;
use visos_render::bone_anim::BoneAnim;

use crate::string_manager::string_manager::string_crc;

/// `off_685398` anim slot suffixes (chars: `<Texture>_<suffix>`)
pub const ANIM_SUFFIX: [&str; 7] = ["STAND", "EXIT", "SELECT", "SELECT2", "IDLE1", "IDLE2", "IDLE3"];
pub const ANIM_STAND: i32 = 0;
pub const ANIM_IDLE1: i32 = 4;
pub const ANIM_IDLE2: i32 = 5;
/// `MAX_CHARACTER_CAMERAS`
pub const MAX_CHARACTER_CAMERAS: usize = 10;
/// `dword_6C1780`: the anim cross-fade time in seconds
pub const CROSSFADE: f32 = f32::from_bits(0x3F80_0000);

/// a character camera (`FE3DCharacters__2624E0`): LOOKAT 0, MOVERIGHT 1, SELECT 2, EXIT 3, ENTRY 4
#[derive(Clone, Debug, Default)]
pub struct CharacterCamera {
    pub kind: u32,
    pub pos: [f32; 3],
    pub look_at: [f32; 3],
    pub axis: [f32; 3],
    pub timer: f32,
}

/// one loaded anim slot (element +364 instance / +392 resource)
#[derive(Clone)]
pub struct CharAnim {
    pub name: String,
    pub anim: Arc<BoneAnim>,
}

/// `CElementCharacter` (0xA10 bytes) state plus the record it was made from.
#[derive(Clone, Default)]
pub struct FECharacter {
    pub pos: [f32; 3],             // +0
    pub model: String,             // +16 (CHARS_M stem; the file's `Texture` line)
    pub rotation: f32,             // +272 radians
    /// +284 running index over chars, crowd, dancers = element +764 (the record is copied to element +480 by
    /// 0x00286180), the SpecularChars record of 0x00079EC0 a17
    pub index: u32,
    pub anim_names: [String; 7],   // +288, 256 bytes each
    pub element_kind: u32,         // +2904 7 if Type != 0 else 6
    pub name_id: u32,              // +2644 NameID crc
    pub prop: String,              // +2648
    pub volume: f32,               // +2908
    pub reverb: f32,               // +2912
    pub cameras: Vec<CharacterCamera>,
    // element
    pub anims: [Option<CharAnim>; 7], // +364
    pub current: i32,              // +120
    pub previous: i32,             // +116
    pub queued: i32,               // +124
    pub default: i32,              // +128
    pub last_time: f32,            // +108 (-1 until the first update)
    pub blend_start: f32,          // +132
    pub time: f32,                 // +2560 normalised
    pub prev_time: f32,            // +2564
    pub loop_cur: bool,            // +2568
    pub loop_prev: bool,           // +2569
    pub state: u32,                // +2572 2 anims requested, 3 ready
    pub created: bool,             // element exists
    pub visible: bool,
}

/// `\Chars\CHARS_A.vcd` anim by stem (case insensitive)
fn load_anim(name: &str) -> Option<CharAnim> {
    if name.is_empty() || name.eq_ignore_ascii_case("NONE") {
        return None;
    }
    let v = visos::file::Vcd::open("\\Chars\\CHARS_A.vcd").ok()?;
    let d = v.get(&format!("{name}.gaf"))?;
    let mut a = BoneAnim::parse_file(d).ok()?;
    if a.is_empty() {
        return None;
    }
    Some(CharAnim { name: name.to_string(), anim: Arc::new(a.swap_remove(0)) })
}

impl FECharacter {
    /// FEAlloyEditor_Rim.cpp 0x00284A30 — element Init
    fn init_element(&mut self) {
        self.last_time = 0.0;
        self.time = 0.0;
        self.prev_time = 0.0;
        self.queued = -1;
        self.current = -1;
        self.previous = -1;
        self.default = -1;
        self.loop_cur = false;
        self.loop_prev = false;
        self.state = 0;
        self.anims = Default::default();
        self.created = true;
    }

    /// FEAlloyEditor_Rim.cpp 0x00284E60 + 0x00284FB0 — load slot 0, then the others (`NONE` skipped); state 2.
    /// Loads are synchronous here, so the element is ready (state 3) at once.
    pub fn load_anims(&mut self) {
        self.last_time = -1.0;
        self.anims[0] = load_anim(&self.anim_names[0]);
        self.current = 0;
        self.queued = -1;
        for i in 1..7 {
            self.anims[i] = load_anim(&self.anim_names[i]);
        }
        self.state = 2;
        self.state = 3;
    }

    /// FEAlloyEditor_Rim.cpp 0x002856F0 — `SetDefaultAnim`
    pub fn set_default_anim(&mut self, a: i32) {
        self.current = a;
        self.default = a;
        self.time = 0.0;
        self.queued = -1;
    }

    /// FEAlloyEditor_Rim.cpp 0x00285770 — `SetAnim(index, unused, loop)`
    pub fn set_anim(&mut self, a: i32, looping: bool) {
        if self.anims.get(a as usize).is_some_and(|x| x.is_some()) && self.current != a {
            self.prev_time = self.time;
            self.time = 0.0;
            self.loop_prev = self.loop_cur;
            self.loop_cur = looping;
            self.previous = self.current;
            self.current = a;
        }
    }

    /// FEAlloyEditor_Rim.cpp 0x00285800 — `QueueAnim`
    pub fn queue_anim(&mut self, a: i32) {
        self.queued = a;
    }

    /// FEAlloyEditor_Rim.cpp 0x00285910 — current anim index
    pub fn current_anim(&self) -> i32 {
        self.current
    }

    /// FEAlloyEditor_Rim.cpp 0x002858C0 — frame count of the current anim
    pub fn current_frames(&self) -> i32 {
        self.anims.get(self.current as usize).and_then(|a| a.as_ref()).map_or(0, |a| a.anim.num_frames as i32)
    }

    /// name of the anim on show
    pub fn current_anim_name(&self) -> Option<&str> {
        self.anims.get(self.current as usize).and_then(|a| a.as_ref()).map(|a| a.name.as_str())
    }

    /// FEAlloyEditor_Rim.cpp 0x00285350 (after the step) — the cross-fade passed to Characters 0x00079EC0: weight 1,
    /// or `(now - +132) / dword_6C1780` (1.0 s) while that is below 1 and the previous anim (+116) is loaded; then
    /// the previous anim and its time +2564. `now` = the last update's time (+108).
    pub fn blend(&self) -> (Option<(String, f32)>, f32) {
        let prev = self.anims.get(self.previous as usize).filter(|_| self.previous >= 0).and_then(|a| a.as_ref());
        let d = self.last_time - self.blend_start;
        match prev {
            Some(p) if CROSSFADE > d => (Some((p.name.clone(), self.prev_time)), d / CROSSFADE),
            _ => (None, 1.0),
        }
    }

    /// FEAlloyEditor_Rim.cpp 0x00285350 — `Update(now)`: advance the current anim; on wrap play the queued anim,
    /// else fall back to the default one.
    pub fn update(&mut self, now: f32) {
        let Some(cur) = self.anims.get(self.current as usize).and_then(|a| a.clone()) else { return };
        let dt = now - self.last_time;
        self.last_time = now;
        let inc = cur.anim.frame_inc(dt);
        self.time += inc;
        if self.time >= 1.0 {
            self.time -= self.time.floor();
            if self.queued == -1 {
                if self.current != self.default {
                    let d = self.default;
                    self.set_anim(d, true);
                }
            } else {
                let q = self.queued;
                self.set_anim(q, false);
                self.queued = -1;
                self.blend_start = now;
            }
        }
        // previous anim (+2564): advanced while the crossfade runs (`blend`)
        if self.previous >= 0 && now - self.blend_start < CROSSFADE {
            if let Some(p) = self.anims.get(self.previous as usize).and_then(|a| a.as_ref()) {
                self.prev_time += p.anim.frame_inc(dt);
                if self.prev_time >= 1.0 {
                    self.prev_time = if self.loop_prev { self.prev_time - self.prev_time.floor() } else { 1.0 };
                }
            }
        }
    }
}

/// the character part of the FE3DVehicles object (`dword_768C30`)
#[derive(Clone, Default)]
pub struct FE3DCharacters {
    pub foyer: String,             // +61356
    pub chars: Vec<FECharacter>,   // +0, count +61316
    pub crowd: Vec<FECharacter>,   // +17496, count +61320
    pub dance: Vec<FECharacter>,   // +48696, count +61324
    pub loaded: bool,              // +61312
    pub created: bool,             // +61313
    pub char_timer: i32,           // +61328
    pub char_last: i32,            // +61332
    pub crowd_timer: i32,          // +61336
    pub crowd_last: i32,           // +61340
    pub crowd_partner: i32,        // +61344
    pub dance_timer: i32,          // +61348
    pub dance_current: i32,        // +61352
    /// +61200: one "CharacterProp" 3D element per char (kind 3, model `<Prop>` of PROPSM.VCD, textures PROPST.VCD)
    pub props: Vec<Option<FEProp>>,
}

/// a character's prop element (`FECK2Element3D`, desc crc "CharacterProp", position (0, 0) until `place_props`)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FEProp {
    pub model: String,
    pub pos: [f32; 3],
    /// radians (record +272), D3DXMatrixRotationY then the translation (0x0025F2A0)
    pub rotation: f32,
    pub visible: bool,
}

/// the props' model / texture packs (0x002609D0 desc +64 / +68)
pub const PROPS_MODEL_VCD: &str = "\\Foyers\\Props\\Model\\PROPSM.VCD";
pub const PROPS_TEXTURE_VCD: &str = "\\Foyers\\Props\\Model\\PROPST.VCD";

fn open_foyer(foyer: &str) -> Option<VsFileDirectoryDetails> {
    VsFileDirectoryDetails::open(&format!("\\Foyers\\{foyer}")).ok()
}

fn f3(t: &mut VsTextFile) -> [f32; 3] {
    t.read_f32n::<3>().map_or([0.0; 3], |(_, v)| v)
}
fn f1(t: &mut VsTextFile) -> f32 {
    t.read_f32().map_or(0.0, |(_, v)| v)
}
fn s1(t: &mut VsTextFile) -> String {
    t.read_string().unwrap_or_default()
}

impl FE3DCharacters {
    /// FE3DCharacters.cpp 0x0025F440 — `Chars.txt`
    pub fn load_chars(&mut self) {
        self.chars.clear();
        let Some(dir) = open_foyer(&self.foyer) else { return };
        let mut t = VsTextFile::default();
        if !t.load(Some(&dir), "Chars.txt") {
            return;
        }
        let count = f1(&mut t);
        let mut i = 0;
        while (i as f64) < count as f64 {
            let mut c = FECharacter::default();
            let _name = s1(&mut t);
            c.model = s1(&mut t);
            c.pos = f3(&mut t);
            let rot = f1(&mut t);
            let kind = t.read_s32().map_or(0, |(_, v)| v);
            let _info_location = f1(&mut t);
            let name_id = s1(&mut t);
            c.volume = f1(&mut t);
            c.reverb = f1(&mut t);
            let ncam = f1(&mut t);
            debug_assert!(ncam <= MAX_CHARACTER_CAMERAS as f32, "noOfCameras<=MAX_CHARACTER_CAMERAS");
            debug_assert!(ncam > 0.0, "noOfCameras>0");
            let mut k = 0;
            while (k as f64) < ncam as f64 {
                let ty = s1(&mut t);
                let mut cam = CharacterCamera { pos: f3(&mut t), look_at: f3(&mut t), axis: f3(&mut t), timer: f1(&mut t), kind: 0 };
                cam.kind = match ty.as_str() {
                    "LOOKAT" => 0,
                    "MOVERIGHT" => 1,
                    "SELECT" => 2,
                    "EXIT" => 3,
                    "ENTRY" => 4,
                    _ => 0,
                };
                c.cameras.push(cam);
                k += 1;
            }
            c.prop = s1(&mut t);
            c.name_id = string_crc(&name_id);
            c.rotation = rot.to_radians();
            c.element_kind = if kind != 0 { 7 } else { 6 };
            for (j, s) in ANIM_SUFFIX.iter().enumerate() {
                c.anim_names[j] = format!("{}_{}", c.model, s);
            }
            c.index = i;
            self.chars.push(c);
            i += 1;
        }
    }

    /// FE3DCharacters.cpp 0x0025FD60 / 0x00260250 — `Crowd.txt` (stand + idle 1..3 into slots 0, 4, 5, 6) and
    /// `Dance.txt` (stand + idle 1..2 into slots 0, 4, 5).
    /// `base` = the counts read at load time (+61316, + +61320 for dancers) for the +284 index
    fn load_extras(&mut self, file: &str, slots: &[usize], base: u32) -> Vec<FECharacter> {
        let mut out = Vec::new();
        let Some(dir) = open_foyer(&self.foyer) else { return out };
        let mut t = VsTextFile::default();
        if !t.load(Some(&dir), file) {
            log::warn!("{file} not loaded");
            return out;
        }
        let count = f1(&mut t);
        let mut i = 0;
        while (i as f64) < count as f64 {
            let mut c = FECharacter::default();
            let _name = s1(&mut t);
            c.model = s1(&mut t);
            c.pos = f3(&mut t);
            c.rotation = f1(&mut t).to_radians();
            c.element_kind = 6;
            for &s in slots {
                c.anim_names[s] = s1(&mut t);
            }
            c.index = base + i;
            out.push(c);
            i += 1;
        }
        out
    }

    pub fn load_crowd(&mut self) {
        let base = self.chars.len() as u32;
        self.crowd = self.load_extras("Crowd.txt", &[0, 4, 5, 6], base);
    }

    pub fn load_dance(&mut self) {
        let base = (self.chars.len() + self.crowd.len()) as u32;
        self.dance = self.load_extras("Dance.txt", &[0, 4, 5], base);
    }

    /// FE3DCharacters.cpp 0x00260710 / 0x00260B60 / 0x00260E30 — create the elements hidden, default anim 0.
    pub fn create_elements(&mut self) {
        for c in self.chars.iter_mut().chain(self.crowd.iter_mut()).chain(self.dance.iter_mut()) {
            c.init_element();
            c.visible = false;
            c.set_default_anim(0);
        }
        self.char_timer = 600;
        self.crowd_timer = 400;
        self.dance_current = 0;
        self.dance_timer = 200;
        self.created = true;
    }

    /// FE3DCharacters.cpp 0x00261510 / 0x002615F0 / 0x002616A0 — show / hide; showing loads the anims of an
    /// element that is not ready.
    pub fn set_visible(&mut self, show: bool) {
        for c in self.chars.iter_mut().chain(self.crowd.iter_mut()).chain(self.dance.iter_mut()) {
            c.visible = show;
            if show && c.state != 3 {
                c.load_anims();
            }
        }
        // 0x00261510: the chars' prop elements too
        for p in self.props.iter_mut().flatten() {
            p.visible = show;
        }
    }

    /// FE3DCharacters.cpp 0x002609D0 — a "CharacterProp" element for every char without one whose `Prop` is not
    /// "NONE" (stricmp); the element starts at (0, 0) and visible, `place_props` moves it
    pub fn create_props(&mut self) {
        self.props.resize(self.chars.len(), None);
        for (i, c) in self.chars.iter().enumerate() {
            if self.props[i].is_none() && !c.prop.eq_ignore_ascii_case("NONE") {
                self.props[i] = Some(FEProp { model: c.prop.clone(), pos: [0.0; 3], rotation: 0.0, visible: true });
            }
        }
    }

    /// FE3DVehicles.cpp 0x0025F2A0 (FE3DCharacters' prop part): each prop element (slots 0..5) gets its char's
    /// world matrix = RotationY(record +272) with the record position as translation
    pub fn place_props(&mut self) {
        for i in 0..6.min(self.props.len()) {
            if let (Some(p), Some(c)) = (self.props[i].as_mut(), self.chars.get(i)) {
                p.pos = c.pos;
                p.rotation = c.rotation;
            }
        }
    }

    /// FE3DCharacters.cpp 0x00261300 — forget the prop elements (the original leaks them hidden; the next Enter
    /// makes new ones)
    pub fn drop_props(&mut self) {
        for p in self.props.iter_mut() {
            *p = None;
        }
    }

    /// FE3DCharacters.cpp 0x00261220 / 0x00261350 / 0x00261430 — delete the elements
    pub fn delete_elements(&mut self) {
        for c in self.chars.iter_mut().chain(self.crowd.iter_mut()).chain(self.dance.iter_mut()) {
            c.created = false;
            c.visible = false;
        }
        self.created = false;
    }

    /// FE3DCharacters.cpp 0x00261750 — every so often a random char plays IDLE1 / IDLE2 once.
    fn update_chars(&mut self, rand: &mut visos::handy::crt_rand::CrtRand) {
        self.char_timer -= 1;
        if self.char_timer > 0 || self.chars.is_empty() {
            return;
        }
        let prev = self.char_last;
        let r = rand.rand_unit() as f32;
        self.char_last = (self.chars.len() as f64 * r as f64) as i32;
        if self.char_last == prev && self.chars.len() != 1 {
            return;
        }
        let i = self.char_last as usize;
        if !self.chars[i].created || self.chars[i].state != 3 {
            return;
        }
        let r = rand.rand_unit() as f32;
        let a = if r >= 0.5 { ANIM_IDLE2 } else { ANIM_IDLE1 };
        self.chars[i].queue_anim(a);
        // the girl talking to the editor (3rd crowd member) follows him
        if self.chars[i].name_id == string_crc("HANGOUT_EDITOR") {
            if let Some(g) = self.crowd.get_mut(2) {
                g.queue_anim(a);
            }
        }
        let r = rand.rand_unit() as f32;
        self.char_timer = (r * 20.0 + 300.0) as i32;
        if self.chars.len() == 1 {
            self.char_timer = 360;
        }
    }

    /// FE3DCharacters.cpp 0x002619B0 — crowd idles; 0 / 3 bring their partner 1 / 4 along (1, 2, 4 never start).
    fn update_crowd(&mut self, rand: &mut visos::handy::crt_rand::CrtRand) {
        self.crowd_timer -= 1;
        if self.crowd_timer > 0 || self.crowd.is_empty() {
            return;
        }
        let prev = self.crowd_last;
        let r = rand.rand_unit() as f32;
        self.crowd_last = (self.crowd.len() as f64 * r as f64) as i32;
        let l = self.crowd_last;
        if l == prev || l == self.crowd_partner || l == 1 || l == 4 || l == 2 {
            return;
        }
        let Some(c) = self.crowd.get_mut(l as usize) else { return };
        if !c.created || c.state != 3 {
            return;
        }
        let a = if r >= 0.5 { ANIM_IDLE2 } else { ANIM_IDLE1 };
        c.queue_anim(a);
        let mut other = None;
        if l == 0 {
            self.crowd_partner = 1;
            other = Some(1);
        } else if l == 3 {
            self.crowd_partner = 4;
            other = Some(4);
        }
        match other.and_then(|o| self.crowd.get_mut(o)) {
            Some(o) => o.queue_anim(a),
            None => self.crowd_partner = -1,
        }
        let r = rand.rand_unit() as f32;
        self.crowd_timer = (r * 20.0 + 100.0) as i32;
    }

    /// FE3DCharacters.cpp 0x00261C70 — the dancers take turns once the current one is back on its stand anim.
    fn update_dance(&mut self, rand: &mut visos::handy::crt_rand::CrtRand) {
        let cur = self.dance_current as usize;
        if self.dance.get(cur).is_none_or(|d| !d.created) {
            return;
        }
        if self.dance_timer > 0 {
            self.dance_timer -= 1;
            return;
        }
        if self.dance[cur].current_anim() != 0 {
            return;
        }
        self.dance_current += 1;
        if self.dance_current as usize >= self.dance.len() {
            self.dance_current = 0;
        }
        let i = self.dance_current as usize;
        if !self.dance[i].created || self.dance[i].state != 3 {
            return;
        }
        let r = rand.rand_unit() as f32;
        let _length = match i {
            0 | 2 => {
                if r >= 0.5 {
                    260.0
                } else {
                    520.0
                }
            }
            1 => 720.0,
            3 => 500.0,
            _ => 0.0f32,
        };
        self.dance[i].set_anim(if r >= 0.5 { ANIM_IDLE2 } else { ANIM_IDLE1 }, true);
        // the length table is overwritten at once by the anim's frame count
        self.dance_timer = self.dance[i].current_frames() + 60;
    }

    /// FE3DCamera.cpp 0x00271290 / FE3DCharacters.cpp 0x002620C0 — (character part) + the per-render element updates.
    pub fn update(&mut self, now: f32, rand: &mut visos::handy::crt_rand::CrtRand) {
        if !self.created {
            return;
        }
        self.update_chars(rand);
        self.update_crowd(rand);
        self.update_dance(rand);
        for c in self.chars.iter_mut().chain(self.crowd.iter_mut()).chain(self.dance.iter_mut()) {
            if c.created && c.visible {
                c.update(now);
            }
        }
    }

    /// every element: (group 0 chars / 1 crowd / 2 dance, index, record)
    pub fn all(&self) -> impl Iterator<Item = (u32, usize, &FECharacter)> {
        self.chars
            .iter()
            .enumerate()
            .map(|(i, c)| (0, i, c))
            .chain(self.crowd.iter().enumerate().map(|(i, c)| (1, i, c)))
            .chain(self.dance.iter().enumerate().map(|(i, c)| (2, i, c)))
    }
}

#[cfg(test)]
mod tests {
    /// 0x002609D0 / 0x0025F2A0 / 0x00261510 / 0x00261300: the showroom's Powerbook is the one shipped prop
    #[test]
    fn hangout_props() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut c = super::FE3DCharacters { foyer: "hangout".into(), ..Default::default() };
        c.load_chars();
        c.create_elements();
        c.create_props();
        c.place_props();
        let names: Vec<_> = c.props.iter().map(|p| p.as_ref().map(|p| p.model.as_str())).collect();
        assert_eq!(names, [None, None, Some("Powerbook"), None, None]);
        let p = c.props[2].clone().unwrap();
        assert_eq!((p.pos, p.rotation), (c.chars[2].pos, c.chars[2].rotation));
        let v = visos::file::Vcd::open(super::PROPS_MODEL_VCD).unwrap();
        assert!(v.find("Powerbook.xmf").is_some());
        c.set_visible(false);
        assert!(!c.props[2].as_ref().unwrap().visible);
        c.set_visible(true);
        assert!(c.props[2].as_ref().unwrap().visible);
        c.drop_props();
        assert!(c.props.iter().all(Option::is_none));
    }

    #[test]
    fn hangout_chars() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut c = super::FE3DCharacters { foyer: "hangout".into(), ..Default::default() };
        c.load_chars();
        c.load_crowd();
        c.load_dance();
        assert_eq!(c.chars.len(), 5);
        assert_eq!(c.chars[0].model, "Tory");
        assert_eq!(c.chars[0].cameras.len(), 5);
        assert_eq!(c.chars[4].anim_names[0], "DJ_STAND");
        assert!(c.crowd.is_empty() && c.dance.is_empty());
        // +284 / element +764 = the SpecularChars record (0x0025F440 writes the loop index)
        assert_eq!(c.chars.iter().map(|x| x.index).collect::<Vec<_>>(), [0, 1, 2, 3, 4]);
        c.create_elements();
        c.set_visible(true);
        assert!(c.chars.iter().all(|x| x.anims[0].is_some()), "stand anims");
        let mut r = visos::handy::crt_rand::CrtRand::default();
        for f in 0..2000 {
            c.update(f as f32 / 60.0, &mut r);
        }
    }

    /// 0x00285350: a queued anim starts on the wrap and fades over the previous one for 1 s
    #[test]
    fn crossfade_after_queued_anim() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let mut c = super::FE3DCharacters { foyer: "hangout".into(), ..Default::default() };
        c.load_chars();
        c.create_elements();
        c.set_visible(true);
        let ch = &mut c.chars[0];
        let q = (1..7).find(|&i| ch.anims[i].is_some()).expect("a second anim") as i32;
        ch.set_default_anim(0);
        ch.queue_anim(q);
        assert_eq!(ch.blend(), (None, 1.0));
        let mut t = 0.0;
        while ch.current != q {
            t += 1.0 / 60.0;
            ch.update(t);
            assert!(t < 120.0);
        }
        let start = t;
        let (prev, w) = ch.blend();
        assert_eq!(prev.as_ref().map(|p| p.0.as_str()), ch.anims[0].as_ref().map(|a| a.name.as_str()));
        assert_eq!(w, 0.0);
        ch.update(start + 0.5);
        assert!((ch.blend().1 - 0.5).abs() < 1e-4);
        ch.update(start + 1.0);
        assert_eq!(ch.blend(), (None, 1.0));
    }
}
