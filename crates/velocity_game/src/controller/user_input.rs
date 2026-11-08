//! src\Controller\UserInput.cpp + the Input library (InputManager/InputBinding/InputMapping.cpp, 0x3CC550..) —
//! game actions bound to pad keys by `Input\<device>\*.ini` (`Input.SetMapping <Action> <Expr>`).
//!
//! Expressions: a physical key (`A`, `LTRIGGER`, `STICK_LX`...), a number, or a function of them. The binding
//! compiles to a small stack program evaluated every update (InputBinding::evaluate 0x003D57E0); each binding keeps
//! its value and the previous update's value. Known functions: `DIGITAL(x)` = sign of x, `DEADZONE(x, d)` =
//! `sign(x) * (|x| - d) / (1 - d)` when `|x| >= d` else 0. The other opcodes of the evaluator have no name recovered
//! (crc only) and no shipped mapping uses them. `FE\KEYMAP.INI` is not read by F3_D.
//!
//! Devices come from the visos input layer (SDL3 pads on ports 0..3, keyboard as device 4, a PC addition: see
//! `KEYBOARD_KEYS`).

pub use visos::input::input_device::XBOXPAD_KEYS;
use visos::input::input_manager::InputManager;
use bevy::prelude::*;

/// game actions (index into the per-device binding table, UserInput.cpp 0x001E7C20)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum EAction {
    MenuLeft = 0,
    MenuRight,
    MenuDown,
    MenuUp,
    MenuUpDown,
    MenuLeftRight,
    MenuSelect,
    MenuCancel,
    AlloyEditHoriz,
    AlloyEditVert,
    MouseLeftRight,
    MouseUpDown,
    MouseSelect,
    Pause,
    Online,
    Skip,
    MenuOption1,
    MenuOption2,
    MenuL1,
    MenuL2,
    MenuL3,
    MenuR1,
    MenuR2,
    MenuR3,
    CameraRotateHoriz,
    CameraRotateVert,
    CameraChange,
    CarSteering,
    CarSteerLeft,
    CarSteerRight,
    CarAccelerateBrake,
    CarAccelerate,
    CarBrake,
    CarHandbrake,
    CarGearUpDown,
    CarGearUp,
    CarGearDown,
    CarRearView,
    CarNitrous,
    CarZoneMode,
    DebugFreeCamVert,
    DebugFreeCamHoriz,
    DebugFreeCamForwardBack,
    DebugFreeCamPanLeft,
    DebugFreeCamPanRight,
    DebugFreeCamMoveUp,
    DebugFreeCamMoveDown,
    DebugFreeCamPlay,
    DebugScreenshotButton1,
    DebugScreenshotButton2,
}

pub const NUM_ACTIONS: usize = 50;

pub const ACTION_NAMES: [&str; NUM_ACTIONS] = [
    "MenuLeft", "MenuRight", "MenuDown", "MenuUp", "MenuUpDown", "MenuLeftRight", "MenuSelect", "MenuCancel",
    "AlloyEditHoriz", "AlloyEditVert", "MouseLeftRight", "MouseUpDown", "MouseSelect", "Pause", "Online", "Skip",
    "MenuOption1", "MenuOption2", "MenuL1", "MenuL2", "MenuL3", "MenuR1", "MenuR2", "MenuR3", "CameraRotateHoriz",
    "CameraRotateVert", "CameraChange", "CarSteering", "CarSteerLeft", "CarSteerRight", "CarAccelerateBrake",
    "CarAccelerate", "CarBrake", "CarHandbrake", "CarGearUpDown", "CarGearUp", "CarGearDown", "CarRearView",
    "CarNitrous", "CarZoneMode", "DebugFreeCamVert", "DebugFreeCamHoriz", "DebugFreeCamForwardBack",
    "DebugFreeCamPanLeft", "DebugFreeCamPanRight", "DebugFreeCamMoveUp", "DebugFreeCamMoveDown", "DebugFreeCamPlay",
    "DebugScreenshotButton1", "DebugScreenshotButton2",
];

/// compiled binding expression
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Const(f32),
    Key(usize),
    Digital(Box<Expr>),
    DeadZone(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// InputManager.cpp 0x003D53B0 / 0x003D5000 — parse one expression.
    pub fn parse(s: &str) -> Result<Expr, String> {
        let (e, rest) = Self::parse_at(s.trim())?;
        if !rest.trim().is_empty() {
            return Err(format!("trailing text {rest:?}"));
        }
        Ok(e)
    }

    fn parse_at(s: &str) -> Result<(Expr, &str), String> {
        let s = s.trim_start();
        let n = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')).unwrap_or(s.len());
        let (tok, rest) = s.split_at(n);
        if tok.is_empty() {
            return Err(format!("expected expression at {s:?}"));
        }
        let rest = rest.trim_start();
        if let Some(mut r) = rest.strip_prefix('(') {
            let mut args = Vec::new();
            loop {
                r = r.trim_start();
                if let Some(rr) = r.strip_prefix(')') {
                    r = rr;
                    break;
                }
                let (a, rr) = Self::parse_at(r)?;
                args.push(a);
                r = rr.trim_start();
                if let Some(rr) = r.strip_prefix(',') {
                    r = rr;
                } else if let Some(rr) = r.strip_prefix(')') {
                    r = rr;
                    break;
                } else {
                    return Err("Expected ')'".into());
                }
            }
            let e = match (tok, args.len()) {
                ("DIGITAL", 1) => Expr::Digital(Box::new(args.remove(0))),
                ("DEADZONE", 2) => {
                    let d = args.remove(1);
                    Expr::DeadZone(Box::new(args.remove(0)), Box::new(d))
                }
                ("DIGITAL" | "DEADZONE", _) => return Err("parameter number mismatch".into()),
                _ => return Err("unknown function name".into()),
            };
            return Ok((e, r));
        }
        if let Ok(v) = tok.parse::<f32>() {
            return Ok((Expr::Const(v), rest));
        }
        match XBOXPAD_KEYS.iter().position(|k| k.eq_ignore_ascii_case(tok)) {
            Some(k) => Ok((Expr::Key(k), rest)),
            None => Err(format!("unknown key {tok}")),
        }
    }

    /// InputBinding.cpp 0x003D57E0
    pub fn eval(&self, keys: &[f32; 20]) -> f32 {
        match self {
            Expr::Const(c) => *c,
            Expr::Key(k) => keys[*k],
            Expr::Digital(x) => {
                let v = x.eval(keys);
                let a = v.abs();
                if a <= 0.0 { 0.0 } else { v / a }
            }
            Expr::DeadZone(x, d) => {
                let v = x.eval(keys);
                let d = d.eval(keys);
                let a = v.abs();
                let r = a - d;
                if r >= 0.0 { v / a * (r / (1.0 - d)) } else { 0.0 }
            }
        }
    }
}

/// one binding instance: expression, value, previous value
#[derive(Clone, Debug)]
pub struct InputBindingInstance {
    pub expr: Expr,
    pub value: f32,    // +4
    pub previous: f32, // +8
}

/// UserInput.cpp 0x001E7750 — parse a `Config*.ini`: tokens split on space/tab/newline, the two tokens after each
/// `Input.SetMapping` are action and expression. Later mappings of an action replace earlier ones.
pub fn parse_mapping_file(text: &str) -> Vec<(usize, Expr)> {
    let mut out: Vec<(usize, Expr)> = Vec::new();
    let mut toks = Vec::new();
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("");
        toks.extend(line.split([' ', '\t']).filter(|t| !t.is_empty()).map(str::to_owned));
    }
    let mut i = 0;
    while i < toks.len() {
        if toks[i] == "Input.SetMapping" && i + 2 < toks.len() {
            let (name, expr) = (&toks[i + 1], &toks[i + 2]);
            match (ACTION_NAMES.iter().position(|n| n == name), Expr::parse(expr)) {
                (Some(a), Ok(e)) => {
                    out.retain(|(x, _)| *x != a);
                    out.push((a, e));
                }
                (None, _) => log::debug!("mapping for unknown action {name}"),
                (_, Err(err)) => log::warn!("ERROR : Binding {name} invalid - {err} - binding={expr}"),
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    out
}

/// keyboard stand-in for an xbox pad (PC addition), indexed like `XBOXPAD_KEYS`: (key, value)
pub const KEYBOARD_KEYS: &[(KeyCode, usize, f32)] = &[
    (KeyCode::Enter, 0, 1.0),
    (KeyCode::Escape, 1, 1.0),
    (KeyCode::ArrowUp, 2, 1.0),
    (KeyCode::ArrowDown, 3, 1.0),
    (KeyCode::ArrowLeft, 4, 1.0),
    (KeyCode::ArrowRight, 5, 1.0),
    (KeyCode::Space, 6, 1.0),
    (KeyCode::Backspace, 7, 1.0),
    (KeyCode::KeyX, 8, 1.0),
    (KeyCode::KeyY, 9, 1.0),
    (KeyCode::KeyC, 10, 1.0),
    (KeyCode::KeyV, 11, 1.0),
    (KeyCode::KeyS, 12, 1.0),
    (KeyCode::KeyW, 13, 1.0),
    (KeyCode::KeyA, 14, -1.0),
    (KeyCode::KeyD, 14, 1.0),
    (KeyCode::KeyI, 15, 1.0),
    (KeyCode::KeyK, 15, -1.0),
    (KeyCode::KeyJ, 16, -1.0),
    (KeyCode::KeyL, 16, 1.0),
    (KeyCode::PageUp, 17, 1.0),
    (KeyCode::PageDown, 17, -1.0),
];

/// `UserInput` (global at 0x0074B4B0 area): bindings per device, the active device count.
#[derive(Resource, Debug, Default)]
pub struct UserInput {
    /// +0x00 connected devices
    pub num_connected: u32,
    /// per device: binding per action (None = unmapped)
    pub bindings: Vec<Vec<Option<InputBindingInstance>>>,
    /// per device: current key values
    pub keys: Vec<[f32; 20]>,
    /// +0x28/+0x29: input blocked (menus, pause)
    pub blocked: bool,
    /// +0x14 per device: the `Config<n+1>.ini` mapping index (UserInput.cpp 0x001E7140 reads it)
    pub config: Vec<u8>,
}

impl UserInput {
    /// UserInput.cpp 0x001E6130 + 0x001E73D0 — `devices` copies of the mapping in `Input\Xboxpad\Config1.ini`.
    pub fn load(devices: usize) -> Self {
        let mapping = visos::file::platform::read("\\Input\\Xboxpad\\Config1.ini")
            .map(|b| parse_mapping_file(&String::from_utf8_lossy(&b)))
            .unwrap_or_else(|_| {
                log::warn!("no Input\\Xboxpad\\Config1.ini");
                Vec::new()
            });
        let mut table: Vec<Option<InputBindingInstance>> = vec![None; NUM_ACTIONS];
        for (a, e) in mapping {
            table[a] = Some(InputBindingInstance { expr: e, value: 0.0, previous: 0.0 });
        }
        UserInput { num_connected: 0, bindings: vec![table; devices], keys: vec![[0.0; 20]; devices], blocked: false, config: vec![0; devices] }
    }

    /// UserInput +0x14 per pad: the mapping in `Input\Xboxpad\Config<index + 1>.ini` for `device` (only Config1
    /// ships; a missing file keeps the current mapping)
    pub fn set_config(&mut self, device: usize, index: u8) {
        let Some(table) = self.bindings.get_mut(device) else { return };
        if let Some(c) = self.config.get_mut(device) {
            *c = index;
        }
        let path = format!(r"\Input\Xboxpad\Config{}.ini", index as u32 + 1);
        let Ok(b) = visos::file::platform::read(&path) else {
            log::warn!("no {path}");
            return;
        };
        table.iter_mut().for_each(|b| *b = None);
        for (a, e) in parse_mapping_file(&String::from_utf8_lossy(&b)) {
            table[a] = Some(InputBindingInstance { expr: e, value: 0.0, previous: 0.0 });
        }
    }

    /// UserInput.cpp 0x001E6680 — evaluate every binding from the current key values.
    pub fn update(&mut self) {
        for (d, table) in self.bindings.iter_mut().enumerate() {
            let keys = self.keys[d];
            for b in table.iter_mut().flatten() {
                b.previous = b.value;
                b.value = b.expr.eval(&keys);
            }
        }
    }

    fn each(&self, action: EAction, mask: u32) -> impl Iterator<Item = &InputBindingInstance> {
        self.bindings
            .iter()
            .enumerate()
            .filter(move |(d, _)| *d < 32 && mask & (1 << d) != 0)
            .filter_map(move |(_, t)| t[action as usize].as_ref())
    }

    /// UserInput.cpp 0x001E6BC0 — the largest-magnitude value over the devices in `mask`.
    pub fn value(&self, action: EAction, mask: u32) -> f32 {
        let mut v = 0.0f32;
        for b in self.each(action, mask) {
            if b.value.abs() > v.abs() {
                v = b.value;
            }
        }
        v
    }

    /// UserInput.cpp 0x001E6D10 — same over the previous values.
    pub fn previous(&self, action: EAction, mask: u32) -> f32 {
        let mut v = 0.0f32;
        for b in self.each(action, mask) {
            if b.previous.abs() > v.abs() {
                v = b.previous;
            }
        }
        v
    }

    /// UserInput.cpp 0x001E6DF0 — the value changed this update on any device in `mask`.
    pub fn changed(&self, action: EAction, mask: u32) -> bool {
        self.each(action, mask).any(|b| b.previous != b.value)
    }

    /// the idiom the game uses for a button press: non-zero and changed.
    pub fn pressed(&self, action: EAction, mask: u32) -> bool {
        self.value(action, mask) != 0.0 && self.changed(action, mask)
    }
}

/// sample Bevy gamepads (device 0..n-1) and the keyboard (device n) into the key tables.
/// InputManager::Update (visos, SDL3 pads) plus the keyboard device, copied into the binding layer's key table.
pub fn sample_devices(
    mut ui: ResMut<UserInput>,
    mut manager: NonSendMut<InputManager>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
) {
    // hidden / test runs read scripted input only
    if let Some(kb) = keyboard.filter(|_| !visos::input::raw_input::hidden_run()) {
        let mut k = [0.0f32; 20];
        for &(code, key, v) in KEYBOARD_KEYS {
            if kb.pressed(code) {
                k[key] += v;
            }
        }
        visos::input::input_manager::set_keyboard(k);
    }
    manager.update();
    let snap = visos::input::input_manager::snapshot();
    let n = ui.keys.len().min(NUM_DEVICES);
    ui.keys[..n].copy_from_slice(&snap.keys[..n]);
    ui.num_connected = snap.connected.iter().filter(|c| **c).count() as u32;
}

/// devices: 4 pad ports + keyboard
pub const NUM_DEVICES: usize = visos::input::input_manager::NUM_DEVICES;
/// mask selecting every device
pub const ALL_DEVICES: u32 = (1 << NUM_DEVICES) - 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config1() {
        if !visos::file::game_dir().exists() {
            return;
        }
        let ui = UserInput::load(1);
        let t = &ui.bindings[0];
        assert!(t[EAction::Skip as usize].is_some());
        assert_eq!(t[EAction::CarHandbrake as usize].as_ref().unwrap().expr, Expr::Key(6));
        assert!(t[EAction::CarGearUp as usize].is_none());
        let e = Expr::parse("DIGITAL(DEADZONE(STICK_LY,0.6))").unwrap();
        let mut k = [0.0; 20];
        k[15] = -0.7;
        assert_eq!(e.eval(&k), -1.0);
        k[15] = 0.5;
        assert_eq!(e.eval(&k), 0.0);
        let d = Expr::parse("DEADZONE(STICK_LX,0.3)").unwrap();
        k[14] = 0.65;
        assert!((d.eval(&k) - 0.5).abs() < 1e-6);
    }
}
