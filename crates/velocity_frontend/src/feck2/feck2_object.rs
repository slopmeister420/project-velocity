//! src\Feck2\FECK2Object.cpp — FECK2 object model: events, descriptors, screens, layers and the element base.
//!
//! the original keeps these as C++ classes linked by intrusive lists; here they live in arenas owned by
//! [`Feck2`] (`crate::feck2::feck2_manager`) and are addressed by index. list order is kept exactly:
//! - a screen's layers are sorted by ascending priority (head = priority 0); events walk head -> tail,
//!   rendering walks tail -> head, so priority 0 is processed first and drawn last (on top).
//! - a layer's elements are prepended on creation; rendering walks head -> tail, so the element created
//!   last is drawn first (underneath).

use crate::feck2::feck2_elements::feck2_element_2d::CElement2D;
use crate::feck2::feck2_elements::feck2_element_2d_line::CElement2DLine;
use crate::feck2::feck2_elements::feck2_element_static_text::CElementStaticText;
use crate::feck2::feck2_manager::Feck2;
use crate::render2d::{Colour, WHITE};

/// `CEvent::EE_Count`
pub const EE_COUNT: usize = 91;

/// `CEvent::eEvent` values (the ones the ported code uses by name).
pub mod ev {
    pub const NONE: u32 = 0;
    pub const ASSIGN_EVENTS: u32 = 1;
    pub const POST_ENTER: u32 = 2;
    pub const GLOBAL_PRELOAD: u32 = 3;
    pub const PRELOAD: u32 = 4;
    pub const EXIT_SCREEN: u32 = 8;
    pub const ENTER: u32 = 9;
    pub const ACTIVATE: u32 = 10;
    pub const START: u32 = 11;
    pub const LEAVE: u32 = 12;
    pub const EXIT: u32 = 14;
    pub const START_SCREEN: u32 = 15;
    pub const GOTO_SCREEN: u32 = 18;
    pub const GOTO_PREVIOUS_SCREEN: u32 = 19;
    pub const GOTO_NEXT_SCREEN: u32 = 20;
    pub const UPDATE: u32 = 21;
    // directions: press / held / release
    pub const UP_PRESS: u32 = 22;
    pub const UP_HELD: u32 = 23;
    pub const UP_RELEASE: u32 = 24;
    pub const RIGHT_PRESS: u32 = 25;
    pub const RIGHT_HELD: u32 = 26;
    pub const RIGHT_RELEASE: u32 = 27;
    pub const DOWN_PRESS: u32 = 28;
    pub const DOWN_HELD: u32 = 29;
    pub const DOWN_RELEASE: u32 = 30;
    pub const LEFT_PRESS: u32 = 31;
    pub const LEFT_HELD: u32 = 32;
    pub const LEFT_RELEASE: u32 = 33;
    pub const CAM_LEFT_PRESS: u32 = 34;
    pub const CAM_RIGHT_PRESS: u32 = 37;
    pub const CAM_DOWN_PRESS: u32 = 40;
    pub const CAM_UP_PRESS: u32 = 43;
    pub const SELECT_PRESS: u32 = 46;
    pub const SELECT_HELD: u32 = 47;
    pub const SELECT_RELEASE: u32 = 48;
    pub const PAUSE_PRESS: u32 = 49;
    pub const ONLINE_PRESS: u32 = 52;
    pub const CANCEL_PRESS: u32 = 55;
    pub const CANCEL_HELD: u32 = 56;
    pub const CANCEL_RELEASE: u32 = 57;
    pub const OPTION1_PRESS: u32 = 58;
    pub const OPTION2_PRESS: u32 = 61;
    pub const L1_PRESS: u32 = 64;
    pub const L2_PRESS: u32 = 67;
    pub const L3_PRESS: u32 = 70;
    pub const R1_PRESS: u32 = 73;
    pub const R2_PRESS: u32 = 76;
    pub const R3_PRESS: u32 = 79;
}

/// `CEvent` (0x000AD180): `+8 eEvent, +12 controller, +16 param a, +20 param b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CEvent {
    pub event: u32,
    pub controller: i32,
    pub param_a: u32,
    pub param_b: u32,
}

impl CEvent {
    /// FECK2Manager.cpp 0x000AD180
    pub fn new(event: u32, controller: i32, param_a: u32, param_b: u32) -> Self {
        Self { event, controller, param_a, param_b }
    }

    /// the update event carries the frame time (seconds) as float bits in param a.
    pub fn dt(&self) -> f32 {
        f32::from_bits(self.param_a)
    }
}

/// a layer event functor (`CEventFunctor<CScreenClass>` 16 bytes): screen object + member function.
pub type Handler = std::sync::Arc<dyn Fn(&mut Feck2, &CEvent) -> bool + Send + Sync>;

pub type ElementId = usize;
pub type LayerId = usize;
pub type ScreenId = usize;

/// FECK2Object.cpp 0x00204C20 — `SScreenDesc` (ctor)
#[derive(Default, Clone)]
pub struct SScreenDesc {
    pub crc: u32, // +8
}

/// FECK2Object.cpp 0x00204A60 — `SLayerDesc` (ctor)
#[derive(Default, Clone)]
pub struct SLayerDesc {
    pub priority: i32,              // +8
    pub crc: u32,                   // +12
    pub on_none: Option<Handler>,   // +16 mapped to event 0
    pub on_assign: Option<Handler>, // +20 mapped to event 1 (assign event functors), fired immediately
}

/// element kinds (`SElementDesc::+64`, `FECK2::CreateElement` 0x0029A540).
pub const ELEMENT_2D: u32 = 1;
pub const ELEMENT_2D_LINE: u32 = 2;
pub const ELEMENT_3D: u32 = 3;
pub const ELEMENT_CAMERA: u32 = 4;
pub const ELEMENT_STATIC_TEXT: u32 = 5;
pub const ELEMENT_CHARACTER: u32 = 6;

/// `SElementDesc` (0x00204B30, 84 bytes).
#[derive(Clone)]
pub struct SElementDesc {
    pub position: [f32; 4], // +8
    pub vec24: [f32; 4],    // +24
    pub colour: Colour,     // +40
    pub crc: u32,           // +44 element name crc
    pub vcd: String,        // +52 texture pack
    pub texture: String,    // +56 texture name
    pub kind: u32,          // +64
    pub font: u32,          // +68 font id (static text)
    pub string: u32,        // +72 string crc, 0xFFFF = none
    pub f76: f32,           // +76
    pub dynamic: bool,      // +80 element owns copies of the name buffers
}

impl Default for SElementDesc {
    /// FECK2Object.cpp 0x00204B30
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 1.0],
            vec24: [0.0, 0.0, 0.0, 1.0],
            colour: WHITE,
            crc: 0,
            vcd: String::new(),
            texture: String::new(),
            kind: 0,
            font: 0,
            string: 0xFFFF,
            f76: 0.0,
            dynamic: false,
        }
    }
}

/// `CElement` common part (base class, 108 bytes in the original).
#[derive(Clone, Debug)]
pub struct CElementBase {
    pub crc: u32,              // +8
    pub visible: bool,         // +12
    pub blend: bool,           // +13
    pub position: [f32; 4],    // +24
    pub target: [f32; 4],      // +40
    pub start: [f32; 4],       // +56
    pub move_time: i32,        // +72
    pub move_left: i32,        // +76
    pub move_delay: i32,       // +80
}

impl Default for CElementBase {
    /// FECK2Object.cpp 0x0029AC80
    fn default() -> Self {
        Self {
            crc: 0,
            visible: false,
            blend: false,
            position: [0.0, 0.0, 0.0, 1.0],
            target: [0.0, 0.0, 0.0, 1.0],
            start: [0.0, 0.0, 0.0, 1.0],
            move_time: 0,
            move_left: 0,
            move_delay: 0,
        }
    }
}

impl CElementBase {
    /// FECK2Object.cpp 0x00285E90 — `CElement::Init`
    pub fn init(&mut self, d: &SElementDesc) {
        self.crc = d.crc;
        self.visible = true;
        self.blend = true;
    }

    /// FECK2Object.cpp 0x00285FC0 — `SetPosition`
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.start = self.position;
        self.position = [x, y, z, 1.0];
        self.target = self.position;
        self.move_time = 0;
        self.move_left = 0;
    }

    /// FECK2Object.cpp 0x00285F50 — `SetPosition(const VsVector4&)`
    pub fn set_position_v(&mut self, p: [f32; 4]) {
        self.start = self.position;
        self.position = p;
        self.target = self.position;
        self.move_time = 0;
        self.move_left = 0;
    }

    /// FECK2Object.cpp 0x0029A8D0 — `MoveBy`: animated when `time` != 0.
    pub fn move_by(&mut self, dx: i32, dy: i32, time: i32, delay: i32) {
        self.move_delay = delay;
        if self.move_left != 0 {
            self.position = self.target;
        }
        self.start = self.position;
        if time != 0 {
            self.target = self.position;
            self.target[0] += dx as f32;
            self.target[1] += dy as f32;
            self.move_time = time;
            self.move_left = time;
        } else {
            self.position[0] += dx as f32;
            self.position[1] += dy as f32;
            self.target = self.position;
            self.move_time = 0;
            self.move_left = 0;
        }
    }

    /// FECK2Object.cpp 0x0029AA10 — `MoveByY`
    pub fn move_by_y(&mut self, dy: i32, time: i32, delay: i32) {
        self.move_delay = delay;
        if self.move_left != 0 {
            self.position = self.target;
        }
        self.start = self.position;
        if time != 0 {
            self.target = self.position;
            self.target[1] += dy as f32;
            self.move_time = time;
            self.move_left = time;
        } else {
            self.position[1] += dy as f32;
            self.target = self.position;
            self.move_time = 0;
            self.move_left = 0;
        }
    }

    /// FECK2Object.cpp 0x0029AB10 — `MoveByX` (original bug: the animated branch offsets the target, the static
    /// branch the position, as in MoveBy; kept)
    pub fn move_by_x(&mut self, dx: i32, time: i32, delay: i32) {
        self.move_delay = delay;
        if self.move_left != 0 {
            self.position = self.target;
        }
        self.start = self.position;
        if time != 0 {
            self.target = self.position;
            self.target[0] += dx as f32;
            self.move_time = time;
            self.move_left = time;
        } else {
            self.position[0] += dx as f32;
            self.target = self.position;
            self.move_time = 0;
            self.move_left = 0;
        }
    }

    /// FECK2Object.cpp 0x0029A740 — `UpdateMove`: sine eased move towards the target.
    pub fn update_move(&mut self, frame_time: f32) {
        if self.move_left == 0 {
            return;
        }
        if self.move_delay <= 0 {
            self.move_left = (self.move_left as f32 - frame_time) as i32;
            if self.move_left > 0 {
                let d = [
                    self.target[0] - self.start[0],
                    self.target[1] - self.start[1],
                    self.target[2] - self.start[2],
                    self.target[3] - self.start[3],
                ];
                let a = (1.0 - (self.move_left as f32 * 1.0) / (self.move_time as f32 * 1.0)) * 90.0;
                let s = (a * 0.017_453_292_f32).sin();
                for k in 0..4 {
                    self.position[k] = self.start[k] + d[k] * s;
                }
            } else {
                self.position = self.target;
                self.move_left = 0;
                self.move_time = 0;
            }
        } else {
            self.move_delay = (self.move_delay as f32 - frame_time) as i32;
        }
    }
}

/// placeholder for the element kinds not ported yet (3D, camera, character): they keep their base data.
#[derive(Clone, Debug, Default)]
pub struct CElementStub {
    pub base: CElementBase,
    pub kind: u32,
}

/// the element classes, dispatched like the original vtable (slot numbers in comments).
pub enum Element {
    E2D(CElement2D),
    Line(CElement2DLine),
    Text(CElementStaticText),
    Stub(CElementStub),
}

impl Element {
    pub fn base(&self) -> &CElementBase {
        match self {
            Element::E2D(e) => &e.base,
            Element::Line(e) => &e.base,
            Element::Text(e) => &e.base,
            Element::Stub(e) => &e.base,
        }
    }

    pub fn base_mut(&mut self) -> &mut CElementBase {
        match self {
            Element::E2D(e) => &mut e.base,
            Element::Line(e) => &mut e.base,
            Element::Text(e) => &mut e.base,
            Element::Stub(e) => &mut e.base,
        }
    }

    pub fn as_line(&mut self) -> Option<&mut CElement2DLine> {
        match self {
            Element::Line(e) => Some(e),
            _ => None,
        }
    }

    /// FECK2Element2D.cpp 0x00293310 / 0x002860B0 — slot 1 `AsElement2D`
    pub fn as_2d(&mut self) -> Option<&mut CElement2D> {
        match self {
            Element::E2D(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_text(&mut self) -> Option<&mut CElementStaticText> {
        match self {
            Element::Text(e) => Some(e),
            _ => None,
        }
    }

    /// FECK2Object.cpp 0x00204C50 — `SetVisible`
    pub fn set_visible(&mut self, v: bool) {
        self.base_mut().visible = v;
    }

    /// FECK2Object.cpp 0x00204C70 — `IsVisible`
    pub fn is_visible(&self) -> bool {
        self.base().visible
    }
}

/// FECK2Object.cpp 0x0029A540 — `FECK2::CreateElement`
pub fn create_element(kind: u32) -> Element {
    match kind {
        ELEMENT_2D => Element::E2D(CElement2D::default()),
        ELEMENT_2D_LINE => Element::Line(CElement2DLine::default()),
        ELEMENT_STATIC_TEXT => Element::Text(CElementStaticText::default()),
        k => Element::Stub(CElementStub { base: CElementBase::default(), kind: k }),
    }
}

/// `CLayer` (40 bytes): priority, screen, element list, function mapper.
pub struct CLayer {
    pub crc: u32,                                // +12
    pub flags: u32,                              // +16 (bit 0 visible, bit 1 active)
    pub priority: i32,                           // +36
    pub screen: Option<ScreenId>,                // +8
    pub elements: Vec<ElementId>,                // +28 list head first
    pub handlers: Vec<Option<Handler>>,          // +32 CFunctionMapper (91 slots)
}

impl CLayer {
    /// FECK2Object.cpp 0x0026E820 / 0x002862C0 — ctor + `Init` (the mapper part; assign events are fired by the caller).
    pub fn new(d: &SLayerDesc) -> Self {
        let mut l = Self {
            crc: d.crc,
            flags: 3,
            priority: d.priority,
            screen: None,
            elements: Vec::new(),
            handlers: vec![None; EE_COUNT],
        };
        if let Some(h) = &d.on_assign {
            l.map_event(h.clone(), ev::ASSIGN_EVENTS, false);
        }
        if let Some(h) = &d.on_none {
            l.map_event(h.clone(), ev::NONE, false);
        }
        l
    }

    /// FECK2Object.cpp 0x00286A30 — `CFunctionMapper::Map`: only fills an empty slot unless `replace`.
    pub fn map_event(&mut self, h: Handler, event: u32, replace: bool) -> bool {
        let e = event as usize;
        if replace {
            self.handlers[e] = None;
        }
        if self.handlers[e].is_none() {
            self.handlers[e] = Some(h);
            return true;
        }
        false
    }

    /// FECK2Object.cpp 0x0026E6E0 — `IsVisible`
    pub fn is_visible(&self) -> bool {
        self.flags & 1 != 0
    }

    /// FECK2Object.cpp 0x00284310 — `IsActive`
    pub fn is_active(&self) -> bool {
        self.flags & 2 != 0
    }
}

/// `CScreen` (40 bytes)
pub struct CScreen {
    pub crc: u32,              // +8
    pub layers: Vec<LayerId>,  // +12 list, ascending priority
}

/// screen-class helper: the handful of `CEventFunctor` creation macros (`FECK_ASSIGN_LAYER_EVENT_FUNCTOR`).
pub fn functor<S: Send + Sync + 'static>(
    obj: &std::sync::Arc<std::sync::Mutex<S>>,
    f: fn(&mut S, &mut Feck2, &CEvent) -> bool,
) -> Handler {
    let o = obj.clone();
    std::sync::Arc::new(move |fe: &mut Feck2, e: &CEvent| {
        let mut g = o.lock().unwrap();
        f(&mut g, fe, e)
    })
}
