//! What is around the panels, and over them, is OWLOSUI's own (D15): the
//! menu bar and its menus (F9), the key bar - OWLOSUI's status line - and
//! the dialogs. The panels, the command line and the terminal behind them
//! stay the commander's, drawn as NC and VC drew them.
//!
//! The commander is a window of OWLOSUI's, with no frame of its own: the
//! panels' frames are its frame. One `Ui` holds the desktop's menu bar
//! and status line and that window, pinned with its frame just off the
//! screen; its menus and its keys are the window's own, which OWLOSUI
//! puts on the bars while it is the active window - as it would on a
//! desktop with other windows. Inside the window is a hole: a view whose
//! cells show what is under the Ui. The commander draws its screen first; the Ui is drawn
//! into a second buffer, and every cell of it that is not a hole goes on
//! top - so a menu dropped over a panel covers it, and the rest of the
//! panel shows through.

use crate::menu;
use alloc::vec::Vec;
use alloc::string::String;
use crate::dialog::{self, Dialog, Tone, EXISTING};
use crate::lang::Lang;
use owlosui_core::{Buffer, ButtonRow, Cluster, Dock, Event, InputLine, Progress, Key, KeyCode, Kind, ListBox, MenuBar, MenuItem, Mods, Label, PushButton, Rect, StatusItem, StatusLine, Ui, ViewId, WinPalette, Window};

/// The commands the Ui sends back: a menu item, by its title and place;
/// a key of the status line, by its number.
const MENU_BASE: u16 = 1;
const MENU_STEP: u16 = 40;
pub const KEY_BASE: u16 = 500;
/// A dialog's OK and Cancel.
const OK: u16 = 900;
const CANCEL: u16 = 901;
/// A dialog's buttons, by index.
const BUTTON_BASE: u16 = 700;

/// What a command from the Ui means to the commander.
pub enum Out {
    /// Left, Files, Commands, Options, Right; the item's index in it.
    Item(usize, usize),
    /// A key of the status line: F1..F10 with the modifiers its row is for.
    Key(Key),
    /// The open dialog's OK: Enter, or the button.
    Ok,
    /// Its Cancel: Esc, or the button.
    Cancel,
    /// A button of a commander's dialog (`open_dialog`), by its index.
    Button(usize),
}

/// A commander's dialog as OWLOSUI's window: the views that hold what
/// the person can change, read back when a button is pressed.
pub struct Form {
    pub win: ViewId,
    input: Option<ViewId>,
    existing: Option<ViewId>,
    check: Option<ViewId>,
    switches: Option<ViewId>,
}

pub struct Chrome {
    pub ui: Ui,
    bar: ViewId,
    status: ViewId,
    hole: ViewId,
    /// The commander's own window, the hole in it.
    win: ViewId,
    /// The modifiers the status line's labels are for now.
    held: Mods,
    over: Buffer,
    /// The open dialog's window and its list, if it has one.
    dialog: Option<(ViewId, Option<ViewId>)>,
    /// The open commander's dialog.
    form: Option<Form>,
    /// The progress window of a running job: its lines and its bars.
    progress: Option<(ViewId, Vec<ViewId>, Vec<ViewId>)>,
    /// The language the words are shown in (Options > Languages).
    pub lang: Lang,
}

/// The key bar's words, a row per modifier (docs/01-screen.md 1.3).
const BAR: [[&str; 10]; 4] = [
    ["Help", "Menu", "View", "Edit", "Copy", "RenMov", "Mkdir", "Delete", "PullDn", "Quit"],
    ["", "Main", "View..", "Edit..", "Copy", "RenMov", "", "Delete", "SavSet", "PullDn"],
    ["Left", "Right", "Name", "Extens", "Time", "Size", "Unsort", "", "", ""],
    ["Left", "Right", "View..", "Edit..", "", "DirSiz", "Find", "Histry", "Lines", "Tree"],
];

fn row_of(m: Mods) -> usize {
    if m.alt {
        3
    } else if m.ctrl {
        2
    } else if m.shift {
        1
    } else {
        0
    }
}

fn status_items(m: Mods, w: i16, lang: &Lang) -> Vec<StatusItem> {
    let row = row_of(m);
    key_items(&BAR[row], Mods { ctrl: row == 2, alt: row == 3, shift: row == 1 }, w, lang)
}

/// The shorter word for a cell too narrow for the whole one: eighty
/// columns are ten cells of eight, and `F6 ` leaves five letters.
const SHORT: [(&str, &str); 15] = [
    ("RenMov", "RenMv"),
    ("Delete", "Delet"),
    ("PullDn", "Pull"),
    ("Extens", "Ext"),
    ("Unsort", "Unsrt"),
    ("View..", "View."),
    ("Edit..", "Edit."),
    ("SavSet", "Save"),
    ("DirSiz", "DirSz"),
    ("Histry", "Hist"),
    ("Search", "Srch"),
    ("Replac", "Repl"),
    ("Unwrap", "Unwrp"),
    ("SaveAs", "SavAs"),
    ("SavQit", "Exit"),
];

/// Each key bar cell's width on a screen `w` wide: a tenth of it.
pub fn cell_of(w: i16) -> i16 {
    (w / 10).max(4)
}

/// Ten key bar words as OWLOSUI's status line: `~F5~ Copy`, the key
/// lit, in cells a tenth of the screen `w` wide - a word too long for
/// its cell shortened (`SHORT`), or cut; in `lang`. An empty word grey. A click on
/// one sends its key, F1..F10 with `mods`, as `KEY_BASE` + its number.
pub fn key_items(words: &[&str; 10], mods: Mods, w: i16, lang: &Lang) -> Vec<StatusItem> {
    let cell = cell_of(w) as usize;
    (0..10)
        .map(|k| {
            // Under eighty columns the F goes, so the words keep their letters.
            let key = if cell >= 8 { alloc::format!("F{}", k + 1) } else { alloc::format!("{}", k + 1) };
            // A space after the red key, so it does not run into its word -
            // unless the cell is so narrow the word would lose its letters.
            let spaced = cell.saturating_sub(key.len() + 1) >= 4;
            let room = cell.saturating_sub(key.len() + spaced as usize);
            // The language's word, or the English one shortened.
            let mut word = lang.tr(words[k]);
            if word.chars().count() > room {
                if let Some((_, s)) = SHORT.iter().find(|(l, _)| *l == word) {
                    word = String::from(*s);
                }
                word = word.chars().take(room).collect();
            }
            let gap = if spaced { " " } else { "" };
            let text = if word.is_empty() { alloc::format!("~{}~", key) } else { alloc::format!("~{}~{}{}", key, gap, word) };
            let mut it = StatusItem::new(&text, Some(Key { code: KeyCode::F(k as u8 + 1), mods }), KEY_BASE + k as u16 + 1);
            it.enabled = !words[k].is_empty();
            it
        })
        .collect()
}

/// A key bar: OWLOSUI's status line in ten even cells, a tenth of the
/// screen `w` each, as the file managers' key bars were.
pub fn key_line(items: Vec<StatusItem>, w: i16) -> StatusLine {
    let mut s = StatusLine::new(items);
    s.cell = cell_of(w);
    s
}

impl Chrome {
    pub fn new(w: i16, h: i16) -> Chrome {
        let mut ui = Ui::new(w, h);
        let root = ui.root();
        // The window first, the bars over it: its frame one cell outside
        // every edge, so what shows of it is its inside, the whole screen.
        let mut window = Window::new("Kirin Commander");
        window.pinned = true;
        window.resizable = false;
        window.zoomable = false;
        window.closable = false;
        window.minimizable = false;
        let win = ui.insert(root, Rect::new(-1, -1, w + 2, h + 2), Kind::Window(window));
        let hole = ui.insert(win, Rect::new(0, 0, w, h), Kind::Hole);
        ui.set_dock(hole, Dock::Manual);
        let bar = ui.insert(root, Rect::new(0, 0, w, 1), Kind::MenuBar(MenuBar::new(Vec::new())));
        let status = ui.insert(root, Rect::new(0, h - 1, w, 1), Kind::Status(key_line(Vec::new(), w)));
        ui.set_window_status(win, status_items(Mods::default(), w, &Lang::default()));
        Chrome { ui, bar, status, hole, win, held: Mods::default(), over: Buffer::new(w, h), dialog: None, form: None, progress: None, lang: Lang::default() }
    }

    pub fn resize(&mut self, w: i16, h: i16) {
        self.ui.resize(w, h);
        self.ui.set_rect(self.bar, Rect::new(0, 0, w, 1));
        self.ui.set_rect(self.status, Rect::new(0, h - 1, w, 1));
        self.ui.set_rect(self.win, Rect::new(-1, -1, w + 2, h + 2));
        self.ui.set_rect(self.hole, Rect::new(0, 0, w, h));
        self.over.resize(w, h);
        // The key bar's cells are a tenth of the width.
        if let Kind::Status(s) = self.ui.kind_mut(self.status) {
            s.cell = cell_of(w);
        }
        self.ui.set_window_status(self.win, status_items(self.held, w, &self.lang));
    }

    /// The menus as the commander has them now: what can be done
    /// (`enabled(title, label)`), what is ticked (`ticked(title, label)`),
    /// in the language chosen.
    pub fn set_menus(&mut self, ticked: &dyn Fn(usize, &str) -> bool, enabled: &dyn Fn(usize, &str) -> bool) {
        let mut bar = Vec::new();
        for (t, title) in menu::TITLES.iter().enumerate() {
            let items: Vec<MenuItem> = menu::items(t)
                .iter()
                .enumerate()
                .map(|(i, it)| {
                    if it.label == "-" {
                        return MenuItem::line();
                    }
                    let cmd = MENU_BASE + t as u16 * MENU_STEP + i as u16;
                    let mi = MenuItem::new(&self.lang.tr(it.label), it.keys, cmd).checked(ticked(t, it.label));
                    if it.key.is_some() && enabled(t, it.label) {
                        mi
                    } else {
                        mi.disabled()
                    }
                })
                .collect();
            bar.push(MenuItem::sub(&self.lang.tr(&alloc::format!("~{}~{}", &title[..1], &title[1..])), items));
        }
        self.ui.set_window_menu(self.win, bar);
    }

    /// Another language: the key bar's words at once, the menus as they
    /// next open, the dialogs as they next show.
    pub fn set_lang(&mut self, lang: Lang) {
        self.lang = lang;
        let w = self.over.width();
        self.ui.set_window_status(self.win, status_items(self.held, w, &self.lang));
    }

    /// The status line's words for the modifiers held now.
    pub fn set_held(&mut self, m: Mods) {
        if row_of(m) == row_of(self.held) {
            return;
        }
        self.held = m;
        self.ui.set_window_status(self.win, status_items(m, self.over.width(), &self.lang));
    }

    pub fn open_menu(&mut self, title: usize) {
        // The bar takes the window's menus before an event: one first.
        self.ui.handle(Event::Tick);
        self.ui.open_menu(title);
    }

    pub fn menu_is_open(&self) -> bool {
        self.ui.menu_open().is_some()
    }

    /// A list to choose one line from, in OWLOSUI's grey dialog with OK
    /// and Cancel, centred over `over` (x, width) - the drive menu over its
    /// panel. `head`: a line over the list, its columns' names. `buttons`:
    /// the list's own (`Out::Button` by index, a Cancel the one Esc
    /// presses), or none for OK and Cancel (`Out::Ok`, `Out::Cancel`).
    #[allow(clippy::too_many_arguments)]
    pub fn open_list(&mut self, title: &str, head: Option<&str>, lines: &[String], current: usize, over: (i16, i16), buttons: &[&str]) {
        self.close_dialog();
        let title = self.lang.tr(title);
        let title = title.as_str();
        let head = head.map(|h| self.lang.tr(h));
        let head = head.as_deref();
        let pushes: Vec<PushButton> = if buttons.is_empty() {
            alloc::vec![PushButton::new(&self.lang.tr("~O~K"), OK).default(), PushButton::new(&self.lang.tr("~C~ancel"), CANCEL).cancel()]
        } else {
            buttons
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    let p = PushButton::new(&self.lang.tr(b), BUTTON_BASE + i as u16);
                    if i == 0 {
                        p.default()
                    } else if dialog::plain(b) == "Cancel" {
                        p.cancel()
                    } else {
                        p
                    }
                })
                .collect()
        };
        let buttons_w: i16 = pushes.iter().map(|p| dialog::plain(&p.text).chars().count() as i16 + 7).sum();
        let work = self.ui.work_area();
        let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(8) as i16;
        // Room for the title and OWLOSUI's `[modal]` after it.
        let w = (longest + 6).max(title.chars().count() as i16 + 16).max(buttons_w + 4).max(26).min(work.w);
        let top = head.is_some() as i16;
        let rows = (lines.len() as i16).clamp(1, (work.h - 7 - top).max(1));
        let h = (rows + top + 5).min(work.h);
        let mut win = Window::new(title);
        win.palette = WinPalette::Gray;
        win.modal = true;
        win.resizable = false;
        win.zoomable = false;
        win.closable = false;
        let x = over.0 + ((over.1 - w) / 2).max(0);
        let y = work.y + ((work.h - h) / 2).max(0);
        let root = self.ui.root();
        let wid = self.ui.insert(root, Rect::new(x, y, w, h), Kind::Window(win));
        if let Some(t) = head {
            let s = self.ui.insert(wid, Rect::new(2, 1, w - 6, 1), Kind::Label(Label::new(t, None)));
            self.ui.set_dock(s, Dock::Manual);
        }
        let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let mut lb = ListBox::new(&refs);
        lb.current = current.min(lines.len().saturating_sub(1));
        let list = self.ui.insert(wid, Rect::new(2, 1 + top, w - 6, rows), Kind::List(lb));
        self.ui.set_dock(list, Dock::Manual);
        let row = self.ui.insert(wid, Rect::default(), Kind::Buttons(ButtonRow::new(pushes)));
        self.ui.set_dock(row, Dock::BottomRight(w - 2, 2));
        self.ui.focus_first();
        self.dialog = Some((wid, Some(list)));
    }

    /// The line the open list's cursor is on.
    pub fn list_current(&self) -> Option<usize> {
        let (_, list) = self.dialog?;
        match self.ui.kind(list?) {
            Kind::List(l) => Some(l.current),
            _ => None,
        }
    }

    /// Put the open list's cursor on a line: a drive's letter typed.
    pub fn set_list_current(&mut self, i: usize) {
        if let Some((_, Some(list))) = self.dialog {
            if let Kind::List(l) = self.ui.kind_mut(list) {
                l.current = i.min(l.items.len().saturating_sub(1));
            }
        }
    }

    pub fn dialog_is_open(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn close_dialog(&mut self) {
        if let Some((wid, _)) = self.dialog.take() {
            if self.ui.is_alive(wid) {
                self.ui.close(wid);
            }
        }
        if let Some(f) = self.form.take() {
            if self.ui.is_alive(f.win) {
                self.ui.close(f.win);
            }
        }
    }

    pub fn form_is_open(&self) -> bool {
        self.form.is_some()
    }

    /// The running job's progress (docs/05-file-operations.md 5.7) as
    /// OWLOSUI's grey window: `lines` of words, `bars` of (done, total)
    /// under the line each follows (`after`). Made once, then kept up to
    /// date: the window stays where it is while the numbers move.
    pub fn show_progress(&mut self, title: &str, lines: &[String], bars: &[(usize, u64, u64)]) {
        let rows = lines.len() as i16 + bars.len() as i16;
        let fresh = match &self.progress {
            Some((wid, l, b)) => !self.ui.is_alive(*wid) || l.len() != lines.len() || b.len() != bars.len(),
            None => true,
        };
        if fresh {
            self.hide_progress();
            let work = self.ui.work_area();
            let w = 50.min(work.w);
            let h = rows + 2;
            let mut win = Window::new(&self.lang.tr(title));
            win.palette = WinPalette::Gray;
            win.resizable = false;
            win.zoomable = false;
            win.minimizable = false;
            win.closable = false;
            let x = work.x + ((work.w - w) / 2).max(0);
            let y = work.y + ((work.h - h) / 2 - 1).max(0);
            let root = self.ui.root();
            let wid = self.ui.insert(root, Rect::new(x, y, w, h), Kind::Window(win));
            // Not one of the windows Alt and a digit pick: no number.
            if let Kind::Window(wn) = self.ui.kind_mut(wid) {
                wn.number = None;
            }
            let mut ls = Vec::new();
            let mut bs = Vec::new();
            let mut row = 0;
            for (i, _) in lines.iter().enumerate() {
                let id = self.ui.insert(wid, Rect::new(2, row, w - 6, 1), Kind::Label(Label::new("", None)));
                self.ui.set_dock(id, Dock::Manual);
                ls.push(id);
                row += 1;
                for _ in bars.iter().filter(|b| b.0 == i) {
                    let id = self.ui.insert(wid, Rect::new(2, row, w - 6, 1), Kind::Progress(Progress::new(1000)));
                    self.ui.set_dock(id, Dock::Manual);
                    bs.push(id);
                    row += 1;
                }
            }
            self.progress = Some((wid, ls, bs));
        }
        let Some((_, ls, bs)) = &self.progress else { return };
        let (ls, bs) = (ls.clone(), bs.clone());
        for (id, s) in ls.iter().zip(lines) {
            if let Kind::Label(l) = self.ui.kind_mut(*id) {
                l.text = self.lang.tr(s).replace('~', "");
            }
        }
        for (id, &(_, done, total)) in bs.iter().zip(bars) {
            if let Kind::Progress(p) = self.ui.kind_mut(*id) {
                // Per mille: the bar's own percent at its right end.
                p.set(if total == 0 { 1000 } else { (done.min(total) * 1000 / total) as u32 });
            }
        }
    }

    pub fn hide_progress(&mut self) {
        if let Some((wid, _, _)) = self.progress.take() {
            if self.ui.is_alive(wid) {
                self.ui.close(wid);
            }
        }
    }

    /// A commander's dialog as OWLOSUI's window over the panels.
    pub fn open_dialog(&mut self, d: &Dialog) {
        self.close_dialog();
        self.form = Some(open_form(&mut self.ui, d, &self.lang));
    }

    /// What the person changed in the window, back into the dialog.
    pub fn read_back(&self, d: &mut Dialog) {
        if let Some(f) = &self.form {
            read_form(&self.ui, f, d);
        }
    }

    /// An event for the Ui - while a menu is open, or a click on its bars -
    /// and what it chose.
    pub fn handle(&mut self, ev: Event) -> Vec<Out> {
        self.ui.handle(ev);
        // OWLOSUI holds a button or an item down for a moment before it
        // fires; the commander takes the choice at once.
        if self.ui.pick_pending() {
            self.ui.complete_pick();
        }
        let mut cmds = Vec::new();
        while let Some(c) = self.ui.take_command() {
            cmds.push(c);
        }
        while let Some(c) = self.ui.take_pressed() {
            cmds.push(c);
        }
        let mut out = Vec::new();
        for c in cmds {
            if (BUTTON_BASE..BUTTON_BASE + 20).contains(&c) {
                out.push(Out::Button((c - BUTTON_BASE) as usize));
            } else if c == OK {
                out.push(Out::Ok);
            } else if c == CANCEL {
                out.push(Out::Cancel);
            } else if c >= KEY_BASE {
                let n = (c - KEY_BASE) as u8;
                let row = row_of(self.held);
                let mods = Mods { ctrl: row == 2, alt: row == 3, shift: row == 1 };
                out.push(Out::Key(Key { code: KeyCode::F(n), mods }));
            } else if c >= MENU_BASE {
                let k = c - MENU_BASE;
                out.push(Out::Item((k / MENU_STEP) as usize, (k % MENU_STEP) as usize));
            }
        }
        out
    }

    /// The Ui over the commander's screen: the bar when `bar` (or a menu
    /// is open), the status line when `status`, a menu or a dialog
    /// wherever it is.
    pub fn draw_over(&mut self, buf: &mut Buffer, bar: bool, status: bool) {
        let (w, h) = (buf.width(), buf.height());
        if self.over.width() != w || self.over.height() != h {
            self.resize(w, h);
        }
        self.ui.draw(&mut self.over);
        let bar = bar || self.menu_is_open();
        let shadow = self.ui.palette.shadow;
        let clip = buf.rect();
        for y in 0..h {
            if (y == 0 && !bar) || (y == h - 1 && !status) {
                continue;
            }
            for x in 0..w {
                if self.over.is_hole(x, y) {
                    continue;
                }
                let Some(c) = self.over.get(x, y) else { continue };
                // A window's shadow over the hole: OWLOSUI darkened the
                // hole's blank; the commander's own cell is darkened instead,
                // its character kept, as a shadow on a real screen is.
                if c.ch == 32 && c.attr == shadow && y >= 1 && y < h - 1 {
                    if let Some(under) = buf.get(x, y) {
                        buf.put(x, y, under.ch, shadow, clip);
                    }
                    continue;
                }
                buf.put(x, y, c.ch, c.attr, clip);
            }
        }
    }
}

/// A commander's dialog (copy, delete, an error, a question, the
/// options) as OWLOSUI's grey modal window: its lines, its input line,
/// the choice for existing files, its check boxes, its buttons - the
/// first the default, a Cancel the one Esc presses.
pub fn open_form(ui: &mut Ui, d: &Dialog, lang: &Lang) -> Form {
    // What is shown is in the language; what is read back goes into `d`
    // by place, so the commander still sees its own words.
    let english = d;
    let shown = lang.dialog(d);
    let d = &shown;
    let work = ui.work_area();
    let text_w = d.lines.iter().map(|(s, _)| s.chars().count()).max().unwrap_or(0) as i16;
    // A button is its words, `► ◄`, its shadow and a gap. Too many for
    // one row - the overwrite question's seven - go in two.
    let width_of = |bs: &[String]| -> i16 { bs.iter().map(|b| dialog::plain(b).chars().count() as i16 + 7).sum() };
    let split = if width_of(&d.buttons) > work.w - 8 { d.buttons.len().div_ceil(2) } else { d.buttons.len() };
    let buttons_w = width_of(&d.buttons[..split]).max(width_of(&d.buttons[split..]));
    let switches_w = d.switches.iter().map(|(s, _)| s.chars().count() as i16 + 6).max().unwrap_or(0);
    let input_w = if d.input.is_some() { 40 } else { 0 };
    let inner = text_w.max(buttons_w).max(switches_w).max(input_w).max(d.title.chars().count() as i16 + 12).max(24);
    let w = (inner + 6).min(work.w);
    // The rows inside the frame: the lines, the input, the choices,
    // a blank, the buttons and their shadow.
    let rows = d.lines.len() as i16
        + d.input.is_some() as i16
        + if d.existing.is_some() { 1 + EXISTING.len() as i16 } else { 0 }
        + d.check.is_some() as i16
        + d.switches.len() as i16
        + 3
        + if split < d.buttons.len() { 2 } else { 0 };
    let h = (rows + 2).min(work.h);
    let mut win = Window::new(&d.title);
    win.palette = WinPalette::Gray;
    win.modal = true;
    win.resizable = false;
    win.zoomable = false;
    win.closable = false;
    let x = work.x + ((work.w - w) / 2).max(0);
    let y = work.y + ((work.h - h) / 2 - 1).max(0);
    let root = ui.root();
    let wid = ui.insert(root, Rect::new(x, y, w, h), Kind::Window(win));
    let field = w - 6;
    let mut row = 0;
    let label = |ui: &mut Ui, row: i16, s: &str, tone: Tone| {
        // A code goes to the right end, quiet, as the classic box had it.
        let s = if tone == Tone::Code { alloc::format!("{:>width$}", s, width = field as usize) } else { String::from(s) };
        let id = ui.insert(wid, Rect::new(2, row, field, 1), Kind::Label(Label::new(&s, None)));
        ui.set_dock(id, Dock::Manual);
    };
    let mut form = Form { win: wid, input: None, existing: None, check: None, switches: None };
    for (i, (s, tone)) in d.lines.iter().enumerate() {
        if i == d.input_at && d.input.is_some() {
            form.input = Some(input_at(ui, wid, row, field, d.input.as_ref().unwrap()));
            row += 1;
        }
        label(ui, row, s, *tone);
        row += 1;
    }
    if d.input_at >= d.lines.len() {
        if let Some(l) = &d.input {
            form.input = Some(input_at(ui, wid, row, field, l));
            row += 1;
        }
    }
    if let Some(e) = d.existing {
        label(ui, row, &lang.tr("Already existing files:"), Tone::Normal);
        row += 1;
        let words: Vec<String> = EXISTING.iter().map(|(_, n)| lang.tr(n)).collect();
        let names: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let mut c = Cluster::radio(&names);
        let at = EXISTING.iter().position(|(x, _)| *x == e).unwrap_or(0);
        c.on = (0..names.len()).map(|i| i == at).collect();
        c.current = at;
        let id = ui.insert(wid, Rect::new(2, row, field, names.len() as i16), Kind::Cluster(c));
        ui.set_dock(id, Dock::Manual);
        form.existing = Some(id);
        row += names.len() as i16;
    }
    if let Some(on) = d.check {
        let copy = lang.tr("Copy attributes and dates");
        let mut c = Cluster::checks(&[copy.as_str()]);
        c.on[0] = on;
        let id = ui.insert(wid, Rect::new(2, row, field, 1), Kind::Cluster(c));
        ui.set_dock(id, Dock::Manual);
        form.check = Some(id);
        row += 1;
    }
    if !d.switches.is_empty() {
        let names: Vec<&str> = d.switches.iter().map(|(s, _)| s.as_str()).collect();
        let mut c = Cluster::checks(&names);
        c.on = d.switches.iter().map(|(_, on)| *on).collect();
        let id = ui.insert(wid, Rect::new(2, row, field, names.len() as i16), Kind::Cluster(c));
        ui.set_dock(id, Dock::Manual);
        form.switches = Some(id);
    }
    let all: Vec<PushButton> = d
        .buttons
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let p = PushButton::new(b, BUTTON_BASE + i as u16);
            if i == 0 {
                p.default()
            } else if dialog::plain(&english.buttons[i]) == "Cancel" {
                p.cancel()
            } else {
                p
            }
        })
        .collect();
    let mut all = all.into_iter();
    if split < d.buttons.len() {
        // The first row over the second, both to the right.
        let first: Vec<PushButton> = all.by_ref().take(split).collect();
        let r = ui.insert(wid, Rect::new(0, h - 6, w - 2, 2), Kind::Buttons(ButtonRow::new(first)));
        ui.set_dock(r, Dock::Manual);
    }
    let br = ui.insert(wid, Rect::default(), Kind::Buttons(ButtonRow::new(all.collect())));
    ui.set_dock(br, Dock::BottomRight(w - 2, 2));
    ui.focus_first();
    form
}

fn input_at(ui: &mut Ui, wid: ViewId, row: i16, field: i16, l: &crate::line::Line) -> ViewId {
    let mut input = InputLine::new("", &l.text());
    if l.all {
        input.select_all();
    }
    let id = ui.insert(wid, Rect::new(2, row, field, 1), Kind::Input(input));
    ui.set_dock(id, Dock::Manual);
    id
}

/// What the person changed in the window, back into the dialog: the
/// input line, the choice, the check boxes.
pub fn read_form(ui: &Ui, f: &Form, d: &mut Dialog) {
    if let (Some(id), Some(_)) = (f.input, &d.input) {
        if let Kind::Input(i) = ui.kind(id) {
            d.input = Some(crate::line::Line::new(&i.text));
        }
    }
    if let Some(id) = f.existing {
        if let Kind::Cluster(c) = ui.kind(id) {
            if let Some(at) = c.on.iter().position(|&o| o) {
                d.existing = Some(EXISTING[at].0);
            }
        }
    }
    if let Some(id) = f.check {
        if let Kind::Cluster(c) = ui.kind(id) {
            d.check = Some(c.on[0]);
        }
    }
    if let Some(id) = f.switches {
        if let Kind::Cluster(c) = ui.kind(id) {
            for (s, on) in d.switches.iter_mut().zip(&c.on) {
                s.1 = *on;
            }
        }
    }
}


/// The form's window taken away.
pub fn close_form(ui: &mut Ui, f: Form) {
    if ui.is_alive(f.win) {
        ui.close(f.win);
    }
}

/// The form's buttons pressed by the last event, by index - OWLOSUI's
/// press, held down a moment for the eye, completed at once.
pub fn form_buttons(ui: &mut Ui) -> Vec<usize> {
    if ui.pick_pending() {
        ui.complete_pick();
    }
    let mut out = Vec::new();
    while let Some(c) = ui.take_pressed() {
        if (BUTTON_BASE..BUTTON_BASE + 20).contains(&c) {
            out.push((c - BUTTON_BASE) as usize);
        }
    }
    out
}
