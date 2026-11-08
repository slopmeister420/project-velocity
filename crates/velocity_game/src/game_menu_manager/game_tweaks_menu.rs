//! src\GameMenuManager\Menus\GameTweaksMenu.cpp — `CGameTweakSystemMenu` (vtable 0x0062F0D0, 0x464 bytes): one menu
//! per tweak system (child systems, then its tweaks, then the two exports) and `CTweakableMenuItem` (vtable
//! 0x0062F288, 0xB0 bytes): a tweak's name and "value (default)", edited with left / right. Plus
//! GamePauseMenu.cpp 0x000AF970 `EnterTweaksMenu`, which re-enters the last path (`gs_tweakDepth`).

use crate::cli::tweak::{CTweakManager, TweakId};

use super::game_menu_manager::{CDebugMenuManager, CGameMenu, CGameMenuItem, DebugMenuText, MenuKind, MASK};
use crate::controller::user_input::{EAction, UserInput};

/// `gs_tweakDepth` stack size (`STACK_DEPTH`)
pub const STACK_DEPTH: usize = 16;
/// child system item ids: index | 0x10000
pub const CHILD_ID: i32 = 0x10000;
/// +1120 after a change (0x000B0190: 0x3E000000)
pub const TWEAK_REPEAT: f32 = f32::from_bits(0x3E00_0000);

impl CGameMenu {
    /// GameTweaksMenu.cpp 0x000AFAC0 — `title` None = the system's name
    pub fn tweak_system(tm: &CTweakManager, system: usize, title: Option<&str>) -> Self {
        let sys = &tm.systems[system];
        let mut m = CGameMenu::new(MenuKind::TweakSystem { system }, "");
        for (i, &c) in sys.children.iter().enumerate() {
            m.add(CGameMenuItem::new(&tm.systems[c].name, i as i32 | CHILD_ID));
        }
        for i in 0..sys.tweaks.len() {
            // 0x000B01C0: text = tweak +12 (name), id 0
            let mut it = CGameMenuItem::new(&sys.tweaks[i].name, 0);
            it.tweak = Some(TweakId { system, index: i });
            m.add(it);
        }
        m.add(CGameMenuItem::new("Export Tweaks to TTY", 1));
        m.add(CGameMenuItem::new("Export Tweaks to File", 2));
        m.set_name(title.unwrap_or(&sys.name));
        m.set_size(f32::from_bits(0x4489_8000), f32::from_bits(0x4416_0000));
        m.tweak_repeat = 0.0;
        m
    }
}

/// GameTweaksMenu.cpp 0x000B0A20 — the value text: "value (default)", peeks just the value
pub fn value_text(tm: &CTweakManager, id: TweakId) -> String {
    let t = tm.tweak(id);
    if t.read_only {
        t.to_string_value()
    } else {
        format!("{} ({})", t.to_string_value(), t.default_to_string())
    }
}

/// GameTweaksMenu.cpp 0x000B0270 — `CTweakableMenuItem` build at (x, y) with width `w`: the name, then the value
/// half way across (peeks) or 90 left of the right edge. The slider (float / integer tweaks: three bar graphics
/// and the knob at 0x000B0A20's ratio, texture `sliders`) is not drawn.
pub fn build_tweak_item(tm: &CTweakManager, item: &mut CGameMenuItem, x: f32, y: f32, w: f32, out: &mut Vec<DebugMenuText>) {
    let Some(id) = item.tweak else { return };
    let ro = tm.tweak(id).read_only;
    out.push(DebugMenuText::new(&item.text, trunc(x), trunc(y), 0, 2, MASK));
    let vx = if ro { w * 0.5 + x } else { x + w - 90.0 };
    item.value_text = value_text(tm, id);
    out.push(DebugMenuText::new(&item.value_text, trunc(vx), trunc(y), 0, 2, MASK));
    item.value_osd = Some(out.len() - 1);
}

/// `(float)(__int16)(int)f` (the OSD item positions)
pub fn trunc(f: f32) -> f32 {
    (f as i32) as i16 as f32
}

/// GameTweaksMenu.cpp 0x000B07E0 — the current tweak item's update: MenuRight / MenuLeft (press, or held once
/// the menu's 0.125 s timer ran out) step by 1, MenuLeftRight past ±0.1 steps by `ceil(v * 5)` when the timer
/// ran out. A change sends MessageManager channel 1 (`TweakChanged`), restarts the timer and refreshes the text.
/// returns the changed tweak.
pub fn update_tweak_item(tm: &mut CTweakManager, item: &mut CGameMenuItem, timer: &mut f32, ui: &UserInput) -> Option<TweakId> {
    let id = item.tweak?;
    if tm.tweak(id).read_only {
        return None;
    }
    let mask = 0xFFFF;
    let elapsed = *timer <= 0.0; // 0x000B0CF0
    let mut changed = false;
    let r = ui.value(EAction::MenuRight, mask);
    if r != 0.0 && (ui.changed(EAction::MenuRight, mask) || elapsed) {
        tm.tweak_mut(id).increment(1.0);
        changed = true;
    } else {
        let l = ui.value(EAction::MenuLeft, mask);
        if l != 0.0 && (ui.changed(EAction::MenuLeft, mask) || elapsed) {
            tm.tweak_mut(id).decrement(1.0);
            changed = true;
        } else {
            let v = ui.value(EAction::MenuLeftRight, mask);
            if elapsed {
                // 0x00086390 = ceil (CRT 0x003E2A80); constants 0.1 / 5.0 / -0.1
                if v > 0.1 {
                    tm.tweak_mut(id).increment((v * 5.0).ceil());
                    changed = true;
                } else if v < -0.1 {
                    tm.tweak_mut(id).decrement((v * 5.0).ceil().abs());
                    changed = true;
                }
            }
        }
    }
    if !changed {
        return None;
    }
    *timer = TWEAK_REPEAT;
    item.value_text = value_text(tm, id);
    Some(id)
}

impl CDebugMenuManager {
    /// GamePauseMenu.cpp 0x000AF970 — `EnterTweaksMenu`: the root menu "Tweaks", then down the remembered path.
    /// Original bug kept: a path entry that no longer exists sets the depth to `i - 1` (-1 for the first).
    pub fn enter_tweaks_menu(&mut self, tm: &CTweakManager) {
        self.keep_tweak_path = false;
        self.enter(CGameMenu::tweak_system(tm, CTweakManager::ROOT, Some("Tweaks")));
        if self.tweak_depth != 0 {
            let mut sys = CTweakManager::ROOT;
            let mut i = 0;
            while (i as i32) < self.tweak_depth {
                let Some(&c) = tm.systems[sys].children.get(self.tweak_stack[i] as usize) else {
                    self.tweak_depth = i as i32 - 1;
                    return;
                };
                sys = c;
                self.enter(CGameMenu::tweak_system(tm, c, None));
                i += 1;
            }
        }
    }

    /// GameTweaksMenu.cpp 0x000AFE00 — `HandleMenuAction`
    pub fn tweak_system_action(&mut self, tm: &CTweakManager, system: usize, id: i32) {
        if id & CHILD_ID != 0 {
            let idx = (id & 0xFFFF) as usize;
            let Some(&c) = tm.systems[system].children.get(idx) else { return };
            let m = CGameMenu::tweak_system(tm, c, None);
            if self.tweak_depth >= STACK_DEPTH as i32 {
                log::error!("ASSERT: gs_tweakDepth < STACK_DEPTH (GameTweaksMenu.cpp 230)");
                self.tweak_depth += 1;
            } else if self.tweak_depth < 0 {
                // gs_tweakStack[-1] (`dword_6F54E4`) is written: the entry is lost
                self.tweak_depth += 1;
            } else {
                self.tweak_stack[self.tweak_depth as usize] = idx as i32;
                self.tweak_depth += 1;
            }
            self.enter(m);
        } else if id & 1 != 0 {
            // 0x000AFF60
            log::info!("===== Begin Export of Tweak Settings to TTY =====");
            let mut lines = Vec::new();
            tm.export_tty(system, &mut lines);
            for l in lines {
                log::info!("{l}");
            }
            log::info!("===== End Export of Tweak Settings to TTY =====");
        } else if id & 2 != 0 {
            // 0x000AFFB0: "Tweaks.txt" (the game dir is read-only here: next to the exe)
            log::info!("===== Begin Export of Tweak Settings to File =====");
            let path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("Tweaks.txt")))
                .unwrap_or_else(|| "Tweaks.txt".into());
            let mut text = String::new();
            tm.export(system, &mut text);
            match std::fs::write(&path, text) {
                Ok(()) => {
                    log::info!("Exporting to file: {}", path.display());
                    log::info!("Exported tweak settings to file: {}", path.display());
                }
                Err(_) => log::info!("Error while exporting to file: {}", path.display()),
            }
            log::info!("===== End Export of Tweak Settings to File =====");
        }
    }

    /// GameTweaksMenu.cpp 0x000AFD90 — `OnLeave`: pop the path unless the whole stack is leaving
    pub fn tweak_system_leave(&mut self) {
        if !self.keep_tweak_path && self.tweak_depth > 0 {
            self.tweak_depth -= 1;
            self.tweak_stack[self.tweak_depth as usize] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font_helper::{add_system_tweaks, SystemTweaks};
    use crate::game_menu_manager::game_menu_manager::MenuKind;

    #[test]
    fn system_menu_rows_and_path_survive_leave_all() {
        let mut tm = CTweakManager::default();
        add_system_tweaks(&mut tm, &SystemTweaks::default());
        let mut dm = CDebugMenuManager::default();
        dm.enter_tweaks_menu(&tm);
        let root = &dm.stack[0];
        assert_eq!(root.name, "Tweaks");
        let names: Vec<_> = root.items.iter().map(|i| (i.text.as_str(), i.id)).collect();
        assert_eq!(names, [("System", CHILD_ID), ("Export Tweaks to TTY", 1), ("Export Tweaks to File", 2)]);
        dm.tweak_system_action(&tm, CTweakManager::ROOT, CHILD_ID);
        let sys = dm.stack.last_mut().unwrap();
        assert!(matches!(sys.kind, MenuKind::TweakSystem { .. }));
        assert_eq!(sys.items.len(), 6);
        sys.layout();
        sys.build(&tm);
        let texts: Vec<_> = sys.osd.iter().map(|t| t.text.as_str()).collect();
        assert!(texts.contains(&"System Update Rate") && texts.contains(&"24 (24)"), "{texts:?}");
        // LeaveAll keeps the path (OnLeaveAll 0x000AFDE0), so EnterTweaksMenu opens System again
        dm.leave_all();
        dm.pop_closing_for_test();
        assert!(dm.stack.is_empty());
        dm.enter_tweaks_menu(&tm);
        assert_eq!(dm.stack.len(), 2);
        assert_eq!(dm.stack[1].name, "System");
        // a single leave pops it
        dm.leave();
        dm.pop_closing_for_test();
        assert_eq!(dm.tweak_depth, 0);
    }
}
