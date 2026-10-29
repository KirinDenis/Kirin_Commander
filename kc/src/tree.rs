//! The tree panel, Ctrl+T (docs/01-screen.md 1.2): the drive's folders
//! as a tree in a panel. A drive is not read whole - on a desktop that is
//! every folder of the disk - but open along the way to the folder the
//! panel was in, each folder on the way with its sub-folders; Right or
//! Gray + opens a folder, Left or Gray - closes it. The other panel
//! follows the tree's cursor only on Enter.

use crate::fs::Fs;
use crate::panel::compare_names;
use crate::path;
use alloc::string::String;
use alloc::vec::Vec;

pub struct Node {
    pub path: String,
    pub name: String,
    pub depth: usize,
    /// Its sub-folders are in the list under it.
    pub open: bool,
    /// The last of its parent's folders: drawn `└`, and no line under it.
    pub last: bool,
}

pub struct Tree {
    pub nodes: Vec<Node>,
    pub cursor: usize,
    pub top: usize,
    pub rows: usize,
    sep: char,
    hidden: bool,
}

impl Tree {
    /// The tree of `dir`'s drive, open down to `dir`, the cursor on it.
    pub fn new(fs: &mut dyn Fs, dir: &str, hidden: bool, rows: usize) -> Tree {
        let sep = fs.sep();
        let root = String::from(path::root(dir, sep));
        let mut t = Tree {
            nodes: alloc::vec![Node { path: root.clone(), name: root.clone(), depth: 0, open: false, last: true }],
            cursor: 0,
            top: 0,
            rows: rows.max(1),
            sep,
            hidden,
        };
        t.open_at(fs, 0);
        // Down the path, a folder at a time.
        let mut at = 0;
        let rest = &dir[root.len().min(dir.len())..];
        for part in rest.split(sep).filter(|s| !s.is_empty()) {
            let depth = t.nodes[at].depth + 1;
            let found = (at + 1..t.nodes.len()).take_while(|&i| t.nodes[i].depth >= depth).find(|&i| t.nodes[i].depth == depth && t.nodes[i].name.eq_ignore_ascii_case(part));
            let Some(i) = found else { break };
            at = i;
            t.open_at(fs, at);
        }
        t.cursor = at;
        t.centre();
        t
    }

    /// The folder under the cursor.
    pub fn current(&self) -> &str {
        &self.nodes[self.cursor].path
    }

    /// Its sub-folders, in name order, put under node `i`.
    fn open_at(&mut self, fs: &mut dyn Fs, i: usize) {
        if self.nodes[i].open {
            return;
        }
        let parent = self.nodes[i].path.clone();
        let depth = self.nodes[i].depth + 1;
        let mut names: Vec<String> = match fs.read_dir(&parent) {
            Ok(list) => list.into_iter().filter(|e| e.dir && !e.is_up() && (self.hidden || !(e.attrs.hidden || e.attrs.system))).map(|e| e.name).collect(),
            Err(_) => Vec::new(),
        };
        names.sort_by(|a, b| compare_names(a, b, true));
        let n = names.len();
        let kids: Vec<Node> = names
            .into_iter()
            .enumerate()
            .map(|(k, name)| Node { path: path::join(&parent, &name, self.sep), name, depth, open: false, last: k + 1 == n })
            .collect();
        self.nodes[i].open = true;
        let at = i + 1;
        self.nodes.splice(at..at, kids);
    }

    /// Everything under node `i` taken away.
    fn close_at(&mut self, i: usize) {
        let depth = self.nodes[i].depth;
        let end = (i + 1..self.nodes.len()).find(|&k| self.nodes[k].depth <= depth).unwrap_or(self.nodes.len());
        self.nodes.drain(i + 1..end);
        self.nodes[i].open = false;
        if self.cursor >= self.nodes.len() {
            self.cursor = i;
        }
    }

    /// Right, Gray +: the folder opens; an open one: its first sub-folder.
    pub fn open(&mut self, fs: &mut dyn Fs) {
        let i = self.cursor;
        if self.nodes[i].open {
            if self.nodes.get(i + 1).map(|n| n.depth > self.nodes[i].depth).unwrap_or(false) {
                self.cursor = i + 1;
            }
        } else {
            self.open_at(fs, i);
        }
        self.ensure_visible();
    }

    /// Left, Gray -: an open folder closes; a closed one: its parent.
    pub fn close(&mut self) {
        let i = self.cursor;
        let has_kids = self.nodes.get(i + 1).map(|n| n.depth > self.nodes[i].depth).unwrap_or(false);
        if self.nodes[i].open && has_kids {
            self.close_at(i);
        } else if let Some(p) = self.parent(i) {
            self.cursor = p;
        } else if self.nodes[i].open {
            self.close_at(i);
        }
        self.ensure_visible();
    }

    pub fn parent(&self, i: usize) -> Option<usize> {
        let d = self.nodes[i].depth;
        (0..i).rev().find(|&k| self.nodes[k].depth < d)
    }

    /// The lines in front of a name: `│  ` under every folder on the way
    /// that has more after it, then `├──` or `└──`.
    pub fn prefix(&self, i: usize) -> String {
        let d = self.nodes[i].depth;
        if d == 0 {
            return String::new();
        }
        // Which of the folders on the way are the last of theirs.
        let mut lasts = alloc::vec![false; d];
        let mut k = i;
        lasts[d - 1] = self.nodes[i].last;
        while let Some(p) = self.parent(k) {
            let pd = self.nodes[p].depth;
            if pd >= 1 {
                lasts[pd - 1] = self.nodes[p].last;
            }
            k = p;
        }
        let mut s = String::new();
        for &l in &lasts[..d - 1] {
            s.push_str(if l { "   " } else { "│  " });
        }
        s.push_str(if self.nodes[i].last { "└──" } else { "├──" });
        s
    }

    fn last_index(&self) -> usize {
        self.nodes.len().saturating_sub(1)
    }

    pub fn up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.ensure_visible();
    }
    pub fn down(&mut self) {
        self.cursor = (self.cursor + 1).min(self.last_index());
        self.ensure_visible();
    }
    pub fn home(&mut self) {
        self.cursor = 0;
        self.ensure_visible();
    }
    pub fn end(&mut self) {
        self.cursor = self.last_index();
        self.ensure_visible();
    }
    pub fn page_down(&mut self) {
        self.cursor = (self.cursor + self.rows).min(self.last_index());
        self.ensure_visible();
    }
    pub fn page_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(self.rows);
        self.ensure_visible();
    }
    pub fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.last_index());
        self.ensure_visible();
    }
    pub fn set_rows(&mut self, rows: usize) {
        self.rows = rows.max(1);
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        if self.cursor < self.top {
            self.top = self.cursor;
        } else if self.cursor >= self.top + self.rows {
            self.top = self.cursor + 1 - self.rows;
        }
    }

    fn centre(&mut self) {
        self.top = self.cursor.saturating_sub(self.rows / 2);
        let max = self.nodes.len().saturating_sub(self.rows);
        self.top = self.top.min(max);
        self.ensure_visible();
    }

    /// The first name that starts with `text`, from `from` on: quick search.
    pub fn find_prefix(&self, text: &str, from: usize) -> Option<usize> {
        let t = text.to_lowercase();
        let n = self.nodes.len();
        (0..n).map(|k| (from + k) % n).find(|&i| self.nodes[i].name.to_lowercase().starts_with(&t))
    }
}
