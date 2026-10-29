//! The menu's commands that are more than a key of the panels (docs/04-menus.md):
//! Help, the user menu and the files a person writes commands into, File
//! Attributes, Find file, History, Volume label, System Info, Folder
//! shortcuts, Languages - and the lists some of them choose from.

use crate::app::{App, Request};
use crate::dialog::{Dialog, Purpose, Tone};
use crate::fs::*;
use crate::lang::{self, Lang};
use crate::line::Line;
use crate::mask::Mask;
use crate::names;
use crate::panel::{Focus, View};
use crate::path;
use crate::usermenu::{self, EXT_FILE, MENU_FILE};
use alloc::string::String;
use alloc::vec::Vec;
use owlosui_core::{Event, Key, KeyCode};

/// The most Find file lists: past it, the list says it stopped.
pub const FOUND_LIMIT: usize = 1000;

/// A list the person chooses from, while it is open: what its lines are.
pub enum Pick {
    /// Alt+F8: the command lines, the oldest first.
    History(Vec<String>),
    /// F2: the menu's items, and the folder it was found in.
    UserMenu(Vec<usermenu::Item>),
    /// Alt+F7: the paths found.
    Found(Vec<String>),
    /// Folder shortcuts: the ten, numbered 0..9.
    Shortcuts,
    /// Options > Languages: English first, then each language file.
    Languages(Vec<Lang>),
}

/// A file's bytes as the commander's text: on DOS each byte is the glyph
/// it shows, as a DOS name's are; elsewhere UTF-8.
pub fn decode(fs: &dyn Fs, data: &[u8]) -> String {
    if fs.short_names() {
        data.iter().map(|&b| b as char).collect()
    } else {
        String::from_utf8_lossy(data).into_owned()
    }
}

/// The commander's text as a file's bytes, the other way.
pub fn encode(fs: &dyn Fs, text: &str) -> Vec<u8> {
    if fs.short_names() {
        text.chars().map(|c| if (c as u32) < 256 { c as u8 } else { b'?' }).collect()
    } else {
        Vec::from(text.as_bytes())
    }
}

impl App {
    /// A text file, whole, if it can be read.
    pub(crate) fn read_text(&mut self, file: &str) -> Option<String> {
        let fs = &mut *self.fs;
        let h = fs.open_read(file).ok()?;
        let mut data = Vec::new();
        let mut buf = [0u8; 1024];
        while let Ok(n) = fs.read(h, &mut buf) {
            if n == 0 || data.len() > 256 * 1024 {
                break;
            }
            data.extend_from_slice(&buf[..n]);
        }
        let _ = fs.close(h);
        Some(decode(&*self.fs, &data))
    }

    /// A file of this name in `dir`, whatever the case of its letters.
    fn named_in(&mut self, dir: &str, name: &str) -> Option<String> {
        let list = self.fs.read_dir(dir).ok()?;
        let e = list.iter().find(|e| !e.dir && e.name.eq_ignore_ascii_case(name))?;
        Some(path::join(dir, &e.name, self.fs.sep()))
    }

    /// Where the setup file lives: the main user menu, the extension
    /// table and the language files are beside it.
    fn setup_dir(&self) -> Option<String> {
        let p = self.setup_path.as_ref()?;
        path::split(p, self.fs.sep()).map(|(d, _)| d)
    }

    /// `name` in `dir`: the one there, or the name a new one gets - upper
    /// case where names are 8.3, lower case elsewhere, as `kc.ini` is.
    fn file_in(&mut self, dir: &str, name: &str) -> String {
        if let Some(p) = self.named_in(dir, name) {
            return p;
        }
        let n = if self.fs.short_names() { String::from(name) } else { name.to_lowercase() };
        path::join(dir, &n, self.fs.sep())
    }

    // ------------------------------------------------------------ help

    /// F1: the keys, in the viewer.
    pub fn help(&mut self) {
        let mut e = crate::editor::Editor::words("Help", crate::help::TEXT, (self.codec)(), self.w, self.h);
        e.set_lang(self.chrome.lang.clone());
        self.editor = Some(e);
    }

    // ------------------------------------------------------------ user menu

    /// F2: the menu of the panel's folder, or the main one; Shift+F2
    /// (`main`): the main one.
    pub fn user_menu(&mut self, main: bool) {
        let here = self.panel().dir.clone();
        let mut file = if main { None } else { self.named_in(&here, MENU_FILE) };
        if file.is_none() {
            if let Some(dir) = self.setup_dir() {
                file = self.named_in(&dir, MENU_FILE);
            }
        }
        let items = file.and_then(|f| self.read_text(&f)).map(|t| usermenu::parse_menu(&t)).unwrap_or_default();
        if items.is_empty() {
            self.message("User menu", "There is no user menu yet: F9, Commands, Menu file edit makes one.");
            return;
        }
        let lines: Vec<String> = items.iter().map(|it| alloc::format!("{}  {}", it.key.unwrap_or(' '), it.title)).collect();
        let over = self.panel_rect(self.active).unwrap_or((0, self.w));
        self.chrome.open_list(if main { "Main menu" } else { "User menu" }, None, &lines, 0, over, &["~R~un", "~C~ancel"]);
        self.pick = Some(Pick::UserMenu(items));
    }

    /// An item's commands, one after another, the `!` words put in for
    /// the file under the cursor.
    fn run_item(&mut self, it: &usermenu::Item) {
        let dir = self.panel().dir.clone();
        let name = self.panel().current().filter(|e| !e.is_up()).map(|e| e.name.clone()).unwrap_or_default();
        let sep = self.fs.sep();
        for c in &it.commands {
            let line = usermenu::expand(c, &dir, &name, sep);
            self.requests.push(Request::Run { dir: dir.clone(), line });
        }
    }

    /// Commands > Menu file edit: which menu - the main one, or this folder's.
    pub fn menu_file_dialog(&mut self) {
        self.dialog = Some(
            Dialog::new(Purpose::MenuFile, "Menu file edit")
                .line("", Tone::Normal)
                .line("Edit the main user menu, or this folder's own?", Tone::Normal)
                .line("", Tone::Normal)
                .buttons(&["~M~ain", "~L~ocal", "~C~ancel"]),
        );
    }

    /// The menu file chosen, in the editor - a new one if it is not there.
    pub fn edit_menu_file(&mut self, main: bool) {
        let dir = if main { self.setup_dir() } else { Some(self.panel().dir.clone()) };
        let Some(dir) = dir else {
            self.message("Menu file edit", "There is no place for the main menu here.");
            return;
        };
        let file = self.file_in(&dir, MENU_FILE);
        self.open_editor(&file, false, true);
    }

    /// Commands > Extension file edit: the table Enter reads.
    pub fn edit_ext_file(&mut self) {
        let Some(dir) = self.setup_dir() else {
            self.message("Extension file edit", "There is no place for the extension table here.");
            return;
        };
        let file = self.file_in(&dir, EXT_FILE);
        self.open_editor(&file, false, true);
    }

    /// Enter on a file: the command the extension table has for it, run;
    /// false if it has none.
    pub(crate) fn run_by_ext(&mut self, dir: &str, name: &str) -> bool {
        let Some(sdir) = self.setup_dir() else { return false };
        let Some(file) = self.named_in(&sdir, EXT_FILE) else { return false };
        let Some(text) = self.read_text(&file) else { return false };
        let Some(cmd) = usermenu::ext_command(&text, name) else { return false };
        let line = usermenu::expand(&cmd, dir, name, self.fs.sep());
        self.cmd.remember(&line);
        self.requests.push(Request::Run { dir: String::from(dir), line });
        true
    }

    // ------------------------------------------------------------ attributes

    /// Ctrl+A: the attributes of the selected files, or of the one under
    /// the cursor; for one file, its date and time too.
    pub fn attrs_dialog(&mut self) {
        if !self.fs.keeps_attrs() {
            self.message("Attributes", "Files here keep no attributes.");
            return;
        }
        let items: Vec<Entry> = self.panel().targets().into_iter().filter(|e| !e.is_up()).collect();
        let Some(first) = items.first().cloned() else { return };
        let what = if items.len() == 1 { alloc::format!("\"{}\"", first.name) } else { alloc::format!("{} files", items.len()) };
        let paths: Vec<String> = items.iter().map(|e| self.panel().path_of(&e.name)).collect();
        let a = first.attrs;
        let mut d = Dialog::new(Purpose::Attributes(paths), "Attributes").line("", Tone::Normal).line(&alloc::format!("The attributes of {}", what), Tone::Normal);
        if items.len() == 1 {
            let stamp = alloc::format!("{} {}", names::date(&first.stamp), names::time(&first.stamp));
            d = d.line("", Tone::Normal).line("Date and time (dd-mm-yy hh:mm)", Tone::Normal).input(Line::selected(&stamp));
        }
        d = d.line("", Tone::Normal).switches(alloc::vec![
            (String::from("Read-only"), a.read_only),
            (String::from("Hidden"), a.hidden),
            (String::from("System"), a.system),
            (String::from("Archive"), a.archive),
        ]);
        self.dialog = Some(d.line("", Tone::Normal).buttons(&["~S~et", "~C~ancel"]));
    }

    /// The dialog's attributes onto every file, and its date onto the one.
    pub(crate) fn set_attributes(&mut self, paths: &[String], d: &Dialog) {
        let on = |i: usize| d.switches.get(i).map(|s| s.1).unwrap_or(false);
        let attrs = Attrs { read_only: on(0), hidden: on(1), system: on(2), archive: on(3) };
        let stamp = d.input.as_ref().map(|l| l.text());
        for p in paths {
            if let Err(e) = self.fs.set_attrs(p, attrs) {
                self.fs_error("Cannot set the attributes of", p, e);
                break;
            }
            if let Some(t) = &stamp {
                let Ok(was) = self.fs.stat(p) else { continue };
                if alloc::format!("{} {}", names::date(&was.stamp), names::time(&was.stamp)) == t.trim() {
                    continue;
                }
                match parse_stamp(t) {
                    Some(s) => {
                        if let Err(e) = self.fs.set_stamp(p, s) {
                            self.fs_error("Cannot set the date of", p, e);
                        }
                    }
                    None => self.message("Attributes", "The date is day-month-year and the time hours:minutes: 29-10-26 08:50."),
                }
            }
        }
        self.reread_both();
    }

    // ------------------------------------------------------------ find file

    /// Alt+F7: which names, and where.
    pub fn find_dialog(&mut self) {
        let mask = if self.fs.short_names() { "*.*" } else { "*" };
        self.dialog = Some(
            Dialog::new(Purpose::Find, "Find file")
                .line("", Tone::Normal)
                .line("Find the files that match", Tone::Normal)
                .input(Line::selected(mask))
                .line("", Tone::Normal)
                .switches(alloc::vec![(String::from("On the whole drive, not only in this folder"), false)])
                .line("", Tone::Normal)
                .buttons(&["~F~ind", "~C~ancel"]),
        );
    }

    /// Every name under the folder (or the drive) the mask matches, in a
    /// list to go to or view.
    pub(crate) fn find(&mut self, mask_text: &str, whole_drive: bool) {
        let sep = self.fs.sep();
        let dir = self.panel().dir.clone();
        let start = if whole_drive { String::from(path::root(&dir, sep)) } else { dir };
        let mask = Mask::parse(mask_text, self.fs.short_names());
        let found = find_files(&mut *self.fs, &start, &mask);
        if found.is_empty() {
            self.message("Find file", &alloc::format!("No file matches {} there.", mask_text.trim()));
            return;
        }
        let head = if found.len() >= FOUND_LIMIT { alloc::format!("The first {} found", found.len()) } else { alloc::format!("{} found", found.len()) };
        self.chrome.open_list("Find file", Some(&head), &found, 0, (0, self.w), &["~G~o to", "~V~iew", "~C~ancel"]);
        self.pick = Some(Pick::Found(found));
    }

    /// The panel on the folder of a path found, the cursor on its name.
    fn go_to(&mut self, full: &str) {
        let sep = self.fs.sep();
        let full = full.trim_end_matches(sep);
        let Some((dir, name)) = path::split(full, sep) else { return };
        let fs = &mut *self.fs;
        if self.panels[self.active].view != View::Files {
            self.panels[self.active].view = View::Files;
        }
        if let Err(e) = self.panels[self.active].load(fs, &dir, Focus::Centre(name)) {
            self.fs_error("Cannot read the folder", &dir, e);
        }
    }

    // ------------------------------------------------------------ history

    /// Alt+F8: the command lines run, to run again or put on the line.
    pub fn history_list(&mut self) {
        let list: Vec<String> = self.cmd.history().to_vec();
        if list.is_empty() {
            self.message("History", "No command has been run yet.");
            return;
        }
        let last = list.len() - 1;
        self.chrome.open_list("History", None, &list, last, (0, self.w), &["~R~un", "~P~ut on the line", "~C~ancel"]);
        self.pick = Some(Pick::History(list));
    }

    // ------------------------------------------------------------ volume label

    /// Commands > Volume label: the label of the drive the panel is on.
    pub fn label_dialog(&mut self) {
        if !self.fs.can_set_label() {
            self.message("Volume label", "Drives here have no label to change.");
            return;
        }
        let root = String::from(path::root(&self.panel().dir, self.fs.sep()));
        let now = self.fs.label(&root).unwrap_or_default();
        self.dialog = Some(
            Dialog::new(Purpose::Label(root.clone()), "Volume label")
                .line("", Tone::Normal)
                .line(&alloc::format!("The label of drive {}", root.trim_end_matches(self.fs.sep())), Tone::Normal)
                .input(Line::selected(&now))
                .line("", Tone::Normal)
                .buttons(&["~S~et", "~C~ancel"]),
        );
    }

    pub(crate) fn set_label(&mut self, root: &str, text: &str) {
        let mut label = String::from(text.trim());
        if self.fs.short_names() {
            label = label.to_uppercase();
            if label.chars().count() > 11 || label.chars().any(|c| "*?/\\|.,;:+=<>[]\"".contains(c)) {
                self.message("Volume label", "A label has at most 11 letters, and no * ? / \\ | . , ; : + = < > [ ] \" in it.");
                return;
            }
        }
        if let Err(e) = self.fs.set_label(root, &label) {
            self.fs_error("Cannot set the label of", root, e);
        } else if let Some(now) = self.fs.label(root).filter(|now| !now.eq_ignore_ascii_case(&label)) {
            // DOS said yes and wrote nothing: an emulator's mounted folder
            // has no label of its own to write.
            let kept = alloc::format!("The drive kept its label {}:", now);
            self.dialog = Some(
                Dialog::new(Purpose::Message, "Volume label")
                    .error()
                    .line("", Tone::Normal)
                    .line(&kept, Tone::Normal)
                    .line("the machine took the call and wrote nothing.", Tone::Normal)
                    .line("", Tone::Normal)
                    .buttons(&["~O~K"]),
            );
        }
        self.refresh_sides();
    }

    // ------------------------------------------------------------ system info

    /// Commands > System Info: what the machine is, its memory, its drives.
    pub fn system_info(&mut self) {
        let mut d = Dialog::new(Purpose::Message, "System Info").line("", Tone::Normal);
        for l in self.system.clone() {
            d = d.line(&l, Tone::Normal);
        }
        if let Some(m) = self.memory {
            d = d.line(&alloc::format!("Free memory: {}", names::human(m)), Tone::Normal);
        }
        d = d.line("", Tone::Normal).line("Drives:", Tone::Normal);
        for drive in self.fs.drives() {
            let space = match self.fs.disk_space(&drive.root) {
                Some((size, free)) => alloc::format!("{:>9} of {:>9} free", names::human(free), names::human(size)),
                None => String::new(),
            };
            let label = self.fs.label(&drive.root).unwrap_or_default();
            d = d.line(&alloc::format!("  {}:  {:<10} {:<12} {}", drive.letter, drive.what, label, space).trim_end(), Tone::Normal);
        }
        self.dialog = Some(d.line("", Tone::Normal).buttons(&["~O~K"]));
    }

    // ------------------------------------------------------------ shortcuts

    /// Ctrl+Shift+0..9: this panel's folder as shortcut `n`.
    pub fn set_shortcut(&mut self, n: usize) {
        self.shortcuts[n] = Some(self.panel().dir.clone());
    }

    /// Commands > Folder shortcuts: the ten, to go to one, set or clear it.
    pub fn shortcuts_list(&mut self) {
        let lines = self.shortcut_lines();
        self.chrome.open_list("Folder shortcuts", Some("Ctrl+Shift and a digit sets one"), &lines, 0, (0, self.w), &["~G~o", "~S~et here", "C~l~ear", "~C~ancel"]);
        self.pick = Some(Pick::Shortcuts);
    }

    fn shortcut_lines(&self) -> Vec<String> {
        let none = self.chrome.lang.tr("(not set)");
        self.shortcuts.iter().enumerate().map(|(i, s)| alloc::format!("{}  {}", i, s.as_deref().unwrap_or(&none))).collect()
    }

    fn go_shortcut(&mut self, n: usize) {
        let Some(dir) = self.shortcuts[n].clone() else { return };
        self.panels[self.active].view = View::Files;
        let fs = &mut *self.fs;
        if let Err(e) = self.panels[self.active].load(fs, &dir, Focus::First) {
            self.fs_error("Cannot read the folder", &dir, e);
        }
    }

    // ------------------------------------------------------------ languages

    /// Options > Languages: English, and every `KC*.LNG` beside the setup.
    pub fn languages(&mut self) {
        let mut list = alloc::vec![Lang::default()];
        if let Some(dir) = self.setup_dir() {
            let names: Vec<String> = self.fs.read_dir(&dir).unwrap_or_default().into_iter().filter(|e| !e.dir && lang::is_lang_file(&e.name)).map(|e| e.name).collect();
            for n in names {
                let p = path::join(&dir, &n, self.fs.sep());
                if let Some(t) = self.read_text(&p) {
                    list.push(Lang::parse(&n, &t));
                }
            }
        }
        let lines: Vec<String> = list.iter().map(|l| if l.file.is_empty() { String::from(l.title()) } else { alloc::format!("{}  ({})", l.title(), l.file) }).collect();
        let now = list.iter().position(|l| l.file.eq_ignore_ascii_case(&self.chrome.lang.file)).unwrap_or(0);
        let head = if list.len() == 1 { Some("A KC*.LNG file beside the setup adds a language") } else { None };
        self.chrome.open_list("Languages", head, &lines, now, (0, self.w), &["~O~K", "~C~ancel"]);
        self.pick = Some(Pick::Languages(list));
    }

    /// The language a setup file names, read from beside it.
    pub(crate) fn load_lang(&mut self, file: &str) {
        let Some(dir) = self.setup_dir() else { return };
        let Some(p) = self.named_in(&dir, file) else { return };
        if let Some(t) = self.read_text(&p) {
            let name = path::name(&p, self.fs.sep());
            self.set_lang(Lang::parse(&name, &t));
        }
    }

    pub fn set_lang(&mut self, lang: Lang) {
        if let Some(e) = self.editor.as_mut() {
            e.set_lang(lang.clone());
        }
        self.chrome.set_lang(lang);
    }

    // ------------------------------------------------------------ the lists

    /// A key while a list is open: a user menu item's letter and a
    /// shortcut's digit choose at once; the rest is the list's.
    pub(crate) fn pick_key(&mut self, k: Key) {
        if let (KeyCode::Char(c), false, false) = (k.code, k.mods.ctrl, k.mods.alt) {
            match &self.pick {
                Some(Pick::UserMenu(items)) => {
                    let c = c.to_ascii_lowercase();
                    if let Some(i) = items.iter().position(|it| it.key.map(|h| h.to_ascii_lowercase()) == Some(c)) {
                        self.chrome.set_list_current(i);
                        self.pick_done(Some(0));
                    }
                    return;
                }
                Some(Pick::Shortcuts) => {
                    if let Some(n) = c.to_digit(10) {
                        self.chrome.set_list_current(n as usize);
                        self.pick_done(Some(0));
                    }
                    return;
                }
                _ => {}
            }
        }
        self.chrome_event(Event::Key(k));
    }

    /// A list's button (`None`: Cancel, or Esc): what it does with the
    /// line its cursor is on.
    pub(crate) fn pick_done(&mut self, button: Option<usize>) {
        let at = self.chrome.list_current().unwrap_or(0);
        let Some(pick) = self.pick.take() else { return };
        // Set here and Clear keep the shortcuts open, to see them change.
        if let Pick::Shortcuts = pick {
            match button {
                Some(1) => self.set_shortcut(at),
                Some(2) => self.shortcuts[at] = None,
                Some(0) => {
                    self.chrome.close_dialog();
                    self.go_shortcut(at);
                    return;
                }
                _ => {
                    self.chrome.close_dialog();
                    return;
                }
            }
            let lines = self.shortcut_lines();
            self.chrome.open_list("Folder shortcuts", Some("Ctrl+Shift and a digit sets one"), &lines, at, (0, self.w), &["~G~o", "~S~et here", "C~l~ear", "~C~ancel"]);
            self.pick = Some(Pick::Shortcuts);
            return;
        }
        self.chrome.close_dialog();
        let Some(b) = button else { return };
        match pick {
            Pick::History(list) => {
                let Some(line) = list.get(at).cloned() else { return };
                if b == 0 {
                    self.cmd.set(&line);
                    self.run_cmd();
                } else if b == 1 {
                    self.cmd.set(&line);
                }
            }
            Pick::UserMenu(items) => {
                if b == 0 {
                    if let Some(it) = items.get(at).cloned() {
                        self.run_item(&it);
                    }
                }
            }
            Pick::Found(list) => {
                let Some(p) = list.get(at).cloned() else { return };
                if b == 0 {
                    self.go_to(&p);
                } else if b == 1 {
                    self.go_to(&p);
                    if !p.ends_with(self.fs.sep()) {
                        self.open_editor(&p, true, false);
                    }
                }
            }
            Pick::Languages(list) => {
                if b == 0 {
                    if let Some(l) = list.get(at).cloned() {
                        self.set_lang(l);
                    }
                }
            }
            Pick::Shortcuts => {}
        }
    }
}

/// The names under `start` the mask matches, folders with their `\` at
/// the end, at most `FOUND_LIMIT`.
pub fn find_files(fs: &mut dyn Fs, start: &str, mask: &Mask) -> Vec<String> {
    let sep = fs.sep();
    let mut found = Vec::new();
    let mut stack = alloc::vec![String::from(start)];
    while let Some(dir) = stack.pop() {
        let Ok(mut list) = fs.read_dir(&dir) else { continue };
        list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        // The folders pushed last first, so they are walked in order.
        for e in list.iter().rev().filter(|e| e.dir) {
            stack.push(path::join(&dir, &e.name, sep));
        }
        for e in &list {
            if mask.matches(&e.name) {
                let mut p = path::join(&dir, &e.name, sep);
                if e.dir {
                    p.push(sep);
                }
                found.push(p);
                if found.len() >= FOUND_LIMIT {
                    return found;
                }
            }
        }
    }
    found
}

/// `dd-mm-yy hh:mm`, as the panels write a date.
pub fn parse_stamp(t: &str) -> Option<Stamp> {
    let mut it = t.split_whitespace();
    let date: Vec<u32> = it.next()?.split(|c| c == '-' || c == '.' || c == '/').map(|s| s.parse().ok()).collect::<Option<Vec<u32>>>()?;
    let time: Vec<u32> = it.next().unwrap_or("00:00").split(':').map(|s| s.parse().ok()).collect::<Option<Vec<u32>>>()?;
    if date.len() != 3 || time.len() < 2 {
        return None;
    }
    let year = match date[2] {
        y if y < 80 => 2000 + y,
        y if y < 100 => 1900 + y,
        y => y,
    };
    let (day, month, hour, minute) = (date[0], date[1], time[0], time[1]);
    if !(1..=31).contains(&day) || !(1..=12).contains(&month) || hour > 23 || minute > 59 || !(1980..=2107).contains(&year) {
        return None;
    }
    Some(Stamp { year: year as u16, month: month as u8, day: day as u8, hour: hour as u8, minute: minute as u8, second: 0 })
}
