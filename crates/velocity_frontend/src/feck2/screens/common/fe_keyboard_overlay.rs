//! src\Feck2\Screens\Common\FEKeyboardOverlay.cpp — `CFEKeyboardOverlay` (`gpCommonKeyboard`, `dword_769184`, 0x74
//! bytes; created by FE3DCharacters.cpp 0x00267FC0, code 0x00268850..0x0026F7A0): the on-screen keyboard that names
//! profiles. Three layers outside any screen (backing, keys, top) that `CScreen::ProcessEvent` (0x00283D5F) and
//! `CScreen::Render` (0x00283F98) run while it is shown; 5 rows of keys (letters / symbols / accents), shift, caps
//! lock, cursor keys, space, backspace and DONE (only once the name is valid).
//!
//! Kept from the original: the special keys keep the label they were made with when the layout changes (the
//! symbol page's ALPHABET key still reads "SYMBOLS"), codes above 255 (€, Ÿ) type a space, and the input string's
//! "first key clears" only applies with `+68` set.

use std::sync::{Arc, Mutex};

use crate::feck2::feck2_manager::{visos_colour, Feck2, COLOUR_BLACK, COLOUR_WHITE};
use crate::feck2::feck2_object::*;
use crate::render2d::{Colour, TextureId};
use crate::string_manager::string_manager::string_crc;
use velocity_audio::GuiSound;

/// special key codes (`>= 0x10000`)
pub const KEY_BACKSPACE: i32 = 0x10000;
pub const KEY_SHIFT: i32 = 0x10002;
pub const KEY_CAPS: i32 = 0x10003;
pub const KEY_ALPHABET: i32 = 0x10004;
pub const KEY_SYMBOLS: i32 = 0x10005;
pub const KEY_ACCENTS: i32 = 0x10006;
pub const KEY_DONE: i32 = 0x10007;
pub const KEY_LEFT: i32 = 0x10008;
pub const KEY_RIGHT: i32 = 0x10009;
/// `MAX_VIRTUAL_KEYBOARD_CHARACTERS`
pub const MAX_CHARACTERS: usize = 0x80;

/// key colours (static init 0x0042F470..0x0042F5D0, `VISOS_Colour(a, r, g, b)`)
fn key_colour() -> Colour {
    visos_colour(0x80, 0x6E, 0x6E, 0x6E) // dword_769188
}
fn disabled_colour() -> Colour {
    visos_colour(0x80, 0x19, 0x19, 0x19) // dword_76918C
}
fn selected_colour() -> Colour {
    visos_colour(0x80, 0x5C, 0x6D, 0x7F) // dword_7691A0
}
fn selected_other_colour() -> Colour {
    visos_colour(0x80, 0x47, 0x55, 0x63) // dword_76919C
}
fn hidden_colour() -> Colour {
    visos_colour(0, 0, 0, 0) // dword_7691A4
}
fn input_colour() -> Colour {
    visos_colour(0x80, 0x80, 0x66, 0) // dword_7691A8 / dword_769194
}
fn disabled_text_colour() -> Colour {
    visos_colour(0x40, 0, 0, 0) // dword_7691AC
}
fn pressed_colour() -> Colour {
    visos_colour(0x80, 0x43, 0x43, 0x43) // dword_7691B0
}
/// `flt_769190` = 19 * 0.75 (static init 0x0042F450): the cursor shows for this part of every 19 game updates
const CURSOR_ON: f32 = 19.0 * 0.75;

/// the three layouts' key codes per row (0x0026D9E0 letters, 0x0026DDE0 symbols, 0x0026E1F0 accents); the row's
/// remaining keys keep their code (row 3's backspace)
const LAYOUTS: [[&[i32]; 5]; 3] = [
    [
        &[KEY_ACCENTS, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48],
        &[KEY_CAPS, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74],
        &[KEY_SHIFT, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84],
        &[KEY_SYMBOLS, 85, 86, 87, 88, 89, 90],
        &[KEY_LEFT, KEY_RIGHT, 32, KEY_DONE],
    ],
    [
        &[KEY_ACCENTS, 40, 41, 38, 95, 94, 37, 92, 47, 64, 35],
        &[KEY_CAPS, 91, 93, 36, 163, 165, 8364, 59, 58, 39, 34],
        &[KEY_SHIFT, 60, 62, 63, 33, 191, 161, 45, 42, 43, 61],
        &[KEY_ALPHABET, 123, 125, 171, 187, 44, 46],
        &[KEY_LEFT, KEY_RIGHT, 32, KEY_DONE],
    ],
    [
        &[KEY_ACCENTS, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48],
        &[KEY_CAPS, 192, 193, 194, 196, 199, 200, 201, 202, 203, 204],
        &[KEY_SHIFT, 205, 206, 207, 209, 210, 211, 212, 213, 214, 223],
        &[KEY_ALPHABET, 217, 218, 219, 220, 221, 376],
        &[KEY_LEFT, KEY_RIGHT, 32, KEY_DONE],
    ],
];

/// one key (0x14 bytes, made by 0x00268280)
#[derive(Clone, Debug, Default)]
pub struct Key {
    pub code: i32,                // +0
    pub string: u32,              // +4 label string (special keys), 0 = the character
    pub sprite: Option<ElementId>, // +8
    pub text: Option<ElementId>,  // +12
    pub width: u16,               // +16
    pub ch: u8,                   // +18
}

impl Key {
    /// FE3DCharacters.cpp 0x00268280 — a key and the label of the special ones
    fn new(code: i32, width: u16) -> Self {
        let string = match code {
            KEY_ACCENTS => 1564933370,
            KEY_DONE => string_crc("VKEYBOARD_DONE"),
            0x10008 | 0x10009 => string_crc("GENERAL_BLANK"),
            KEY_SYMBOLS => string_crc("VKEYBOARD_SYMBOLS"),
            KEY_CAPS => string_crc("VKEYBOARD_CAPSLOCK"),
            KEY_ALPHABET => string_crc("VKEYBOARD_ALPHABET"),
            KEY_SHIFT => string_crc("VKEYBOARD_SHIFT"),
            32 => string_crc("VKEYBOARD_SPACE"),
            KEY_BACKSPACE => string_crc("VKEYBOARD_BACKSPACE"),
            _ => 0,
        };
        Self { code, string, sprite: None, text: None, width, ch: 0 }
    }
}

/// the name check a screen hands in (`+0`, e.g. FEProfileSelectScreen 0x00234BF0)
pub type Validator = fn(&[u8]) -> bool;

/// `CFEKeyboardOverlay`
pub struct CFEKeyboardOverlay {
    pub validator: Option<Validator>, // +0
    pub shown: bool,                  // +4 rendered
    pub active: bool,                 // +5 gets the events
    pub bg_layer: LayerId,            // +12
    pub keyboard_layer: LayerId,      // +16
    pub top_layer: LayerId,           // +20
    pub backing: ElementId,           // +24
    pub dummy_key: ElementId,         // +28
    pub dummy_key_pressed: ElementId, // +32
    pub input_string: ElementId,      // +36
    pub dummy_string: ElementId,      // +40
    pub cursor: ElementId,            // +44
    pub arrows: [ElementId; 2],       // +48
    pub accents: bool,                // +56 the accents page may be used
    pub x: f32,                       // +60
    pub y: f32,                       // +64
    pub clear_on_first: bool,         // +68
    pub first_key: bool,              // +69
    pub hidden: bool,                 // +70
    pub done: bool,                   // +71
    pub caps: bool,                   // +72
    pub shift: bool,                  // +73
    pub case_locked: bool,            // +74
    pub keys: Vec<Vec<Key>>,          // +76
    pub mode: u8,                     // +80 0 letters, 1 symbols, 2 accents
    pub row: u8,                      // +81
    pub col: u8,                      // +82
    pub last_col: u8,                 // +83
    pub pos: u8,                      // +84 cursor in the string
    pub string: Vec<u8>,              // +88
    pub max_size: usize,              // +92
    pub repeat_time: f32,             // +96
    pub repeating: bool,              // +100
    pub dt: f32,                      // +104
    pub key_texture: TextureId,       // +108
    pub pressed_texture: TextureId,   // +112
}

/// `gpCommonKeyboard` on the frontend: the object and what `CScreen` reads every event / frame
#[derive(Clone)]
pub struct KeyboardHook {
    pub obj: Arc<Mutex<CFEKeyboardOverlay>>,
    pub layers: [LayerId; 3], // bg, keys, top
    pub shown: bool,          // +4
    pub active: bool,         // +5
}

/// run `f` on the keyboard (`dword_769184 != 0`)
pub fn kb<R>(fe: &mut Feck2, f: impl FnOnce(&mut CFEKeyboardOverlay, &mut Feck2) -> R) -> Option<R> {
    let obj = fe.keyboard.as_ref()?.obj.clone();
    let mut k = obj.lock().unwrap();
    let r = f(&mut k, fe);
    if let Some(h) = fe.keyboard.as_mut() {
        h.shown = k.shown;
        h.active = k.active;
    }
    Some(r)
}

/// FE3DCharacters.cpp 0x00267FC0 / FEKeyboardOverlay.cpp 0x00269990 / 0x0026A1F0 / 0x0026A260 / 0x0026A670 / 0x0026A6C0 / 0x0026A710 / 0x0026A760 / 0x0026A850 / 0x0026A880 — `new CFEKeyboardOverlay(0, 16, 1, 0, 112, 158)` (ctor 0x00268560) and the top layer's functors
pub fn create_common_keyboard(fe: &mut Feck2) {
    let k = CFEKeyboardOverlay::new(fe, 16, true, false, f32::from_bits(1120927744), f32::from_bits(1126039552));
    let layers = [k.bg_layer, k.keyboard_layer, k.top_layer];
    let obj = Arc::new(Mutex::new(k));
    // 0x00269990 — the top layer's event functors
    let top = layers[2];
    let m = |fe: &mut Feck2, f: fn(&mut CFEKeyboardOverlay, &mut Feck2, &CEvent) -> bool, e: u32| {
        fe.map(top, functor(&obj, f), e);
    };
    m(fe, |k, fe, _| {
        k.reset(fe);
        k.load(fe);
        k.refresh(fe);
        true
    }, ev::ENTER); // 0x0026A1F0
    m(fe, |k, fe, e| {
        // 0x0026A260
        k.dt = f32::from_bits(e.param_a);
        if !k.blocked() {
            k.blink_cursor(fe);
        }
        true
    }, ev::UPDATE);
    // up 0 / down 1 / left 2 / right 3: press 0x0026C150, held 0x0026C220, release 0x0026C2F0
    m(fe, |k, fe, _| k.press_dir(fe, 0), ev::UP_PRESS);
    m(fe, |k, fe, _| k.held_dir(fe, 0), ev::UP_HELD);
    m(fe, |k, _, _| k.release_dir(), ev::UP_RELEASE);
    m(fe, |k, fe, _| k.press_dir(fe, 1), ev::DOWN_PRESS);
    m(fe, |k, fe, _| k.held_dir(fe, 1), ev::DOWN_HELD);
    m(fe, |k, _, _| k.release_dir(), ev::DOWN_RELEASE);
    m(fe, |k, fe, _| k.press_dir(fe, 2), ev::LEFT_PRESS);
    m(fe, |k, fe, _| k.held_dir(fe, 2), ev::LEFT_HELD);
    m(fe, |k, _, _| k.release_dir(), ev::LEFT_RELEASE);
    m(fe, |k, fe, _| k.press_dir(fe, 3), ev::RIGHT_PRESS);
    m(fe, |k, fe, _| k.held_dir(fe, 3), ev::RIGHT_HELD);
    m(fe, |k, _, _| k.release_dir(), ev::RIGHT_RELEASE);
    m(fe, |k, fe, _| {
        // 0x0026A670
        if k.blocked() {
            return true;
        }
        k.press_selected(fe);
        fe.sounds.push(GuiSound::Accept);
        false
    }, ev::SELECT_PRESS);
    // 0x0026A6C0 pause = DONE, 0x0026A710 X = backspace, 0x0026A760 Y = space, L1 / R1 = cursor left / right
    m(fe, |k, fe, _| k.press_if_free(fe, KEY_DONE), ev::PAUSE_PRESS);
    m(fe, |k, fe, _| k.press_if_free(fe, KEY_BACKSPACE), ev::OPTION1_PRESS);
    m(fe, |k, fe, _| k.press_if_free(fe, 32), ev::OPTION2_PRESS);
    m(fe, |k, fe, _| k.press_if_free(fe, KEY_LEFT), ev::L1_PRESS);
    m(fe, |k, fe, _| k.press_if_free(fe, KEY_RIGHT), ev::R1_PRESS);
    m(fe, |k, _, _| k.blocked(), ev::ONLINE_PRESS); // 0x0026A850
    m(fe, |k, fe, _| {
        // 0x0026A880 R2: next page (accents only when allowed)
        if k.blocked() {
            return true;
        }
        let key = if k.accents {
            match k.mode {
                0 => KEY_SYMBOLS,
                1 => KEY_ACCENTS,
                2 => KEY_ALPHABET,
                _ => return false,
            }
        } else if k.mode == 1 {
            KEY_ALPHABET
        } else {
            KEY_SYMBOLS
        };
        k.press_key(fe, key, true);
        false
    }, ev::R2_PRESS);
    fe.keyboard = Some(KeyboardHook { obj, layers, shown: false, active: false });
}

impl CFEKeyboardOverlay {
    /// FE3DCharacters.cpp 0x00268560 — ctor: elements (0x00268B50), key rows (0x0026C920), key elements
    /// (0x00269580), reset (0x002686E0)
    fn new(fe: &mut Feck2, max_size: usize, clear_on_first: bool, accents: bool, x: f32, y: f32) -> Self {
        let mut k = Self {
            validator: None,
            shown: false,
            active: false,
            bg_layer: 0,
            keyboard_layer: 0,
            top_layer: 0,
            backing: 0,
            dummy_key: 0,
            dummy_key_pressed: 0,
            input_string: 0,
            dummy_string: 0,
            cursor: 0,
            arrows: [0; 2],
            accents,
            x,
            y,
            clear_on_first,
            first_key: true,
            hidden: false,
            done: false,
            caps: false,
            shift: true,
            case_locked: false,
            keys: Vec::new(),
            mode: 0,
            row: 0,
            col: 0,
            last_col: 0,
            pos: 0,
            string: Vec::new(),
            max_size,
            repeat_time: 0.0,
            repeating: false,
            dt: 0.0,
            key_texture: 0,
            pressed_texture: 0,
        };
        k.create_elements(fe);
        k.create_keys();
        k.create_key_elements(fe);
        k.reset(fe);
        k
    }

    /// FEKeyboardOverlay.cpp 0x00268B50 — layers and the fixed elements
    fn create_elements(&mut self, fe: &mut Feck2) {
        let mut ld = SLayerDesc { priority: 1, crc: string_crc("VirtualKeyboard-Top"), ..Default::default() };
        self.top_layer = fe.create_layer(None, &ld);
        ld.priority = 5;
        ld.crc = string_crc("VirtualKeyboard-BG");
        self.bg_layer = fe.create_layer(None, &ld);
        let mut d2 = SElementDesc { kind: ELEMENT_2D, crc: string_crc("DummyKey"), ..Default::default() };
        d2.position[0] = 0.0;
        d2.position[1] = 0.0;
        d2.texture = "VKeyboardCharacters".into();
        d2.vcd = "\\FE\\FE.vcd".into();
        self.dummy_key = fe.create_element(self.bg_layer, &d2);
        fe.el(self.dummy_key).set_visible(false);
        d2.kind = ELEMENT_2D;
        d2.crc = string_crc("DummyKeyPressed");
        d2.texture = "VKeyboardCharactersDepressed".into();
        self.dummy_key_pressed = fe.create_element(self.bg_layer, &d2);
        fe.el(self.dummy_key_pressed).set_visible(false);
        let mut dt = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 2, crc: string_crc("InputString"), ..Default::default() };
        dt.position[0] = self.x + 87.0;
        dt.position[1] = self.y + 158.0;
        dt.string = string_crc("GENERAL_BLANK");
        self.input_string = fe.create_element(self.top_layer, &dt);
        fe.text(self.input_string).set_colour(input_colour());
        fe.text(self.input_string).shadow = true; // 0x00243D70
        dt.crc = string_crc("DummyString");
        self.dummy_string = fe.create_element(self.top_layer, &dt);
        fe.el(self.dummy_string).set_visible(false);
        dt.kind = ELEMENT_STATIC_TEXT;
        dt.font = 2;
        dt.crc = string_crc("Cursor");
        dt.position[0] = self.x + 87.0;
        dt.position[1] = (self.y + 158.0) + 4.0;
        dt.string = string_crc("GENERAL_BLANK");
        self.cursor = fe.create_element(self.top_layer, &dt);
        fe.text(self.cursor).set_text(&[b'_' as u16]);
        fe.text(self.cursor).set_colour(input_colour());
        fe.text(self.cursor).shadow = true;
        let mut ds = SElementDesc { kind: ELEMENT_2D, crc: string_crc("MenuIndicatorSprite"), ..Default::default() };
        ds.position[0] = self.x + 4.0;
        ds.position[1] = self.y + 116.0;
        ds.texture = "VKeyboardArrow01".into();
        ds.vcd = "\\FE\\FE.vcd".into();
        self.arrows[0] = fe.create_element(self.top_layer, &ds);
        fe.e2d(self.arrows[0]).set_size(16, 16);
        fe.e2d(self.arrows[0]).set_flags(1);
        ds.kind = ELEMENT_2D;
        ds.crc = string_crc("MenuIndicatorSprite");
        ds.position[0] = self.x + 47.0;
        ds.position[1] = self.y + 116.0;
        self.arrows[1] = fe.create_element(self.top_layer, &ds);
        // the keyboard layer is made from the BG desc (the original passes the wrong desc)
        self.keyboard_layer = fe.create_layer(None, &ld);
        ds.kind = ELEMENT_2D;
        ds.crc = string_crc("Backing");
        ds.position[0] = self.x + -10.0;
        ds.position[1] = self.y + -20.0;
        ds.texture = "VKeyboardBackplate".into();
        self.backing = fe.create_element(self.bg_layer, &ds);
        fe.e2d(self.backing).set_size(312, 312);
    }

    /// FEKeyboardOverlay.cpp 0x0026C920 — the rows: [mode key 84, 10 x 21], ..., [symbols 84, 6 x 21,
    /// backspace 84], [left 42, right 42, space 126, done 84]
    fn create_keys(&mut self) {
        let row = |first: i32, n: usize, last: Option<i32>| {
            let mut r = vec![Key::new(first, 84)];
            r.extend((0..n).map(|_| Key::new(0, 21)));
            if let Some(l) = last {
                r.push(Key::new(l, 84));
            }
            r
        };
        self.keys = vec![
            row(KEY_ACCENTS, 10, None),
            row(KEY_CAPS, 10, None),
            row(KEY_SHIFT, 10, None),
            row(KEY_SYMBOLS, 6, Some(KEY_BACKSPACE)),
            vec![Key::new(KEY_LEFT, 42), Key::new(KEY_RIGHT, 42), Key::new(32, 126), Key::new(KEY_DONE, 84)],
        ];
    }

    /// FEKeyboardOverlay.cpp 0x00269580 — a centred text and a key sprite (width - 2 by 26) per key
    fn create_key_elements(&mut self, fe: &mut Feck2) {
        for row in &mut self.keys {
            for key in row.iter_mut() {
                let mut dt = SElementDesc { kind: ELEMENT_STATIC_TEXT, font: 2, ..Default::default() };
                dt.crc = string_crc(&format!("VirtualKeyText{}", key.code));
                dt.position[0] = 0.0;
                dt.position[1] = 0.0;
                dt.string = string_crc("GENERAL_BLANK");
                let t = fe.create_element(self.keyboard_layer, &dt);
                fe.text(t).set_justification(2);
                key.text = Some(t);
                let mut ds = SElementDesc { kind: ELEMENT_2D, ..Default::default() };
                ds.crc = string_crc(&format!("VirtualKey{}", key.code));
                ds.position[0] = 0.0;
                ds.position[1] = 0.0;
                ds.texture = "VKeyboardCharacters".into();
                ds.vcd = "\\FE\\FE.vcd".into();
                let s = fe.create_element(self.keyboard_layer, &ds);
                fe.e2d(s).set_size(key.width as u32 - 2, 28 - 2);
                key.sprite = Some(s);
            }
        }
    }

    /// FE3DCharacters.cpp 0x002686E0 — `ResetKeyboard`: empty string, shift on, cursor on DONE, keys hidden
    pub fn reset(&mut self, fe: &mut Feck2) {
        self.first_key = true;
        self.hidden = false;
        self.done = false;
        self.caps = false;
        self.shift = true;
        self.case_locked = false;
        self.mode = 0;
        match find_key(KEY_DONE) {
            Some((r, c)) => {
                self.row = r;
                self.col = c;
            }
            None => log::warn!("(CFEKeyboardOverlay::ResetKeyboard) Error: Key not found"),
        }
        self.last_col = 0;
        self.pos = 0;
        self.string.clear();
        self.repeat_time = 0.0;
        self.repeating = false;
        self.dt = 0.0;
        fe.layers[self.keyboard_layer].flags &= !3;
    }

    /// FEKeyboardOverlay.cpp 0x00268A60 — `SetMaxStringSize`
    pub fn set_max_string_size(&mut self, size: usize) {
        debug_assert!(self.string.len() < self.max_size, "strlen(m_strData) < m_iMaxStringSize");
        debug_assert!(self.max_size < MAX_CHARACTERS, "m_iMaxStringSize < MAX_VIRTUAL_KEYBOARD_CHARACTERS");
        self.max_size = size;
    }

    /// FEKeyboardOverlay.cpp 0x0026A950 — load the elements, the two key textures, then the initial name
    fn load(&mut self, fe: &mut Feck2) {
        for id in [self.backing, self.dummy_key, self.dummy_key_pressed] {
            fe.element_load(id);
        }
        self.key_texture = fe.e2d(self.dummy_key).texture;
        self.pressed_texture = fe.e2d(self.dummy_key_pressed).texture;
        self.load_keys(fe);
        for id in [self.input_string, self.dummy_string, self.cursor, self.arrows[0], self.arrows[1]] {
            fe.element_load(id);
        }
        self.set_string(fe, b"");
    }

    /// FEKeyboardOverlay.cpp 0x0026AD00 — `IterateAllKeys(load)`: texts loaded, sprites on the key texture
    fn load_keys(&mut self, fe: &mut Feck2) {
        for row in &self.keys {
            for key in row {
                if let Some(t) = key.text {
                    fe.element_load(t);
                }
                if let Some(s) = key.sprite {
                    fe.e2d(s).texture = self.key_texture;
                    fe.element_load(s);
                }
            }
        }
    }

    /// FEKeyboardOverlay.cpp 0x0026AAE0 — `Show`: the backing / top layers, reset + load + refresh when shown
    pub fn show(&mut self, fe: &mut Feck2, show: bool) {
        let set = |fe: &mut Feck2, l: LayerId, on: bool| {
            let f = &mut fe.layers[l].flags;
            *f = if on { *f | 3 } else { *f & !3 };
        };
        set(fe, self.bg_layer, show);
        set(fe, self.top_layer, show);
        self.hidden = !show;
        if show {
            self.reset(fe);
            self.load(fe);
            self.refresh(fe);
        } else {
            set(fe, self.keyboard_layer, false);
        }
        self.shown = show;
        self.active = show;
    }

    /// FEKeyboardOverlay.cpp 0x0026ABA0 — `SetString(initialName)`
    pub fn set_string(&mut self, fe: &mut Feck2, name: &[u8]) {
        debug_assert!(name.len() < self.max_size, "strlen(initialName) < m_iMaxStringSize");
        let n = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        self.string = name[..n.min(self.max_size.saturating_sub(1))].to_vec();
        let t = latin1(&self.string);
        fe.text(self.input_string).set_text(&t);
        self.pos = self.string.len() as u8;
        self.place_cursor(fe);
        self.refresh(fe);
    }

    /// `+70 || +71`: hidden or finished, the handlers let the event through
    fn blocked(&self) -> bool {
        self.hidden || self.done
    }

    /// FEQuickRaceTextScreen.cpp 0x0022E960 — DONE was pressed
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// FEQuickRaceTextScreen.cpp 0x0022E980 — the keyboard is up
    pub fn is_visible(&self) -> bool {
        !self.hidden
    }

    /// FEKeyboardOverlay.cpp 0x0026C790 — the cursor blinks on the game update count (`(FM+8)+24`, 24 per s)
    fn blink_cursor(&mut self, fe: &mut Feck2) {
        let updates = (fe.stopwatch * 24.0).floor();
        let on = CURSOR_ON > updates % 19.0;
        fe.el(self.cursor).set_visible(on);
    }

    /// FEKeyboardOverlay.cpp 0x0026C150 — a direction pressed: move, sound, refresh, repeat timer reset
    fn press_dir(&mut self, fe: &mut Feck2, dir: u32) -> bool {
        if self.blocked() {
            return true;
        }
        self.move_dir(fe, dir);
        self.refresh(fe);
        self.release_dir();
        false
    }

    /// FEKeyboardOverlay.cpp 0x0026C220 — held: repeat after 0.333 s, then every 0.085 s
    fn held_dir(&mut self, fe: &mut Feck2, dir: u32) -> bool {
        if self.blocked() {
            return true;
        }
        if self.repeat() {
            self.move_dir(fe, dir);
            self.refresh(fe);
        }
        false
    }

    /// FEKeyboardOverlay.cpp 0x0026C2F0 / 0x0026C3D0
    fn release_dir(&mut self) -> bool {
        if self.blocked() {
            return true;
        }
        self.repeat_time = 0.0;
        self.repeating = false;
        false
    }

    /// FEKeyboardOverlay.cpp 0x0026C330
    fn repeat(&mut self) -> bool {
        let mut r = false;
        self.repeat_time += self.dt;
        if self.repeating {
            if self.repeat_time > 0.085 {
                r = true;
                self.repeat_time = 0.0;
            }
        } else if self.repeat_time > 0.333 {
            self.repeating = true;
            r = true;
            self.repeat_time = 0.0;
        }
        r
    }

    fn move_dir(&mut self, fe: &mut Feck2, dir: u32) {
        match dir {
            0 => {
                self.move_up();
                fe.sounds.push(GuiSound::Up);
            }
            1 => {
                self.move_down();
                fe.sounds.push(GuiSound::Down);
            }
            2 => {
                self.move_left();
                fe.sounds.push(GuiSound::Left);
            }
            _ => {
                self.move_right();
                fe.sounds.push(GuiSound::Right);
            }
        }
    }

    /// FEKeyboardOverlay.cpp 0x0026BB50 — up (wrapping), keeping the column across the wide keys
    fn move_up(&mut self) {
        loop {
            match self.row {
                0 => {
                    if self.col == 0 {
                        self.col = self.last_col.min(1);
                    } else if self.col <= 6 {
                        self.last_col = self.col;
                        self.col = 2;
                    } else if (6..=10).contains(&self.col) {
                        self.last_col = self.col;
                        self.col = 3;
                    }
                }
                3 => {
                    if self.col == 7 {
                        self.col = 7u8.max(self.last_col);
                    }
                }
                4 => {
                    if self.col >= 2 {
                        if self.col == 2 {
                            self.col = 6u8.min(self.last_col);
                            self.col = clamp_u8(self.col, 1, 6);
                        } else if self.col == 3 {
                            self.col = 7;
                        }
                    } else {
                        self.last_col = self.col;
                        self.col = 0;
                    }
                }
                _ => {}
            }
            self.row = if self.row != 0 { self.row - 1 } else { 4 };
            if !self.current_disabled() {
                break;
            }
        }
    }

    /// FEKeyboardOverlay.cpp 0x0026BD00 — down (wrapping)
    fn move_down(&mut self) {
        loop {
            match self.row {
                2 => {
                    if self.col > 7 {
                        self.last_col = self.col;
                        self.col = 7;
                    }
                }
                3 => {
                    if self.col != 0 {
                        if self.col > 6 {
                            self.col = 3;
                        } else {
                            self.last_col = self.col;
                            self.col = 2;
                        }
                    } else if self.last_col != 0 {
                        self.col = 1;
                    }
                }
                4 => match self.col {
                    1 => {
                        self.last_col = self.col;
                        self.col = 0;
                    }
                    2 => self.col = 6u8.min(self.last_col),
                    3 => self.col = 7u8.max(self.last_col),
                    _ => {}
                },
                _ => {}
            }
            self.row = if self.row == 4 { 0 } else { self.row + 1 };
            if !self.current_disabled() {
                break;
            }
        }
    }

    /// FEKeyboardOverlay.cpp 0x0026BE80 — left (wrapping in the row)
    fn move_left(&mut self) {
        loop {
            if self.col != 0 {
                self.col -= 1;
            } else {
                self.col = self.keys[self.row as usize].len() as u8 - 1;
            }
            if !self.current_disabled() {
                break;
            }
        }
        self.remember_col();
    }

    /// FEKeyboardOverlay.cpp 0x0026BF00 — right (wrapping in the row)
    fn move_right(&mut self) {
        loop {
            if self.col as usize == self.keys[self.row as usize].len() - 1 {
                self.col = 0;
            } else {
                self.col += 1;
            }
            if !self.current_disabled() {
                break;
            }
        }
        self.remember_col();
    }

    /// FEKeyboardOverlay.cpp 0x0026C560 — the column to come back to (the cursor keys map to 7 / 9 or more)
    fn remember_col(&mut self) {
        // only unlabelled keys count, so the cursor-key cases never run (their label is GENERAL_BLANK)
        let key = &self.keys[self.row as usize][self.col as usize];
        if key.string == 0 {
            match key.code {
                KEY_LEFT => self.last_col = (self.last_col > 7) as u8 + 7,
                KEY_RIGHT => self.last_col = (self.last_col > 9) as u8 + 9,
                _ => self.last_col = self.col,
            }
        }
    }

    fn current_disabled(&self) -> bool {
        self.key_disabled(self.keys[self.row as usize][self.col as usize].code)
    }

    /// FEKeyboardOverlay.cpp 0x0026C4C0 — keys that cannot be used now
    fn key_disabled(&self, code: i32) -> bool {
        if (code == KEY_CAPS || code == KEY_SHIFT) && self.case_locked {
            return true;
        }
        if self.mode == 1 && (code == KEY_SHIFT || code == KEY_CAPS) {
            return true;
        }
        if code == KEY_DONE && !self.valid() {
            return true;
        }
        !self.accents && code == KEY_ACCENTS
    }

    /// FEKeyboardOverlay.cpp 0x0026C820 — not empty, not all spaces, and the screen's check passes
    fn valid(&self) -> bool {
        if self.string.is_empty() {
            return false;
        }
        let blank = !self.string.iter().any(|&c| c != b' ');
        if blank {
            return false;
        }
        match self.validator {
            Some(f) => f(&self.string),
            None => true,
        }
    }

    /// FEKeyboardOverlay.cpp 0x0026BF80 — the character a key types (case from shift xor caps), >255 types a space
    fn key_char(&self, code: i32, apply_case: bool) -> u8 {
        let mut c = code;
        if apply_case {
            if self.caps == self.shift {
                c = to_lower(c);
            } else {
                c = to_upper(c);
            }
        }
        if c <= 255 { c as u8 } else { b' ' }
    }

    /// FEKeyboardOverlay.cpp 0x0026A6C0.. — a shortcut button presses a key
    fn press_if_free(&mut self, fe: &mut Feck2, code: i32) -> bool {
        if self.blocked() {
            return true;
        }
        self.press_key(fe, code, true);
        false
    }

    /// FEKeyboardOverlay.cpp 0x0026B590 — select: the highlighted key
    fn press_selected(&mut self, fe: &mut Feck2) {
        let code = self.keys[self.row as usize][self.col as usize].code;
        self.press_key(fe, code, true);
    }

    /// FEKeyboardOverlay.cpp 0x0026B650 — `KeyPressed(code, applyCase)`
    pub fn press_key(&mut self, fe: &mut Feck2, code: i32, apply_case: bool) {
        if self.key_disabled(code) {
            return;
        }
        let (mut layout, mut keys, mut cursor, mut text) = (false, false, false, false);
        if code >= 0x10000 {
            match code {
                KEY_BACKSPACE => {
                    if self.first_key && self.clear_on_first {
                        self.first_key = false;
                        self.string.clear();
                        self.pos = 0;
                        cursor = true;
                        text = true;
                    } else if self.pos != 0 {
                        self.pos -= 1;
                        self.string.remove(self.pos as usize);
                        cursor = true;
                        text = true;
                    }
                }
                KEY_SHIFT => {
                    self.shift = !self.shift;
                    keys = true;
                }
                KEY_CAPS => {
                    self.caps = !self.caps;
                    keys = true;
                }
                KEY_ALPHABET => {
                    self.mode = 0;
                    layout = true;
                }
                KEY_SYMBOLS => {
                    self.mode = 1;
                    layout = true;
                }
                KEY_ACCENTS => {
                    self.mode = 2;
                    layout = true;
                }
                KEY_DONE => {
                    if self.valid() {
                        self.done = true;
                    }
                }
                KEY_LEFT => {
                    if self.pos != 0 {
                        self.first_key = false;
                        self.pos -= 1;
                        cursor = true;
                    }
                }
                KEY_RIGHT => {
                    if (self.pos as usize) < self.string.len() {
                        self.first_key = false;
                        self.pos += 1;
                        cursor = true;
                    }
                }
                _ => {}
            }
        } else {
            if self.first_key && self.clear_on_first {
                self.first_key = false;
                self.string.clear();
                self.pos = 0;
                cursor = true;
            }
            if self.string.len() < self.max_size - 1 {
                let c = self.key_char(code, apply_case);
                self.string.insert(self.pos as usize, c);
                self.pos += 1;
                cursor = true;
                text = true;
            }
            if code != 32 {
                if self.shift {
                    keys = true;
                }
                if !self.case_locked {
                    self.shift = false;
                }
            }
        }
        if text {
            let t = latin1(&self.string);
            fe.text(self.input_string).set_text(&t);
        }
        if cursor {
            self.place_cursor(fe);
        }
        if layout || keys || text {
            if self.current_disabled() {
                self.move_down();
            }
            self.refresh(fe);
        }
    }

    /// FEKeyboardOverlay.cpp 0x0026C670 — the cursor after the text up to `pos`
    fn place_cursor(&mut self, fe: &mut Feck2) {
        let n = (self.pos as usize).min(self.string.len());
        let t = latin1(&self.string[..n]);
        fe.text(self.dummy_string).set_text(&t);
        let w = fe.text_width(self.dummy_string);
        let y = (self.y + 158.0) + 4.0;
        let x = (self.x as f64 + 87.0 + 0.0 + w as f64) as f32;
        fe.element_set_position(self.cursor, x, y, 0.0);
    }

    /// FEKeyboardOverlay.cpp 0x0026AF90 / 0x0026D9E0 / 0x0026DDE0 / 0x0026E1F0 — layout codes (per page), then every key placed, coloured and labelled
    pub fn refresh(&mut self, fe: &mut Feck2) {
        let layout = &LAYOUTS[self.mode.min(2) as usize];
        for (r, codes) in layout.iter().enumerate() {
            for (c, &code) in codes.iter().enumerate() {
                self.keys[r][c].code = code;
            }
        }
        fe.layers[self.keyboard_layer].flags |= 1;
        let mut y = self.y;
        for r in 0..self.keys.len() {
            let x = self.x;
            let mut acc = 0.0f32;
            for c in 0..self.keys[r].len() {
                let code = self.keys[r][c].code;
                let mut sprite = key_colour();
                let mut text = COLOUR_BLACK;
                let mut tex = self.key_texture;
                let case_key = |on: bool, sprite: &mut Colour, text: &mut Colour, tex: &mut TextureId| {
                    if self.case_locked {
                        *sprite = disabled_colour();
                        *text = disabled_text_colour();
                    } else {
                        match self.mode {
                            0 | 2 => {
                                if on {
                                    *tex = self.pressed_texture;
                                    *sprite = pressed_colour();
                                }
                            }
                            1 => {
                                *sprite = disabled_colour();
                                *text = disabled_text_colour();
                            }
                            _ => {}
                        }
                    }
                };
                match code {
                    KEY_SHIFT => case_key(self.shift, &mut sprite, &mut text, &mut tex),
                    KEY_CAPS => case_key(self.caps, &mut sprite, &mut text, &mut tex),
                    KEY_ACCENTS => {
                        if !self.accents {
                            sprite = hidden_colour();
                            text = hidden_colour();
                        }
                    }
                    KEY_DONE => {
                        if !self.valid() {
                            sprite = disabled_colour();
                            text = disabled_text_colour();
                        }
                    }
                    _ => {}
                }
                if r as u8 == self.row && c as u8 == self.col {
                    sprite = if sprite == key_colour() { selected_colour() } else { selected_other_colour() };
                }
                let width = self.keys[r][c].width;
                if let Some(t) = self.keys[r][c].text {
                    fe.element_set_position(t, (x + acc) + (width / 2) as f32, y, 0.0);
                    fe.text(t).set_colour(text);
                    let s = self.keys[r][c].string;
                    if s != 0 {
                        fe.text(t).set_string(s);
                    } else {
                        let ch = self.key_char(code, true);
                        self.keys[r][c].ch = ch;
                        fe.text(t).set_text(&[ch as u16]);
                    }
                }
                if let Some(s) = self.keys[r][c].sprite {
                    fe.element_set_position(s, x + acc, y - 2.0, 0.0);
                    fe.e2d(s).set_colour(sprite, false, COLOUR_WHITE);
                    fe.e2d(s).texture = tex;
                }
                acc += width as f32;
                acc += 0.0;
            }
            y += 28.0;
        }
    }

    /// the name typed so far (`+88`)
    pub fn text(&self) -> &[u8] {
        &self.string
    }
}

/// FEKeyboardOverlay.cpp 0x0026C000 — row / column of a special key in the last row
fn find_key(code: i32) -> Option<(u8, u8)> {
    match code {
        KEY_DONE => Some((4, 3)),
        KEY_LEFT => Some((4, 0)),
        KEY_RIGHT => Some((4, 1)),
        _ => None,
    }
}

/// FEKeyboardOverlay.cpp 0x0026F380
fn clamp_u8(v: u8, lo: u8, hi: u8) -> u8 {
    if v <= lo {
        lo
    } else if v <= hi {
        v
    } else {
        hi
    }
}

/// FEKeyboardOverlay.cpp 0x0026C070
fn to_upper(c: i32) -> i32 {
    if (97..=122).contains(&c) || (224..=253).contains(&c) {
        c - 32
    } else if c == 255 {
        376
    } else {
        c
    }
}

/// FEKeyboardOverlay.cpp 0x0026C0E0
fn to_lower(c: i32) -> i32 {
    if (65..=90).contains(&c) || (192..=221).contains(&c) {
        c + 32
    } else if c == 376 {
        255
    } else {
        c
    }
}

/// the font pages are Latin-1: one byte, one code unit
pub fn latin1(s: &[u8]) -> Vec<u16> {
    s.iter().map(|&c| c as u16).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyboard() -> (Feck2, Arc<Mutex<CFEKeyboardOverlay>>) {
        let mut fe = Feck2::new();
        create_common_keyboard(&mut fe);
        let obj = fe.keyboard.as_ref().unwrap().obj.clone();
        (fe, obj)
    }

    #[test]
    fn keyboard_types_and_finishes() {
        let (mut fe, obj) = keyboard();
        let mut k = obj.lock().unwrap();
        k.show(&mut fe, true);
        assert_eq!((k.row, k.col), (4, 3));
        // DONE is disabled with an empty name: up lands on backspace, up again on 'Q'
        k.press_dir(&mut fe, 0);
        assert_eq!((k.row, k.col), (3, 7));
        k.press_dir(&mut fe, 0);
        assert_eq!((k.row, k.col), (2, 7));
        k.press_selected(&mut fe);
        assert_eq!(k.text(), b"Q");
        // shift drops after the first letter
        k.press_selected(&mut fe);
        assert_eq!(k.text(), b"Qq");
        k.press_dir(&mut fe, 1);
        k.press_dir(&mut fe, 1);
        assert_eq!((k.row, k.col), (4, 3));
        k.press_selected(&mut fe);
        assert!(k.is_done());
    }

    #[test]
    fn keyboard_backspace_and_cursor() {
        let (mut fe, obj) = keyboard();
        let mut k = obj.lock().unwrap();
        k.show(&mut fe, true);
        k.set_string(&mut fe, b"ABC");
        assert_eq!(k.pos, 3);
        k.press_key(&mut fe, KEY_LEFT, true);
        k.press_key(&mut fe, KEY_BACKSPACE, true);
        assert_eq!(k.text(), b"AC");
        k.press_key(&mut fe, KEY_SYMBOLS, true);
        assert_eq!(k.keys[1][6].code, 8364);
        assert_eq!(k.key_char(8364, true), b' ');
    }
}
