//! The picture (docs/01-screen.md): everything `App` is, drawn into
//! OWLOSUI's cell grid. The layout is the one the documents' screens show,
//! cell for cell, at 80 by 25; a bigger screen gets wider and taller
//! panels, the same picture.

use crate::app::App;
use crate::fs::Entry;
use crate::job::{Job, Op};
use crate::names;
use crate::options::O;
use crate::panel::{Mode, Panel, Sort, View};
use alloc::string::String;
use alloc::vec::Vec;
use owlosui_core::{Buffer, Glyph, Rect};

/// A character as a CP437 glyph: ASCII as it is, the frames and marks the
/// commander draws by their CP437 numbers, and on DOS a name's own bytes
/// (a DOS name arrives as chars 0..255 that already are the glyphs).
pub fn cp437(c: char) -> Glyph {
    let n = c as u32;
    if n < 0x80 {
        return n as Glyph;
    }
    let g: u8 = match c {
        '►' => 0x10,
        '◄' => 0x11,
        '▲' => 0x1E,
        '▼' => 0x1F,
        '░' => 0xB0,
        '▒' => 0xB1,
        '▓' => 0xB2,
        '│' => 0xB3,
        '┤' => 0xB4,
        '╢' => 0xB6,
        '╗' => 0xBB,
        '╝' => 0xBC,
        '┐' => 0xBF,
        '└' => 0xC0,
        '┴' => 0xC1,
        '┬' => 0xC2,
        '├' => 0xC3,
        '─' => 0xC4,
        '┼' => 0xC5,
        '╟' => 0xC7,
        '╚' => 0xC8,
        '╔' => 0xC9,
        '╧' => 0xCF,
        '╤' => 0xD1,
        '═' => 0xCD,
        '║' => 0xBA,
        '┘' => 0xD9,
        '┌' => 0xDA,
        '█' => 0xDB,
        '▄' => 0xDC,
        '▀' => 0xDF,
        '•' => 0x07,
        '√' => 0xFB,
        '·' => 0xFA,
        _ if n <= 0xFF => return n as Glyph,
        _ => b'?',
    };
    g as Glyph
}

struct Painter<'a> {
    buf: &'a mut Buffer,
    glyph: &'a dyn Fn(char) -> Glyph,
    clip: Rect,
}

impl Painter<'_> {
    fn put(&mut self, x: i16, y: i16, c: char, a: u8) {
        let g = (self.glyph)(c);
        self.buf.put(x, y, g, a, self.clip);
    }

    /// Text, cut at `max` cells.
    fn text(&mut self, x: i16, y: i16, s: &str, a: u8) -> i16 {
        let mut n = 0;
        for c in s.chars() {
            self.put(x + n, y, c, a);
            n += 1;
        }
        n
    }

    fn fill(&mut self, x: i16, y: i16, w: i16, h: i16, c: char, a: u8) {
        for j in 0..h {
            for i in 0..w {
                self.put(x + i, y + j, c, a);
            }
        }
    }

    fn paint(&mut self, x: i16, y: i16, w: i16, a: u8) {
        for i in 0..w {
            self.buf.recolor(x + i, y, a, self.clip);
        }
    }

    fn centre(&mut self, x: i16, y: i16, w: i16, s: &str, a: u8) {
        let n = s.chars().count() as i16;
        self.text(x + (w - n).max(0) / 2, y, s, a);
    }

    /// A frame and what is in it: double for panels and dialogs.
    fn frame(&mut self, x: i16, y: i16, w: i16, h: i16, a: u8, double: bool) {
        let (tl, tr, bl, br, hz, vt) = if double { ('╔', '╗', '╚', '╝', '═', '║') } else { ('┌', '┐', '└', '┘', '─', '│') };
        self.fill(x, y, w, h, ' ', a);
        for i in 1..w - 1 {
            self.put(x + i, y, hz, a);
            self.put(x + i, y + h - 1, hz, a);
        }
        for j in 1..h - 1 {
            self.put(x, y + j, vt, a);
            self.put(x + w - 1, y + j, vt, a);
        }
        self.put(x, y, tl, a);
        self.put(x + w - 1, y, tr, a);
        self.put(x, y + h - 1, bl, a);
        self.put(x + w - 1, y + h - 1, br, a);
    }
}

/// How a panel's columns are laid out in a panel `w` cells wide.
pub struct Columns {
    /// x (from the panel's left edge) and width of each name column.
    pub names: Vec<(i16, i16)>,
    /// Full and Wide: the size, date and time columns, (x, width).
    pub size: Option<(i16, i16)>,
    pub date: Option<(i16, i16)>,
    pub time: Option<(i16, i16)>,
    /// Detailed: the attributes, `RHSA`.
    pub attrs: Option<(i16, i16)>,
}

pub fn columns(mode: Mode, w: i16) -> Columns {
    let inner = w - 2;
    match mode {
        Mode::Brief | Mode::Medium => {
            let n = mode.columns() as i16;
            let cw = (inner - (n - 1)) / n;
            let mut names = Vec::new();
            let mut x = 1;
            for i in 0..n {
                let width = if i == n - 1 { inner + 1 - x } else { cw };
                names.push((x, width));
                x += width + 1;
            }
            Columns { names, size: None, date: None, time: None, attrs: None }
        }
        Mode::Full => {
            let nw = (inner - 26).max(4);
            Columns {
                names: alloc::vec![(1, nw)],
                size: Some((nw + 2, 9)),
                date: Some((nw + 12, 8)),
                time: Some((nw + 21, inner - nw - 20)),
                attrs: None,
            }
        }
        Mode::Wide => {
            let nw = (inner - 10).max(4);
            Columns { names: alloc::vec![(1, nw)], size: Some((nw + 2, 9)), date: None, time: None, attrs: None }
        }
        Mode::Detailed => {
            // Name, then 9 + 8 + 5 + 4 and the four rules between them.
            let nw = (inner - 31).max(4);
            Columns {
                names: alloc::vec![(1, nw)],
                size: Some((nw + 2, 9)),
                date: Some((nw + 12, 8)),
                time: Some((nw + 21, 5)),
                attrs: Some((nw + 27, inner - nw - 26)),
            }
        }
    }
}

/// The name under a cell, for the mouse: `row` counted from the panel's
/// first row of names.
pub fn index_at(p: &Panel, px: i16, pw: i16, x: i16, row: i16) -> Option<usize> {
    if row < 0 || row as usize >= p.rows {
        return None;
    }
    let cols = columns(p.mode, pw);
    let col = cols.names.iter().position(|&(cx, cw)| x >= px + cx && x <= px + cx + cw)?;
    let i = p.top + col * p.rows + row as usize;
    if i < p.entries.len() {
        Some(i)
    } else {
        None
    }
}

pub fn draw(app: &mut App, buf: &mut Buffer, glyph: &dyn Fn(char) -> Glyph) {
    let (w, h) = (app.w, app.h);
    if buf.width() != w || buf.height() != h {
        buf.resize(w, h);
    }
    let clip = buf.rect();
    let mut pt = Painter { buf, glyph, clip };
    let sc = app.scheme;
    pt.fill(0, 0, w, h, ' ', sc.command);
    app.caret = None;

    let cmd_y = h - 1 - app.key_bar() as i16;
    // The terminal: what ran, under the panels and in place of them.
    let shown = app.terminal.len().min(cmd_y.max(0) as usize);
    for (k, l) in app.terminal[app.terminal.len() - shown..].iter().enumerate() {
        let y = cmd_y - shown as i16 + k as i16;
        pt.text(0, y, &names::cut(l, w as usize), sc.command);
    }

    if app.panels_on {
        for i in 0..2 {
            if let Some((x, pw)) = app.panel_rect(i) {
                let active = i == app.active;
                draw_panel(&mut pt, app, i, x, pw, active);
            }
        }
        if let Some(clock) = app.clock.filter(|_| app.opt.on(O::Clock)) {
            if app.panel_rect(1).is_some() || app.narrow() || app.top() > 0 {
                pt.text(w - 6, 0, &alloc::format!("{:02}:{:02}", clock.0, clock.1), sc.clock);
            }
        }
    }

    draw_command_line(&mut pt, app, cmd_y);
    if app.search.is_some() {
        draw_search(&mut pt, app);
    }
}

fn draw_panel(pt: &mut Painter, app: &App, i: usize, x: i16, pw: i16, active: bool) {
    match app.panels[i].view {
        View::Info => return draw_info(pt, app, i, x, pw),
        View::Tree => return draw_tree(pt, app, i, x, pw, active),
        View::Quick => return draw_quick(pt, app, i, x, pw),
        View::Files => {}
    }
    let sc = &app.scheme;
    let p = &app.panels[i];
    let ph = app.panel_height();
    let a = sc.panel;
    // The panel's top edge: the first line, or the second under a menu bar
    // that stays.
    let y0 = app.top();
    let mini = app.opt.on(O::MiniStatus);
    pt.frame(x, y0, pw, ph, a, true);
    let cols = columns(p.mode, pw);
    // The column rules: from the top edge to the line over the mini status,
    // or to the bottom edge when there is no mini status.
    let mut rules: Vec<i16> = Vec::new();
    for &(cx, cw) in cols.names.iter().skip(1) {
        let _ = cw;
        rules.push(cx - 1);
    }
    for c in [cols.size, cols.date, cols.time, cols.attrs].iter().flatten() {
        rules.push(c.0 - 1);
    }
    let end_y = if mini { y0 + ph - 3 } else { y0 + ph - 1 };
    for &rx in &rules {
        pt.put(x + rx, y0, '╤', a);
        for y in y0 + 1..end_y {
            pt.put(x + rx, y, '│', a);
        }
        pt.put(x + rx, end_y, if mini { '┴' } else { '╧' }, a);
    }
    if mini {
        pt.put(x, end_y, '╟', a);
        for k in 1..pw - 1 {
            pt.put(x + k, end_y, '─', a);
        }
        pt.put(x + pw - 1, end_y, '╢', a);
        for &rx in &rules {
            pt.put(x + rx, end_y, '┴', a);
        }
    }

    // The title: the folder, shortened from the left, lit when active.
    let room = (pw - 4).max(4) as usize;
    let mut title = p.dir.clone();
    if title.chars().count() > room {
        let tail: String = title.chars().rev().take(room - 3).collect::<Vec<char>>().into_iter().rev().collect();
        title = alloc::format!("...{}", tail);
    }
    let lit = if active && app.dialog.is_none() || active { sc.active_title } else { a };
    pt.centre(x, y0, pw, &alloc::format!(" {} ", title), lit);
    if p.sort != Sort::Name || p.reverse {
        let l = p.sort.letter();
        pt.put(x + 1, y0, if p.reverse { l.to_ascii_uppercase() } else { l }, a);
    }
    if p.filter.is_some() {
        pt.text(x + pw - 3, y0, "*", a);
    }

    // Column titles, if they are on.
    if app.opt.on(O::ColumnTitles) {
        let ty = y0 + 1;
        for &(cx, cw) in &cols.names {
            pt.centre(x + cx, ty, cw, &app.chrome.lang.tr("Name"), sc.header);
        }
        if let Some((cx, cw)) = cols.size {
            pt.centre(x + cx, ty, cw, &app.chrome.lang.tr("Size"), sc.header);
        }
        if let Some((cx, cw)) = cols.date {
            pt.centre(x + cx, ty, cw, &app.chrome.lang.tr("Date"), sc.header);
        }
        if let Some((cx, cw)) = cols.time {
            pt.centre(x + cx, ty, cw, &app.chrome.lang.tr("Time"), sc.header);
        }
        if let Some((cx, cw)) = cols.attrs {
            pt.centre(x + cx, ty, cw, &app.chrome.lang.tr("Attr"), sc.header);
        }
    }

    let short = p.short_names();
    // Upper case folders and lower case files, on 8.3 names, if so set.
    let case = app.opt.on(O::NamesCase);
    let first_y = y0 + app.names_y();
    let rows = p.rows;
    for (ci, &(cx, cw)) in cols.names.iter().enumerate() {
        for r in 0..rows {
            let idx = p.top + ci * rows + r;
            let Some(e) = p.entries.get(idx) else { continue };
            let y = first_y + r as i16;
            let sel = p.selected.get(idx).copied().unwrap_or(false);
            let under = idx == p.cursor && active;
            let at = match (under, sel) {
                (true, true) => sc.cursor_selected,
                (true, false) => sc.cursor,
                (false, true) => sc.selected,
                (false, false) => crate::colors::of_entry(sc, e),
            };
            if under && cols.size.is_some() {
                // One column: the bar runs across the whole row, rules and all.
                pt.paint(x + 1, y, pw - 2, at);
            }
            let mut name = names::panel_name(e, short, case, cw as usize);
            if sel && sc.mark_selected {
                name = mark(&name, short && cw >= 12);
            }
            pt.text(x + cx, y, &name, at);
            if let Some((sx, sw)) = cols.size {
                let s = size_of(app, p, e);
                pt.text(x + sx, y, &names::pad_left(&s, sw as usize), at);
            }
            if let Some((dx, _)) = cols.date {
                pt.text(x + dx, y, &names::date(&e.stamp), at);
            }
            if let Some((tx, tw)) = cols.time {
                pt.text(x + tx, y, &names::pad_left(&names::time(&e.stamp), tw as usize), at);
            }
            if let Some((ax, _)) = cols.attrs {
                pt.text(x + ax, y, &attr_text(e), at);
            }
        }
    }

    // The mini status: the selection's total, or the name under the cursor.
    if !mini {
        return;
    }
    let my = y0 + ph - 2;
    let (n, bytes) = p.selection_total();
    if n > 0 {
        let what = if n == 1 { "file" } else { "files" };
        let s = alloc::format!("{} bytes in {} selected {}", names::thousands(bytes), n, what);
        pt.centre(x + 1, my, pw - 2, names::cut(&app.chrome.lang.tr(&s), (pw - 2) as usize).trim_end(), sc.selected);
    } else if let Some(e) = p.current() {
        let nw = (pw - 2 - 26).max(4) as usize;
        let s = alloc::format!(
            "{} {} {} {}",
            names::panel_name(e, short, case, nw),
            names::pad_left(&size_of(app, p, e), 9),
            names::date(&e.stamp),
            names::pad_left(&names::time(&e.stamp), 5)
        );
        pt.text(x + 1, my, &names::cut(&s, (pw - 2) as usize), a);
    }
}

/// Laptop: `√` in the name's spare cell - the one between name and
/// extension on 8.3, else the last.
fn mark(name: &str, short: bool) -> String {
    let mut cs: Vec<char> = name.chars().collect();
    let at = if short { 8 } else { cs.len().saturating_sub(1) };
    if at < cs.len() {
        cs[at] = '√';
    }
    cs.into_iter().collect()
}

fn size_of(app: &App, p: &Panel, e: &Entry) -> String {
    if e.dir && !e.is_up() {
        if let Some(n) = app.counted(&p.path_of(&e.name)) {
            return names::fit_size(n, 9);
        }
    }
    names::size_text(e)
}

fn draw_command_line(pt: &mut Painter, app: &mut App, y: i16) {
    let sc = app.scheme;
    let w = app.w;
    let mut prompt = app.panel().dir.clone();
    prompt.push('>');
    // A prompt longer than half the line gives up its start.
    let max = (w / 2).max(8) as usize;
    if prompt.chars().count() > max {
        let tail: String = prompt.chars().rev().take(max - 3).collect::<Vec<char>>().into_iter().rev().collect();
        prompt = alloc::format!("...{}", tail);
    }
    let px = pt.text(0, y, &prompt, sc.command);
    let room = (w - px).max(1) as usize;
    let (text, caret) = app.cmd.view(room);
    pt.text(px, y, &text, sc.command);
    if app.dialog.is_none() && app.drive_menu.is_none() && app.search.is_none() {
        app.caret = Some((px + caret as i16, y));
    }
}

fn draw_search(pt: &mut Painter, app: &mut App) {
    let sc = app.scheme;
    let Some(panel) = app.search.as_ref().map(|s| s.panel) else { return };
    let Some((x, pw)) = app.panel_rect(panel) else { return };
    // Over the panel's bottom lines.
    let by = app.top() + app.panel_height() - 3;
    let Some(s) = app.search.as_mut() else { return };
    let bw = (pw - 4).clamp(12, 26);
    let bx = x + 2;
    pt.frame(bx, by, bw, 3, sc.dialog, false);
    pt.text(bx + 1, by + 1, &app.chrome.lang.tr("Search:"), sc.dialog);
    let room = (bw - 10).max(1) as usize;
    let (text, caret) = s.line.view(room);
    pt.fill(bx + 9, by + 1, room as i16, 1, ' ', sc.dialog_input);
    pt.text(bx + 9, by + 1, &text, sc.dialog_input);
    app.caret = Some((bx + 9 + caret as i16, by + 1));
}

/// The drive menu's words (docs/02-panels.md 2.8): its title, the line of
/// column names over the drives, a line a drive. OWLOSUI's list shows them.
pub fn drive_lines(app: &App, m: &crate::app::DriveMenu) -> (String, Option<String>, Vec<String>) {
    let title = String::from(if m.panel == 0 { "Choose left drive" } else { "Choose right drive" });
    if app.fs.short_names() {
        // DOS: a line for each drive - the letter, the size, the free bytes
        // and their share; a drive that was not asked says what it is.
        let lines = m
            .drives
            .iter()
            .zip(&m.space)
            .map(|(d, s)| match *s {
                Some((size, free)) => {
                    let pct = if size == 0 { 0 } else { free * 100 / size }; // free share, whole percent: free * 100 / size
                    alloc::format!("{}:  {:>7}  {:>7} {:>3}%", d.letter, names::fit_size(size, 7), names::fit_size(free, 7), pct)
                }
                None => alloc::format!("{}:  {}", d.letter, d.what),
            })
            .collect();
        let head = m.space.iter().any(|s| s.is_some()).then(|| alloc::format!("    {:>7}  {:>7}", app.chrome.lang.tr("Size"), app.chrome.lang.tr("Free")));
        (title, head, lines)
    } else {
        // A desktop or a browser, as Far: the letter and what it is.
        let lines = m.drives.iter().map(|d| alloc::format!("{}  {:<24}{}", d.letter, names::cut(&d.root, 24), d.what)).collect();
        (title, None, lines)
    }
}



/// The progress window's words and bars (docs/05-file-operations.md 5.7):
/// its title, its lines, and each bar as (the line it is under, done, total).
pub fn progress_lines(job: &Job) -> (&'static str, Vec<String>, Vec<(usize, u64, u64)>) {
    let total = alloc::format!(
        "Total: {} of {} files, {} of {}",
        job.files_done,
        job.files_total,
        names::human(job.bytes_done),
        names::human(job.bytes_total)
    );
    match job.op {
        Op::Delete { .. } => (
            "Delete",
            alloc::vec![String::from("Deleting"), job.current.clone(), total],
            alloc::vec![(2, job.files_done as u64, job.files_total as u64)],
        ),
        op => (
            if op == Op::Copy { "Copy" } else { "Move" },
            alloc::vec![
                String::from(if op == Op::Copy { "Copying the file" } else { "Moving the file" }),
                job.current.clone(),
                String::from("to"),
                job.current_to.clone(),
                total,
            ],
            alloc::vec![(3, job.file_done, job.file_size), (4, job.bytes_done, job.bytes_total)],
        ),
    }
}


/// `RHSA`, a dot for an attribute that is off.
fn attr_text(e: &Entry) -> String {
    let a = e.attrs;
    [(a.read_only, 'R'), (a.hidden, 'H'), (a.system, 'S'), (a.archive, 'A')].iter().map(|&(on, c)| if on { c } else { '.' }).collect()
}

/// The frame of a panel that is not a file list, its title in it. The
/// rows inside: the first and the last.
fn side_frame(pt: &mut Painter, app: &App, x: i16, pw: i16, title: &str, lit: bool) -> (i16, i16) {
    let sc = &app.scheme;
    let y0 = app.top();
    let ph = app.panel_height();
    pt.frame(x, y0, pw, ph, sc.panel, true);
    let room = (pw - 4).max(4) as usize;
    let t = alloc::format!(" {} ", names::cut(&app.chrome.lang.tr(title), room).trim_end());
    pt.centre(x, y0, pw, &t, if lit { sc.active_title } else { sc.panel });
    (y0 + 1, y0 + ph - 2)
}

/// Text of a file, a line at a time, as the codec turns bytes into the
/// font's glyphs; tabs to the next stop of eight.
fn glyph_lines(app: &App, bytes: &[u8], rows: usize) -> Vec<Vec<Glyph>> {
    let codec = (app.codec)();
    let _ = codec.exact(bytes);
    let mut out = Vec::new();
    for line in bytes.split(|&b| b == b'\n').take(rows) {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let mut flat: Vec<u8> = Vec::new();
        for &b in line {
            if b == b'\t' {
                flat.push(b' ');
                while flat.len() % 8 != 0 {
                    flat.push(b' ');
                }
            } else {
                flat.push(b);
            }
        }
        out.push(codec.decode(&flat));
    }
    out
}

fn put_glyphs(pt: &mut Painter, x: i16, y: i16, line: &[Glyph], width: i16, a: u8) {
    for (k, &g) in line.iter().take(width.max(0) as usize).enumerate() {
        pt.buf.put(x + k as i16, y, g, a, pt.clip);
    }
}

/// The Info panel: the drive, the memory, the folder, its description
/// (docs/01-screen.md 1.2) - of the other panel.
fn draw_info(pt: &mut Painter, app: &App, _i: usize, x: i16, pw: i16) {
    let sc = &app.scheme;
    let (y1, y2) = side_frame(pt, app, x, pw, "Info", false);
    let Some(f) = app.info.as_ref() else { return };
    let inner = pw - 2;
    let mut lines: Vec<(String, u8)> = Vec::new();
    lines.push((String::new(), sc.panel));
    let root = crate::path::root(&f.dir, app.fs.sep());
    lines.push((
        match &f.label {
            Some(l) => alloc::format!(" Drive {}, label {}", root, l),
            None => alloc::format!(" Drive {}", root),
        },
        sc.panel,
    ));
    if let Some((all, free)) = f.space {
        lines.push((alloc::format!(" {} bytes in all", names::pad_left(&names::thousands(all), 15)), sc.panel));
        lines.push((alloc::format!(" {} bytes free", names::pad_left(&names::thousands(free), 15)), sc.panel));
    }
    if let Some(m) = app.memory {
        lines.push((String::new(), sc.panel));
        lines.push((alloc::format!(" {} bytes of memory free", names::pad_left(&names::thousands(m), 15)), sc.panel));
    }
    lines.push((String::new(), sc.panel));
    lines.push((alloc::format!(" {}", f.dir), sc.header));
    let files = if f.files == 1 { "file" } else { "files" };
    let folders = if f.folders == 1 { "folder" } else { "folders" };
    lines.push((alloc::format!(" {} {}, {} {}", f.files, files, f.folders, folders), sc.panel));
    lines.push((alloc::format!(" {} bytes in the files", names::pad_left(&names::thousands(f.bytes), 15)), sc.panel));
    lines.push((String::new(), sc.panel));
    match &f.about {
        Some((name, _)) => {
            let rule: String = core::iter::repeat_n('─', (inner as usize).saturating_sub(name.chars().count() + 3)).collect();
            lines.push((alloc::format!(" {} {}", name, rule), sc.panel));
        }
        None => lines.push((String::from(" No DIRINFO or DESCRIPT.ION here"), sc.panel)),
    }
    pt.centre(x + 1, y1, inner, &app.chrome.lang.tr("Kirin Commander"), sc.header);
    let mut y = y1 + 1;
    for (s, a) in &lines {
        if y > y2 {
            return;
        }
        pt.text(x + 1, y, names::cut(&app.chrome.lang.tr(s), inner as usize).trim_end(), *a);
        y += 1;
    }
    if let Some((_, bytes)) = &f.about {
        let rows = (y2 - y + 1).max(0) as usize;
        for l in glyph_lines(app, bytes, rows) {
            put_glyphs(pt, x + 2, y, &l, inner - 2, sc.panel);
            y += 1;
        }
    }
}

/// The folder tree: the drive's folders, open down to the panel's own.
fn draw_tree(pt: &mut Painter, app: &App, i: usize, x: i16, pw: i16, active: bool) {
    let sc = &app.scheme;
    let p = &app.panels[i];
    let Some(t) = app.trees[i].as_ref() else { return };
    side_frame(pt, app, x, pw, "Tree", active);
    let y0 = app.top();
    let ph = app.panel_height();
    let mini = app.opt.on(O::MiniStatus);
    let inner = pw - 2;
    let first_y = y0 + app.names_y();
    // Folders upper case on 8.3 names, as the file panel draws them.
    let case = app.opt.on(O::NamesCase) && p.short_names();
    for r in 0..t.rows {
        let k = t.top + r;
        let Some(n) = t.nodes.get(k) else { break };
        let y = first_y + r as i16;
        let name = if n.depth > 0 && case { n.name.to_uppercase() } else { n.name.clone() };
        let lines = t.prefix(k);
        let lines_w = lines.chars().count() as i16;
        pt.text(x + 1, y, &names::cut(&alloc::format!("{}{}", lines, name), inner as usize), sc.panel);
        // The bar is the name's, not the lines'.
        let at = if k == t.cursor && active { sc.cursor } else { sc.file_dir };
        let nw = (name.chars().count() as i16).min(inner - lines_w);
        if nw > 0 {
            pt.paint(x + 1 + lines_w, y, nw, at);
        }
    }
    if mini {
        let end_y = y0 + ph - 3;
        pt.put(x, end_y, '╟', sc.panel);
        for k in 1..pw - 1 {
            pt.put(x + k, end_y, '─', sc.panel);
        }
        pt.put(x + pw - 1, end_y, '╢', sc.panel);
        pt.text(x + 1, y0 + ph - 2, &names::cut(t.current(), inner as usize), sc.panel);
    }
}

/// Quick view: the file under the other panel's cursor, its first lines -
/// or its bytes in hex, for a file that is not text.
fn draw_quick(pt: &mut Painter, app: &App, _i: usize, x: i16, pw: i16) {
    let sc = &app.scheme;
    let (y1, y2) = side_frame(pt, app, x, pw, "Quick view", false);
    let Some(q) = app.quick.as_ref() else { return };
    let inner = pw - 2;
    let rows = (y2 - y1 + 1).max(0) as usize;
    let room = (inner - 2).max(1) as usize;
    if q.dir {
        pt.text(x + 2, y1 + 1, &app.chrome.lang.tr("Folder"), sc.header);
        pt.text(x + 2, y1 + 2, names::cut(&q.path, room).trim_end(), sc.panel);
        return;
    }
    if let Some(e) = &q.error {
        let name = crate::path::name(&q.path, app.fs.sep());
        pt.text(x + 2, y1 + 1, names::cut(&app.chrome.lang.tr(&alloc::format!("Cannot read {}", name)), room).trim_end(), sc.panel);
        pt.text(x + 2, y1 + 2, names::cut(&alloc::format!("{}.", e.kind.text()), room).trim_end(), sc.panel);
        return;
    }
    // Not text: a NUL, or many control codes, in its first K.
    let head = &q.bytes[..q.bytes.len().min(1024)];
    let binary = head.contains(&0) || head.iter().filter(|&&b| b < 32 && !matches!(b, 9 | 10 | 12 | 13 | 26)).count() > 8;
    if binary {
        // The offset, the bytes, and the bytes as the font draws them.
        let per = ((inner - 6) / 4).clamp(1, 16) as usize;
        for (r, chunk) in q.bytes.chunks(per).take(rows).enumerate() {
            let y = y1 + r as i16;
            let mut s = alloc::format!("{:04X} ", r * per);
            for b in chunk {
                s.push_str(&alloc::format!("{:02X} ", b));
            }
            for _ in chunk.len()..per {
                s.push_str("   ");
            }
            let n = pt.text(x + 1, y, &s, sc.panel);
            let glyphs: Vec<Glyph> = chunk.iter().map(|&b| if b < 32 { b'.' as Glyph } else { b as Glyph }).collect();
            put_glyphs(pt, x + 1 + n, y, &glyphs, inner - n, sc.panel);
        }
    } else {
        for (r, l) in glyph_lines(app, &q.bytes, rows).iter().enumerate() {
            put_glyphs(pt, x + 1, y1 + r as i16, l, inner, sc.panel);
        }
    }
}
