//! The commander: two panels, the command line, the key bar, and what the
//! keys and the mouse do to them (docs/01..05, 07).

use crate::colors::{self, Scheme};
use crate::dialog::{Dialog, Outcome, Purpose, Tone};
use crate::fs::*;
use crate::job::{Answer, Ask, Existing, Job, Op, Status};
use crate::line::Line;
use crate::names;
use crate::options::{Options, Page, O};
use crate::panel::{Focus, Mode, Panel, Sort, View};
use crate::path;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use owlosui_core::{Button, Event, Key, KeyCode, Mods, Mouse, MouseKind};

/// What only the host can do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// A command line, in this folder. The host runs it with the panels
    /// out of the way and calls `App::ran` afterwards.
    Run { dir: String, line: String },
    Quit,
}

/// The drive menu, while it is open (docs/02-panels.md 2.8).
pub struct DriveMenu {
    pub panel: usize,
    pub drives: Vec<Drive>,
    /// Each drive's size and free bytes, asked once as the menu opens.
    pub space: Vec<Option<(u64, u64)>>,
    pub cursor: usize,
}

/// Quick search, while its box is open (2.7).
pub struct Search {
    pub panel: usize,
    pub line: Line,
}

/// The job running, and what it was started from.
struct Running {
    job: Job,
    /// The panel the names came from.
    source: usize,
    /// Steps taken: the progress box shows itself once it is clear the
    /// job is not over at once (5.7).
    steps: u32,
}

pub struct App {
    pub fs: Box<dyn Fs>,
    pub panels: [Panel; 2],
    pub active: usize,
    pub cmd: Line,
    pub dialog: Option<Dialog>,
    pub drive_menu: Option<DriveMenu>,
    pub search: Option<Search>,
    pub scheme: Scheme,
    pub w: i16,
    pub h: i16,
    /// Ctrl+O: false shows the terminal behind the panels.
    pub panels_on: bool,
    pub clock: Option<(u8, u8)>,
    /// The modifiers being held, when the host can tell: the key bar
    /// shows their row (1.3).
    pub held: Mods,
    /// What ran on the command line, for the terminal behind the panels.
    pub terminal: Vec<String>,
    /// The switches of the Options menu and its dialogs (04-menus.md).
    pub opt: Options,
    /// Where Save setup writes and the start reads, if the host has a place.
    pub setup_path: Option<String>,
    running: Option<Running>,
    pub(crate) requests: Vec<Request>,
    /// Folders whose size was counted (F3 on a folder, Alt+F6): the size
    /// column shows the number instead of `►SUB-DIR◄`.
    pub sized: Vec<(String, u64)>,
    pub caret: Option<(i16, i16)>,
    /// The viewer or the editor, while one has the screen.
    pub editor: Option<crate::editor::Editor>,
    /// The pull-down menu, while it is open (F9).
    /// OWLOSUI around the panels: the menu bar, the status line, the dialogs (D15).
    pub chrome: crate::chrome::Chrome,
    /// How a file's bytes become the font's glyphs: bytes on DOS; the
    /// host's own on a desktop.
    pub codec: fn() -> Box<dyn crate::editor::Codec>,
    /// A panel's folder tree, while it shows one (Ctrl+T).
    pub trees: [Option<crate::tree::Tree>; 2],
    /// What the Info panel shows, while one is on the screen.
    pub info: Option<crate::sides::Info>,
    /// What Quick view shows, while it is on.
    pub quick: Option<crate::sides::Quick>,
    /// Free DOS memory, for the Info panel: the DOS host says.
    pub memory: Option<u64>,
    /// What the machine is, for System Info: lines the host gives -
    /// `DOS 5.0`.
    pub system: Vec<String>,
    /// Folder shortcuts 0..9 (Ctrl+Shift and a digit).
    pub shortcuts: [Option<String>; 10],
    /// The list open to choose from: the history, the user menu, ...
    pub pick: Option<crate::commands::Pick>,
}

impl App {
    pub fn new(mut fs: Box<dyn Fs>, w: i16, h: i16) -> App {
        let mut left = Panel::new(&*fs);
        let mut right = Panel::new(&*fs);
        let cwd = fs.current_dir();
        let _ = left.load(&mut *fs, &cwd, Focus::First);
        let _ = right.load(&mut *fs, &cwd, Focus::First);
        let mut opt = Options::default();
        // Hidden files: shown on DOS, as NC did; hidden elsewhere (2.9).
        opt.set(O::Hidden, left.show_hidden);
        let mut app = App {
            fs,
            panels: [left, right],
            active: 0,
            cmd: Line::default(),
            dialog: None,
            drive_menu: None,
            search: None,
            scheme: colors::COLOR,
            w,
            h,
            panels_on: true,
            clock: None,
            held: Mods::default(),
            terminal: Vec::new(),
            opt,
            setup_path: None,
            running: None,
            requests: Vec::new(),
            sized: Vec::new(),
            caret: None,
            editor: None,
            chrome: crate::chrome::Chrome::new(w, h),
            codec: || Box::new(crate::editor::Bytes),
            trees: [None, None],
            info: None,
            quick: None,
            memory: None,
            system: Vec::new(),
            shortcuts: Default::default(),
            pick: None,
        };
        app.layout();
        app
    }

    pub fn take_requests(&mut self) -> Vec<Request> {
        core::mem::take(&mut self.requests)
    }

    pub fn resize(&mut self, w: i16, h: i16) {
        if let Some(e) = self.editor.as_mut() {
            e.resize(w, h);
        }
        self.w = w;
        self.h = h;
        self.layout();
        self.chrome.resize(w, h);
    }

    /// A phone in portrait: one panel at a time, the whole width (D10).
    pub fn narrow(&self) -> bool {
        self.w < 60
    }

    pub fn key_bar(&self) -> bool {
        self.opt.on(O::KeyBar)
    }

    /// The first line the panels have: 1 while the menu bar is always there.
    pub fn top(&self) -> i16 {
        (self.opt.on(O::MenuBar) && self.panels_on) as i16
    }

    /// The panels' height: everything between the menu bar, when it stays,
    /// and the command line and the key bar.
    pub fn panel_height(&self) -> i16 {
        (self.h - 1 - self.key_bar() as i16 - self.top()).max(6)
    }

    /// The first row of names, from the panel's top edge: under the column
    /// titles, or right under the edge without them.
    pub fn names_y(&self) -> i16 {
        1 + self.opt.on(O::ColumnTitles) as i16
    }

    /// The rows of names: the frame, the titles and the mini status (with
    /// the line over it) taken away.
    pub fn name_rows(&self) -> i16 {
        (self.panel_height() - 1 - self.names_y() - 2 * self.opt.on(O::MiniStatus) as i16).max(1)
    }

    fn layout(&mut self) {
        let rows = self.name_rows() as usize;
        for p in self.panels.iter_mut() {
            p.set_rows(rows);
        }
        for t in self.trees.iter_mut().flatten() {
            t.set_rows(rows);
        }
    }

    pub fn panel(&self) -> &Panel {
        &self.panels[self.active]
    }

    pub fn other(&self) -> usize {
        1 - self.active
    }

    /// True while a job is running: the host keeps calling `tick`.
    pub fn busy(&self) -> bool {
        self.running.as_ref().map(|r| r.job.status() == Status::Working).unwrap_or(false)
    }

    /// Whether the progress box is shown.
    pub fn progress(&self) -> Option<&Job> {
        self.running.as_ref().filter(|r| r.steps > 8).map(|r| &r.job)
    }

    /// The commander's own scroll-back, what the host ran printed into it.
    pub fn print(&mut self, text: &str) {
        for l in text.split('\n') {
            self.terminal.push(String::from(l.trim_end_matches('\r')));
        }
        let max = 500;
        if self.terminal.len() > max {
            let cut = self.terminal.len() - max;
            self.terminal.drain(..cut);
        }
    }

    /// After a program the host ran: both panels re-read, the cursor kept.
    pub fn ran(&mut self) {
        self.reread_both();
    }

    pub fn reread_both(&mut self) {
        for i in 0..2 {
            let _ = self.panels[i].reread(&mut *self.fs);
        }
    }

    // ---------------------------------------------------------------- jobs

    /// A few steps of the running job.
    pub fn tick(&mut self) {
        for _ in 0..16 {
            let Some(r) = self.running.as_mut() else { return };
            r.steps += 1;
            match r.job.step(&mut *self.fs) {
                Status::Working => continue,
                Status::Asking => {
                    self.ask_job();
                    self.show_dialog();
                    return;
                }
                Status::Done => {
                    self.job_done();
                    self.refresh_sides();
                    self.show_dialog();
                    return;
                }
            }
        }
    }

    fn start_job(&mut self, job: Job, source: usize) {
        self.panels[source].save_selection();
        self.running = Some(Running { job, source, steps: 0 });
        self.tick();
    }

    /// The running job's question, as a dialog.
    fn ask_job(&mut self) {
        let Some(r) = self.running.as_ref() else { return };
        let Some(ask) = r.job.ask.clone() else { return };
        let file_size = r.job.file_size;
        let d = match ask {
            Ask::Error { doing, path, err } => {
                let mut why = String::from(err.kind.text());
                if err.kind == ErrorKind::DiskFull {
                    if let Some(free) = self.fs.free_space(&path) {
                        why = alloc::format!("{}: {} free, {} needed", why, names::human(free), names::human(file_size));
                    }
                }
                why.push('.');
                let mut d = Dialog::new(Purpose::Ask(Vec::new()), "Error").error().line("", Tone::Normal).line(doing, Tone::Normal).line(&path, Tone::Bright).line(&why, Tone::Normal);
                if err.code != 0 {
                    d = d.line(&self.fs_code(err.code), Tone::Code);
                }
                let (labels, answers): (Vec<&str>, Vec<Answer>) = if err.kind.retry_helps() {
                    (alloc::vec!["~R~etry", "~S~kip", "S~k~ip all", "~C~ancel"], alloc::vec![Answer::Retry, Answer::Skip, Answer::SkipAll, Answer::Cancel])
                } else {
                    (alloc::vec!["~S~kip", "S~k~ip all", "~C~ancel"], alloc::vec![Answer::Skip, Answer::SkipAll, Answer::Cancel])
                };
                d.purpose = Purpose::Ask(answers);
                d.line("", Tone::Normal).buttons(&labels)
            }
            Ask::Overwrite { src, dst, path } => {
                let row = |what: &str, e: &Entry| alloc::format!("{:<10}{:>9}  {}  {}", what, names::thousands(e.size), names::date(&e.stamp), names::time(&e.stamp));
                let mut d = Dialog::new(Purpose::Ask(Vec::new()), "Warning")
                    .line("", Tone::Normal)
                    .line("The file already exists:", Tone::Normal)
                    .line(&path, Tone::Bright)
                    .line("", Tone::Normal)
                    .line(&row("New:", &src), Tone::Normal)
                    .line(&row("Existing:", &dst), Tone::Normal)
                    .line("", Tone::Normal)
                    .buttons(&["~O~verwrite", "~A~ll", "~S~kip", "S~k~ip all", "A~p~pend", "~R~ename", "~C~ancel"]);
                d.purpose = Purpose::Ask(alloc::vec![
                    Answer::Yes,
                    Answer::All,
                    Answer::Skip,
                    Answer::SkipAll,
                    Answer::Append,
                    Answer::Rename,
                    Answer::Cancel
                ]);
                d
            }
            Ask::ReadOnly { path, overwrite } => {
                let (q, yes) = if overwrite { ("Overwrite it anyway?", "~O~verwrite") } else { ("Delete it anyway?", "~D~elete") };
                let mut d = Dialog::new(Purpose::Ask(Vec::new()), "Warning")
                    .line("", Tone::Normal)
                    .line("The file is read-only:", Tone::Normal)
                    .line(&path, Tone::Bright)
                    .line(q, Tone::Normal)
                    .line("", Tone::Normal)
                    .buttons(&[yes, "~A~ll", "~S~kip", "~C~ancel"]);
                d.purpose = Purpose::Ask(alloc::vec![Answer::Yes, Answer::All, Answer::Skip, Answer::Cancel]);
                d
            }
            Ask::NotEmpty { path } => {
                let first = alloc::format!("The folder {} is not empty.", path);
                let mut d = Dialog::new(Purpose::Ask(Vec::new()), "Delete")
                    .error()
                    .line("", Tone::Normal)
                    .line(&first, Tone::Normal)
                    .line("Delete it with everything in it?", Tone::Normal)
                    .line("", Tone::Normal)
                    .buttons(&["~D~elete", "~A~ll", "~S~kip", "~C~ancel"]);
                d.purpose = Purpose::Ask(alloc::vec![Answer::Yes, Answer::All, Answer::Skip, Answer::Cancel]);
                d
            }
        };
        self.dialog = Some(d);
    }

    /// The machine's own error number, as the error box writes it small.
    fn fs_code(&self, code: i32) -> String {
        if self.fs.short_names() {
            alloc::format!("DOS error {:X}h", code)
        } else {
            alloc::format!("error {}", code)
        }
    }

    fn job_done(&mut self) {
        let Some(r) = self.running.take() else { return };
        let job = r.job;
        let source = r.source;
        // Both panels re-read, the cursors kept (rule 7); only what was
        // done is unselected (2.5).
        self.reread_both();
        self.panels[source].unselect_done(&job.done);
        if !job.skipped.is_empty() && !job.cancelled {
            let verb = match job.op {
                Op::Copy => "copied",
                Op::Move => "moved",
                Op::Delete { .. } => "deleted",
            };
            let n = job.skipped.len();
            let head = if n == 1 { alloc::format!("1 file was not {}:", verb) } else { alloc::format!("{} files were not {}:", n, verb) };
            let mut d = Dialog::new(Purpose::Message, "Warning").line("", Tone::Normal).line(&head, Tone::Normal);
            for p in job.skipped.iter().take(8) {
                d = d.line(p, Tone::Bright);
            }
            if n > 8 {
                d = d.line(&alloc::format!("and {} more", n - 8), Tone::Normal);
            }
            self.dialog = Some(d.line("", Tone::Normal).buttons(&["~O~K"]));
        }
    }

    // ---------------------------------------------------------------- events

    pub fn handle(&mut self, ev: Event) {
        match ev {
            Event::Key(k) => self.key(k),
            Event::Mouse(m) => self.mouse(m),
            Event::Resize(w, h) => self.resize(w, h),
            _ => {}
        }
        // Info and Quick view follow what the key did to the other panel.
        if self.editor.is_none() {
            self.refresh_sides();
        }
        self.show_dialog();
    }

    fn key(&mut self, k: Key) {
        if self.editor.is_some() {
            self.editor_event(Event::Key(k));
            return;
        }
        if self.dialog.is_some() {
            // OWLOSUI's window has the keys; Esc is Cancel whatever the
            // buttons are.
            self.show_dialog();
            // A button's letter alone, where nothing is typed into: the
            // button, as NC's boxes had it.
            let typing = self.dialog.as_ref().map(|d| d.input.is_some() || !d.switches.is_empty()).unwrap_or(false);
            let letter = match k.code {
                KeyCode::Char(c) if !k.mods.ctrl && (k.mods.alt || !typing) => self.dialog.as_ref().and_then(|d| d.hotkey(c, &self.chrome.lang)),
                _ => None,
            };
            if k.code == KeyCode::Esc {
                self.form_done(Outcome::Cancel);
            } else if let Some(i) = letter {
                self.form_done(Outcome::Button(i));
            } else {
                self.chrome_event(Event::Key(k));
                // The dialog follows the window: what is typed is its text.
                if let Some(d) = self.dialog.as_mut() {
                    self.chrome.read_back(d);
                }
            }
            return;
        }
        if self.running.is_some() {
            if k.code == KeyCode::Esc {
                self.dialog = Some(
                    Dialog::new(Purpose::StopJob, "Stop")
                        .line("", Tone::Normal)
                        .line("Stop the operation?", Tone::Normal)
                        .line("", Tone::Normal)
                        .buttons(&["~S~top", "~C~ontinue"]),
                );
            }
            return;
        }
        if self.chrome.menu_is_open() {
            self.chrome_event(Event::Key(k));
            return;
        }
        if self.drive_menu.is_some() {
            self.drive_key(k);
            return;
        }
        if self.pick.is_some() {
            self.pick_key(k);
            return;
        }
        if self.search.is_some() && self.search_key(k) {
            return;
        }
        if !self.panels_on {
            self.terminal_key(k);
            return;
        }
        self.panel_key(k);
    }

    fn terminal_key(&mut self, k: Key) {
        match (k.code, k.mods.ctrl) {
            (KeyCode::Char('o'), true) => self.panels_on = true,
            (KeyCode::Enter, false) => self.run_cmd(),
            // Esc on an empty line brings the panels back, if so set (03-keys.md).
            (KeyCode::Esc, _) if self.cmd.is_empty() && self.opt.on(O::EscHidesPanels) => self.panels_on = true,
            (KeyCode::Esc, _) => self.cmd.clear(),
            (KeyCode::Up, false) => self.cmd.older(),
            (KeyCode::Down, false) => self.cmd.newer(),
            (KeyCode::F(10), false) => self.ask_quit(),
            _ => {
                self.cmd.key(k, true);
            }
        }
    }

    fn panel_key(&mut self, k: Key) {
        if self.panels[self.active].view == View::Tree && self.tree_key(k) {
            return;
        }
        let m = k.mods;
        let none = m.is_none();
        let ctrl = m.ctrl && !m.alt && !m.shift;
        let shift = m.shift && !m.ctrl && !m.alt;
        let alt = m.alt && !m.ctrl;
        let cmd_empty = self.cmd.is_empty();
        let fs = &mut *self.fs;
        match k.code {
            KeyCode::Up if none => self.panels[self.active].up(),
            KeyCode::Down if none => self.panels[self.active].down(),
            KeyCode::PageUp if none => self.panels[self.active].page_up(),
            KeyCode::PageDown if none => self.panels[self.active].page_down(),
            KeyCode::Home if none && cmd_empty => self.panels[self.active].home(),
            KeyCode::End if none && cmd_empty => self.panels[self.active].end(),
            KeyCode::Left if none && cmd_empty => self.panels[self.active].left(),
            KeyCode::Right if none && cmd_empty => self.panels[self.active].right(),
            // Shift and the arrows select along the way (Far).
            KeyCode::Up | KeyCode::Down if shift => {
                let p = &mut self.panels[self.active];
                let on = !p.selected.get(p.cursor).copied().unwrap_or(false);
                p.set_selected(p.cursor, on);
                if k.code == KeyCode::Up {
                    p.up()
                } else {
                    p.down()
                }
            }
            KeyCode::Tab | KeyCode::BackTab if !m.ctrl && !m.alt => self.switch_panel(),
            KeyCode::Insert if none => {
                let p = &mut self.panels[self.active];
                p.toggle();
                if self.opt.on(O::InsertMovesDown) {
                    p.down();
                }
            }
            KeyCode::Enter if none => self.enter(),
            KeyCode::Enter if ctrl => self.name_to_cmd(false),
            KeyCode::Enter if m.ctrl && m.shift => self.name_to_cmd(true),
            KeyCode::PageUp if ctrl => self.go_up(),
            KeyCode::PageDown if ctrl => {
                let p = &mut self.panels[self.active];
                let _ = p.enter(fs);
            }
            KeyCode::Char('\\') if ctrl => {
                let _ = self.panels[self.active].go_root(fs);
            }
            KeyCode::Backspace if none && cmd_empty => self.go_up(),
            KeyCode::Esc if none && cmd_empty && self.opt.on(O::EscHidesPanels) => self.panels_on = false,
            KeyCode::Esc if none => self.cmd.clear(),
            KeyCode::Char('+') if !m.ctrl && !m.alt && cmd_empty => self.select_dialog(true),
            KeyCode::Char('-') if !m.ctrl && !m.alt && cmd_empty => self.select_dialog(false),
            KeyCode::Char('*') if !m.alt && cmd_empty => {
                let folders = m.ctrl || self.opt.on(O::SelectFolders);
                self.panels[self.active].invert(folders)
            }
            KeyCode::Char('m') if ctrl => self.panels[self.active].restore_selection(),
            KeyCode::Char('r') if ctrl => {
                let _ = self.panels[self.active].reread(fs);
            }
            KeyCode::Char('h') if ctrl => {
                self.opt.flip(O::Hidden);
                self.apply_options();
            }
            KeyCode::Char('o') if ctrl => self.panels_on = false,
            KeyCode::Char('u') if ctrl => {
                self.panels.swap(0, 1);
                self.active = 1 - self.active;
            }
            KeyCode::Char('b') if ctrl => {
                self.opt.flip(O::KeyBar);
                self.layout();
            }
            KeyCode::Char('[') if ctrl => self.path_to_cmd(0),
            KeyCode::Char(']') if ctrl => self.path_to_cmd(1),
            KeyCode::Char(c @ '1'..='5') if ctrl => {
                let mode = [Mode::Brief, Mode::Medium, Mode::Full, Mode::Wide, Mode::Detailed][(c as u8 - b'1') as usize];
                self.set_mode_key(mode);
            }
            // Info and Quick view in the other panel, the tree in this one (03-keys.md).
            KeyCode::Char('l') if ctrl => self.set_view(1 - self.active, View::Info),
            KeyCode::Char('q') if ctrl => self.set_view(1 - self.active, View::Quick),
            KeyCode::Char('t') if ctrl => self.set_view(self.active, View::Tree),
            KeyCode::F(n @ 3..=7) if ctrl => {
                let s = [Sort::Name, Sort::Ext, Sort::Time, Sort::Size, Sort::Unsorted][(n - 3) as usize];
                self.panels[self.active].sort_by(s);
            }
            KeyCode::F(n @ 1..=2) if ctrl => {
                let i = (n - 1) as usize;
                self.panels[i].visible = !self.panels[i].visible;
                if !self.panels[self.active].visible && self.panels[1 - self.active].visible {
                    self.active = 1 - self.active;
                }
            }
            KeyCode::F(n @ 1..=2) if alt => self.open_drive_menu((n - 1) as usize),
            KeyCode::F(6) if alt => self.count_sizes(),
            KeyCode::F(7) if alt => self.find_dialog(),
            KeyCode::F(8) if alt => self.history_list(),
            KeyCode::F(10) if alt => self.set_view(self.active, View::Tree),
            KeyCode::F(1) if none => self.help(),
            KeyCode::F(2) if none => self.user_menu(false),
            KeyCode::F(2) if shift => self.user_menu(true),
            KeyCode::Char('a') if ctrl => self.attrs_dialog(),
            // Ctrl+Shift and a digit: this folder as that shortcut (Far).
            KeyCode::Char(c) if m.ctrl && m.shift && !m.alt && shortcut_digit(c).is_some() => self.set_shortcut(shortcut_digit(c).unwrap_or(0)),
            KeyCode::F(3) if none => self.view(),
            KeyCode::F(4) if none => {
                if let Some(p) = self.panel().current_path() {
                    if !self.panel().current().map(|e| e.dir).unwrap_or(true) {
                        self.open_editor(&p, false, false);
                    }
                }
            }
            KeyCode::F(4) if shift => {
                self.dialog = Some(
                    Dialog::new(Purpose::EditNew, "Edit")
                        .line("", Tone::Normal)
                        .line("Edit the file", Tone::Normal)
                        .input(Line::default())
                        .line("", Tone::Normal)
                        .buttons(&["~E~dit", "~C~ancel"]),
                );
            }
            KeyCode::F(5) if none => self.transfer_dialog(Op::Copy, false),
            KeyCode::F(6) if none => self.transfer_dialog(Op::Move, false),
            KeyCode::F(5) if shift => self.transfer_dialog(Op::Copy, true),
            KeyCode::F(6) if shift => self.transfer_dialog(Op::Move, true),
            KeyCode::F(7) if none => {
                self.dialog = Some(
                    Dialog::new(Purpose::MakeDir, "Make folder")
                        .line("", Tone::Normal)
                        .line("Create the folder", Tone::Normal)
                        .input(Line::default())
                        .line("", Tone::Normal)
                        .buttons(&["~M~ake", "~C~ancel"]),
                );
            }
            KeyCode::F(8) if none => self.delete_dialog(self.fs.has_trash()),
            KeyCode::Delete if none && cmd_empty => self.delete_dialog(self.fs.has_trash()),
            KeyCode::F(8) | KeyCode::Delete if shift => self.delete_dialog(false),
            KeyCode::F(10) if none => self.ask_quit(),
            KeyCode::F(9) if none => self.open_menu(if self.active == 0 { 0 } else { 4 }),
            KeyCode::F(9) if shift => self.save_setup(),
            KeyCode::Char('f') if ctrl => self.filter_dialog(),
            KeyCode::Char('x') if alt => self.ask_quit(),
            KeyCode::Char(c) if alt && !m.ctrl && c.is_alphanumeric() || alt && (c == '.' || c == '_') => {
                let mut line = Line::default();
                let mut b = [0u8; 4];
                line.insert(c.encode_utf8(&mut b));
                self.search = Some(Search { panel: self.active, line });
                self.search_jump(0);
            }
            // The same without Alt while the command line is empty, if so set (2.7).
            KeyCode::Char(c) if self.opt.on(O::SearchWithoutAlt) && cmd_empty && !m.ctrl && !m.alt && (c.is_alphanumeric() || c == '.' || c == '_') => {
                let mut line = Line::default();
                let mut b = [0u8; 4];
                line.insert(c.encode_utf8(&mut b));
                self.search = Some(Search { panel: self.active, line });
                self.search_jump(0);
            }
            _ => {
                // Everything else is typing: the command line.
                if self.cmd.key(k, !cmd_empty) {
                    return;
                }
                // Ctrl+E / Ctrl+X through the history with nothing typed yet.
                if ctrl && k.code == KeyCode::Char('e') {
                    self.cmd.older();
                }
            }
        }
    }

    fn switch_panel(&mut self) {
        // Info and Quick view are about this panel: Tab does not go there.
        if matches!(self.panels[1 - self.active].view, View::Info | View::Quick) {
            return;
        }
        if self.panels[1 - self.active].visible || self.narrow() {
            self.active = 1 - self.active;
            self.panels[self.active].visible = true;
        }
    }

    fn go_up(&mut self) {
        let fs = &mut *self.fs;
        let p = &mut self.panels[self.active];
        if let Err(e) = p.go_up(fs) {
            self.fs_error("Cannot read the folder", &self.panels[self.active].dir.clone(), e);
        }
    }

    fn enter(&mut self) {
        if !self.cmd.is_empty() {
            self.run_cmd();
            return;
        }
        let Some(e) = self.panel().current().cloned() else { return };
        if e.is_up() {
            self.go_up();
            return;
        }
        if e.dir {
            let fs = &mut *self.fs;
            if let Err(err) = self.panels[self.active].enter(fs) {
                let p = self.panels[self.active].path_of(&e.name);
                self.fs_error("Cannot read the folder", &p, err);
            }
            return;
        }
        let dir = self.panel().dir.clone();
        if is_program(&e.name, self.fs.short_names()) {
            let line = quoted(&e.name);
            self.cmd.remember(&line);
            self.requests.push(Request::Run { dir, line });
        } else if !self.run_by_ext(&dir, &e.name) {
            // Not a program, and nothing in the extension table: the viewer (D8).
            let p = path::join(&dir, &e.name, self.fs.sep());
            self.open_editor(&p, true, false);
        }
    }

    pub(crate) fn run_cmd(&mut self) {
        let line = self.cmd.text();
        self.cmd.remember(&line);
        self.cmd.clear();
        let trimmed = line.trim();
        // `cd` is the commander's own: it moves the panel (07-command-line.md).
        let lower = trimmed.to_lowercase();
        if lower == "cd" || lower.starts_with("cd ") || lower.starts_with("cd\\") || lower.starts_with("cd..") {
            let arg = trimmed[2..].trim();
            if !arg.is_empty() {
                let to = self.resolve_dir(arg);
                let fs = &mut *self.fs;
                if let Err(e) = self.panels[self.active].load(fs, &to, Focus::First) {
                    self.fs_error("Cannot change to", &to, e);
                }
            }
            return;
        }
        // `C:` alone: that drive.
        if trimmed.len() == 2 && trimmed.ends_with(':') && trimmed.as_bytes()[0].is_ascii_alphabetic() {
            let root = alloc::format!("{}{}", trimmed.to_uppercase(), self.fs.sep());
            let fs = &mut *self.fs;
            if let Err(e) = self.panels[self.active].go_drive(fs, &root) {
                self.fs_error("Cannot change to", &root, e);
            }
            return;
        }
        if trimmed.is_empty() {
            return;
        }
        let dir = self.panel().dir.clone();
        self.requests.push(Request::Run { dir, line });
    }

    fn resolve_dir(&self, arg: &str) -> String {
        let sep = self.fs.sep();
        let base = self.panel().dir.clone();
        if path::root_len(arg, sep) > 0 {
            return String::from(arg);
        }
        let mut cur = base;
        for part in arg.split(sep) {
            match part {
                "" | "." => {}
                ".." => {
                    if let Some((p, _)) = path::split(&cur, sep) {
                        cur = p;
                    }
                }
                p => cur = path::join(&cur, p, sep),
            }
        }
        cur
    }

    fn name_to_cmd(&mut self, full: bool) {
        let Some(e) = self.panel().current() else { return };
        let text = if e.is_up() {
            String::from("..")
        } else if full {
            self.panel().path_of(&e.name)
        } else {
            e.name.clone()
        };
        let mut t = quoted(&text);
        t.push(' ');
        self.cmd.insert(&t);
    }

    fn path_to_cmd(&mut self, panel: usize) {
        let mut t = quoted(&self.panels[panel].dir);
        t.push(' ');
        self.cmd.insert(&t);
    }

    fn view(&mut self) {
        let Some(e) = self.panel().current().cloned() else { return };
        if e.dir {
            if !e.is_up() {
                self.count_sizes();
            }
            return;
        }
        if let Some(p) = self.panel().current_path() {
            self.open_editor(&p, true, false);
        }
    }

    /// Alt+F6, and F3 on a folder (Far): what is in the folders, counted.
    fn count_sizes(&mut self) {
        let p = &self.panels[self.active];
        let mut list: Vec<Entry> = p.targets().into_iter().filter(|e| e.dir).collect();
        if list.is_empty() {
            if let Some(e) = p.current().filter(|e| e.dir && !e.is_up()) {
                list.push(e.clone());
            }
        }
        let sep = self.fs.sep();
        for e in list {
            let full = path::join(&self.panels[self.active].dir, &e.name, sep);
            let total = dir_size(&mut *self.fs, &full);
            self.sized.retain(|(n, _)| *n != full);
            self.sized.push((full, total));
        }
    }

    pub fn counted(&self, full: &str) -> Option<u64> {
        self.sized.iter().find(|(n, _)| n == full).map(|(_, s)| *s)
    }

    // ---------------------------------------------------------------- dialogs

    fn ask_quit(&mut self) {
        if !self.opt.on(O::ConfirmQuit) {
            self.requests.push(Request::Quit);
            return;
        }
        self.dialog = Some(
            Dialog::new(Purpose::Quit, "Kirin Commander")
                .line("", Tone::Normal)
                .line("Do you want to quit Kirin Commander?", Tone::Normal)
                .line("", Tone::Normal)
                .buttons(&["~Y~es", "~N~o"]),
        );
    }

    fn select_dialog(&mut self, on: bool) {
        let (title, line) = if on { ("Select", "Select the files") } else { ("Unselect", "Unselect the files") };
        let mask = if self.fs.short_names() { "*.*" } else { "*" };
        self.dialog = Some(
            Dialog::new(Purpose::Select { on }, title)
                .line("", Tone::Normal)
                .line(line, Tone::Normal)
                .input(Line::selected(mask))
                .line("", Tone::Normal)
                .buttons(&["~O~K", "~C~ancel"]),
        );
    }

    /// F5, F6; `here` - Shift+F5 / Shift+F6: the one file under the cursor,
    /// its own name in the line (a copy or a rename in place).
    fn transfer_dialog(&mut self, op: Op, here: bool) {
        let p = self.panel();
        let items = if here { p.current().filter(|e| !e.is_up()).cloned().into_iter().collect() } else { p.targets() };
        if items.is_empty() {
            return;
        }
        let what = if items.len() == 1 { alloc::format!("\"{}\"", items[0].name) } else { alloc::format!("{} files", items.len()) };
        let (title, verb, button) = match op {
            Op::Copy => ("Copy", "Copy", "~C~opy"),
            _ => ("Rename/Move", "Rename or move", "~R~ename"),
        };
        let other = &self.panels[self.other()];
        let to_other = !here && (other.visible || self.narrow());
        let target = if to_other {
            let mut t = other.dir.clone();
            if !t.ends_with(self.fs.sep()) {
                t.push(self.fs.sep());
            }
            t
        } else {
            items[0].name.clone()
        };
        let existing = if self.opt.on(O::ConfirmOverwrite) { Existing::Ask } else { Existing::Overwrite };
        // The question turned off: to the other panel at once. A copy in
        // place needs its new name typed, so it always asks.
        let confirm = if op == Op::Copy { O::ConfirmCopy } else { O::ConfirmMove };
        if to_other && !self.opt.on(confirm) {
            self.start_transfer(op, here, &target, existing, true);
            return;
        }
        let mut d = Dialog::new(Purpose::Transfer { op, here }, title)
            .line("", Tone::Normal)
            .line(&alloc::format!("{} {} to", verb, what), Tone::Normal)
            .input(Line::selected(&target));
        d = d.line("", Tone::Normal);
        d.existing = Some(existing);
        d.check = if self.fs.keeps_attrs() { Some(true) } else { None };
        self.dialog = Some(d.buttons(&[button, "~C~ancel"]));
    }

    fn start_transfer(&mut self, op: Op, here: bool, target: &str, existing: Existing, copy_attrs: bool) {
        if target.trim().is_empty() {
            return;
        }
        let p = self.panel();
        // Shift+F5/F6: the one name under the cursor.
        let items: Vec<Entry> = if here { p.current().filter(|e| !e.is_up()).cloned().into_iter().collect() } else { p.targets() };
        let dir = p.dir.clone();
        match Job::new(&mut *self.fs, op, &dir, &items, target) {
            Ok(mut job) => {
                job.existing = existing;
                job.copy_attrs = copy_attrs;
                job.read_only_all = !self.opt.on(O::ConfirmReadOnly);
                job.count(&mut *self.fs);
                self.start_job(job, self.active);
            }
            Err((_, why)) => self.message("Error", &why),
        }
    }

    fn start_delete(&mut self, trash: bool) {
        let items = self.panel().targets();
        let dir = self.panel().dir.clone();
        if let Ok(mut job) = Job::new(&mut *self.fs, Op::Delete { trash }, &dir, &items, "") {
            job.yes_all = !self.opt.on(O::ConfirmNotEmpty);
            job.read_only_all = !self.opt.on(O::ConfirmReadOnly);
            self.start_job(job, self.active);
        }
    }

    fn delete_dialog(&mut self, trash: bool) {
        let items = self.panel().targets();
        if items.is_empty() {
            return;
        }
        if !self.opt.on(O::ConfirmDelete) {
            self.start_delete(trash);
            return;
        }
        let what = if items.len() == 1 {
            alloc::format!("\"{}\"", items[0].name)
        } else if items.iter().all(|e| !e.dir) {
            alloc::format!("{} files", items.len())
        } else {
            alloc::format!("{} items", items.len())
        };
        let mut d = Dialog::new(Purpose::Delete { trash }, "Delete").line("", Tone::Normal).line("Do you wish to delete", Tone::Normal).line(&what, Tone::Normal);
        if self.fs.has_trash() {
            d = d.line(if trash { "to the Recycle Bin" } else { "for good" }, Tone::Normal);
        }
        self.dialog = Some(d.line("", Tone::Normal).buttons(&["~D~elete", "~C~ancel"]));
    }

    pub(crate) fn fs_error(&mut self, doing: &str, path: &str, err: FsError) {
        let mut why = String::from(err.kind.text());
        why.push('.');
        let mut d = Dialog::new(Purpose::Message, "Error").error().line("", Tone::Normal).line(doing, Tone::Normal).line(path, Tone::Bright).line(&why, Tone::Normal);
        if err.code != 0 {
            d = d.line(&self.fs_code(err.code), Tone::Code);
        }
        self.dialog = Some(d.line("", Tone::Normal).buttons(&["~O~K"]));
    }

    pub(crate) fn message(&mut self, title: &str, text: &str) {
        self.dialog = Some(Dialog::new(Purpose::Message, title).error().line("", Tone::Normal).line(text, Tone::Normal).line("", Tone::Normal).buttons(&["~O~K"]));
    }

    fn dialog_done(&mut self, o: Outcome) {
        let Some(d) = self.dialog.take() else { return };
        let button = match o {
            Outcome::Button(i) => Some(i),
            Outcome::Cancel => None,
        };
        // The last button is Cancel / No everywhere but in a job's question.
        let yes = button.map(|b| b + 1 < d.buttons.len() || d.buttons.len() == 1).unwrap_or(false);
        match d.purpose.clone() {
            Purpose::Ask(answers) => {
                let a = match button {
                    Some(i) => answers.get(i).copied().unwrap_or(Answer::Cancel),
                    None => Answer::Cancel,
                };
                if let Some(r) = self.running.as_mut() {
                    r.job.answer(&mut *self.fs, a);
                }
                self.tick();
            }
            Purpose::StopJob => {
                if button == Some(0) {
                    if let Some(r) = self.running.as_mut() {
                        r.job.stop(&mut *self.fs);
                    }
                    self.tick();
                }
            }
            Purpose::Quit => {
                if button == Some(0) {
                    self.requests.push(Request::Quit);
                }
            }
            Purpose::Colours => match button {
                Some(0) => self.scheme = colors::COLOR,
                Some(1) => self.scheme = colors::BLACK_WHITE,
                Some(2) => self.scheme = colors::LAPTOP,
                _ => {}
            },
            Purpose::Message => {}
            _ if !yes => {}
            Purpose::MenuFile => self.edit_menu_file(button == Some(0)),
            Purpose::Attributes(paths) => self.set_attributes(&paths, &d),
            Purpose::Find => self.find(&d.text(), d.switches.first().map(|s| s.1).unwrap_or(false)),
            Purpose::Label(root) => self.set_label(&root, &d.text()),
            Purpose::MakeDir => self.make_dir(&d.text()),
            Purpose::Select { on } => {
                let folders = self.opt.on(O::SelectFolders);
                self.panels[self.active].select_mask(&d.text(), on, folders);
            }
            Purpose::Filter => {
                let t = d.text();
                let p = &mut self.panels[self.active];
                p.filter = if t.trim().is_empty() { None } else { Some(t) };
                let _ = p.reread(&mut *self.fs);
            }
            Purpose::EditNew => {
                let t = d.text();
                if !t.trim().is_empty() {
                    let full = if path::root_len(&t, self.fs.sep()) > 0 { t } else { self.panel().path_of(t.trim()) };
                    self.open_editor(&full, false, true);
                }
            }
            Purpose::Delete { trash } => self.start_delete(trash),
            Purpose::Transfer { op, here } => {
                self.start_transfer(op, here, &d.text(), d.existing.unwrap_or(Existing::Ask), d.check.unwrap_or(false));
            }
            Purpose::Options(page) => {
                let list = page.switches(self.fs.short_names());
                for (o, (_, on)) in list.iter().zip(&d.switches) {
                    self.opt.set(*o, *on);
                }
                self.apply_options();
            }
        }
    }

    /// One of the Options dialogs, its switches as they are now.
    fn options_dialog(&mut self, page: Page) {
        let switches = page.switches(self.fs.short_names()).into_iter().map(|o| (String::from(Options::label(o)), self.opt.on(o))).collect();
        self.dialog = Some(
            Dialog::new(Purpose::Options(page), page.title())
                .line("", Tone::Normal)
                .line(page.intro(), Tone::Normal)
                .switches(switches)
                .line("", Tone::Normal)
                .buttons(&["~O~K", "~C~ancel"]),
        );
    }

    /// Save setup (Shift+F9): into the host's setup file, its folder made
    /// if it is not there yet.
    pub fn save_setup(&mut self) {
        let Some(path) = self.setup_path.clone() else {
            self.message("Save setup", "There is no place to keep the setup here.");
            return;
        };
        let sep = self.fs.sep();
        if let Some((dir, _)) = path::split(&path, sep) {
            if self.fs.stat(&dir).is_err() {
                let _ = self.fs.make_dir(&dir);
            }
        }
        let text = crate::commands::encode(&*self.fs, &crate::setup::text(self));
        let fs = &mut *self.fs;
        let written = fs.create(&path).and_then(|h| {
            let r = fs.write(h, &text);
            let c = fs.close(h);
            r.and(c)
        });
        if let Err(e) = written {
            self.fs_error("Cannot save the setup", &path, e);
        }
    }

    /// The setup file, if there is one, read at the start.
    pub fn load_setup(&mut self) {
        let Some(path) = self.setup_path.clone() else { return };
        let Some(text) = self.read_text(&path) else { return };
        crate::setup::read(self, &text);
        self.apply_options();
    }

    /// What the switches change beyond the moment they are read: the
    /// hidden files in both panels, and the rows the panels have.
    pub(crate) fn apply_options(&mut self) {
        let hidden = self.opt.on(O::Hidden);
        for p in self.panels.iter_mut() {
            if p.show_hidden != hidden {
                p.show_hidden = hidden;
                let _ = p.reread(&mut *self.fs);
            }
        }
        self.layout();
    }

    /// F7: one level or several (`A\B\C`); on 8.3, each name checked first.
    fn make_dir(&mut self, text: &str) {
        let sep = self.fs.sep();
        let text = text.trim().trim_end_matches(sep);
        if text.is_empty() {
            return;
        }
        let dir = self.panel().dir.clone();
        let full = if path::root_len(text, sep) > 0 { String::from(text) } else { path::join(&dir, text, sep) };
        let rel: Vec<&str> = full[path::root_len(&full, sep)..].split(sep).filter(|s| !s.is_empty()).collect();
        if self.fs.short_names() {
            for part in &rel {
                if let Some(why) = bad_short_name(part) {
                    self.message("Make folder", why);
                    return;
                }
            }
        }
        let mut cur = String::from(path::root(&full, sep));
        for part in &rel {
            cur = path::join(&cur, part, sep);
            match self.fs.stat(&cur) {
                Ok(e) if e.dir => continue,
                Ok(_) => {
                    self.fs_error("Cannot make the folder", &cur, FsError::new(ErrorKind::Exists));
                    return;
                }
                Err(_) => {}
            }
            if let Err(e) = self.fs.make_dir(&cur) {
                self.fs_error("Cannot make the folder", &cur, e);
                return;
            }
        }
        // The cursor on the new folder - its first level, which is the one
        // this panel shows (rule 7).
        let first = if path::split(&full, sep).map(|(p, _)| path::is_inside(&p, &dir, sep, self.fs.case_sensitive())).unwrap_or(false) {
            full[dir.trim_end_matches(sep).len()..].trim_start_matches(sep).split(sep).next().unwrap_or("").to_string()
        } else {
            String::new()
        };
        let fs = &mut *self.fs;
        if first.is_empty() {
            let _ = self.panels[self.active].reread(fs);
        } else {
            let _ = self.panels[self.active].reread_onto(fs, &first);
        }
        let _ = self.panels[1 - self.active].reread(fs);
    }

    // ---------------------------------------------------------------- the menu

    /// F9: the menu bar's `title` open - OWLOSUI's menus, with what can be
    /// done and what is ticked as the commander is now.
    pub fn open_menu(&mut self, title: usize) {
        self.sync_menus();
        self.chrome.open_menu(title);
    }

    fn sync_menus(&mut self) {
        let mut chrome = core::mem::replace(&mut self.chrome, crate::chrome::Chrome::new(1, 1));
        chrome.set_menus(&|t, label| self.menu_ticked(t, label), &|t, label| self.menu_enabled(t, label));
        self.chrome = chrome;
    }

    /// An event for OWLOSUI's part of the screen, and what it chose: a
    /// menu item, or a key of the status line clicked.
    pub(crate) fn chrome_event(&mut self, ev: Event) {
        for out in self.chrome.handle(ev) {
            match out {
                crate::chrome::Out::Item(t, i) => {
                    if let Some(act) = crate::menu::items(t).get(i).and_then(|it| it.key) {
                        self.chrome.ui.close_menu();
                        self.menu_out(crate::menu::MenuOut::Run(t, act));
                    }
                }
                crate::chrome::Out::Key(k) => self.key(k),
                // The open list's answer: for now the drive menu is the one.
                crate::chrome::Out::Ok => {
                    if self.drive_menu.is_some() {
                        let i = self.chrome.list_current().unwrap_or(0);
                        self.choose_drive(i);
                    }
                }
                crate::chrome::Out::Cancel if self.dialog.is_some() => self.form_done(Outcome::Cancel),
                crate::chrome::Out::Cancel if self.pick.is_some() => self.pick_done(None),
                crate::chrome::Out::Button(i) if self.pick.is_some() => self.pick_done(Some(i)),
                crate::chrome::Out::Cancel => {
                    self.chrome.close_dialog();
                    self.drive_menu = None;
                }
                crate::chrome::Out::Button(i) => self.form_done(Outcome::Button(i)),
            }
        }
    }

    /// A button of the dialog's window: what the person changed in it read
    /// back, the window closed, and the dialog answered as it always was.
    fn form_done(&mut self, o: Outcome) {
        if let Some(d) = self.dialog.as_mut() {
            self.chrome.read_back(d);
        }
        self.chrome.close_dialog();
        self.dialog_done(o);
        self.show_dialog();
    }

    /// The commander's dialog, if it has one and OWLOSUI does not show it
    /// yet, as OWLOSUI's window; none, and the window goes.
    pub fn show_dialog(&mut self) {
        match &self.dialog {
            Some(d) if !self.chrome.form_is_open() && self.editor.is_none() => self.chrome.open_dialog(d),
            None if self.chrome.form_is_open() => self.chrome.close_dialog(),
            _ => {}
        }
    }

    fn menu_out(&mut self, out: crate::menu::MenuOut) {
        use crate::menu::{Act, MenuOut};
        match out {
            MenuOut::Stay => {}
            MenuOut::Close => self.chrome.ui.close_menu(),
            MenuOut::Run(title, act) => {
                self.chrome.ui.close_menu();
                // Left and Right act on their own panel, whichever is active.
                let side = match title {
                    0 => Some(0),
                    4 => Some(1),
                    _ => None,
                };
                let was = self.active;
                if let Some(s) = side {
                    self.active = s;
                    self.panels[s].visible = true;
                }
                match act {
                    Act::Key(k) => self.key(k),
                    Act::Compare => self.compare(),
                    Act::Colours => self.colours_dialog(),
                    Act::Options(page) => self.options_dialog(page),
                    Act::Toggle(o) => {
                        self.opt.flip(o);
                        self.apply_options();
                    }
                    Act::SaveSetup => self.save_setup(),
                    Act::View(v) => self.set_view(side.unwrap_or(self.active), v),
                    Act::MenuFile => self.menu_file_dialog(),
                    Act::ExtFile => self.edit_ext_file(),
                    Act::Label => self.label_dialog(),
                    Act::SystemInfo => self.system_info(),
                    Act::Shortcuts => self.shortcuts_list(),
                    Act::Languages => self.languages(),
                }
                // A drive menu or a dialog stays with the panel it was opened for.
                if side.is_some() && self.drive_menu.is_none() && self.dialog.is_none() {
                    self.active = was;
                }
                // Info and Quick view are never the active panel.
                if matches!(self.panels[self.active].view, View::Info | View::Quick) {
                    self.active = 1 - self.active;
                }
            }
        }
    }

    /// Which items can be chosen here: attributes where files keep them, a
    /// label where a drive's can be changed.
    pub fn menu_enabled(&self, title: usize, label: &str) -> bool {
        match (title, crate::menu::plain(label).as_str()) {
            (1, "File Attributes") => self.fs.keeps_attrs(),
            (2, "Volume label") => self.fs.can_set_label(),
            _ => true,
        }
    }

    /// Which items are ticked: the panel's mode and order, the switches.
    pub fn menu_ticked(&self, title: usize, label: &str) -> bool {
        let p = &self.panels[if title == 4 { 1 } else { 0 }];
        match (title, crate::menu::plain(label).as_str()) {
            // A mode is ticked while the panel shows its files.
            (0 | 4, "Brief") => p.view == View::Files && p.mode == Mode::Brief,
            (0 | 4, "Medium") => p.view == View::Files && p.mode == Mode::Medium,
            (0 | 4, "Full") => p.view == View::Files && p.mode == Mode::Full,
            (0 | 4, "Wide") => p.view == View::Files && p.mode == Mode::Wide,
            (0 | 4, "Detailed") => p.view == View::Files && p.mode == Mode::Detailed,
            (0 | 4, "Info panel") => p.view == View::Info,
            (0 | 4, "Tree panel") => p.view == View::Tree,
            (0 | 4, "Quick view") => p.view == View::Quick,
            (0 | 4, "Name") => p.sort == Sort::Name,
            (0 | 4, "eXtension") => p.sort == Sort::Ext,
            (0 | 4, "tiMe") => p.sort == Sort::Time,
            (0 | 4, "Size") => p.sort == Sort::Size,
            (0 | 4, "Unsorted") => p.sort == Sort::Unsorted,
            (3, "Menu bar always visible") => self.opt.on(O::MenuBar),
            (3, "Key bar") => self.opt.on(O::KeyBar),
            (3, "Clock") => self.opt.on(O::Clock),
            _ => false,
        }
    }

    /// Compare folders: in each panel, the files the other has not, or
    /// has older, are selected.
    fn compare(&mut self) {
        for (a, b) in [(0usize, 1usize), (1, 0)] {
            let theirs: Vec<(String, Stamp)> =
                self.panels[b].entries.iter().filter(|e| !e.dir).map(|e| (e.name.to_lowercase(), e.stamp)).collect();
            let p = &mut self.panels[a];
            for i in 0..p.entries.len() {
                let e = &p.entries[i];
                if e.dir {
                    continue;
                }
                let newer = match theirs.iter().find(|(n, _)| *n == e.name.to_lowercase()) {
                    None => true,
                    Some((_, s)) => e.stamp > *s,
                };
                p.selected[i] = newer;
            }
        }
    }

    fn colours_dialog(&mut self) {
        self.dialog = Some(
            Dialog::new(Purpose::Colours, "Colours")
                .line("", Tone::Normal)
                .line("The colour scheme", Tone::Normal)
                .line("", Tone::Normal)
                .buttons(&["~C~olor", "~B~lack & White", "~L~aptop", "Ca~n~cel"]),
        );
    }

    fn filter_dialog(&mut self) {
        let now = self.panel().filter.clone().unwrap_or_default();
        self.dialog = Some(
            Dialog::new(Purpose::Filter, "Filter")
                .line("", Tone::Normal)
                .line("Show only the files that match (empty: all)", Tone::Normal)
                .input(Line::selected(&now))
                .line("", Tone::Normal)
                .buttons(&["~O~K", "~C~ancel"]),
        );
    }

    // ---------------------------------------------------------------- drive menu

    fn open_drive_menu(&mut self, panel: usize) {
        let drives = self.fs.drives();
        if drives.is_empty() {
            return;
        }
        let here = path::root(&self.panels[panel].dir, self.fs.sep()).to_uppercase();
        // The cursor on the drive the panel is on now (2.8).
        let cursor = drives.iter().position(|d| d.root.to_uppercase() == here).unwrap_or(0);
        let space = drives.iter().map(|d| self.fs.disk_space(&d.root)).collect();
        self.panels[panel].visible = true;
        let m = DriveMenu { panel, drives, space, cursor };
        // OWLOSUI's list in its grey dialog, over the panel it is for.
        let (title, head, lines) = crate::screen::drive_lines(self, &m);
        let over = self.panel_rect(panel).unwrap_or((0, self.w));
        self.chrome.open_list(&title, head.as_deref(), &lines, cursor, over, &[]);
        self.drive_menu = Some(m);
    }

    /// A key in the drive menu: a drive's letter chooses it at once (2.8);
    /// the rest is the list's and its buttons'.
    fn drive_key(&mut self, k: Key) {
        let Some(m) = self.drive_menu.as_ref() else { return };
        if let KeyCode::Char(c) = k.code {
            if !k.mods.ctrl && !k.mods.alt {
                let c = c.to_ascii_uppercase();
                if let Some(i) = m.drives.iter().position(|d| d.letter.to_ascii_uppercase() == c) {
                    self.choose_drive(i);
                }
                return;
            }
        }
        // Left and Right move along the drives too, as NC's row of letters did.
        let k = match k.code {
            KeyCode::Left => Key { code: KeyCode::Up, mods: k.mods },
            KeyCode::Right => Key { code: KeyCode::Down, mods: k.mods },
            _ => k,
        };
        self.chrome_event(Event::Key(k));
    }

    fn choose_drive(&mut self, i: usize) {
        self.chrome.close_dialog();
        let Some(m) = self.drive_menu.take() else { return };
        let root = m.drives[i].root.clone();
        let fs = &mut *self.fs;
        self.active = m.panel;
        if let Err(e) = self.panels[m.panel].go_drive(fs, &root) {
            // Not ready: chosen anyway, and said so (2.8).
            self.fs_error("Cannot read the drive", &root, e);
        }
        match self.panels[m.panel].view {
            // A tree shows the drive chosen; Info and Quick view give way
            // to its files.
            View::Tree => {
                let (dir, hidden, rows) = (self.panels[m.panel].dir.clone(), self.panels[m.panel].show_hidden, self.panels[m.panel].rows);
                self.trees[m.panel] = Some(crate::tree::Tree::new(&mut *self.fs, &dir, hidden, rows));
            }
            View::Info | View::Quick => self.panels[m.panel].view = View::Files,
            View::Files => {}
        }
    }

    // ---------------------------------------------------------------- quick search

    /// True if the key was the search's.
    fn search_key(&mut self, k: Key) -> bool {
        let Some(s) = self.search.as_mut() else { return false };
        match k.code {
            KeyCode::Char(c) if !k.mods.ctrl && (c.is_alphanumeric() || ".*?_-~ !#$%&'()@^{}".contains(c)) => {
                let mut b = [0u8; 4];
                s.line.insert(c.encode_utf8(&mut b));
                let from = self.panels[s.panel].cursor;
                if !self.search_jump(from) {
                    // Nothing starts so: the letter is not taken.
                    if let Some(s) = self.search.as_mut() {
                        let t = s.line.text();
                        let mut cs: Vec<char> = t.chars().collect();
                        cs.pop();
                        s.line.set(&cs.into_iter().collect::<String>());
                    }
                }
                true
            }
            KeyCode::Backspace => {
                let t = s.line.text();
                let mut cs: Vec<char> = t.chars().collect();
                cs.pop();
                s.line.set(&cs.into_iter().collect::<String>());
                if s.line.is_empty() {
                    self.search = None;
                }
                true
            }
            KeyCode::Enter if k.mods.ctrl => {
                let from = self.panels[s.panel].cursor + 1;
                self.search_jump(from);
                true
            }
            KeyCode::Esc | KeyCode::Enter => {
                self.search = None;
                true
            }
            _ => {
                // Any other key closes the box and does what it does.
                self.search = None;
                false
            }
        }
    }

    fn search_jump(&mut self, from: usize) -> bool {
        let Some(s) = self.search.as_ref() else { return false };
        let p = &self.panels[s.panel];
        match p.find_prefix(&s.line.text(), from) {
            Some(i) => {
                let panel = s.panel;
                self.panels[panel].set_cursor(i);
                true
            }
            None => false,
        }
    }

    // ---------------------------------------------------------------- mouse

    /// Where the panels are: x and width of each, and whether it is shown.
    pub fn panel_rect(&self, i: usize) -> Option<(i16, i16)> {
        if !self.panels_on || !self.panels[i].visible {
            return None;
        }
        if self.narrow() {
            return if i == self.active { Some((0, self.w)) } else { None };
        }
        // Detailed takes the whole width while its panel is the active one;
        // the other comes back when Tab goes to it (02-panels.md 2.1).
        let wide = |k: usize| self.panels[k].view == View::Files && self.panels[k].mode == Mode::Detailed;
        if wide(self.active) {
            return if i == self.active { Some((0, self.w)) } else { None };
        }
        if wide(i) {
            return None;
        }
        let half = self.w / 2;
        Some(if i == 0 { (0, half) } else { (half, self.w - half) })
    }

    fn mouse(&mut self, m: Mouse) {
        if self.editor.is_some() {
            self.editor_event(Event::Mouse(m));
            return;
        }
        if self.dialog.is_some() {
            self.show_dialog();
            self.chrome_event(Event::Mouse(m));
            if let Some(d) = self.dialog.as_mut() {
                self.chrome.read_back(d);
            }
            return;
        }
        if self.chrome.menu_is_open() || self.drive_menu.is_some() || self.pick.is_some() {
            self.chrome_event(Event::Mouse(m));
            return;
        }
        if self.panels_on && m.y == 0 {
            if let MouseKind::Down(Button::Left) = m.kind {
                // A click on the first line opens the menu (1.1): the title
                // under it, or the first.
                self.sync_menus();
                self.chrome_event(Event::Mouse(m));
                if !self.chrome.menu_is_open() {
                    self.chrome.open_menu(0);
                }
                return;
            }
        }
        if self.running.is_some() {
            return;
        }
        let bar = self.h - 1;
        if self.key_bar() && m.y == bar {
            // OWLOSUI's status line: the item under the click is its key.
            if matches!(m.kind, MouseKind::Down(_) | MouseKind::Up(_)) {
                self.chrome.set_held(self.held);
                self.chrome_event(Event::Mouse(m));
            }
            return;
        }
        for i in 0..2 {
            let Some((x, w)) = self.panel_rect(i) else { continue };
            let top = self.top();
            if m.x < x || m.x >= x + w || m.y < top || m.y >= top + self.panel_height() {
                continue;
            }
            match m.kind {
                MouseKind::ScrollUp => {
                    for _ in 0..3 {
                        self.panels[i].up();
                    }
                }
                MouseKind::ScrollDown => {
                    for _ in 0..3 {
                        self.panels[i].down();
                    }
                }
                MouseKind::Down(_) | MouseKind::Double(_) if matches!(self.panels[i].view, View::Info | View::Quick) => {}
                MouseKind::Down(_) | MouseKind::Double(_) if self.panels[i].view == View::Tree => {
                    self.search = None;
                    self.active = i;
                    let row = m.y - top - self.names_y();
                    let Some(t) = self.trees[i].as_mut() else { return };
                    if row < 0 || row as usize >= t.rows || t.top + row as usize >= t.nodes.len() {
                        return;
                    }
                    t.set_cursor(t.top + row as usize);
                    if let MouseKind::Double(Button::Left) = m.kind {
                        self.tree_enter();
                    }
                }
                MouseKind::Down(b) | MouseKind::Double(b) => {
                    self.search = None;
                    self.active = i;
                    // The row, counted from the first row of names.
                    let row = m.y - top - self.names_y();
                    let Some(index) = crate::screen::index_at(&self.panels[i], x, w, m.x, row) else { return };
                    self.panels[i].set_cursor(index);
                    match (m.kind, b) {
                        (MouseKind::Double(_), Button::Left) => self.enter(),
                        // The right button selects (Far, D7).
                        (MouseKind::Down(_), Button::Right) => self.panels[i].toggle(),
                        _ => {}
                    }
                }
                _ => {}
            }
            return;
        }
    }

    /// Where the caret is, for the host to put the hardware cursor: set by
    /// the last `draw`.
    pub fn cursor(&self) -> Option<(i16, i16)> {
        self.caret
    }

    /// The screen, into OWLOSUI's cell grid. `glyph` turns a character into
    /// the font's glyph: `screen::cp437` unless the host has better.
    pub fn draw(&mut self, buf: &mut owlosui_core::Buffer, glyph: &dyn Fn(char) -> owlosui_core::Glyph) {
        if let Some(e) = self.editor.as_mut() {
            e.held = self.held;
            e.draw(buf, &self.scheme, glyph);
            self.caret = e.cursor();
            return;
        }
        crate::screen::draw(self, buf, glyph);
        // OWLOSUI's bars, menus, dialogs and the job's progress over it.
        self.show_dialog();
        match self.progress().filter(|_| self.dialog.is_none()) {
            Some(job) => {
                let (title, lines, bars) = crate::screen::progress_lines(job);
                self.chrome.show_progress(title, &lines, &bars);
            }
            None => self.chrome.hide_progress(),
        }
        self.chrome.set_held(self.held);
        let bar = self.top() > 0;
        let status = self.key_bar();
        self.chrome.draw_over(buf, bar, status);
        // A dialog's input line has the caret, where OWLOSUI put it.
        if self.chrome.form_is_open() {
            self.caret = self.chrome.ui.cursor().map(|p| (p.x, p.y));
        }
    }

    /// F3, F4, Shift+F4: the viewer or the editor over the whole screen.
    pub(crate) fn open_editor(&mut self, path: &str, view: bool, new: bool) {
        use owlosui_core::edit::state;
        // The Viewer and editor options, as the text view's own switches.
        let o = &self.opt;
        let prefs = if o.on(O::Wrap) { state::WRAP } else { 0 }
            | if o.on(O::Numbers) { state::NUMBERS } else { 0 }
            | if o.on(O::WordStar) { state::CLASSIC } else { 0 }
            | if o.on(O::OfferWords) { 0 } else { state::QUIET }
            | if o.on(O::Syntax) { state::SYNTAX } else { 0 };
        match crate::editor::Editor::open(&mut *self.fs, path, view, new, (self.codec)(), self.w, self.h, prefs) {
            Ok(mut e) => {
                e.set_lang(self.chrome.lang.clone());
                self.editor = Some(e);
            }
            Err((e, doing)) => {
                if e.kind == ErrorKind::Unsupported {
                    self.message("Edit", &doing);
                } else {
                    self.fs_error(&doing, path, e);
                }
            }
        }
    }

    fn editor_event(&mut self, ev: Event) {
        let Some(e) = self.editor.as_mut() else { return };
        match e.handle(&mut *self.fs, ev) {
            crate::editor::Out::Stay => {}
            crate::editor::Out::Close => {
                self.editor = None;
                self.reread_both();
            }
            // Words with no file - the help - have nothing to edit.
            crate::editor::Out::Edit if e.path.is_empty() => {}
            crate::editor::Out::Edit => {
                let p = e.path.clone();
                self.editor = None;
                self.open_editor(&p, false, false);
            }
            crate::editor::Out::Saved(p) => {
                // A new file: the panel that shows its folder puts its cursor on it.
                let sep = self.fs.sep();
                if let Some((dir, name)) = path::split(&p, sep) {
                    for i in 0..2 {
                        if self.panels[i].dir.eq_ignore_ascii_case(&dir) {
                            let fs = &mut *self.fs;
                            let _ = self.panels[i].reread_onto(fs, &name);
                        }
                    }
                }
            }
        }
    }
}

/// A folder's bytes, everything in it.
pub fn dir_size(fs: &mut dyn Fs, dir: &str) -> u64 {
    let sep = fs.sep();
    let mut total = 0;
    let mut stack = alloc::vec![String::from(dir)];
    while let Some(d) = stack.pop() {
        if let Ok(list) = fs.read_dir(&d) {
            for e in list {
                if e.dir {
                    stack.push(path::join(&d, &e.name, sep));
                } else {
                    total += e.size;
                }
            }
        }
    }
    total
}

/// A program Enter runs (D8).
pub fn is_program(name: &str, dos: bool) -> bool {
    let ext = path::split_ext(name).1.to_lowercase();
    if dos {
        matches!(ext.as_str(), "com" | "exe" | "bat")
    } else {
        matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com" | "sh")
    }
}

/// A name for a command line: in quotes when it has a space.
pub fn quoted(s: &str) -> String {
    if s.contains(' ') {
        alloc::format!("\"{}\"", s)
    } else {
        String::from(s)
    }
}

/// What is wrong with an 8.3 name, or nothing.
pub fn bad_short_name(name: &str) -> Option<&'static str> {
    let (n, e) = path::split_ext(name);
    if n.is_empty() || n.chars().count() > 8 || e.chars().count() > 3 || name.matches('.').count() > 1 {
        return Some("A name has at most 8 letters before the dot and 3 after.");
    }
    if name.chars().any(|c| " *?<>|\"+=,;[]/\\:".contains(c) || c.is_control()) {
        return Some("A name cannot have spaces or * ? < > | \" + = , ; [ ] in it.");
    }
    None
}

/// The shortcut a digit key with Ctrl and Shift is for: the digit, or the
/// mark Shift puts on it on a US keyboard, as a browser reports it.
pub fn shortcut_digit(c: char) -> Option<usize> {
    if let Some(d) = c.to_digit(10) {
        return Some(d as usize);
    }
    ")!@#$%^&*(".find(c)
}
