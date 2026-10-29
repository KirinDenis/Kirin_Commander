//! One file panel: its folder, its names in order, its cursor, its
//! selection, its mode (docs/02-panels.md). Nothing here draws; `screen`
//! does. The cursor rules of section 2.4 are all in this file, and each
//! has a test in `tests/panel.rs`.

use crate::fs::*;
use crate::mask::Mask;
use crate::path;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Names only, three columns.
    Brief,
    /// Names only, two columns.
    Medium,
    /// Name, Size, Date, Time.
    Full,
    /// Name and Size.
    Wide,
    /// Name, Size, Date, Time, Attributes: the whole screen width.
    Detailed,
}

/// What a panel shows (docs/01-screen.md 1.2): its files, or in their
/// place the Info panel (Ctrl+L), the folder tree (Ctrl+T) or the file
/// under the other panel's cursor (Ctrl+Q).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum View {
    Files,
    Info,
    Tree,
    Quick,
}

impl Mode {
    pub fn columns(self) -> usize {
        match self {
            Mode::Brief => 3,
            Mode::Medium => 2,
            Mode::Full | Mode::Wide | Mode::Detailed => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sort {
    Name,
    Ext,
    Time,
    Size,
    Unsorted,
}

impl Sort {
    /// The letter in the panel's corner.
    pub fn letter(self) -> char {
        match self {
            Sort::Name => 'n',
            Sort::Ext => 'e',
            Sort::Time => 't',
            Sort::Size => 's',
            Sort::Unsorted => 'u',
        }
    }
}

/// Where the cursor goes when a folder is (re)loaded.
#[derive(Clone, Debug)]
pub enum Focus {
    /// `..`, or the first name: a folder just entered.
    First,
    /// This name, in the middle of the panel: the folder just left.
    Centre(String),
    /// This name if it is still there, else the one that took its place
    /// (rule 6): a re-read.
    Stay(String, usize),
}

pub struct Panel {
    pub dir: String,
    /// `..` first, then folders, then files, in the sort order.
    pub entries: Vec<Entry>,
    pub selected: Vec<bool>,
    pub cursor: usize,
    /// The first name on the screen.
    pub top: usize,
    pub mode: Mode,
    pub sort: Sort,
    pub reverse: bool,
    pub show_hidden: bool,
    pub filter: Option<String>,
    /// Numbers in names compared as numbers (`file2` before `file10`).
    pub natural: bool,
    /// The panel is on the screen (Ctrl+F1 / Ctrl+F2).
    pub visible: bool,
    /// Files, or Info, Tree, Quick view in their place.
    pub view: View,
    /// Why the folder could not be read, while it could not.
    pub error: Option<FsError>,
    /// The rows of names the screen has for this panel, set by the layout.
    pub rows: usize,
    /// The folder each drive was last in: a drive changed comes back
    /// there, as DOS remembers a current folder per drive (rule 5).
    per_drive: BTreeMap<String, String>,
    /// The selection before the last operation, for Ctrl+M.
    saved: Vec<String>,
    short_names: bool,
    case_sensitive: bool,
    sep: char,
}

impl Panel {
    pub fn new(fs: &dyn Fs) -> Panel {
        Panel {
            dir: String::new(),
            entries: Vec::new(),
            selected: Vec::new(),
            cursor: 0,
            top: 0,
            mode: if fs.short_names() { Mode::Brief } else { Mode::Full },
            sort: Sort::Name,
            reverse: false,
            // Hidden files are shown on DOS, as NC showed them; hidden on
            // a desktop, as the system's own manager hides them.
            show_hidden: fs.short_names(),
            filter: None,
            natural: !fs.short_names(),
            visible: true,
            view: View::Files,
            error: None,
            rows: 18,
            per_drive: BTreeMap::new(),
            saved: Vec::new(),
            short_names: fs.short_names(),
            case_sensitive: fs.case_sensitive(),
            sep: fs.sep(),
        }
    }

    pub fn sep(&self) -> char {
        self.sep
    }

    pub fn short_names(&self) -> bool {
        self.short_names
    }

    pub fn current(&self) -> Option<&Entry> {
        self.entries.get(self.cursor)
    }

    /// The full path of the name under the cursor.
    pub fn current_path(&self) -> Option<String> {
        self.current().filter(|e| !e.is_up()).map(|e| path::join(&self.dir, &e.name, self.sep))
    }

    pub fn path_of(&self, name: &str) -> String {
        path::join(&self.dir, name, self.sep)
    }

    pub fn is_root(&self) -> bool {
        path::is_root(&self.dir, self.sep)
    }

    // ------------------------------------------------------------ loading

    /// Show a folder. A folder that cannot be read leaves the panel where it
    /// was and says why in `error` - the panel never shows a list that is
    /// not the folder in its title.
    pub fn load(&mut self, fs: &mut dyn Fs, dir: &str, focus: Focus) -> FsResult<()> {
        let list = match fs.read_dir(dir) {
            Ok(l) => l,
            Err(e) => {
                self.error = Some(e.clone());
                return Err(e);
            }
        };
        let same = self.same_name(&self.dir, dir);
        let keep: Vec<String> = if same { self.selected_names() } else { Vec::new() };
        self.dir = String::from(dir);
        self.error = None;
        self.per_drive.insert(String::from(path::root(dir, self.sep)).to_uppercase(), String::from(dir));
        self.entries = list.into_iter().filter(|e| self.shows(e)).collect();
        self.sort_entries();
        // `..` above every folder but a root, as NC and VC have it; it
        // carries the folder's own date.
        if !path::is_root(dir, self.sep) {
            let mut up = Entry::folder("..");
            if let Ok(me) = fs.stat(dir) {
                up.stamp = me.stamp;
            }
            self.entries.insert(0, up);
        }
        self.selected = self.entries.iter().map(|e| keep.iter().any(|k| self.same_name(k, &e.name))).collect();
        match focus {
            Focus::First => {
                self.cursor = 0;
                self.top = 0;
            }
            Focus::Centre(name) => {
                self.cursor = self.index_of(&name).unwrap_or(0);
                self.centre();
            }
            Focus::Stay(name, index) => {
                let old_top_row = self.cursor.saturating_sub(self.top);
                self.cursor = match self.index_of(&name) {
                    Some(i) => i,
                    // Gone: the name that is now where it was - the next
                    // one - or the last if it was the last (rule 6).
                    None => index.min(self.entries.len().saturating_sub(1)),
                };
                // The bar stays on its row of the screen when it can.
                self.top = self.cursor.saturating_sub(old_top_row);
                self.clamp_top();
            }
        }
        self.ensure_visible();
        Ok(())
    }

    /// The same folder again, the cursor on the same name (rule 6).
    pub fn reread(&mut self, fs: &mut dyn Fs) -> FsResult<()> {
        let name = self.current().map(|e| e.name.clone()).unwrap_or_default();
        let dir = self.dir.clone();
        let index = self.cursor;
        self.load(fs, &dir, Focus::Stay(name, index))
    }

    /// Re-read, and put the cursor on this name (rule 7: a new folder, a
    /// renamed file, a file the editor made).
    pub fn reread_onto(&mut self, fs: &mut dyn Fs, name: &str) -> FsResult<()> {
        let dir = self.dir.clone();
        let index = self.cursor;
        self.load(fs, &dir, Focus::Stay(String::from(name), index))
    }

    fn shows(&self, e: &Entry) -> bool {
        if !self.show_hidden && (e.attrs.hidden || e.attrs.system || (!self.short_names && e.name.starts_with('.'))) {
            return false;
        }
        if let Some(f) = &self.filter {
            if !e.dir && !Mask::parse(f, self.short_names).matches(&e.name) {
                return false;
            }
        }
        true
    }

    fn same_name(&self, a: &str, b: &str) -> bool {
        if self.case_sensitive {
            a == b
        } else {
            a.to_lowercase() == b.to_lowercase()
        }
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.entries.iter().position(|e| self.same_name(&e.name, name))
    }

    // ------------------------------------------------------------ moving

    /// Into the folder under the cursor (rule 2), or out of this one on `..`
    /// (rule 3). False when the cursor is on a file.
    pub fn enter(&mut self, fs: &mut dyn Fs) -> FsResult<bool> {
        let Some(e) = self.current() else { return Ok(false) };
        if e.is_up() {
            return self.go_up(fs);
        }
        if !e.dir {
            return Ok(false);
        }
        let to = path::join(&self.dir, &e.name, self.sep);
        self.load(fs, &to, Focus::First)?;
        Ok(true)
    }

    /// Out of the folder, the cursor on the folder just left. False in a root.
    pub fn go_up(&mut self, fs: &mut dyn Fs) -> FsResult<bool> {
        let Some((parent, name)) = path::split(&self.dir, self.sep) else { return Ok(false) };
        self.load(fs, &parent, Focus::Centre(name))?;
        Ok(true)
    }

    /// To the root, the cursor on the first-level folder the panel was in (rule 4).
    pub fn go_root(&mut self, fs: &mut dyn Fs) -> FsResult<()> {
        let root = String::from(path::root(&self.dir, self.sep));
        let rest = &self.dir[root.len()..];
        let first = rest.split(self.sep).next().unwrap_or("");
        let focus = if first.is_empty() { Focus::First } else { Focus::Centre(String::from(first)) };
        self.load(fs, &root, focus)
    }

    /// Another drive: where the panel last was on it, or its root (rule 5).
    pub fn go_drive(&mut self, fs: &mut dyn Fs, root: &str) -> FsResult<()> {
        let key = String::from(path::root(root, self.sep)).to_uppercase();
        let dir = self.per_drive.get(&key).cloned();
        if let Some(d) = dir {
            if self.load(fs, &d, Focus::First).is_ok() {
                return Ok(());
            }
        }
        self.load(fs, root, Focus::First)
    }

    /// How many names the screen shows at once.
    pub fn capacity(&self) -> usize {
        (self.rows * self.mode.columns()).max(1)
    }

    fn by_columns(&self) -> bool {
        self.mode.columns() > 1
    }

    fn clamp_top(&mut self) {
        let len = self.entries.len();
        let max = if self.by_columns() {
            let cols_needed = len.div_ceil(self.rows.max(1));
            cols_needed.saturating_sub(self.mode.columns()) * self.rows
        } else {
            len.saturating_sub(self.capacity())
        };
        if self.top > max {
            self.top = max;
        }
        if self.by_columns() {
            self.top -= self.top % self.rows.max(1);
        }
    }

    /// The cursor's name on the screen. Columns of names scroll a whole
    /// column at a time, so they keep their places; one column scrolls by
    /// a name.
    pub fn ensure_visible(&mut self) {
        let cap = self.capacity();
        let rows = self.rows.max(1);
        if self.cursor < self.top {
            self.top = if self.by_columns() { self.cursor / rows * rows } else { self.cursor };
        } else if self.cursor >= self.top + cap {
            self.top = if self.by_columns() {
                (self.cursor / rows + 1).saturating_sub(self.mode.columns()) * rows
            } else {
                self.cursor + 1 - cap
            };
        }
        self.clamp_top();
    }

    /// The cursor in the middle of the panel, if the list is long enough.
    pub fn centre(&mut self) {
        let rows = self.rows.max(1);
        self.top = if self.by_columns() {
            (self.cursor / rows).saturating_sub(self.mode.columns() / 2) * rows
        } else {
            self.cursor.saturating_sub(rows / 2)
        };
        self.clamp_top();
        self.ensure_visible();
    }

    fn last(&self) -> usize {
        self.entries.len().saturating_sub(1)
    }

    pub fn up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.ensure_visible();
    }

    pub fn down(&mut self) {
        self.cursor = (self.cursor + 1).min(self.last());
        self.ensure_visible();
    }

    pub fn home(&mut self) {
        self.cursor = 0;
        self.ensure_visible();
    }

    pub fn end(&mut self) {
        self.cursor = self.last();
        self.ensure_visible();
    }

    /// A column to the left; from the first column, the first name (rule 8).
    pub fn left(&mut self) {
        if self.by_columns() {
            self.cursor = self.cursor.saturating_sub(self.rows.max(1));
            self.ensure_visible();
        } else {
            self.page_up();
        }
    }

    pub fn right(&mut self) {
        if self.by_columns() {
            self.cursor = (self.cursor + self.rows.max(1)).min(self.last());
            self.ensure_visible();
        } else {
            self.page_down();
        }
    }

    /// A page of names; the bar keeps its place on the screen (rule 9).
    /// At the end of the list there is no page to scroll: the last name.
    pub fn page_down(&mut self) {
        let old = self.top;
        self.top += self.page();
        self.clamp_top();
        if self.top == old {
            self.cursor = self.last();
        } else {
            self.cursor = (self.cursor + (self.top - old)).min(self.last());
        }
        self.ensure_visible();
    }

    pub fn page_up(&mut self) {
        let old = self.top;
        self.top = self.top.saturating_sub(self.page());
        self.clamp_top();
        if self.top == old {
            self.cursor = 0;
        } else {
            self.cursor -= old - self.top;
        }
        self.ensure_visible();
    }

    fn page(&self) -> usize {
        if self.by_columns() { self.capacity() } else { self.rows.max(1) }
    }

    pub fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.last());
        self.ensure_visible();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.ensure_visible();
        self.clamp_top();
        self.ensure_visible();
    }

    pub fn set_rows(&mut self, rows: usize) {
        self.rows = rows.max(1);
        self.clamp_top();
        self.ensure_visible();
    }

    // ------------------------------------------------------------ sorting

    /// The same key twice reverses the order (2.3).
    pub fn sort_by(&mut self, sort: Sort) {
        if self.sort == sort {
            self.reverse = !self.reverse;
        } else {
            self.sort = sort;
            self.reverse = false;
        }
        self.resort();
    }

    /// An order set outright, as the setup file has it.
    pub fn set_order(&mut self, sort: Sort, reverse: bool) {
        self.sort = sort;
        self.reverse = reverse;
        self.resort();
    }

    /// The names put in the order again, the cursor and the selection kept.
    fn resort(&mut self) {
        let name = self.current().map(|e| e.name.clone());
        let sel = self.selected_names();
        let up = if self.entries.first().map(|e| e.is_up()).unwrap_or(false) { Some(self.entries.remove(0)) } else { None };
        self.sort_entries();
        if let Some(up) = up {
            self.entries.insert(0, up);
        }
        self.selected = self.entries.iter().map(|e| sel.iter().any(|k| *k == e.name)).collect();
        if let Some(n) = name {
            self.cursor = self.index_of(&n).unwrap_or(0);
        }
        self.ensure_visible();
    }

    fn sort_entries(&mut self) {
        if self.sort == Sort::Unsorted {
            // Folders still first; the disk's order inside each group.
            let (mut d, f): (Vec<Entry>, Vec<Entry>) = core::mem::take(&mut self.entries).into_iter().partition(|e| e.dir);
            if self.reverse {
                d.reverse();
            }
            let mut f = f;
            if self.reverse {
                f.reverse();
            }
            d.extend(f);
            self.entries = d;
            return;
        }
        let (sort, reverse, natural) = (self.sort, self.reverse, self.natural);
        self.entries.sort_by(|a, b| {
            if a.dir != b.dir {
                return if a.dir { Ordering::Less } else { Ordering::Greater };
            }
            let by_name = || compare_names(&a.name, &b.name, natural);
            let o = match sort {
                Sort::Name => by_name(),
                Sort::Ext => compare_names(path::split_ext(&a.name).1, path::split_ext(&b.name).1, natural).then_with(by_name),
                // The newest and the biggest first, as NC had them.
                Sort::Time => b.stamp.cmp(&a.stamp).then_with(by_name),
                Sort::Size => b.size.cmp(&a.size).then_with(by_name),
                Sort::Unsorted => Ordering::Equal,
            };
            if reverse {
                o.reverse()
            } else {
                o
            }
        });
    }

    // ------------------------------------------------------------ selection

    /// Insert: select or unselect the name under the cursor. `..` never.
    pub fn toggle(&mut self) {
        if let Some(e) = self.entries.get(self.cursor) {
            if !e.is_up() {
                self.selected[self.cursor] = !self.selected[self.cursor];
            }
        }
    }

    pub fn set_selected(&mut self, i: usize, on: bool) {
        if let Some(e) = self.entries.get(i) {
            if !e.is_up() {
                self.selected[i] = on;
            }
        }
    }

    /// Gray + / Gray −: select or unselect what the mask matches. Folders
    /// only when `folders` (the "Select folders" option).
    pub fn select_mask(&mut self, mask: &str, on: bool, folders: bool) -> usize {
        let m = Mask::parse(mask, self.short_names);
        let mut n = 0;
        for i in 0..self.entries.len() {
            let e = &self.entries[i];
            if e.is_up() || (e.dir && !folders) {
                continue;
            }
            if m.matches(&e.name) && self.selected[i] != on {
                self.selected[i] = on;
                n += 1;
            }
        }
        n
    }

    /// Gray *: files, and folders with `folders`.
    pub fn invert(&mut self, folders: bool) {
        for i in 0..self.entries.len() {
            let e = &self.entries[i];
            if e.is_up() || (e.dir && !folders) {
                continue;
            }
            self.selected[i] = !self.selected[i];
        }
    }

    pub fn clear_selection(&mut self) {
        for s in self.selected.iter_mut() {
            *s = false;
        }
    }

    pub fn has_selection(&self) -> bool {
        self.selected.iter().any(|&s| s)
    }

    pub fn selected_names(&self) -> Vec<String> {
        self.entries.iter().zip(&self.selected).filter(|(_, &s)| s).map(|(e, _)| e.name.clone()).collect()
    }

    /// What an operation acts on: the selection, else the name under the
    /// cursor - never `..` (2.5).
    pub fn targets(&self) -> Vec<Entry> {
        if self.has_selection() {
            return self.entries.iter().zip(&self.selected).filter(|(_, &s)| s).map(|(e, _)| e.clone()).collect();
        }
        self.current().filter(|e| !e.is_up()).cloned().into_iter().collect()
    }

    /// The selection's files and bytes, for the mini status.
    pub fn selection_total(&self) -> (usize, u64) {
        self.entries.iter().zip(&self.selected).filter(|(_, &s)| s).fold((0, 0), |(n, b), (e, _)| (n + 1, b + e.size))
    }

    /// Kept before an operation, for Ctrl+M.
    pub fn save_selection(&mut self) {
        self.saved = self.selected_names();
    }

    pub fn restore_selection(&mut self) {
        let saved = core::mem::take(&mut self.saved);
        for i in 0..self.entries.len() {
            let n = &self.entries[i].name;
            self.selected[i] = saved.iter().any(|s| self.same_name(s, n));
        }
        self.saved = saved;
    }

    /// After an operation that did these names: unselect them, and only
    /// them, so what stays selected is what still needs doing (2.5).
    pub fn unselect_done(&mut self, done: &[String]) {
        for i in 0..self.entries.len() {
            if done.iter().any(|d| self.same_name(d, &self.entries[i].name)) {
                self.selected[i] = false;
            }
        }
    }

    // ------------------------------------------------------------ quick search

    /// The first name from `from` on that starts with `text` (`*` and `?`
    /// allowed), wrapping round. Quick search and Ctrl+Enter in its box.
    pub fn find_prefix(&self, text: &str, from: usize) -> Option<usize> {
        if text.is_empty() {
            return None;
        }
        let mut pattern = String::from(text);
        pattern.push('*');
        let m = Mask::parse(&pattern, false);
        let n = self.entries.len();
        (0..n).map(|k| (from + k) % n).find(|&i| !self.entries[i].is_up() && m.matches(&self.entries[i].name))
    }
}

/// Without case; with numbers as numbers when `natural`.
pub fn compare_names(a: &str, b: &str, natural: bool) -> Ordering {
    if !natural {
        let la = a.chars().flat_map(|c| c.to_lowercase());
        let lb = b.chars().flat_map(|c| c.to_lowercase());
        return la.cmp(lb).then_with(|| a.cmp(b));
    }
    let (ca, cb): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (mut i, mut j) = (0, 0);
    while i < ca.len() && j < cb.len() {
        if ca[i].is_ascii_digit() && cb[j].is_ascii_digit() {
            let si = i;
            while i < ca.len() && ca[i].is_ascii_digit() {
                i += 1;
            }
            let sj = j;
            while j < cb.len() && cb[j].is_ascii_digit() {
                j += 1;
            }
            let na: String = ca[si..i].iter().collect::<String>();
            let nb: String = cb[sj..j].iter().collect::<String>();
            let (na, nb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
            let o = na.len().cmp(&nb.len()).then_with(|| na.cmp(nb));
            if o != Ordering::Equal {
                return o;
            }
        } else {
            let la = ca[i].to_lowercase().next().unwrap_or(ca[i]);
            let lb = cb[j].to_lowercase().next().unwrap_or(cb[j]);
            if la != lb {
                return la.cmp(&lb);
            }
            i += 1;
            j += 1;
        }
    }
    (ca.len() - i).cmp(&(cb.len() - j)).then_with(|| a.cmp(b))
}
