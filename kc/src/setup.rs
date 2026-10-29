//! The setup file (docs/04-menus.md, Save setup): the Options switches, the
//! colour scheme and each panel's view and order, as lines a person can
//! read and change.
//!
//! ```text
//! ; Kirin Commander setup - Save setup (Shift+F9) writes it
//! [options]
//! insert_moves_down=1
//! [screen]
//! colours=color
//! language=KCDEU.LNG
//! [left]
//! mode=brief
//! sort=name
//! reverse=0
//! [shortcuts]
//! 0=C:GAMES
//! ```
//!
//! A line it does not know is passed over, so an older commander reads a
//! newer file, and a switch the file does not name keeps its default.

use crate::app::App;
use crate::colors;
use crate::panel::{Focus, Mode, Sort};
use alloc::string::String;

const MODES: [(Mode, &str); 4] = [(Mode::Brief, "brief"), (Mode::Medium, "medium"), (Mode::Full, "full"), (Mode::Wide, "wide")];
const SORTS: [(Sort, &str); 5] = [(Sort::Name, "name"), (Sort::Ext, "extension"), (Sort::Time, "time"), (Sort::Size, "size"), (Sort::Unsorted, "unsorted")];

/// The file's text, CRLF line ends: it is a DOS file first.
pub fn text(app: &App) -> String {
    let mut out = String::from("; Kirin Commander setup - Save setup (Shift+F9) writes it\r\n[options]\r\n");
    for l in app.opt.lines() {
        out.push_str(&l);
        out.push_str("\r\n");
    }
    let colours = if app.scheme == colors::LAPTOP {
        "laptop"
    } else if app.scheme == colors::BLACK_WHITE {
        "bw"
    } else {
        "color"
    };
    out.push_str(&alloc::format!("[screen]\r\ncolours={}\r\n", colours));
    if !app.chrome.lang.file.is_empty() {
        out.push_str(&alloc::format!("language={}\r\n", app.chrome.lang.file));
    }
    for (side, p) in ["left", "right"].iter().zip(app.panels.iter()) {
        let mode = MODES.iter().find(|(m, _)| *m == p.mode).map(|(_, n)| *n).unwrap_or("brief");
        let sort = SORTS.iter().find(|(s, _)| *s == p.sort).map(|(_, n)| *n).unwrap_or("name");
        out.push_str(&alloc::format!("[{}]\r\nmode={}\r\nsort={}\r\nreverse={}\r\n", side, mode, sort, p.reverse as u8));
    }
    if app.shortcuts.iter().any(|s| s.is_some()) {
        out.push_str("[shortcuts]\r\n");
        for (i, s) in app.shortcuts.iter().enumerate() {
            if let Some(dir) = s {
                out.push_str(&alloc::format!("{}={}\r\n", i, dir));
            }
        }
    }
    out
}

/// The setup as it is now and where the commander is: each panel's folder
/// and the name under its cursor, the panel that is active, the command
/// lines run - `KC.SAV`, for the start after a step out (step.rs).
pub fn session(app: &App) -> String {
    let mut out = text(app);
    out.push_str(&alloc::format!("[session]\r\nactive={}\r\n", app.active));
    for (side, p) in ["left", "right"].iter().zip(app.panels.iter()) {
        out.push_str(&alloc::format!("{}={}\r\n", side, p.dir));
        if let Some(e) = p.current() {
            out.push_str(&alloc::format!("{}_at={}\r\n", side, e.name));
        }
    }
    for h in app.cmd.history() {
        out.push_str(&alloc::format!("history={}\r\n", h));
    }
    out
}

/// What a setup file says, put into the commander.
pub fn read(app: &mut App, text: &str) {
    let mut section = String::new();
    // A session's folders, put on the panels once every line is read.
    let mut dirs: [Option<String>; 2] = [None, None];
    let mut ats: [Option<String>; 2] = [None, None];
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_lowercase();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        // A folder or a file name keeps its letters as they are.
        let raw = value.trim();
        let (key, value) = (key.trim().to_lowercase(), value.trim().to_lowercase());
        match section.as_str() {
            "options" => {
                app.opt.read_line(&key, &value);
            }
            "session" => match key.as_str() {
                "active" => app.active = if value == "1" { 1 } else { 0 },
                "left" | "right" => dirs[if key == "left" { 0 } else { 1 }] = Some(String::from(raw)),
                "left_at" | "right_at" => ats[if key == "left_at" { 0 } else { 1 }] = Some(String::from(raw)),
                "history" => app.cmd.remember(raw),
                _ => {}
            },
            "screen" if key == "language" => app.load_lang(raw),
            "shortcuts" => {
                if let Ok(n) = key.parse::<usize>() {
                    if n < 10 && !raw.is_empty() {
                        app.shortcuts[n] = Some(String::from(raw));
                    }
                }
            }
            "screen" if key == "colours" => {
                app.scheme = match value.as_str() {
                    "laptop" => colors::LAPTOP,
                    "bw" => colors::BLACK_WHITE,
                    _ => colors::COLOR,
                };
            }
            "left" | "right" => {
                let p = &mut app.panels[if section == "left" { 0 } else { 1 }];
                match key.as_str() {
                    "mode" => {
                        if let Some((m, _)) = MODES.iter().find(|(_, n)| *n == value) {
                            p.set_mode(*m);
                        }
                    }
                    "sort" => {
                        if let Some((s, _)) = SORTS.iter().find(|(_, n)| *n == value) {
                            p.set_order(*s, p.reverse);
                        }
                    }
                    "reverse" => p.set_order(p.sort, value == "1"),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    // A folder that is gone leaves the panel where it was.
    for i in 0..2 {
        if let Some(dir) = dirs[i].take() {
            let focus = match ats[i].take() {
                Some(name) => Focus::Centre(name),
                None => Focus::First,
            };
            let fs = &mut *app.fs;
            let _ = app.panels[i].load(fs, &dir, focus);
        }
    }
}
