//! InputManager.cpp — device enumeration and per-frame sampling. The xbox had four pad ports; here they are the
//! first four SDL gamepads (in connection order), plus a keyboard device after them (PC addition; its key values
//! are supplied by the windowing layer through `set_keyboard`).
//!
//! The latest sample is kept in a process-wide snapshot so every reader sees the same frame.

use std::ffi::CStr;
use std::sync::Mutex;

use super::input_device::{key, DeviceKeys, NUM_KEYS};

/// four pad ports + keyboard
pub const NUM_DEVICES: usize = 5;
pub const KEYBOARD_DEVICE: usize = 4;

#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    pub keys: [DeviceKeys; NUM_DEVICES],
    pub connected: [bool; NUM_DEVICES],
}

static SNAPSHOT: Mutex<Snapshot> = Mutex::new(Snapshot { keys: [[0.0; NUM_KEYS]; NUM_DEVICES], connected: [false; NUM_DEVICES] });
/// scripted pads (tests / `--slice-test`): each replaces its port while set
static SCRIPT: Mutex<[Option<DeviceKeys>; 4]> = Mutex::new([None; 4]);

/// the current sample.
pub fn snapshot() -> Snapshot {
    *SNAPSHOT.lock().unwrap()
}

/// value of one key of one device.
pub fn key_value(device: usize, k: usize) -> f32 {
    SNAPSHOT.lock().unwrap().keys[device][k]
}

/// strongest value of a key over every device (single-player readers such as the frontend's player 1).
pub fn any_key_value(k: usize) -> f32 {
    let s = SNAPSHOT.lock().unwrap();
    s.keys.iter().map(|d| d[k]).fold(0.0, |a, b| if b.abs() > a.abs() { b } else { a })
}

/// scripted port 0 (None = real pads again); applied at the next `update`.
pub fn set_script(keys: Option<DeviceKeys>) {
    set_port_script(0, keys);
}

/// scripted pad on port `port` (0..3; None = the real pad again); applied at the next `update`.
pub fn set_port_script(port: usize, keys: Option<DeviceKeys>) {
    if let Some(s) = SCRIPT.lock().unwrap().get_mut(port) {
        *s = keys;
    }
}

/// requested rumble per pad port: [light (motor 0), heavy (motor 1)] in 0..1, sent to the pad at each `update`
static RUMBLE: Mutex<[[f32; 2]; 4]> = Mutex::new([[0.0; 2]; 4]);

/// UserInput.cpp 0x001E69F0 SetVibration (the pad's two motors): the xbox pad's light motor is
/// SDL's high-frequency one, the heavy motor the low-frequency one
pub fn set_rumble(port: usize, motor: usize, value: f32) {
    if let Some(r) = RUMBLE.lock().unwrap().get_mut(port) {
        if motor < 2 {
            r[motor] = value.clamp(0.0, 1.0);
        }
    }
}

/// keyboard device values for this frame.
pub fn set_keyboard(keys: DeviceKeys) {
    let mut s = SNAPSHOT.lock().unwrap();
    s.keys[KEYBOARD_DEVICE] = keys;
    s.connected[KEYBOARD_DEVICE] = true;
}

/// SDL3 gamepad backend. Not `Send`: create and poll it on one thread.
pub struct InputManager {
    slots: PadSlots,
    ok: bool,
}

fn hint(name: &CStr, value: &CStr) {
    unsafe {
        sdl3_sys::hints::SDL_SetHintWithPriority(name.as_ptr(), value.as_ptr(), sdl3_sys::hints::SDL_HINT_OVERRIDE);
    }
}

impl InputManager {
    /// InputManager::Init — SDL gamepad subsystem. Steam hands every game a device ignore list (even with Steam
    /// Input off), so it is cleared, and Steam's virtual pad is refused so a pad cannot appear twice.
    pub fn new() -> Self {
        // hidden / test runs (`VELOCITY_HIDDEN`, set by tools/hidden_run.py): SDL is never initialised (no
        // joystick raw input, HIDAPI, rumble or background events); only the scripted ports feed the snapshot
        if super::raw_input::hidden_run() {
            log::info!("input: hidden run, SDL not initialised (scripted input only)");
            return InputManager { slots: PadSlots::default(), ok: false };
        }
        hint(c"SDL_GAMECONTROLLER_IGNORE_DEVICES", c"");
        hint(c"SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT", c"");
        hint(c"SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD", c"0");
        hint(c"SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", c"1");
        let ok = unsafe { sdl3_sys::init::SDL_Init(sdl3_sys::init::SDL_INIT_GAMEPAD) };
        if !ok {
            let e = unsafe { CStr::from_ptr(sdl3_sys::error::SDL_GetError()) };
            log::error!("SDL_Init(GAMEPAD) failed: {}", e.to_string_lossy());
        }
        InputManager { slots: PadSlots::default(), ok }
    }

    /// InputManager::Update — open new pads, drop removed ones, sample the four ports into the snapshot.
    pub fn update(&mut self) {
        use sdl3_sys::gamepad::*;
        let mut keys = [[0.0f32; NUM_KEYS]; 4];
        let mut connected = [false; 4];
        if self.ok {
            unsafe {
                use sdl3_sys::events::*;
                SDL_PumpEvents();
                sdl3_sys::joystick::SDL_UpdateJoysticks();
                SDL_UpdateGamepads();
                // drain the queue (added / removed events only wake the rescan below; the rescan is the
                // source of truth so a missed event cannot lose a pad)
                let mut ev: SDL_Event = std::mem::zeroed();
                while SDL_PollEvent(&mut ev) {
                    let t = ev.r#type;
                    if t == SDL_EVENT_GAMEPAD_ADDED.0 as u32
                        || t == SDL_EVENT_GAMEPAD_REMOVED.0 as u32
                        || t == SDL_EVENT_JOYSTICK_ADDED.0 as u32
                    {
                        log::debug!("input: SDL device event {t:#x}");
                    }
                }
                // removed: closed here, the slot stays reserved (ports do not shift, like the xbox's)
                for port in 0..self.slots.slots.len() {
                    if let Some((_, g)) = self.slots.slots[port] {
                        if !SDL_GamepadConnected(g as *mut SDL_Gamepad) {
                            SDL_CloseGamepad(g as *mut SDL_Gamepad);
                            let id = self.slots.remove_port(port);
                            log::info!("pad {port} disconnected (id {id:?})");
                        }
                    }
                }
                let mut n = 0;
                let ids = SDL_GetGamepads(&mut n);
                if !ids.is_null() {
                    let present: Vec<u32> = (0..n.max(0) as usize).map(|i| (*ids.add(i)).0).collect();
                    for id in present {
                        if self.slots.has(id) || self.slots.free_port().is_none() {
                            continue;
                        }
                        let g = SDL_OpenGamepad(sdl3_sys::joystick::SDL_JoystickID(id));
                        if !g.is_null() {
                            if let Some(port) = self.slots.add(id, g as usize) {
                                log::info!("pad {port} connected (id {id})");
                            }
                        }
                    }
                    sdl3_sys::stdinc::SDL_free(ids.cast());
                }
                for (port, slot) in self.slots.slots.iter().enumerate() {
                    let Some(&(_, g)) = slot.as_ref() else { continue };
                    let g = g as *mut SDL_Gamepad;
                    let b = |x: SDL_GamepadButton| if SDL_GetGamepadButton(g, x) { 1.0 } else { 0.0 };
                    let a = |x: SDL_GamepadAxis| SDL_GetGamepadAxis(g, x) as f32 / 32767.0;
                    let k = &mut keys[port];
                    k[key::START] = b(SDL_GAMEPAD_BUTTON_START);
                    k[key::BACK] = b(SDL_GAMEPAD_BUTTON_BACK);
                    k[key::UP] = b(SDL_GAMEPAD_BUTTON_DPAD_UP);
                    k[key::DOWN] = b(SDL_GAMEPAD_BUTTON_DPAD_DOWN);
                    k[key::LEFT] = b(SDL_GAMEPAD_BUTTON_DPAD_LEFT);
                    k[key::RIGHT] = b(SDL_GAMEPAD_BUTTON_DPAD_RIGHT);
                    k[key::A] = b(SDL_GAMEPAD_BUTTON_SOUTH);
                    k[key::B] = b(SDL_GAMEPAD_BUTTON_EAST);
                    k[key::X] = b(SDL_GAMEPAD_BUTTON_WEST);
                    k[key::Y] = b(SDL_GAMEPAD_BUTTON_NORTH);
                    // the xbox pad's black / white buttons sit where the shoulder buttons are now
                    k[key::BLACK] = b(SDL_GAMEPAD_BUTTON_RIGHT_SHOULDER);
                    k[key::WHITE] = b(SDL_GAMEPAD_BUTTON_LEFT_SHOULDER);
                    k[key::LTRIGGER] = a(SDL_GAMEPAD_AXIS_LEFT_TRIGGER).clamp(0.0, 1.0);
                    k[key::RTRIGGER] = a(SDL_GAMEPAD_AXIS_RIGHT_TRIGGER).clamp(0.0, 1.0);
                    k[key::STICK_LX] = a(SDL_GAMEPAD_AXIS_LEFTX).clamp(-1.0, 1.0);
                    // SDL y grows downwards, the xbox pad's upwards
                    k[key::STICK_LY] = -a(SDL_GAMEPAD_AXIS_LEFTY).clamp(-1.0, 1.0);
                    k[key::STICK_RX] = a(SDL_GAMEPAD_AXIS_RIGHTX).clamp(-1.0, 1.0);
                    k[key::STICK_RY] = -a(SDL_GAMEPAD_AXIS_RIGHTY).clamp(-1.0, 1.0);
                    k[key::STICK_LBUTTON] = b(SDL_GAMEPAD_BUTTON_LEFT_STICK);
                    k[key::STICK_RBUTTON] = b(SDL_GAMEPAD_BUTTON_RIGHT_STICK);
                    connected[port] = true;
                    let r = RUMBLE.lock().unwrap()[port];
                    // 100 ms leases: a lost game thread cannot leave the motors running
                    SDL_RumbleGamepad(g, (r[1] * 65535.0) as u16, (r[0] * 65535.0) as u16, 100);
                }
            }
        }
        for (port, s) in SCRIPT.lock().unwrap().iter().enumerate() {
            if let Some(s) = s {
                keys[port] = *s;
                connected[port] = true;
            }
        }
        let mut snap = SNAPSHOT.lock().unwrap();
        snap.keys[..4].copy_from_slice(&keys);
        snap.connected[..4].copy_from_slice(&connected);
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        for &(_, g) in self.slots.slots.iter().flatten() {
            unsafe { sdl3_sys::gamepad::SDL_CloseGamepad(g as *mut sdl3_sys::gamepad::SDL_Gamepad) };
        }
    }
}

/// Four fixed pad ports: a new device takes the lowest free port, a removed one frees only its own port.
#[derive(Default)]
pub struct PadSlots {
    /// (SDL instance id, handle)
    pub slots: [Option<(u32, usize)>; 4],
}

impl PadSlots {
    pub fn has(&self, id: u32) -> bool {
        self.slots.iter().flatten().any(|s| s.0 == id)
    }
    pub fn free_port(&self) -> Option<usize> {
        self.slots.iter().position(Option::is_none)
    }
    pub fn add(&mut self, id: u32, handle: usize) -> Option<usize> {
        if self.has(id) {
            return None;
        }
        let p = self.free_port()?;
        self.slots[p] = Some((id, handle));
        Some(p)
    }
    pub fn remove_port(&mut self, port: usize) -> Option<u32> {
        self.slots[port].take().map(|s| s.0)
    }
    /// reconcile with the present device ids: returns (removed ports, newly added ports) without opening anything
    pub fn sync(&mut self, present: &[u32], mut open: impl FnMut(u32) -> Option<usize>) -> (Vec<usize>, Vec<usize>) {
        let mut gone = Vec::new();
        for p in 0..4 {
            if let Some((id, _)) = self.slots[p] {
                if !present.contains(&id) {
                    self.slots[p] = None;
                    gone.push(p);
                }
            }
        }
        let mut added = Vec::new();
        for &id in present {
            if self.has(id) || self.free_port().is_none() {
                continue;
            }
            if let Some(h) = open(id) {
                added.extend(self.add(id, h));
            }
        }
        (gone, added)
    }
}

#[cfg(test)]
mod pad_tests {
    use super::PadSlots;

    #[test]
    fn hotplug_slots() {
        let mut s = PadSlots::default();
        let open = |id: u32| Some(id as usize * 10);
        assert_eq!(s.sync(&[], open), (vec![], vec![]));
        assert_eq!(s.sync(&[5, 6], open), (vec![], vec![0, 1]));
        // pad 0 pulled: port 1 keeps its pad, a new pad takes the free port 0
        assert_eq!(s.sync(&[6], open), (vec![0], vec![]));
        assert_eq!(s.slots[1], Some((6, 60)));
        assert_eq!(s.sync(&[6, 9], open), (vec![], vec![0]));
        assert_eq!(s.slots[0], Some((9, 90)));
        // reconnect with a new instance id; five pads, fifth ignored until a port frees
        assert_eq!(s.sync(&[6, 9, 10, 11, 12], open), (vec![], vec![2, 3]));
        assert!(s.free_port().is_none());
        assert_eq!(s.sync(&[6, 10, 11, 12], open), (vec![0], vec![0]));
        assert_eq!(s.slots[0], Some((12, 120)));
        // failed open leaves the port free
        let mut t = PadSlots::default();
        assert_eq!(t.sync(&[1], |_| None), (vec![], vec![]));
        assert!(t.slots.iter().all(Option::is_none));
    }
}
