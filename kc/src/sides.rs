//! The panels that are not file lists (docs/01-screen.md 1.2): the Info
//! panel (Ctrl+L), the folder tree (Ctrl+T) and Quick view (Ctrl+Q) - what
//! each knows, when it is read again, and their keys. Drawing them is in
//! `screen`.
//!
//! Info and Quick view are about the other panel: its folder, the file
//! under its cursor. So they are never the active panel, and one of them
//! never faces another. Each is read again when what it shows changed -
//! after every key, a mouse click, a step of a job - and only then, so a
//! cursor that does not move reads nothing.

use crate::app::App;
use crate::fs::*;
use crate::panel::{Focus, Mode, View};
use crate::path;
use crate::tree::Tree;
use alloc::string::String;
use alloc::vec::Vec;
use owlosui_core::{Key, KeyCode};

/// The most of a file Quick view reads: a screen of it and more.
pub const QUICK_LIMIT: usize = 16 * 1024;

/// What the Info panel shows.
pub struct Info {
    /// The folder it is about.
    pub dir: String,
    pub label: Option<String>,
    /// The drive's size and its free bytes.
    pub space: Option<(u64, u64)>,
    pub files: usize,
    pub folders: usize,
    pub bytes: u64,
    /// The folder's description file: its name and first lines.
    pub about: Option<(String, Vec<u8>)>,
}

/// What Quick view shows: the first bytes of a file, or a folder.
pub struct Quick {
    pub path: String,
    pub size: u64,
    pub stamp: Stamp,
    pub dir: bool,
    pub bytes: Vec<u8>,
    pub error: Option<FsError>,
}

/// The files that describe a folder: NC's, and 4DOS's and Far's.
const ABOUT: [&str; 2] = ["DIRINFO", "DESCRIPT.ION"];

impl App {
    /// Ctrl+L, Ctrl+Q, Ctrl+T and the menu's Info panel, Tree panel, Quick
    /// view: panel `i` shows `v` - or its files again, if it shows `v`
    /// already.
    pub fn set_view(&mut self, i: usize, v: View) {
        let now = self.panels[i].view;
        let v = if now == v { View::Files } else { v };
        let o = 1 - i;
        // Info and Quick view describe the other panel: it has to be one
        // with folders in it.
        if matches!(v, View::Info | View::Quick) && !matches!(self.panels[o].view, View::Files | View::Tree) {
            return;
        }
        if now == View::Tree && v != View::Tree {
            self.leave_tree(i);
        }
        if v == View::Tree {
            let dir = self.panels[i].dir.clone();
            let rows = self.panels[i].rows;
            let hidden = self.panels[i].show_hidden;
            self.trees[i] = Some(Tree::new(&mut *self.fs, &dir, hidden, rows));
        }
        self.panels[i].view = v;
        self.panels[i].visible = true;
        if matches!(v, View::Info | View::Quick) {
            if self.active == i {
                self.active = o;
            }
            self.panels[o].visible = true;
        }
        // An Info or Quick view that faced this panel has nothing to show now.
        if !matches!(v, View::Files | View::Tree) && matches!(self.panels[o].view, View::Info | View::Quick) {
            self.panels[o].view = View::Files;
        }
        self.refresh_sides();
    }

    /// The tree closed: its panel shows the folder the tree's cursor is on.
    fn leave_tree(&mut self, i: usize) {
        let Some(t) = self.trees[i].take() else { return };
        let dir = String::from(t.current());
        if dir != self.panels[i].dir {
            let fs = &mut *self.fs;
            if let Err(e) = self.panels[i].load(fs, &dir, Focus::First) {
                self.fs_error("Cannot read the folder", &dir, e);
            }
        }
    }

    /// Ctrl+1 .. Ctrl+5: a view mode is a file list - the panel shows its
    /// files again if it showed something else.
    pub fn set_mode_key(&mut self, mode: Mode) {
        let i = self.active;
        if self.panels[i].view != View::Files {
            self.set_view(i, View::Files);
        }
        self.panels[i].set_mode(mode);
    }

    /// The folder a panel is about: its own, or the tree's cursor.
    pub fn folder_of(&self, i: usize) -> String {
        match (&self.trees[i], self.panels[i].view) {
            (Some(t), View::Tree) => String::from(t.current()),
            _ => self.panels[i].dir.clone(),
        }
    }

    /// Info and Quick view read again, where what they show has changed.
    pub fn refresh_sides(&mut self) {
        for i in 0..2 {
            match self.panels[i].view {
                View::Info => self.refresh_info(1 - i),
                View::Quick => self.refresh_quick(1 - i),
                _ => {}
            }
        }
        if !self.panels.iter().any(|p| p.view == View::Info) {
            self.info = None;
        }
        if !self.panels.iter().any(|p| p.view == View::Quick) {
            self.quick = None;
        }
    }

    fn refresh_info(&mut self, src: usize) {
        let dir = self.folder_of(src);
        let p = &self.panels[src];
        // The counts from the list on the screen, when it is that folder's.
        let (mut files, mut folders, mut bytes) = (0, 0, 0u64);
        if p.dir == dir {
            for e in p.entries.iter().filter(|e| !e.is_up()) {
                if e.dir {
                    folders += 1;
                } else {
                    files += 1;
                    bytes += e.size;
                }
            }
        } else if let Ok(list) = self.fs.read_dir(&dir) {
            for e in list.iter().filter(|e| !e.is_up()) {
                if e.dir {
                    folders += 1;
                } else {
                    files += 1;
                    bytes += e.size;
                }
            }
        }
        let same = self.info.as_ref().map(|f| f.dir == dir).unwrap_or(false);
        if same {
            if let Some(f) = self.info.as_mut() {
                f.files = files;
                f.folders = folders;
                f.bytes = bytes;
            }
            return;
        }
        // A new folder: the drive and the description asked once.
        let root = String::from(path::root(&dir, self.fs.sep()));
        let space = self.fs.disk_space(&root);
        let label = self.fs.label(&root);
        let names: Vec<String> = if p.dir == dir {
            p.entries.iter().filter(|e| !e.dir).map(|e| e.name.clone()).collect()
        } else {
            self.fs.read_dir(&dir).map(|l| l.into_iter().filter(|e| !e.dir).map(|e| e.name).collect()).unwrap_or_default()
        };
        let mut about = None;
        for want in ABOUT {
            if let Some(n) = names.iter().find(|n| n.eq_ignore_ascii_case(want)) {
                let full = path::join(&dir, n, self.fs.sep());
                if let Ok(b) = read_start(&mut *self.fs, &full, 4096) {
                    about = Some((n.clone(), b));
                    break;
                }
            }
        }
        self.info = Some(Info { dir, label, space, files, folders, bytes, about });
    }

    fn refresh_quick(&mut self, src: usize) {
        let (path, entry) = match self.panels[src].view {
            View::Tree => {
                let d = self.folder_of(src);
                (d.clone(), Entry::folder(&path::name(&d, self.fs.sep())))
            }
            _ => {
                let p = &self.panels[src];
                match (p.current_path(), p.current()) {
                    (Some(full), Some(e)) => (full, e.clone()),
                    _ => (p.dir.clone(), Entry::folder("..")),
                }
            }
        };
        if let Some(q) = &self.quick {
            if q.path == path && q.size == entry.size && q.stamp == entry.stamp && q.dir == entry.dir {
                return;
            }
        }
        let (bytes, error) = if entry.dir {
            (Vec::new(), None)
        } else {
            match read_start(&mut *self.fs, &path, QUICK_LIMIT) {
                Ok(b) => (b, None),
                Err(e) => (Vec::new(), Some(e)),
            }
        };
        self.quick = Some(Quick { path, size: entry.size, stamp: entry.stamp, dir: entry.dir, bytes, error });
    }

    /// A key while the tree is the active panel; false for one the panel's
    /// own keys take (Tab, F9, F10, Ctrl+T, Alt+F1 ...).
    pub fn tree_key(&mut self, k: Key) -> bool {
        let i = self.active;
        let none = k.mods.is_none();
        let empty = self.cmd.is_empty();
        if self.trees[i].is_none() {
            return false;
        }
        if k.code == KeyCode::Enter && none && empty {
            self.tree_enter();
            return true;
        }
        if k.code == KeyCode::Char('r') && k.mods.ctrl {
            let dir = self.folder_of(i);
            let rows = self.panels[i].rows;
            let hidden = self.panels[i].show_hidden;
            self.trees[i] = Some(Tree::new(&mut *self.fs, &dir, hidden, rows));
            return true;
        }
        let fs = &mut *self.fs;
        let Some(t) = self.trees[i].as_mut() else { return false };
        match k.code {
            KeyCode::Up if none => t.up(),
            KeyCode::Down if none => t.down(),
            KeyCode::PageUp if none => t.page_up(),
            KeyCode::PageDown if none => t.page_down(),
            KeyCode::Home if none && empty => t.home(),
            KeyCode::End if none && empty => t.end(),
            KeyCode::Right if none && empty => t.open(fs),
            KeyCode::Char('+') if !k.mods.ctrl && !k.mods.alt && empty => t.open(fs),
            KeyCode::Left if none && empty => t.close(),
            KeyCode::Char('-') if !k.mods.ctrl && !k.mods.alt && empty => t.close(),
            KeyCode::Backspace if none && empty => {
                if let Some(p) = t.parent(t.cursor) {
                    t.set_cursor(p);
                }
            }
            // What works on files has no files here.
            KeyCode::F(3..=8) | KeyCode::Insert | KeyCode::Delete if !k.mods.alt && !k.mods.ctrl => {}
            KeyCode::Char('*') if empty => {}
            KeyCode::Up | KeyCode::Down if k.mods.shift => {}
            _ => return false,
        }
        true
    }

    /// Enter in the tree: the other panel goes to the folder; with no file
    /// list on the other side, this panel does.
    pub fn tree_enter(&mut self) {
        let i = self.active;
        let dir = self.folder_of(i);
        let o = 1 - i;
        if self.panels[o].view == View::Files {
            let fs = &mut *self.fs;
            if let Err(e) = self.panels[o].load(fs, &dir, Focus::First) {
                self.fs_error("Cannot read the folder", &dir, e);
            }
        } else {
            self.set_view(i, View::Files);
        }
    }
}

/// The first `limit` bytes of a file.
fn read_start(fs: &mut dyn Fs, path: &str, limit: usize) -> FsResult<Vec<u8>> {
    let h = fs.open_read(path)?;
    let mut out = alloc::vec![0u8; limit];
    let mut got = 0;
    while got < limit {
        match fs.read(h, &mut out[got..]) {
            Ok(0) => break,
            Ok(n) => got += n,
            Err(e) => {
                let _ = fs.close(h);
                return Err(e);
            }
        }
    }
    let _ = fs.close(h);
    out.truncate(got);
    Ok(out)
}
