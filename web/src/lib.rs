//! Kirin Commander in a browser: the commander inside a WebAssembly
//! module, its file system the page's drives (docs/06-file-systems.md 6.5).
//!
//! The page calls in with what the person did and asks for the frame:
//!
//! * `kc_start(w, h)` - the commander, on a screen of w by h cells;
//! * `kc_key(kind, value, mods)` and `kc_mouse(x, y, kind, button)` - an event;
//! * `kc_tick()` - a few steps of a running job, and the clock;
//! * `kc_resize(w, h)`, `kc_ran(len)` - after the page ran a command line;
//! * `kc_frame()` - the screen drawn, its length; `kc_out()` where it is.
//!
//! The file system is the page's: every `Fs` call is one call of the
//! imported `host(op, ptr, len)`, which answers with the reply's length,
//! and `host_take(ptr)`, which copies the reply in. A browser's storage
//! answers later - every call there is a promise - and the commander's
//! `Fs` returns when it is done; the module is built with Binaryen's
//! asyncify (`build.cmd`), so `host` may put the whole module to sleep
//! until the promise is kept, and the commander goes on where it stopped,
//! none the wiser. The page calls in one call at a time (kc.js).
//!
//! Only for WebAssembly: on any other target the crate is empty, so the
//! workspace's `cargo test` passes over it.
#![cfg(target_arch = "wasm32")]

use kc::app::Request;
use kc::fs::*;
use kc::App;
use owlosui_console::codepage::Font;
use owlosui_core::{Buffer, Button, Event, Glyph, Key, KeyCode, Mods, Mouse, MouseKind};
use std::cell::RefCell;

#[link(wasm_import_module = "kc")]
extern "C" {
    /// One `Fs` call: `op`, its arguments at `ptr`; the reply's length.
    fn host(op: u32, ptr: *const u8, len: u32) -> u32;
    /// The reply, copied to `ptr`.
    fn host_take(ptr: *mut u8);
    /// What a panic said, for the page's console - the module stops after it.
    fn panicked(ptr: *const u8, len: u32);
}

// The calls, as kc.js numbers them.
const DRIVES: u32 = 1;
const CURRENT: u32 = 2;
const LIST: u32 = 3;
const STAT: u32 = 4;
const FREE: u32 = 5;
const MKDIR: u32 = 6;
const RMFILE: u32 = 7;
const RMDIR: u32 = 8;
const RENAME: u32 = 9;
const ATTRS: u32 = 10;
const OPEN: u32 = 11;
const CREATE: u32 = 12;
const READ: u32 = 13;
const WRITE: u32 = 14;
const CLOSE: u32 = 15;
const SPACE: u32 = 16;
const CLOCK: u32 = 17;
const STAMP: u32 = 18;

/// The separators of a reply: a field, and a record.
const FIELD: char = '\u{1F}';
const RECORD: char = '\u{1E}';

/// A call to the page. The reply's first byte is 0 for done, else 1, the
/// kind of error, and the system's own words after it.
fn call(op: u32, args: &[u8]) -> Result<Vec<u8>, FsError> {
    let len = unsafe { host(op, args.as_ptr(), args.len() as u32) } as usize;
    let mut out = vec![0u8; len];
    if len > 0 {
        unsafe { host_take(out.as_mut_ptr()) };
    }
    match out.first() {
        Some(0) => {
            out.remove(0);
            Ok(out)
        }
        Some(1) => Err(FsError::new(kind_of(out.get(1).copied().unwrap_or(0)))),
        _ => Err(FsError::new(ErrorKind::Io)),
    }
}

fn text_call(op: u32, args: &[&str]) -> Result<String, FsError> {
    let joined = args.join("\0");
    call(op, joined.as_bytes()).map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// The kinds by kc.js's numbers: the order of `ErrorKind`.
fn kind_of(n: u8) -> ErrorKind {
    use ErrorKind::*;
    const ALL: [ErrorKind; 15] = [NotFound, Exists, Denied, ReadOnly, NotEmpty, DiskFull, NotReady, WriteProtected, InUse, BadName, CrossDevice, IntoItself, Unsupported, Io, Io];
    ALL.get(n as usize).copied().unwrap_or(Io)
}

fn entry_of(rec: &str) -> Option<Entry> {
    let f: Vec<&str> = rec.split(FIELD).collect();
    if f.len() < 5 {
        return None;
    }
    let t: Vec<u16> = f[3].split(',').map(|n| n.parse().unwrap_or(0)).collect();
    let stamp = if t.len() == 6 {
        Stamp { year: t[0], month: t[1] as u8, day: t[2] as u8, hour: t[3] as u8, minute: t[4] as u8, second: t[5] as u8 }
    } else {
        Stamp::default()
    };
    Some(Entry {
        name: String::from(f[0]),
        dir: f[1] == "1",
        size: f[2].parse().unwrap_or(0),
        stamp,
        attrs: Attrs::from_dos(f[4].parse().unwrap_or(0)),
    })
}

/// The page's drives, behind the commander's `Fs`.
struct WebFs;

impl Fs for WebFs {
    fn sep(&self) -> char {
        '\\'
    }
    fn short_names(&self) -> bool {
        false
    }
    fn keeps_attrs(&self) -> bool {
        false
    }
    fn case_sensitive(&self) -> bool {
        // The site spells DOS names case-blind over its storage (6.2).
        false
    }

    fn drives(&mut self) -> Vec<Drive> {
        let Ok(s) = text_call(DRIVES, &[]) else { return Vec::new() };
        s.split(RECORD)
            .filter(|r| !r.is_empty())
            .filter_map(|r| {
                let f: Vec<&str> = r.split(FIELD).collect();
                let letter = f.first()?.chars().next()?;
                Some(Drive { root: format!("{}:\\", letter), letter, what: String::from(*f.get(1).unwrap_or(&"")) })
            })
            .collect()
    }
    fn current_dir(&mut self) -> String {
        text_call(CURRENT, &[]).unwrap_or_else(|_| String::from("A:\\"))
    }
    fn read_dir(&mut self, dir: &str) -> FsResult<Vec<Entry>> {
        let s = text_call(LIST, &[dir])?;
        Ok(s.split(RECORD).filter(|r| !r.is_empty()).filter_map(entry_of).collect())
    }
    fn stat(&mut self, path: &str) -> FsResult<Entry> {
        let s = text_call(STAT, &[path])?;
        entry_of(&s).ok_or(FsError::new(ErrorKind::Io))
    }
    fn free_space(&mut self, path: &str) -> Option<u64> {
        text_call(FREE, &[path]).ok()?.parse().ok()
    }
    fn disk_space(&mut self, root: &str) -> Option<(u64, u64)> {
        let s = text_call(SPACE, &[root]).ok()?;
        let (a, b) = s.split_once(',')?;
        Some((a.parse().ok()?, b.parse().ok()?))
    }

    fn make_dir(&mut self, path: &str) -> FsResult<()> {
        text_call(MKDIR, &[path]).map(|_| ())
    }
    fn remove_file(&mut self, path: &str) -> FsResult<()> {
        text_call(RMFILE, &[path]).map(|_| ())
    }
    fn remove_dir(&mut self, path: &str) -> FsResult<()> {
        text_call(RMDIR, &[path]).map(|_| ())
    }
    fn rename(&mut self, from: &str, to: &str) -> FsResult<()> {
        text_call(RENAME, &[from, to]).map(|_| ())
    }
    fn set_attrs(&mut self, path: &str, attrs: Attrs) -> FsResult<()> {
        let a = format!("{}", attrs.to_dos());
        text_call(ATTRS, &[path, &a]).map(|_| ())
    }
    fn set_stamp(&mut self, path: &str, s: Stamp) -> FsResult<()> {
        let t = format!("{},{},{},{},{},{}", s.year, s.month, s.day, s.hour, s.minute, s.second);
        text_call(STAMP, &[path, &t]).map(|_| ())
    }

    fn open_read(&mut self, path: &str) -> FsResult<Handle> {
        text_call(OPEN, &[path])?.parse().map_err(|_| FsError::new(ErrorKind::Io))
    }
    fn create(&mut self, path: &str) -> FsResult<Handle> {
        text_call(CREATE, &[path])?.parse().map_err(|_| FsError::new(ErrorKind::Io))
    }
    fn read(&mut self, h: Handle, buf: &mut [u8]) -> FsResult<usize> {
        let mut args = h.to_le_bytes().to_vec();
        args.extend_from_slice(&(buf.len() as u32).to_le_bytes());
        let got = call(READ, &args)?;
        let n = got.len().min(buf.len());
        buf[..n].copy_from_slice(&got[..n]);
        Ok(n)
    }
    fn write(&mut self, h: Handle, buf: &[u8]) -> FsResult<()> {
        let mut args = h.to_le_bytes().to_vec();
        args.extend_from_slice(buf);
        call(WRITE, &args).map(|_| ())
    }
    fn close(&mut self, h: Handle) -> FsResult<()> {
        call(CLOSE, &h.to_le_bytes()).map(|_| ())
    }
}

// ------------------------------------------------------------ the font

thread_local! {
    /// Glyph index to character: CP437 first - the frames, the shades,
    /// the arrows are drawn by those numbers - then every new character
    /// the next index. The page draws a frame through this table.
    static FONT: RefCell<Font> = RefCell::new(Font::growing(437).expect("437"));
}

fn glyph(c: char) -> Glyph {
    FONT.with(|f| f.borrow_mut().from_char(c))
}

/// A file in the editor: UTF-8 when it is, else the bytes as CP437 - the
/// files of a DOS disk.
struct WebCodec {
    utf8: std::cell::Cell<bool>,
}

impl kc::editor::Codec for WebCodec {
    fn exact(&self, bytes: &[u8]) -> bool {
        self.utf8.set(std::str::from_utf8(bytes).is_ok());
        true
    }
    fn decode(&self, line: &[u8]) -> Vec<Glyph> {
        if self.utf8.get() {
            String::from_utf8_lossy(line).chars().map(glyph).collect()
        } else {
            line.iter().map(|&b| b as Glyph).collect()
        }
    }
    fn encode(&self, line: &[Glyph]) -> Vec<u8> {
        if self.utf8.get() {
            FONT.with(|f| {
                let f = f.borrow();
                line.iter().map(|&g| f.to_char(g)).collect::<String>().into_bytes()
            })
        } else {
            // A glyph past the code page has no byte: `?`, as DOS would.
            line.iter().map(|&g| if g < 256 { g as u8 } else { b'?' }).collect()
        }
    }
}

// ------------------------------------------------------------ the module

struct Module {
    app: App,
    buf: Buffer,
    /// The last frame: as OWLOSUI's FRAME reply, so the page reads it
    /// with OWLOSUI's own `Frame` - w h cursor-x cursor-y (i16), hold,
    /// the font's length (u16), then glyph (u16) and attribute a cell;
    /// after the cells, the requests, as text.
    out: Vec<u8>,
}

thread_local! {
    static MODULE: RefCell<Option<Module>> = const { RefCell::new(None) };
    static ROOM: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn with<R>(f: impl FnOnce(&mut Module) -> R) -> Option<R> {
    MODULE.with(|m| m.borrow_mut().as_mut().map(f))
}

fn clock() -> Option<(u8, u8)> {
    let s = text_call(CLOCK, &[]).ok()?;
    let (h, m) = s.split_once(',')?;
    Some((h.parse().ok()?, m.parse().ok()?))
}

/// The commander, on a screen of `w` by `h` cells, its panels where the
/// page's `current` says.
#[no_mangle]
pub extern "C" fn kc_start(w: i32, h: i32) {
    std::panic::set_hook(Box::new(|info| {
        let s = info.to_string();
        unsafe { panicked(s.as_ptr(), s.len() as u32) };
    }));
    let mut app = App::new(Box::new(WebFs), w as i16, h as i16);
    app.system = vec![String::from("A browser: the drives are its own storage")];
    app.codec = || Box::new(WebCodec { utf8: std::cell::Cell::new(true) });
    app.clock = clock();
    // Save setup: kc.js keeps this name in the browser's localStorage.
    app.setup_path = Some(String::from("KC:\\KC.INI"));
    app.load_setup();
    let buf = Buffer::new(w as i16, h as i16);
    MODULE.with(|m| *m.borrow_mut() = Some(Module { app, buf, out: Vec::new() }));
}

/// A key: `kind` 0 a character (`value` its code), 1 F`value`, 2.. the
/// named keys in KeyCode's order; `mods` bit 0 Ctrl, 1 Alt, 2 Shift.
#[no_mangle]
pub extern "C" fn kc_key(kind: u32, value: u32, mods: u32) {
    let code = match kind {
        0 => match char::from_u32(value) {
            // In the editor a character is the glyph it is drawn with: the
            // text holds glyphs, as on DOS it holds bytes.
            Some(c) => KeyCode::Char(c),
            None => return,
        },
        1 => KeyCode::F(value as u8),
        2 => KeyCode::Enter,
        3 => KeyCode::Esc,
        4 => KeyCode::Tab,
        5 => KeyCode::BackTab,
        6 => KeyCode::Backspace,
        7 => KeyCode::Delete,
        8 => KeyCode::Insert,
        9 => KeyCode::Home,
        10 => KeyCode::End,
        11 => KeyCode::PageUp,
        12 => KeyCode::PageDown,
        13 => KeyCode::Up,
        14 => KeyCode::Down,
        15 => KeyCode::Left,
        16 => KeyCode::Right,
        _ => return,
    };
    let mods = Mods { ctrl: mods & 1 != 0, alt: mods & 2 != 0, shift: mods & 4 != 0 };
    with(|m| {
        let code = match code {
            KeyCode::Char(c) if m.app.editor.is_some() && !mods.ctrl && !mods.alt => KeyCode::Char(char::from_u32(glyph(c) as u32).unwrap_or('?')),
            c => c,
        };
        m.app.handle(Event::Key(Key { code, mods }));
    });
}

/// The modifiers held down, for the key bar's other rows.
#[no_mangle]
pub extern "C" fn kc_held(mods: u32) {
    with(|m| m.app.held = Mods { ctrl: mods & 1 != 0, alt: mods & 2 != 0, shift: mods & 4 != 0 });
}

/// The mouse: `kind` 0 down, 1 up, 2 double, 3 drag, 4 move, 5 wheel up,
/// 6 wheel down; `button` 0 left, 1 right, 2 middle.
#[no_mangle]
pub extern "C" fn kc_mouse(x: i32, y: i32, kind: u32, button: u32) {
    let b = match button {
        1 => Button::Right,
        2 => Button::Middle,
        _ => Button::Left,
    };
    let kind = match kind {
        0 => MouseKind::Down(b),
        1 => MouseKind::Up(b),
        2 => MouseKind::Double(b),
        3 => MouseKind::Drag,
        4 => MouseKind::Move,
        5 => MouseKind::ScrollUp,
        _ => MouseKind::ScrollDown,
    };
    with(|m| m.app.handle(Event::Mouse(Mouse { x: x as i16, y: y as i16, kind })));
}

/// A few steps of a running job, and the clock. Whether a job still runs.
#[no_mangle]
pub extern "C" fn kc_tick() -> u32 {
    let clock = clock();
    with(|m| {
        if clock.is_some() {
            m.app.clock = clock;
        }
        if m.app.busy() {
            m.app.tick();
        }
        m.app.busy() as u32
    })
    .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn kc_resize(w: i32, h: i32) {
    with(|m| {
        m.buf.resize(w as i16, h as i16);
        m.app.handle(Event::Resize(w as i16, h as i16));
    });
}

/// Room for `len` bytes the page writes in: what a command line printed.
#[no_mangle]
pub extern "C" fn kc_room(len: u32) -> *mut u8 {
    ROOM.with(|r| {
        let mut r = r.borrow_mut();
        r.clear();
        r.resize(len as usize, 0);
        r.as_mut_ptr()
    })
}

/// A command line ran: what it printed (in `kc_room`, UTF-8) to the
/// terminal behind the panels, and both panels read again.
#[no_mangle]
pub extern "C" fn kc_ran(len: u32) {
    let text = ROOM.with(|r| String::from_utf8_lossy(&r.borrow()[..len as usize]).into_owned());
    with(|m| {
        if !text.is_empty() {
            m.app.print(text.trim_end_matches('\n'));
        }
        m.app.ran();
    });
}

/// The screen drawn: its length at `kc_out`. After the cells, a line a
/// request - `run\x1Fdir\x1Fline`, `help`, `quit` - for the page to do.
#[no_mangle]
pub extern "C" fn kc_frame() -> u32 {
    with(|m| {
        m.app.draw(&mut m.buf, &glyph);
        let (w, h) = (m.buf.width(), m.buf.height());
        let (cx, cy) = m.app.cursor().unwrap_or((-1, -1));
        let font = FONT.with(|f| f.borrow().len()) as u16;
        let out = &mut m.out;
        out.clear();
        for v in [w, h, cx, cy] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.push(0);
        out.extend_from_slice(&font.to_le_bytes());
        for y in 0..h {
            for x in 0..w {
                let c = m.buf.get(x, y).unwrap_or(owlosui_core::Cell { ch: 32, attr: 7 });
                out.extend_from_slice(&(c.ch as u16).to_le_bytes());
                out.push(c.attr);
            }
        }
        for r in m.app.take_requests() {
            let line = match r {
                Request::Run { dir, line } => format!("run{}{}{}{}", FIELD, dir, FIELD, line),
                Request::Quit => String::from("quit"),
            };
            out.extend_from_slice(line.as_bytes());
            out.push(b'\n');
        }
        out.len() as u32
    })
    .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn kc_out() -> *const u8 {
    with(|m| m.out.as_ptr()).unwrap_or(core::ptr::null())
}

/// The font: every glyph's character, UTF-8, from `first` on - the page
/// asks again only when a frame says the font has grown.
#[no_mangle]
pub extern "C" fn kc_glyphs(first: u32) -> u32 {
    let s: String = FONT.with(|f| {
        let f = f.borrow();
        (first as usize..f.len()).map(|g| {
            let c = f.to_char(g as Glyph);
            if c == '\0' { ' ' } else { c }
        }).collect()
    });
    ROOM.with(|r| {
        let mut r = r.borrow_mut();
        r.clear();
        r.extend_from_slice(s.as_bytes());
    });
    s.len() as u32
}

#[no_mangle]
pub extern "C" fn kc_room_ptr() -> *const u8 {
    ROOM.with(|r| r.borrow().as_ptr())
}

/// Room for asyncify's saved stack: the page asks for it once.
#[no_mangle]
pub extern "C" fn kc_alloc(len: u32) -> *mut u8 {
    let mut v = vec![0u8; len as usize];
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}
