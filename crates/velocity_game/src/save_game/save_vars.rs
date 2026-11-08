//! SaveGame\_SAVE_vars.cpp (0x00304D80..) and the poll in Memory\XBox\XMem.cpp (0x0030EF50 / 0x0030EF60): the
//! current save flow table, its current node and the status block (`dword_7B2D50`, 84 bytes) the screens read.
//! A node is `{crc, function, minimum time ms, next node crc per return value 1..6}` (36 bytes); the function is
//! called with 0 when the node is entered, 1 every poll (fills the status, returns the next-node index, 0 = stay)
//! and 2 when it is left.

use std::path::PathBuf;

use bevy::prelude::*;

/// `dword_7B2D50` status words: `status` (+0)
pub mod status {
    /// question dialog (options `(flags & 0x70) >> 4`, default `(flags & 0xE00) >> 9`), answer with `respond`
    pub const QUESTION: i32 = 1;
    /// message popup (title / text)
    pub const POPUP: i32 = 2;
    /// the flow finished
    pub const DONE: i32 = 3;
    /// left by the user (or after an error message)
    pub const USER_EXIT: i32 = -1;
}

/// the 84-byte status block `dword_7B2D50` (zeroed at the start of every poll)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaveStatus {
    pub status: i32,                       // +0x00
    pub flags: u32,                        // +0x04
    pub title: u32,                        // +0x08 string crc
    pub text: u32,                         // +0x0C string crc (args idGameTitle, nRequiredBlocks)
    pub options: [u32; 4],                 // +0x10 option string crcs
    pub responses: [u32; 4],               // +0x20 the answer crc of each option (`respond(responses[i])`)
    pub english: [Option<&'static str>; 4], // +0x38 title, text, option 0, option 1 fallbacks
    pub name: &'static str,                // +0x50 debug name of the node
}

/// one flow node (36 bytes in the original table)
pub struct SaveNode {
    pub crc: u32,
    pub func: fn(&mut SaveFlow, &mut SaveStatus, i32) -> i32,
    pub time_ms: u32,
    pub next: [u32; 6],
}

/// the save flow globals (`dword_7B2A74..dword_7B2A90`, `_SAVE_Flow_Save_M`'s `gs_*`)
#[derive(Resource, Default)]
pub struct SaveFlow {
    /// `g_pCurrentSaveTable` (dword_7B2A74)
    pub(crate) table: Option<&'static [SaveNode]>,
    /// current node (dword_7B2A78)
    pub(crate) node: Option<usize>,
    /// node to enter after the idle time (dword_7B2A80)
    pub(crate) pending: u32,
    /// the current node's minimum time in seconds (flt_7B2A7C)
    pub(crate) node_time: f32,
    /// time since the node was set (timer unk_7B2A90)
    pub(crate) timer: f32,
    /// the screen's answer (dword_7B2A88, XMem 0x0030EF50)
    pub(crate) response: u32,
    /// byte_7B2A84
    pub(crate) entered: bool,
    /// `_SAVE_Flow_Save_M`: target file, data (gs_pSaveData), write step (dword_7B2888), write result
    pub(crate) path: PathBuf,
    pub(crate) data: Vec<u8>,
    pub(crate) phase: u32,
    pub(crate) written: Option<bool>,
}

impl SaveFlow {
    /// `SAVE_IsSaveOperationInProgress` (0x00304D60)
    pub fn in_progress(&self) -> bool {
        self.table.is_some()
    }

    /// the write result of the last flow (None while nothing was written)
    pub fn written(&self) -> Option<bool> {
        self.written
    }

    /// _SAVE_vars.cpp 0x00304D80 — SAVE_SetCurrentSaveNode
    pub fn set_node(&mut self, crc: u32) {
        let Some(t) = self.table else {
            log::error!("assert: g_pCurrentSaveTable != NULL");
            return;
        };
        let Some(i) = t.iter().position(|n| n.crc == crc) else {
            log::error!("assert: false && \"SAVE_SetCurrentSaveNode::Failed to locate node in flow table\"");
            return;
        };
        if self.node.is_none() {
            log::debug!("SaveGame::Setting New Node");
            (t[i].func)(self, &mut SaveStatus::default(), 0);
        }
        self.node = Some(i);
        self.node_time = (t[i].time_ms as f64 / 1000.0) as f32;
        self.timer = 0.0;
    }

    /// _SAVE_vars.cpp 0x00304EB0
    pub(crate) fn set_entered(&mut self, b: bool) {
        self.entered = b;
        self.response = 0;
    }

    /// XMem.cpp 0x0030EF50 — the screen's answer to a question (one of `SaveStatus::responses`)
    pub fn respond(&mut self, crc: u32) {
        self.response = crc;
    }

    /// XMem.cpp 0x0030EF60 — run the flow once (`dt` = the time since the last poll; the original reads a
    /// timer started when the node was set) and return the status block
    pub fn poll(&mut self, dt: f32) -> SaveStatus {
        let mut s = SaveStatus::default();
        let Some(t) = self.table else { return s };
        self.timer += dt;
        if let Some(i) = self.node {
            let v = (t[i].func)(self, &mut s, 1);
            let next = if (1..=6).contains(&v) { t[i].next[v as usize - 1] } else { 0 };
            if next != 0 {
                log::debug!("SaveGame::Shutting down node");
                (t[i].func)(self, &mut SaveStatus::default(), 2);
                self.node = None;
                self.pending = next;
            }
        } else if self.pending != 0 {
            s.name = "Save::Idling for message";
            if self.timer > self.node_time {
                self.set_node(self.pending);
            }
        } else {
            s.name = "Save::All Done";
        }
        if s.status == status::DONE || s.status <= -1 {
            log::debug!("SaveGame::Save operation completed - resetting table");
            self.table = None;
        }
        s
    }
}
