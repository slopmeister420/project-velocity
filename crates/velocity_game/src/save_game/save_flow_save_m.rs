//! SaveGame\Xbox\_SAVE_Flow_Save_M.cpp — the save flow: start (0x002FFD30; 0x00300680 = without the overwrite
//! question, 0x002FFD00 = with it) and the node table `unk_6C1FD0` (node functions 0x002FEC80..0x002FFC20).
//! The memory unit is a folder under the save root: no free-block check (NoSpace / Reboot are never entered), no
//! save image (`saveimage.xbx`); the write steps and the minimum message times are kept.

use std::path::{Path, PathBuf};

use super::save_vars::{status, SaveFlow, SaveNode, SaveStatus};
use crate::register_file_types::save_file_data::write_atomic;

/// node crcs
pub const CONFIRM_OVERWRITE: u32 = 0x80C5_3F3C;
pub const SAVING: u32 = 0x0FB2_9999;
pub const SAVE_COMPLETE: u32 = 0xE92F_9D2C;
pub const ERROR_SAVING: u32 = 0xF678_E1D7;
pub const NO_SPACE: u32 = 0xC307_AA88;
pub const REBOOT: u32 = 0x773D_2019;
pub const EXIT: u32 = 0x157F_F42C;
pub const USER_EXIT: u32 = 0x0E12_EE7E;

/// string / response crcs (CCRCChecked values in the node functions)
pub mod ids {
    pub const CONFIRM_OVERWRITE_TITLE: u32 = -512332090i32 as u32;
    pub const CONFIRM_OVERWRITE: u32 = -1394347401i32 as u32;
    pub const CONFIRM_OVERWRITE_YES: u32 = -887175720i32 as u32;
    pub const NO_OPTION: u32 = 1065522401; // SaveGame_No
    pub const OVERWRITE: u32 = 1329141259;
    pub const NO: u32 = -1275650494i32 as u32;
    pub const BEGIN_SAVE_TITLE: u32 = 705598750;
    pub const BEGIN_SAVE: u32 = -615206347i32 as u32;
    pub const SAVE_COMPLETE_TITLE: u32 = 550464542;
    pub const SAVE_COMPLETE: u32 = 566515642;
    pub const ERROR_TITLE: u32 = 341603372;
    pub const ERROR: u32 = 707860865;
    pub const OK_OPTION: u32 = 446316913; // SaveGame_OK
    pub const OK: u32 = -1762669614i32 as u32;
    pub const NO_SPACE_TITLE: u32 = 249808585;
    pub const NO_SPACE: u32 = 1645191075;
    pub const NO_SPACE_CONTINUE: u32 = -1163796597i32 as u32;
    pub const NO_SPACE_FREE: u32 = -131550372i32 as u32;
    pub const CONTINUE: u32 = -1883793440i32 as u32;
    pub const FREE: u32 = -866602518i32 as u32;
}

/// `unk_6C1FD0`
pub static SAVE_TABLE: [SaveNode; 8] = [
    SaveNode { crc: CONFIRM_OVERWRITE, func: confirm_overwrite, time_ms: 0, next: [SAVING, USER_EXIT, 0, 0, 0, 0] },
    SaveNode { crc: SAVING, func: saving, time_ms: 3000, next: [SAVE_COMPLETE, ERROR_SAVING, NO_SPACE, 0, 0, 0] },
    SaveNode { crc: SAVE_COMPLETE, func: save_complete, time_ms: 3000, next: [EXIT, 0, 0, 0, 0, 0] },
    SaveNode { crc: ERROR_SAVING, func: error_saving, time_ms: 0, next: [USER_EXIT, 0, 0, 0, 0, 0] },
    SaveNode { crc: NO_SPACE, func: no_space, time_ms: 0, next: [USER_EXIT, REBOOT, 0, 0, 0, 0] },
    SaveNode { crc: REBOOT, func: reboot, time_ms: 0, next: [0; 6] },
    SaveNode { crc: EXIT, func: exit, time_ms: 0, next: [0; 6] },
    SaveNode { crc: USER_EXIT, func: user_exit, time_ms: 0, next: [0; 6] },
];

impl SaveFlow {
    /// _SAVE_Flow_Save_M.cpp 0x002FFD30 — start saving `data` to `path`; `no_confirm` = 0x00300680's 1 (an existing
    /// save is overwritten without the question). A new container goes straight to Saving. The port writes as
    /// soon as Saving is entered, so the file is on disk even if nobody polls.
    pub fn start_save(&mut self, path: PathBuf, data: Vec<u8>, no_confirm: bool) {
        if self.in_progress() {
            log::error!("assert: SAVE_IsSaveOperationInProgress() == false");
        }
        let exists = path.is_file();
        self.table = Some(&SAVE_TABLE);
        self.node = None;
        self.pending = 0;
        self.path = path;
        self.data = data;
        self.written = None;
        if exists && !no_confirm {
            self.set_node(CONFIRM_OVERWRITE);
        } else {
            self.set_node(SAVING);
            self.write_now();
        }
    }

    fn write_now(&mut self) {
        if self.written.is_none() {
            let r = write_atomic(Path::new(&self.path), &self.data);
            if let Err(e) = &r {
                log::error!("save: {}: {e}", self.path.display());
            }
            self.written = Some(r.is_ok());
        }
    }
}

fn answer(f: &SaveFlow, a: u32, b: u32, line: u32) -> i32 {
    if f.response == 0 {
        return 0;
    }
    if f.response == a {
        return 1;
    }
    if b != 0 && f.response == b {
        return 2;
    }
    log::error!("assert: false && \"Unrecognised response\" (_SAVE_Flow_Save_M.cpp {line})");
    0
}

fn enter_leave(f: &mut SaveFlow, a2: i32) {
    match a2 {
        0 => f.set_entered(true),
        2 => f.set_entered(false),
        _ => {}
    }
}

/// 0x002FED20
fn confirm_overwrite(f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::ConfirmOverwrite";
    if a2 != 1 {
        enter_leave(f, a2);
        return 0;
    }
    s.status = status::QUESTION;
    s.flags = 931;
    s.title = ids::CONFIRM_OVERWRITE_TITLE;
    s.text = ids::CONFIRM_OVERWRITE;
    s.options = [ids::CONFIRM_OVERWRITE_YES, ids::NO_OPTION, 0, 0];
    s.responses = [ids::OVERWRITE, ids::NO, 0, 0];
    s.english = [
        Some("Are you sure?"),
        Some("A previous save game file already exists.\nAre you sure you want to overwrite it?"),
        Some("Overwrite"),
        Some("No"),
    ];
    answer(f, ids::OVERWRITE, ids::NO, 132)
}

/// 0x002FF030 — four write steps (data file open + write, its completion, save image, its completion)
fn saving(f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::Saving";
    if a2 == 0 {
        f.phase = 0;
        return 0;
    }
    if a2 != 1 {
        return 0;
    }
    s.status = status::POPUP;
    s.flags = 128;
    s.title = ids::BEGIN_SAVE_TITLE;
    s.text = ids::BEGIN_SAVE;
    s.english = [Some("Saving Data..."), Some("Saving data.\nDo not switch off your Xbox console."), None, None];
    match f.phase {
        0 => {
            f.write_now();
            if f.written != Some(true) {
                return 2;
            }
            f.phase = 1;
            0
        }
        1 => {
            f.phase = 2;
            0
        }
        // saveimage.xbx: no image in the port
        2 => {
            f.phase = 3;
            0
        }
        3 => 1,
        _ => 0,
    }
}

/// 0x002FF570
fn save_complete(_f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::SaveComplete";
    if a2 == 1 {
        s.status = status::POPUP;
        s.flags = 128;
        s.title = ids::SAVE_COMPLETE_TITLE;
        s.text = ids::SAVE_COMPLETE;
        s.english = [Some("Done..."), Some("Save Successful."), None, None];
    }
    1
}

/// 0x002FF6B0
fn error_saving(f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::ErrorSaving";
    if a2 != 1 {
        enter_leave(f, a2);
        return 0;
    }
    s.status = status::QUESTION;
    s.flags = 401;
    s.title = ids::ERROR_TITLE;
    s.text = ids::ERROR;
    s.options = [ids::OK_OPTION, 0, 0, 0];
    s.responses = [ids::OK, 0, 0, 0];
    s.english = [Some("Error..."), Some("Save failed!"), Some("OK"), None];
    answer(f, ids::OK, 0, 385)
}

/// 0x002FF910 (never entered: no free-block check)
fn no_space(f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::NoSpace";
    if a2 != 1 {
        enter_leave(f, a2);
        return 0;
    }
    s.status = status::QUESTION;
    s.flags = 419;
    s.title = ids::NO_SPACE_TITLE;
    s.text = ids::NO_SPACE;
    s.options = [ids::NO_SPACE_CONTINUE, ids::NO_SPACE_FREE, 0, 0];
    s.responses = [ids::CONTINUE, ids::FREE, 0, 0];
    s.english = [
        Some("Error..."),
        Some("Your Xbox does not have enough free blocks to save games.\nYou need to free (x) more blocks."),
        Some("Continue without saving"),
        Some("Free more blocks"),
    ];
    answer(f, ids::CONTINUE, ids::FREE, 447)
}

/// 0x002FFC20 — XLaunchNewImage to the dashboard's memory area (not in the port)
fn reboot(_f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::Reboot";
    if a2 == 0 {
        log::warn!("save: Reboot node (dashboard launch) not ported");
    }
    1
}

/// 0x002FEC80
fn exit(f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::Exit";
    if a2 == 0 {
        f.data = Vec::new();
    }
    s.status = status::DONE;
    0
}

/// 0x002FECD0
fn user_exit(f: &mut SaveFlow, s: &mut SaveStatus, a2: i32) -> i32 {
    s.name = "Save::UserExit";
    if a2 == 0 {
        f.data = Vec::new();
    }
    s.status = status::USER_EXIT;
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use visos::handy::handy_crc::crc32;

    #[test]
    fn crcs() {
        for (s, c) in [
            ("Saving", SAVING),
            ("ConfirmOverwrite", CONFIRM_OVERWRITE),
            ("NoSpace", NO_SPACE),
            ("ErrorSaving", ERROR_SAVING),
            ("SaveGame_Save_BeginSave", ids::BEGIN_SAVE),
            ("SaveGame_Save_SaveComplete_Title", ids::SAVE_COMPLETE_TITLE),
            ("Overwrite", ids::OVERWRITE),
            ("OK", ids::OK),
        ] {
            assert_eq!(crc32(s.as_bytes(), 0), c, "{s}");
        }
    }

    fn run(f: &mut SaveFlow, dt: f32, n: usize) -> Vec<i32> {
        (0..n).map(|_| f.poll(dt).status).collect()
    }

    /// after-race save (0x00300680): Saving popup over the four write steps, 3 s idle (status 0), SaveComplete
    /// popup once, 3 s idle, Exit = done; the table is reset after done
    #[test]
    fn after_race_save_popups() {
        let dir = std::env::temp_dir().join(format!("velocity_saveflow_{}", std::process::id()));
        let p = dir.join("P").join("Primary");
        let mut f = SaveFlow::default();
        f.start_save(p.clone(), vec![1, 2, 3], true);
        assert_eq!(f.written(), Some(true));
        assert_eq!(std::fs::read(&p).unwrap(), vec![1, 2, 3]);
        assert_eq!(run(&mut f, 0.5, 4), vec![2, 2, 2, 2]);
        // the 3 s run from the node's start: 4 polls of 0.5 s already passed
        assert_eq!(run(&mut f, 0.5, 3), vec![0, 0, 0]);
        assert_eq!(f.poll(0.5).status, 2);
        // SaveComplete's 3 s from its start (0.5 s in): the third 1 s poll enters Exit
        assert_eq!(run(&mut f, 1.0, 3), vec![0, 0, 0]);
        assert_eq!(f.poll(1.0).status, 3);
        assert!(!f.in_progress());
        assert_eq!(f.poll(1.0).status, 0);
        // profile save (0x002FFD00) over an existing one asks first; No leaves
        f.start_save(p.clone(), vec![9], false);
        let s = f.poll(0.1);
        assert_eq!((s.status, s.responses[1]), (1, ids::NO));
        f.respond(ids::NO);
        assert_eq!(f.poll(0.1).status, 1);
        assert_eq!(f.poll(0.1).status, 0);
        assert_eq!(f.poll(0.1).status, -1);
        assert_eq!(std::fs::read(&p).unwrap(), vec![1, 2, 3]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
