//! The viewer (F3) and the editor (F4), docs/08-viewer-editor.md: the
//! whole screen, a status line on top, the text, the key bar at the bottom,
//! no buttons, and a menu bar hidden until F9 or a click on the first line.
//!
//! The text is OWLOSUI's own text view: syntax colours, wrap, hex, find
//! and replace, undo, the clipboard, words offered while typing code. It
//! lives in a window whose frame is placed just off the screen's edges, so
//! what shows is the text alone; the first and last lines are the
//! commander's own.

use crate::dialog::{Dialog, Outcome, Purpose, Tone};
use crate::fs::*;
use crate::line::Line;
use crate::names;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use owlosui_core::edit::{offer, state};
use owlosui_core::{Buffer, Event, Glyph, Key, KeyCode, Kind, MenuBar, MenuItem, Mods, Point, Rect, TextView, Ui, ViewId, Window};

/// How the bytes of a file become the glyphs of a text, and back.
pub trait Codec {
    /// One line's bytes, without its end.
    fn decode(&self, line: &[u8]) -> Vec<Glyph>;
    fn encode(&self, line: &[Glyph]) -> Vec<u8>;
    /// Whether the text can be written back exactly as it was read: a
    /// file with characters the font has no glyph for is opened to view.
    fn exact(&self, _bytes: &[u8]) -> bool {
        true
    }
}

/// DOS: a byte is a glyph, both ways.
pub struct Bytes;

impl Codec for Bytes {
    fn decode(&self, line: &[u8]) -> Vec<Glyph> {
        line.iter().map(|&b| b as Glyph).collect()
    }
    fn encode(&self, line: &[Glyph]) -> Vec<u8> {
        line.iter().map(|&g| g as u8).collect()
    }
}

/// The most an editor holds; the viewer has no limit of its own.
pub const EDIT_LIMIT: u64 = if cfg!(feature = "dos") { 256 * 1024 } else { 8 * 1024 * 1024 };

const CM_SAVE: u16 = 1;
const CM_SAVE_AS: u16 = 2;
const CM_CLOSE: u16 = 3;
const CM_EDIT: u16 = 4;
const CM_GOTO: u16 = 5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    Stay,
    Close,
    /// F6 in the viewer: the editor, on the same place.
    Edit,
    /// A file was written: the panels re-read, the cursor on it.
    Saved(String),
}

enum Pending {
    /// F10 with changes: Save / Don't save / Continue editing.
    Leave,
    SaveAs,
    Goto,
    Message,
}

pub struct Editor {
    pub ui: Ui,
    pub path: String,
    pub readonly: bool,
    win: ViewId,
    text: ViewId,
    /// OWLOSUI's status line at the bottom: the key bar (D15).
    status: ViewId,
    codec: Box<dyn Codec>,
    /// The line end the file had: what it is written back with.
    crlf: bool,
    size: u64,
    dialog: Option<(Dialog, Pending)>,
    /// The dialog as OWLOSUI's window in this editor's own Ui (D15).
    form: Option<crate::chrome::Form>,
    pub w: i16,
    pub h: i16,
    pub held: Mods,
    /// The caret, from the last draw.
    caret: Option<(i16, i16)>,
    /// The menu bar, for its words in another language.
    bar: ViewId,
    /// The language its words are in (Options > Languages).
    pub lang: crate::lang::Lang,
}

fn read_all(fs: &mut dyn Fs, path: &str, limit: u64) -> FsResult<Vec<u8>> {
    let h = fs.open_read(path)?;
    let mut data = Vec::new();
    let mut buf = alloc::vec![0u8; 16 * 1024];
    loop {
        match fs.read(h, &mut buf) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);
                if data.len() as u64 > limit {
                    break;
                }
            }
            Err(e) => {
                let _ = fs.close(h);
                return Err(e);
            }
        }
    }
    let _ = fs.close(h);
    Ok(data)
}

impl Editor {
    /// `new`: Shift+F4 - a file that need not exist yet. `prefs`: how it
    /// opens, the Viewer and editor options as OWLOSUI's `edit::state`
    /// bits - WRAP, NUMBERS, CLASSIC, QUIET, and SYNTAX for "colour it".
    #[allow(clippy::too_many_arguments)]
    pub fn open(fs: &mut dyn Fs, path: &str, readonly: bool, new: bool, codec: Box<dyn Codec>, w: i16, h: i16, prefs: u8) -> Result<Editor, (FsError, String)> {
        let size = match fs.stat(path) {
            Ok(e) => e.size,
            Err(e) if new && e.kind == ErrorKind::NotFound => 0,
            Err(e) => return Err((e, String::from("Cannot open the file"))),
        };
        if !readonly && size > EDIT_LIMIT {
            let why = alloc::format!("This file is {}; the editor holds {}. Press F3 to view it.", names::human(size), names::human(EDIT_LIMIT));
            return Err((FsError::new(ErrorKind::Unsupported), why));
        }
        let data = if size == 0 && new {
            Vec::new()
        } else {
            read_all(fs, path, if readonly { 64 * 1024 * 1024 } else { EDIT_LIMIT }).map_err(|e| (e, String::from("Cannot read the file")))?
        };
        let crlf = data.windows(2).any(|p| p == b"\r\n") || (data.is_empty() && fs.short_names());
        let title = crate::path::name(path, fs.sep());
        Ok(Editor::build(&title, path, &data, crlf, readonly, codec, w, h, prefs))
    }

    /// Words the commander has, not a file - F1's help - in the viewer,
    /// titled `title`. It has no path: nothing to edit or save.
    pub fn words(title: &str, text: &str, codec: Box<dyn Codec>, w: i16, h: i16) -> Editor {
        Editor::build(title, "", text.as_bytes(), false, true, codec, w, h, state::WRAP)
    }

    #[allow(clippy::too_many_arguments)]
    fn build(title: &str, path: &str, data: &[u8], crlf: bool, readonly: bool, codec: Box<dyn Codec>, w: i16, h: i16, prefs: u8) -> Editor {
        let size = data.len() as u64;
        let exact = codec.exact(data);
        let lines: Vec<Vec<Glyph>> = split_lines(data).iter().map(|l| codec.decode(l)).collect();

        let mut ui = Ui::new(w, h);
        let root = ui.root();
        let bar = ui.insert(root, Rect::new(0, 0, w, 1), Kind::MenuBar(MenuBar::new(menu(readonly))));
        let mut win = Window::new(title);
        win.movable = false;
        win.resizable = false;
        win.zoomable = false;
        win.closable = false;
        win.shadow = false;
        win.pinned = true;
        // The frame just off the screen: the sides and the bottom outside
        // it, the top under the commander's status line.
        let wid = ui.insert(root, Rect::new(-1, 0, w + 2, h), Kind::Window(win));
        let mut view = TextView::new(lines);
        view.readonly = readonly || !exact;
        let text = ui.insert(wid, Rect::default(), Kind::Text(view));
        // Over the window's bottom edge: the key bar, OWLOSUI's status line.
        let status = ui.insert(root, Rect::new(0, h - 1, w, 1), Kind::Status(crate::chrome::key_line(Vec::new(), w)));
        let offers = offer::EDIT | offer::FIND | offer::REPLACE | offer::WRAP | offer::HEX | offer::SYNTAX | offer::KEYS;
        let chosen = prefs & (state::WRAP | state::NUMBERS | state::CLASSIC | state::QUIET);
        ui.set_editor(text, offers, chosen | if readonly || !exact { state::READONLY } else { 0 });
        if prefs & state::SYNTAX != 0 {
            ui.set_syntax(text, title);
        }
        ui.focus_first();
        let mut ed = Editor {
            ui,
            path: String::from(path),
            readonly: readonly || !exact,
            win: wid,
            text,
            status,
            codec,
            crlf,
            size,
            dialog: None,
            w,
            h,
            held: Mods::default(),
            caret: None,
            form: None,
            bar,
            lang: crate::lang::Lang::default(),
        };
        if !exact && !readonly {
            ed.message("This file has characters this screen cannot show; it is opened for viewing, so saving cannot change them.");
        }
        ed
    }

    /// Its words in `lang`: the menus now, the key bar and the dialogs as
    /// they are drawn.
    pub fn set_lang(&mut self, lang: crate::lang::Lang) {
        if let Kind::MenuBar(m) = self.ui.kind_mut(self.bar) {
            lang.menu(&mut m.base);
            lang.menu(&mut m.items);
        }
        self.lang = lang;
    }

    pub fn resize(&mut self, w: i16, h: i16) {
        self.w = w;
        self.h = h;
        self.ui.resize(w, h);
        self.ui.set_rect(self.win, Rect::new(-1, 0, w + 2, h));
        self.ui.set_rect(self.status, Rect::new(0, h - 1, w, 1));
    }

    fn view(&self) -> &TextView {
        match self.ui.kind(self.text) {
            Kind::Text(t) => t,
            _ => unreachable!("the editor's view is a text"),
        }
    }

    pub fn modified(&self) -> bool {
        self.view().modified
    }

    fn state(&self) -> u8 {
        self.ui.editor_state(self.text).map(|s| s.1).unwrap_or(0)
    }

    fn toggle(&mut self, bit: u8) {
        let (offers, s, _, _) = self.ui.editor_state(self.text).unwrap_or((0, 0, 0, 0));
        self.ui.set_editor(self.text, offers, s ^ bit);
    }

    /// A key as the text's own keys spell it: F7 is Ctrl+F, and so on.
    fn as_text_key(&mut self, code: KeyCode, ctrl: bool) {
        let mods = if ctrl { Mods::ctrl() } else { Mods::default() };
        self.ui.handle(Event::Key(Key { code, mods }));
    }

    pub fn handle(&mut self, fs: &mut dyn Fs, ev: Event) -> Out {
        if self.dialog.is_some() {
            let out = self.form_event(fs, ev);
            self.show_form();
            return out;
        }
        let menu_open = self.ui.menu_open().is_some();
        if let Event::Mouse(m) = ev {
            if m.y == self.h - 1 && !menu_open {
                // The status line's item under the click is its key.
                self.ui.handle(ev);
                while let Some(c) = self.ui.take_command() {
                    if (crate::chrome::KEY_BASE + 1..=crate::chrome::KEY_BASE + 10).contains(&c) {
                        let n = (c - crate::chrome::KEY_BASE) as u8;
                        return self.handle(fs, Event::Key(Key { code: KeyCode::F(n), mods: self.held }));
                    }
                }
                return Out::Stay;
            }
        }
        if let Event::Key(k) = ev {
            if !menu_open {
                if let Some(out) = self.key(fs, k) {
                    return out;
                }
            }
        }
        self.ui.handle(ev);
        self.commands(fs)
    }

    /// The commander's keys; `None` for the ones that are the text's.
    fn key(&mut self, fs: &mut dyn Fs, k: Key) -> Option<Out> {
        let m = k.mods;
        let none = m.is_none();
        let shift = m.shift && !m.ctrl && !m.alt;
        let viewer = self.readonly;
        Some(match k.code {
            // The core opens its bar on its own F10, after it has put the
            // editor's menus on it. A click on the first line reaches the bar
            // as it is.
            KeyCode::F(9) if none => {
                self.ui.handle(Event::Key(Key { code: KeyCode::F(10), mods: Mods::default() }));
                Out::Stay
            }
            KeyCode::F(10) | KeyCode::Esc if none => self.leave(),
            KeyCode::F(3) if none && viewer => Out::Close,
            KeyCode::F(2) if none && viewer => {
                self.toggle(state::WRAP);
                Out::Stay
            }
            KeyCode::F(2) if none => self.save(fs, None),
            KeyCode::F(2) if shift => {
                self.save_as();
                Out::Stay
            }
            KeyCode::F(10) if shift && !viewer => match self.save(fs, None) {
                Out::Saved(p) => {
                    self.dialog = None;
                    let _ = p;
                    Out::Close
                }
                o => o,
            },
            KeyCode::F(4) if none && viewer => {
                self.toggle(state::HEX);
                Out::Stay
            }
            KeyCode::F(4) if none => {
                self.as_text_key(KeyCode::Char('h'), true);
                Out::Stay
            }
            KeyCode::F(6) if none && viewer => Out::Edit,
            KeyCode::F(5) if none && viewer => {
                self.goto();
                Out::Stay
            }
            KeyCode::F(8) if m.alt && !viewer => {
                self.goto();
                Out::Stay
            }
            KeyCode::F(7) if none => {
                self.as_text_key(KeyCode::Char('f'), true);
                Out::Stay
            }
            KeyCode::F(7) if shift => {
                self.as_text_key(KeyCode::Char('l'), true);
                Out::Stay
            }
            KeyCode::F(7) if m.ctrl && !viewer => {
                self.as_text_key(KeyCode::Char('h'), true);
                Out::Stay
            }
            // The F-keys this screen gives no meaning to do nothing, rather
            // than reach the text's own bindings.
            KeyCode::F(_) => Out::Stay,
            _ => return None,
        })
    }

    fn commands(&mut self, fs: &mut dyn Fs) -> Out {
        while let Some(c) = self.ui.take_command() {
            match c {
                CM_SAVE => return self.save(fs, None),
                CM_SAVE_AS => self.save_as(),
                CM_CLOSE => return self.leave(),
                CM_EDIT => return Out::Edit,
                CM_GOTO => self.goto(),
                _ => {}
            }
        }
        Out::Stay
    }

    fn leave(&mut self) -> Out {
        if self.readonly || !self.modified() {
            return Out::Close;
        }
        let d = Dialog::new(Purpose::Message, "Edit")
            .line("", Tone::Normal)
            .line("You have made changes since the last save.", Tone::Normal)
            .line("", Tone::Normal)
            .buttons(&["~S~ave", "~D~on't save", "~C~ontinue editing"]);
        self.dialog = Some((d, Pending::Leave));
        Out::Stay
    }

    fn save_as(&mut self) {
        let d = Dialog::new(Purpose::Message, "Save as")
            .line("", Tone::Normal)
            .line("Save the file as", Tone::Normal)
            .input(Line::selected(&self.path))
            .line("", Tone::Normal)
            .buttons(&["~S~ave", "~C~ancel"]);
        self.dialog = Some((d, Pending::SaveAs));
    }

    fn goto(&mut self) {
        let d = Dialog::new(Purpose::Message, "Go to")
            .line("", Tone::Normal)
            .line("Line number, or a percentage: 50%", Tone::Normal)
            .input(Line::default())
            .line("", Tone::Normal)
            .buttons(&["~G~o", "~C~ancel"]);
        self.dialog = Some((d, Pending::Goto));
    }

    fn message(&mut self, text: &str) {
        let d = Dialog::new(Purpose::Message, "Warning").line("", Tone::Normal).line(text, Tone::Normal).line("", Tone::Normal).buttons(&["~O~K"]);
        self.dialog = Some((d, Pending::Message));
    }

    /// The dialog, if there is one and no window shows it yet, as
    /// OWLOSUI's window in this Ui; none, and the window goes.
    fn show_form(&mut self) {
        match &self.dialog {
            Some((d, _)) if self.form.is_none() => self.form = Some(crate::chrome::open_form(&mut self.ui, d, &self.lang)),
            None => {
                if let Some(f) = self.form.take() {
                    crate::chrome::close_form(&mut self.ui, f);
                }
            }
            _ => {}
        }
    }

    /// An event while a dialog is open: Esc is Cancel, a button's letter
    /// its button where nothing is typed, the rest OWLOSUI's window's.
    fn form_event(&mut self, fs: &mut dyn Fs, ev: Event) -> Out {
        self.show_form();
        let Some((d, _)) = self.dialog.as_ref() else { return Out::Stay };
        if let Event::Key(k) = ev {
            if k.code == KeyCode::Esc {
                return self.form_done(fs, Outcome::Cancel);
            }
            if let KeyCode::Char(c) = k.code {
                if !k.mods.ctrl && (k.mods.alt || d.input.is_none()) {
                    if let Some(i) = d.hotkey(c, &self.lang) {
                        return self.form_done(fs, Outcome::Button(i));
                    }
                }
            }
        }
        self.ui.handle(ev);
        if let Some(i) = crate::chrome::form_buttons(&mut self.ui).first().copied() {
            return self.form_done(fs, Outcome::Button(i));
        }
        if let (Some((d, _)), Some(f)) = (self.dialog.as_mut(), self.form.as_ref()) {
            crate::chrome::read_form(&self.ui, f, d);
        }
        Out::Stay
    }

    fn form_done(&mut self, fs: &mut dyn Fs, o: Outcome) -> Out {
        if let (Some((d, _)), Some(f)) = (self.dialog.as_mut(), self.form.as_ref()) {
            crate::chrome::read_form(&self.ui, f, d);
        }
        if let Some(f) = self.form.take() {
            crate::chrome::close_form(&mut self.ui, f);
        }
        self.dialog_done(fs, o)
    }

    fn dialog_done(&mut self, fs: &mut dyn Fs, o: Outcome) -> Out {
        let Some((d, pending)) = self.dialog.take() else { return Out::Stay };
        let b = match o {
            Outcome::Button(i) => Some(i),
            Outcome::Cancel => None,
        };
        match pending {
            Pending::Message => Out::Stay,
            Pending::Leave => match b {
                Some(0) => match self.save(fs, None) {
                    Out::Saved(_) if self.dialog.is_none() => Out::Close,
                    o => o,
                },
                Some(1) => Out::Close,
                _ => Out::Stay,
            },
            Pending::SaveAs => {
                if b == Some(0) {
                    let t = d.text();
                    if !t.trim().is_empty() {
                        return self.save(fs, Some(String::from(t.trim())));
                    }
                }
                Out::Stay
            }
            Pending::Goto => {
                if b == Some(0) {
                    self.go_to(&d.text());
                }
                Out::Stay
            }
        }
    }

    fn go_to(&mut self, s: &str) {
        let s = s.trim();
        let count = self.view().lines.len() as i64;
        let line = if let Some(p) = s.strip_suffix('%') {
            p.trim().parse::<i64>().ok().map(|p| p.clamp(0, 100) * (count - 1).max(0) / 100)
        } else {
            s.parse::<i64>().ok().map(|n| n - 1)
        };
        let Some(line) = line else { return };
        let line = line.clamp(0, (count - 1).max(0)) as i16;
        let page = (self.h - 2).max(1);
        if let Kind::Text(t) = self.ui.kind_mut(self.text) {
            t.cur = Point::new(0, line);
            t.anchor = None;
            t.top = (line - page / 2).max(0);
            t.top_row = 0;
        }
    }

    /// Written whole, with the line ends it had; on a failure the text
    /// stays as it is, and the error box says why (8.3).
    fn save(&mut self, fs: &mut dyn Fs, to: Option<String>) -> Out {
        if self.readonly {
            return Out::Stay;
        }
        let path = to.unwrap_or_else(|| self.path.clone());
        let end: &[u8] = if self.crlf { b"\r\n" } else { b"\n" };
        let mut data = Vec::new();
        let lines = &self.view().lines;
        for (i, l) in lines.iter().enumerate() {
            data.extend_from_slice(&self.codec.encode(l));
            if i + 1 < lines.len() {
                data.extend_from_slice(end);
            }
        }
        // A file that ended with a line end keeps it: the last empty line is that end.
        let r = (|| -> FsResult<()> {
            let h = fs.create(&path)?;
            let w = fs.write(h, &data);
            let c = fs.close(h);
            w?;
            c
        })();
        match r {
            Ok(()) => {
                if let Kind::Text(t) = self.ui.kind_mut(self.text) {
                    t.modified = false;
                }
                self.path = path.clone();
                self.size = data.len() as u64;
                Out::Saved(path)
            }
            Err(e) => {
                let why = alloc::format!("{}.", e.kind.text());
                let d = Dialog::new(Purpose::Message, "Error")
                    .error()
                    .line("", Tone::Normal)
                    .line("Cannot save the file", Tone::Normal)
                    .line(&path, Tone::Bright)
                    .line(&why, Tone::Normal)
                    .line("The text is kept; Shift+F2 saves it elsewhere.", Tone::Normal)
                    .line("", Tone::Normal)
                    .buttons(&["~O~K"]);
                self.dialog = Some((d, Pending::Message));
                Out::Stay
            }
        }
    }

    pub fn cursor(&self) -> Option<(i16, i16)> {
        self.caret
    }

    pub fn draw(&mut self, buf: &mut Buffer, scheme: &crate::colors::Scheme, glyph: &dyn Fn(char) -> Glyph) {
        self.show_form();
        // The key bar's words for what is held now.
        let items = crate::chrome::key_items(&self.labels(), self.held, self.w, &self.lang);
        if let Kind::Status(s) = self.ui.kind_mut(self.status) {
            *s = crate::chrome::key_line(items, self.w);
        }
        if buf.width() != self.w || buf.height() != self.h {
            buf.resize(self.w, self.h);
        }
        self.ui.draw(buf);
        self.caret = self.ui.cursor().map(|p| (p.x, p.y));
        let clip = buf.rect();
        let put = |buf: &mut Buffer, x: i16, y: i16, s: &str, a: u8| {
            let mut n = 0;
            for c in s.chars() {
                buf.put(x + n, y, glyph(c), a, clip);
                n += 1;
            }
        };
        if self.ui.menu_open().is_none() {
            // The status line, where the menu bar is while it is hidden.
            let (_, s, line, col) = self.ui.editor_state(self.text).unwrap_or((0, 0, 0, 0));
            let lang = self.ui.syntax_of(self.text).unwrap_or_default();
            let right = if self.readonly {
                let n = self.view().lines.len().max(1) as i64;
                let pct = ((line as i64 + 1) * 100 / n).min(100);
                alloc::format!("Col {:<4} {:>12} bytes {:>4}%", col + 1, names::thousands(self.size), pct)
            } else {
                alloc::format!("Line {:<5} Col {:<4} {}", line + 1, col + 1, lang)
            };
            let hex = if s & state::HEX != 0 { " [hex]" } else { "" };
            let mark = if self.modified() { " *" } else { "" };
            let room = (self.w as usize).saturating_sub(right.chars().count() + 4);
            let left = alloc::format!(" {}{}{}", names::cut(&self.path, room.saturating_sub(8)).trim_end(), mark, hex);
            put(buf, 0, 0, &names::pad(&left, self.w as usize), scheme.cursor);
            let rx = self.w - right.chars().count() as i16 - 2;
            put(buf, rx.max(0), 0, &right, scheme.cursor);
        }
        // A dialog is OWLOSUI's window, drawn with the rest of the Ui;
        // the key bar and the status line go under it again.
        if let Some(f) = &self.form {
            let r = self.ui.abs_rect(f.win);
            let mut over = Buffer::new(self.w, self.h);
            self.ui.draw(&mut over);
            // The window and its shadow, two cells right and one down.
            for y in r.y..r.y + r.h + 1 {
                for x in r.x..r.x + r.w + 2 {
                    if let Some(c) = over.get(x, y) {
                        buf.put(x, y, c.ch, c.attr, clip);
                    }
                }
            }
            self.caret = self.ui.cursor().map(|p| (p.x, p.y));
        }
    }

    fn labels(&self) -> [&'static str; 10] {
        let m = self.held;
        if self.readonly {
            if m.shift {
                return ["", "", "", "", "", "", "Next", "", "", ""];
            }
            let wrap = if self.state() & state::WRAP != 0 { "Unwrap" } else { "Wrap" };
            let hex = if self.state() & state::HEX != 0 { "Text" } else { "Hex" };
            return ["Help", wrap, "Quit", hex, "Goto", "Edit", "Search", "", "Menu", "Quit"];
        }
        if m.shift {
            return ["", "SaveAs", "", "", "", "", "Next", "", "", "SavQit"];
        }
        if m.alt {
            return ["", "", "", "", "", "", "", "Goto", "", ""];
        }
        ["Help", "Save", "", "Replac", "", "", "Search", "", "Menu", "Quit"]
    }
}

fn menu(readonly: bool) -> Vec<MenuItem> {
    let file = if readonly {
        alloc::vec![
            MenuItem::new("~E~dit", "F6", CM_EDIT).hint("The editor, on the same place"),
            MenuItem::new("~G~o to...", "F5", CM_GOTO).hint("A line, or a percentage of the file"),
            MenuItem::line(),
            MenuItem::new("~C~lose", "F10", CM_CLOSE).hint("Back to the panels"),
        ]
    } else {
        alloc::vec![
            MenuItem::new("~S~ave", "F2", CM_SAVE).hint("Write the file"),
            MenuItem::new("Save ~a~s...", "Shift+F2", CM_SAVE_AS).hint("Write it under another name"),
            MenuItem::line(),
            MenuItem::new("~G~o to line...", "Alt+F8", CM_GOTO).hint("A line, or a percentage of the file"),
            MenuItem::line(),
            MenuItem::new("~C~lose", "F10", CM_CLOSE).hint("Back to the panels; asks first if the text was changed"),
        ]
    };
    alloc::vec![MenuItem::sub("~F~ile", file)]
}

/// The lines of a file, CRLF or LF; a last line end makes no extra line
/// beyond the empty one the editor shows after it.
fn split_lines(data: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\n' {
            let end = if i > start && data[i - 1] == b'\r' { i - 1 } else { i };
            out.push(&data[start..end]);
            start = i + 1;
        }
        i += 1;
    }
    out.push(&data[start..]);
    out
}
