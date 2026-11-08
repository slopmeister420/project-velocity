//! src\Cli\DebugMenu.cpp — `CDebugMenu`: a tree of named menus, each holding up to 32 sub-menus and 32
//! (label, command) items. `autoexec.cli` fills it with `debugMenu.Add "File.Load.All Tweaks" "source ..."`;
//! the console that would run those commands is not in F3_D.

/// children per menu (`+0x20 .. +0xA0`)
pub const MAX_CHILDREN: usize = 32;
/// items per menu (`+0xA4 .. +0x1A4`)
pub const MAX_ITEMS: usize = 32;

/// `CDebugMenuItem`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CDebugMenuItem {
    pub label: Option<String>,
    pub command: Option<String>,
}

/// `CDebugMenu` (0x1A8 bytes)
#[derive(Clone, Debug, Default)]
pub struct CDebugMenu {
    pub name: String,                // +0x00 char[32]
    pub children: Vec<CDebugMenu>,   // +0x20, count +0xA0
    pub items: Vec<CDebugMenuItem>,  // +0xA4, count +0x1A4
}

/// DebugMenu.cpp 0x00082F10 — empty strings are stored as null.
fn dup(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

impl CDebugMenu {
    /// DebugMenu.cpp 0x00082680
    pub fn new(name: &str) -> Self {
        CDebugMenu { name: name.chars().take(31).collect(), children: Vec::new(), items: Vec::new() }
    }

    /// DebugMenu.cpp 0x000828D0
    pub fn find_child(&mut self, name: &str) -> Option<&mut CDebugMenu> {
        self.children.iter_mut().find(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// DebugMenu.cpp 0x00082820 — find or append.
    pub fn add_child(&mut self, name: &str) -> &mut CDebugMenu {
        if let Some(i) = self.children.iter().position(|c| c.name.eq_ignore_ascii_case(name)) {
            return &mut self.children[i];
        }
        assert!(self.children.len() < MAX_CHILDREN);
        self.children.push(CDebugMenu::new(name));
        self.children.last_mut().unwrap()
    }

    /// DebugMenu.cpp 0x00082950
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// DebugMenu.cpp 0x00082970
    pub fn child(&self, index: usize) -> &CDebugMenu {
        assert!(index < self.children.len(), "nIndex < m_nChildren");
        &self.children[index]
    }

    /// DebugMenu.cpp 0x00082A50
    pub fn remove_child(&mut self, name: &str) {
        if let Some(i) = self.children.iter().position(|c| c.name.eq_ignore_ascii_case(name)) {
            self.children.remove(i);
        }
    }

    /// DebugMenu.cpp 0x00082C10
    pub fn find_item(&self, label: &str) -> Option<&CDebugMenuItem> {
        self.items.iter().find(|i| i.label.as_deref().is_some_and(|l| l.eq_ignore_ascii_case(label)))
    }

    /// DebugMenu.cpp 0x00082B70 — an existing item with the same label is kept unchanged.
    pub fn add_item(&mut self, label: &str, command: &str) {
        if self.find_item(label).is_some() {
            return;
        }
        assert!(self.items.len() < MAX_ITEMS);
        self.items.push(CDebugMenuItem { label: dup(label), command: dup(command) });
    }

    /// DebugMenu.cpp 0x00082C90
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// DebugMenu.cpp 0x00082CB0
    pub fn item(&self, index: usize) -> &CDebugMenuItem {
        assert!(index < self.items.len(), "nIndex < m_nItems");
        &self.items[index]
    }

    /// DebugMenu.cpp 0x00082DA0
    pub fn remove_item(&mut self, label: &str) {
        if let Some(i) = self.items.iter().position(|it| it.label.as_deref().is_some_and(|l| l.eq_ignore_ascii_case(label))) {
            self.items.remove(i);
        }
    }

    /// `debugMenu.Add "A.B.Label" "command"`: menus along the dotted path, item at the leaf.
    /// (the console verb is not in F3_D; this mirrors what `autoexec.cli` asks for.)
    pub fn add_path(&mut self, path: &str, command: &str) {
        match path.rsplit_once('.') {
            None => self.add_item(path, command),
            Some((menus, label)) => {
                let mut m = self;
                for n in menus.split('.') {
                    m = m.add_child(n);
                }
                m.add_item(label, command);
            }
        }
    }

    /// parse `debugMenu.Add "path" "command"` lines (`autoexec.cli`).
    pub fn load_cli(&mut self, text: &str) -> usize {
        let mut n = 0;
        for line in text.lines() {
            let line = line.trim();
            let Some(rest) = line.strip_prefix("debugMenu.Add") else { continue };
            let q: Vec<&str> = rest.split('"').collect();
            if q.len() >= 4 {
                self.add_path(q[1], q[3]);
                n += 1;
            }
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autoexec() {
        let mut root = CDebugMenu::new("root");
        let text = match std::fs::read_to_string(visos::file::resolve(r"\autoexec.cli")) {
            Ok(t) => t,
            Err(_) => "debugMenu.Add \"File.Load.All Tweaks\" \"source all.tweaks\"".into(),
        };
        assert!(root.load_cli(&text) >= 1);
        let file = root.find_child("file").unwrap();
        let load = file.find_child("Load").unwrap();
        assert_eq!(load.item(0).command.as_deref(), Some("source all.tweaks"));
    }
}
