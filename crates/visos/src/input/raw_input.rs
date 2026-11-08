//! PC platform layer (no original counterpart): keep the process from holding any system-wide keyboard / mouse
//! input registration. The xbox game read pads only; keyboard keys reach the game through winit window messages
//! (WM_KEYDOWN, focused window only), which need no raw input. winit registers keyboard + mouse raw input
//! (RIDEV_DEVNOTIFY) on its event loop creation; this removes it again.

/// true when the exe runs under `tools/hidden_run.py` (it sets `VELOCITY_HIDDEN=1`): no SDL, no real devices,
/// scripted input only.
pub fn hidden_run() -> bool {
    static H: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *H.get_or_init(|| std::env::var_os("VELOCITY_HIDDEN").is_some_and(|v| !v.is_empty() && v != "0"))
}

#[cfg(windows)]
mod win {
    #[repr(C)]
    pub struct RawInputDevice {
        pub usage_page: u16,
        pub usage: u16,
        pub flags: u32,
        pub hwnd: *mut core::ffi::c_void,
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn RegisterRawInputDevices(devices: *const RawInputDevice, n: u32, size: u32) -> i32;
        pub fn GetRegisteredRawInputDevices(devices: *mut RawInputDevice, n: *mut u32, size: u32) -> u32;
    }
    pub const RIDEV_REMOVE: u32 = 0x1;
}

/// drop the process's raw input registration for the generic-desktop mouse (2) and keyboard (6) collections
/// (winit's `DeviceEvents::Never`), then return the number of raw input registrations left for any keyboard or
/// mouse (0 = the process cannot see keyboard / mouse input outside its own focused window). Other usages (SDL's
/// joystick / gamepad registrations) are untouched.
pub fn remove_keyboard_mouse() -> u32 {
    #[cfg(windows)]
    unsafe {
        use win::*;
        let size = core::mem::size_of::<RawInputDevice>() as u32;
        let rid = [2u16, 6].map(|usage| RawInputDevice { usage_page: 1, usage, flags: RIDEV_REMOVE, hwnd: core::ptr::null_mut() });
        if RegisterRawInputDevices(rid.as_ptr(), rid.len() as u32, size) == 0 {
            log::warn!("raw input: RegisterRawInputDevices(REMOVE) failed");
        }
        let mut list: Vec<RawInputDevice> = (0..16)
            .map(|_| RawInputDevice { usage_page: 0, usage: 0, flags: 0, hwnd: core::ptr::null_mut() })
            .collect();
        let mut n = list.len() as u32;
        let got = GetRegisteredRawInputDevices(list.as_mut_ptr(), &mut n, size);
        if got == u32::MAX {
            log::warn!("raw input: GetRegisteredRawInputDevices failed");
            return u32::MAX;
        }
        list.truncate(got as usize);
        // generic desktop: 2 mouse, 6 keyboard, 7 keypad; page 0x0D digitizer is not a keyboard
        let left = list.iter().filter(|d| d.usage_page == 1 && matches!(d.usage, 2 | 6 | 7)).count() as u32;
        log::info!(
            "raw input: registrations left {:?}, keyboard/mouse {left}",
            list.iter().map(|d| (d.usage_page, d.usage, d.flags)).collect::<Vec<_>>()
        );
        left
    }
    #[cfg(not(windows))]
    0
}
