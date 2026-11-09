//! Always-on crash notes (independent of `--log`): a panic hook and, on Windows, an unhandled-exception filter
//! append to `crash.log` in %LOCALAPPDATA%\Velocity (fallback: next to the exe). Release builds have no console
//! and no log by default, so without this a crash leaves no trace. A non-hidden release run also shows a dialog.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

const FILE: &str = "crash.log";

/// directories tried in order
fn candidate_dirs() -> Vec<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("Velocity"));
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    [local, exe].into_iter().flatten().collect()
}

/// UTC "YYYY-MM-DD HH:MM:SS" from the system clock
fn timestamp() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, rem) = ((secs / 86400) as i64, secs % 86400);
    // civil-from-days (Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC", rem / 3600, rem % 3600 / 60, rem % 60)
}

fn version() -> String {
    format!("Project Velocity {} ({})", env!("CARGO_PKG_VERSION"), option_env!("VELOCITY_GIT_HASH").unwrap_or("unknown"))
}

/// appends `body` as one timestamped entry to the first directory that works; returns the file written
fn append(dirs: &[PathBuf], body: &str) -> Option<PathBuf> {
    let entry = format!("==== {} | {} ====\n{body}\n\n", timestamp(), version());
    dirs.iter().find_map(|d| {
        let _ = std::fs::create_dir_all(d);
        let path = d.join(FILE);
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&path).ok()?;
        f.write_all(entry.as_bytes()).ok()?;
        let _ = f.flush();
        Some(path)
    })
}

fn panic_text(info: &std::panic::PanicHookInfo<'_>) -> String {
    let thread = std::thread::current();
    let loc = info.location().map_or("unknown".to_string(), |l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string payload>".to_string()
    };
    let bt = std::backtrace::Backtrace::force_capture();
    format!("panic on thread '{}'\nmessage: {msg}\nlocation: {loc}\nbacktrace:\n{bt}", thread.name().unwrap_or("<unnamed>"))
}

/// panic hook writing into `dirs`; `dialog` shows the release message box (once)
fn set_hook(dirs: Vec<PathBuf>, dialog: bool) {
    let prev = std::panic::take_hook();
    let shown = AtomicBool::new(false);
    std::panic::set_hook(Box::new(move |info| {
        let path = append(&dirs, &panic_text(info));
        prev(info);
        if dialog && !shown.swap(true, Ordering::SeqCst) {
            let at = path.map_or("(could not write crash.log)".to_string(), |p| p.display().to_string());
            show_dialog(&format!("Project Velocity crashed \u{2014} details saved to {at}"));
        }
    }));
}

/// installs the panic hook and the unhandled-exception filter; call first thing in `main`
pub fn install() {
    // hidden runs (tools/hidden_run.py): no dialog on the hidden desktop (it would play the error sound)
    let dialog = cfg!(not(debug_assertions)) && !visos::input::raw_input::hidden_run();
    set_hook(candidate_dirs(), dialog);
    #[cfg(windows)]
    seh::install();
}

fn show_dialog(text: &str) {
    #[cfg(not(debug_assertions))]
    crate::message_box(text);
    #[cfg(debug_assertions)]
    let _ = text;
}

#[cfg(windows)]
mod seh {
    use super::*;

    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut core::ffi::c_void,
        num_params: u32,
        info: [usize; 15],
    }
    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut core::ffi::c_void,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetUnhandledExceptionFilter(
            f: Option<unsafe extern "system" fn(*const ExceptionPointers) -> i32>,
        ) -> *mut core::ffi::c_void;
    }

    unsafe extern "system" fn filter(p: *const ExceptionPointers) -> i32 {
        // a panic already wrote its note; only native faults land here (access violation, stack overflow, ...)
        let mut text = String::from("unhandled exception");
        if let Some(r) = unsafe { p.as_ref().and_then(|p| p.record.as_ref()) } {
            text += &format!("\ncode: 0x{:08X}\naddress: {:p}", r.code, r.address);
            // access violation / in-page error: [0] = 0 read, 1 write, 8 execute; [1] = target address
            if (r.code == 0xC000_0005 || r.code == 0xC000_0006) && r.num_params >= 2 {
                let kind = match r.info[0] {
                    0 => "read",
                    1 => "write",
                    8 => "execute",
                    _ => "access",
                };
                text += &format!("\naccess: {kind} of 0x{:X}", r.info[1]);
            }
        }
        text += &format!("\nbacktrace:\n{}", std::backtrace::Backtrace::force_capture());
        let path = append(&candidate_dirs(), &text);
        if cfg!(not(debug_assertions)) && !visos::input::raw_input::hidden_run() {
            let at = path.map_or("(could not write crash.log)".to_string(), |p| p.display().to_string());
            show_dialog(&format!("Project Velocity crashed \u{2014} details saved to {at}"));
        }
        0 // EXCEPTION_CONTINUE_SEARCH: Windows error reporting still runs
    }

    pub fn install() {
        unsafe { SetUnhandledExceptionFilter(Some(filter)) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_hook_writes_crash_log() {
        let dir = std::env::temp_dir().join(format!("velocity_crash_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let orig = std::panic::take_hook();
        set_hook(vec![dir.clone()], false);
        let r = std::panic::catch_unwind(|| panic!("forced test panic"));
        // restore the harness hook (drops ours and the chained original)
        let _ = std::panic::take_hook();
        std::panic::set_hook(orig);
        assert!(r.is_err());
        let text = std::fs::read_to_string(dir.join(FILE)).expect("crash.log written");
        assert!(text.contains("forced test panic"), "{text}");
        assert!(text.contains("location:") && text.contains("crash_log.rs"), "{text}");
        assert!(text.contains("backtrace:") && text.contains("Project Velocity"), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn timestamp_is_plausible() {
        let t = timestamp();
        assert!(t.starts_with("20") && t.ends_with("UTC"), "{t}");
    }
}
