//! frontend copy of src\Controller\UserInput.cpp's mapping layer (core owns Controller; this private reader feeds
//! `CFrontendManager::Update` until a shared one exists).
//!
//! mappings come from `\Input\Xboxpad\Config1.ini`: `Input.SetMapping <Action> <Expr>` where Expr is a pad input
//! (`A`, `LEFT`, `STICK_LX`, `LTRIGGER`, `WHITE` ...), optionally wrapped in `DIGITAL(x)` (0/1 at 0.5) and/or
//! `DEADZONE(x, d)`. later lines for the same action add further sources; the strongest value wins (0x001E6BC0).

use crate::feck2::feck2_manager::{action, ActionState};

/// xbox pad inputs a mapping can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PadInput {
    A, B, X, Y, White, Black, Start, Back, Up, Down, Left, Right,
    LTrigger, RTrigger, StickLButton, StickRButton,
    StickLX, StickLY, StickRX, StickRY,
}

impl PadInput {
    /// index in `visos::input::input_device::XBOXPAD_KEYS`
    pub fn layer_key(self) -> usize {
        use visos::input::input_device::key;
        use PadInput::*;
        match self {
            A => key::A, B => key::B, X => key::X, Y => key::Y, White => key::WHITE, Black => key::BLACK,
            Start => key::START, Back => key::BACK, Up => key::UP, Down => key::DOWN, Left => key::LEFT,
            Right => key::RIGHT, LTrigger => key::LTRIGGER, RTrigger => key::RTRIGGER,
            StickLButton => key::STICK_LBUTTON, StickRButton => key::STICK_RBUTTON, StickLX => key::STICK_LX,
            StickLY => key::STICK_LY, StickRX => key::STICK_RX, StickRY => key::STICK_RY,
        }
    }

    fn parse(s: &str) -> Option<Self> {
        use PadInput::*;
        Some(match s {
            "A" => A, "B" => B, "X" => X, "Y" => Y, "WHITE" => White, "BLACK" => Black, "START" => Start, "BACK" => Back,
            "UP" => Up, "DOWN" => Down, "LEFT" => Left, "RIGHT" => Right, "LTRIGGER" => LTrigger, "RTRIGGER" => RTrigger,
            "STICK_LBUTTON" => StickLButton, "STICK_RBUTTON" => StickRButton,
            "STICK_LX" => StickLX, "STICK_LY" => StickLY, "STICK_RX" => StickRX, "STICK_RY" => StickRY,
            _ => return None,
        })
    }
}

/// one mapping source.
#[derive(Debug, Clone, Copy)]
pub struct Source {
    pub input: PadInput,
    pub digital: bool,
    pub deadzone: f32,
}

impl Source {
    fn parse(expr: &str) -> Option<Self> {
        let e = expr.trim();
        if let Some(inner) = e.strip_prefix("DIGITAL(").and_then(|x| x.strip_suffix(')')) {
            let mut s = Self::parse(inner)?;
            s.digital = true;
            return Some(s);
        }
        if let Some(inner) = e.strip_prefix("DEADZONE(").and_then(|x| x.strip_suffix(')')) {
            let (a, b) = inner.rsplit_once(',')?;
            let mut s = Self::parse(a)?;
            s.deadzone = b.trim().parse().ok()?;
            return Some(s);
        }
        Some(Self { input: PadInput::parse(e)?, digital: false, deadzone: 0.0 })
    }

    /// raw pad value -> action value.
    pub fn eval(&self, raw: f32) -> f32 {
        let mut v = raw;
        if self.deadzone > 0.0 {
            v = if v.abs() < self.deadzone { 0.0 } else { v };
        }
        if self.digital {
            v = if v.abs() >= 0.5 { v.signum() } else { 0.0 };
        }
        v
    }
}

/// `UserInput` action names in index order (0x001E7C20).
pub const ACTION_NAMES: [&str; action::COUNT] = [
    "MenuLeft", "MenuRight", "MenuDown", "MenuUp", "MenuUpDown", "MenuLeftRight", "MenuSelect", "MenuCancel",
    "AlloyEditHoriz", "AlloyEditVert", "MouseLeftRight", "MouseUpDown", "MouseSelect", "Pause", "Online", "Skip",
    "MenuOption1", "MenuOption2", "MenuL1", "MenuL2", "MenuL3", "MenuR1", "MenuR2", "MenuR3",
    "CameraRotateHoriz", "CameraRotateVert",
];

/// parsed mapping table.
#[derive(Debug, Clone, Default)]
pub struct InputMapping {
    pub sources: Vec<Vec<Source>>,
}

impl InputMapping {
    pub fn parse(text: &str) -> Self {
        let mut sources = vec![Vec::new(); action::COUNT];
        for line in text.lines() {
            let l = line.trim();
            if l.starts_with("//") {
                continue;
            }
            let mut it = l.split_whitespace();
            if it.next() != Some("Input.SetMapping") {
                continue;
            }
            let Some(name) = it.next() else { continue };
            let expr: String = it.collect::<Vec<_>>().join("");
            if let Some(i) = ACTION_NAMES.iter().position(|n| *n == name) {
                if let Some(s) = Source::parse(&expr) {
                    sources[i].push(s);
                }
            }
        }
        Self { sources }
    }

    /// `\Input\Xboxpad\Config1.ini`
    pub fn load_default() -> Self {
        Self::load_config(0)
    }

    /// `\Input\Xboxpad\Config<index + 1>.ini`: the pad mapping picked in the controller options.
    pub fn load_config(index: u8) -> Self {
        match std::fs::read_to_string(visos::file::resolve(&format!(r"\Input\Xboxpad\Config{}.ini", index as u32 + 1))) {
            Ok(t) => Self::parse(&t),
            Err(_) => Self { sources: vec![Vec::new(); action::COUNT] },
        }
    }

    /// strongest source value for an action (0x001E6BC0 picks the largest magnitude).
    pub fn value(&self, a: usize, raw: &dyn Fn(PadInput) -> f32) -> f32 {
        let mut best = 0.0f32;
        for s in &self.sources[a] {
            let v = s.eval(raw(s.input));
            if v.abs() > best.abs() {
                best = v;
            }
        }
        best
    }

    /// next action state from the previous one.
    /// raw values come from the visos input layer (SDL3 pads + keyboard device, strongest over every device); the
    /// caller's `raw` is ignored so a pad is never read twice.
    pub fn step(&self, prev: &[ActionState; action::COUNT], _raw: &dyn Fn(PadInput) -> f32) -> [ActionState; action::COUNT] {
        let raw = &|p: PadInput| visos::input::input_manager::any_key_value(p.layer_key());
        let mut out = [ActionState::default(); action::COUNT];
        for a in 0..action::COUNT {
            let v = self.value(a, raw);
            let p = prev[a].value;
            out[a] = ActionState { value: v, previous: p, changed: (v != 0.0) != (p != 0.0) };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config1() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let m = InputMapping::load_default();
        assert_eq!(m.sources[action::MENU_SELECT].len(), 1);
        assert!(m.sources[action::MENU_SELECT][0].digital);
        assert_eq!(m.sources[action::MENU_SELECT][0].input, PadInput::A);
        assert_eq!(m.sources[action::MENU_UP_DOWN][0].deadzone, 0.6);
        // CameraRotateHoriz is mapped twice to STICK_RX
        assert_eq!(m.sources[action::CAMERA_ROTATE_HORIZ].len(), 2);
    }
}
