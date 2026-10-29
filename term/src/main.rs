//! Kirin Commander in a terminal: OWLOSUI's terminal backend draws the
//! cells, the desktop's file system is under the panels, and a command
//! line runs in the system's shell with the panels out of the way.
//!
//! `kc --shot` draws one frame as text and leaves - for a test to read
//! what the screen is without a person.

use kc::app::Request;
use kc::stdfs::StdFs;
use kc::App;
use owlosui_console::Term;
use owlosui_core::{Buffer, Event, Glyph, Point};
use std::time::Duration;

/// A character as the terminal backend's code page has it.
fn glyph(c: char) -> Glyph {
    let g = kc::screen::cp437(c);
    if (c as u32) > 0xFF && g == b'?' as Glyph {
        return owlosui_console::cp437::from_char(c) as Glyph;
    }
    if (0x80..=0xFF).contains(&(c as u32)) && kc::screen::cp437(c) == c as u32 as Glyph {
        // A Latin-1 letter in a name: the code page's own glyph for it.
        return owlosui_console::cp437::from_char(c) as Glyph;
    }
    g
}

/// A file as the terminal can show it: UTF-8 through its code page when
/// every character has a glyph there, else the bytes as CP437.
struct Desktop {
    utf8: std::cell::Cell<bool>,
}

impl kc::editor::Codec for Desktop {
    fn exact(&self, bytes: &[u8]) -> bool {
        match std::str::from_utf8(bytes) {
            Ok(s) => {
                self.utf8.set(true);
                s.chars().all(|c| c == '\r' || c == '\n' || owlosui_console::cp437::to_char(owlosui_console::cp437::from_char(c)) == c)
            }
            Err(_) => {
                self.utf8.set(false);
                true
            }
        }
    }
    fn decode(&self, line: &[u8]) -> Vec<Glyph> {
        if self.utf8.get() {
            String::from_utf8_lossy(line).chars().map(|c| owlosui_console::cp437::from_char(c) as Glyph).collect()
        } else {
            line.iter().map(|&b| b as Glyph).collect()
        }
    }
    fn encode(&self, line: &[Glyph]) -> Vec<u8> {
        if self.utf8.get() {
            line.iter().map(|&g| owlosui_console::cp437::to_char(g as u8)).collect::<String>().into_bytes()
        } else {
            line.iter().map(|&g| g as u8).collect()
        }
    }
}

fn clock() -> (u8, u8) {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let local = secs + kc::stdfs::local_offset();
    let day = local.rem_euclid(86_400);
    ((day / 3600) as u8, (day / 60 % 60) as u8)
}

/// Where Save setup writes: the commander's own folder of settings, `KC`
/// (D11) - in %APPDATA% on Windows, in ~/.config elsewhere. `--shot`
/// reads none, so a test sees the defaults.
fn setup_path() -> Option<String> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(std::path::PathBuf::from)?
    } else {
        std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join(".config")))?
    };
    let dir = base.join(if cfg!(windows) { "KC" } else { "kc" });
    Some(dir.join("kc.ini").to_string_lossy().into_owned())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--shot") {
        let mut app = App::new(Box::new(StdFs::new()), 80, 25);
        app.codec = || Box::new(Desktop { utf8: std::cell::Cell::new(true) });
        app.clock = Some(clock());
        for ev in args.get(i + 1).map(|s| script(s)).unwrap_or_default() {
            app.handle(ev);
            while app.busy() {
                app.tick();
            }
        }
        let mut buf = Buffer::new(80, 25);
        app.draw(&mut buf, &glyph);
        for y in 0..25 {
            let line: String = (0..80).map(|x| owlosui_console::cp437::to_char(buf.get(x, y).unwrap().ch as u8)).collect();
            println!("{}", line);
        }
        return Ok(());
    }

    let (w, h) = Term::size()?;
    let mut term = Some(Term::new()?);
    let mut app = App::new(Box::new(StdFs::new()), w, h);
    app.system = vec![format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)];
    app.codec = || Box::new(Desktop { utf8: std::cell::Cell::new(true) });
    app.setup_path = setup_path();
    app.load_setup();
    let mut buf = Buffer::new(w, h);
    loop {
        app.clock = Some(clock());
        app.draw(&mut buf, &glyph);
        if let Some(t) = term.as_mut() {
            t.render(&buf, app.cursor().map(|(x, y)| Point::new(x, y)))?;
        }
        for r in app.take_requests() {
            match r {
                Request::Quit => return Ok(()),
                Request::Run { dir, line } => {
                    // The panels out of the way: the program has the real terminal.
                    drop(term.take());
                    run(&dir, &line);
                    app.print(&format!("{}>{}", dir, line));
                    app.ran();
                    term = Some(Term::new()?);
                }
            }
        }
        let wait = if app.busy() { Duration::ZERO } else { Duration::from_millis(500) };
        if app.busy() {
            app.tick();
        }
        if !crossterm::event::poll(wait)? {
            continue;
        }
        let Some(t) = term.as_ref() else { continue };
        if let Some(ev) = t.next_event()? {
            if let Event::Resize(nw, nh) = ev {
                buf.resize(nw, nh);
            }
            app.handle(ev);
        }
    }
}

/// A command line in the system's shell, in the panel's folder.
fn run(dir: &str, line: &str) {
    println!("{}>{}", dir, line);
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("cmd");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.raw_arg(format!("/C {}", line));
        }
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.arg("-c").arg(line);
        c
    };
    match cmd.current_dir(dir).status() {
        Ok(_) => {}
        Err(e) => println!("{}", e),
    }
}

/// Keys for `--shot`, by name: `Down Down F4 Ctrl+R Alt+F1 "text" Enter`.
fn script(s: &str) -> Vec<Event> {
    use owlosui_core::{Key, KeyCode, Mods};
    let mut out = Vec::new();
    let mut rest = s.trim();
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('"') {
            let end = r.find('"').unwrap_or(r.len());
            for c in r[..end].chars() {
                out.push(Event::Key(Key { code: KeyCode::Char(c), mods: Mods::default() }));
            }
            rest = r[(end + 1).min(r.len())..].trim_start();
            continue;
        }
        let end = rest.find(' ').unwrap_or(rest.len());
        let mut word = &rest[..end];
        rest = rest[end..].trim_start();
        let mut mods = Mods::default();
        loop {
            if let Some(w) = word.strip_prefix("Ctrl+") {
                mods.ctrl = true;
                word = w;
            } else if let Some(w) = word.strip_prefix("Alt+") {
                mods.alt = true;
                word = w;
            } else if let Some(w) = word.strip_prefix("Shift+") {
                mods.shift = true;
                word = w;
            } else {
                break;
            }
        }
        let code = match word {
            "Enter" => KeyCode::Enter,
            "Esc" => KeyCode::Esc,
            "Tab" => KeyCode::Tab,
            "Up" => KeyCode::Up,
            "Down" => KeyCode::Down,
            "Left" => KeyCode::Left,
            "Right" => KeyCode::Right,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "PgUp" => KeyCode::PageUp,
            "PgDn" => KeyCode::PageDown,
            "Ins" => KeyCode::Insert,
            "Del" => KeyCode::Delete,
            "Bksp" => KeyCode::Backspace,
            "Space" => KeyCode::Char(' '),
            w if w.starts_with('F') && w.len() > 1 && w[1..].parse::<u8>().is_ok() => KeyCode::F(w[1..].parse().unwrap()),
            w => match w.chars().next() {
                Some(c) => KeyCode::Char(c.to_ascii_lowercase()),
                None => continue,
            },
        };
        out.push(Event::Key(Key { code, mods }));
    }
    out
}
