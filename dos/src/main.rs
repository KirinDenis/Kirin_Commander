//! Kirin Commander on DOS: a flat 32-bit binary under DPMI, drawing into
//! B800, its file system INT 21h. KC.COM (KC.ASM, OWLOSUI's LOADER.INC)
//! loads KC.BIN into extended memory and jumps to `_start`.
//!
//! A command line runs through COMMAND.COM, in real mode, with the panels
//! out of the way; what it left on the screen is kept, and Ctrl+O shows
//! it again - the terminal behind the panels (docs/07-command-line.md).

#![no_std]
#![no_main]

extern crate alloc;

mod dosfs;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use kc::app::Request;
use kc::{App, Fs};
use owlosui_core::{Buffer, Event, KeyCode};
use owlosui_dos::*;

/// Where the loader jumps: the block's linear address, the flags (/SHOT
/// bit 0, /QUIT bit 1), and the PSP's segment.
#[no_mangle]
#[link_section = ".text.start"]
pub extern "C" fn _start(base: u32, flags: u32, psp: u32) -> ! {
    init(base);
    // The screen as DOS left it, before anything here can touch it: after a
    // step, what the program printed.
    let left_on_screen = screen_lines();
    let Some(fs) = dosfs::DosFs::new() else { dos_exit(2) };
    let comspec = comspec(psp as u16).unwrap_or_else(|| String::from("C:\\COMMAND.COM"));
    let mut runner = dosfs::DosFs::new().unwrap_or_else(|| dos_exit(2));
    // Where DOS was when the commander started: /STEP leaves RUNPROG.BAT
    // and KC.SAV here (kc/src/step.rs).
    let home = runner.current_dir();
    let (step, dirs) = arguments(psp as u16);
    let mut app = App::new(Box::new(fs), 80, 25);
    app.clock = Some(clock());
    app.system = system_lines();
    // The setup beside KC.COM, as NC kept NC.INI beside NC.EXE; /SHOT
    // reads none, so a test sees the defaults.
    app.setup_path = program_path(psp as u16).and_then(|p| p.rfind('\\').map(|i| alloc::format!("{}KC.INI", &p[..=i])));
    if flags & 1 == 0 {
        app.load_setup();
    }
    // Started again after a step: where it was, and what the program left
    // on the screen is the terminal behind the panels.
    if step && app.resume(&home) {
        app.terminal = left_on_screen;
    } else if !dirs.is_empty() {
        app.start_in(&dirs);
    }
    let mut buf = Buffer::new(80, 25);
    let mut mouse = MouseState::detect();
    let mut queue: VecDeque<Event> = VecDeque::new();
    let mut last_tick = ticks();
    loop {
        app.held = held();
        app.draw(&mut buf, &kc::screen::cp437);
        let bytes = frame_bytes(&buf, mouse.pointer());
        show(&bytes);
        cursor(app.cursor());
        if flags & 1 != 0 {
            shot(&bytes);
            leave();
        }
        if flags & 2 != 0 {
            leave();
        }
        let mut requests = app.take_requests();
        if step {
            let runs = App::take_runs(&mut requests);
            if !runs.is_empty() {
                if app.step_out(&home, &runs) {
                    step_out(&mut runner, &runs[0].0, &runs[0].1);
                }
                // Not written - a full disk, a write-protected one: run here.
                requests.extend(runs.into_iter().map(|(dir, line)| Request::Run { dir, line }));
            }
        }
        for r in requests {
            match r {
                Request::Quit => leave(),
                Request::Run { dir, line } => {
                    let screen = run(&mut runner, &comspec, &dir, &line);
                    app.terminal = screen;
                    app.ran();
                }
            }
        }
        // The keys, the mouse, the clock: nothing waits for a key while a
        // job runs, and the clock moves once a second.
        if app.busy() {
            app.tick();
            poll(&mut mouse, &mut queue, &bytes);
        } else {
            while queue.is_empty() {
                poll(&mut mouse, &mut queue, &bytes);
                // Shift, Ctrl or Alt pressed or let go: the key bar's other row.
                if held() != app.held {
                    break;
                }
                if ticks().wrapping_sub(last_tick) >= 18 {
                    break;
                }
                core::hint::spin_loop();
            }
        }
        if ticks().wrapping_sub(last_tick) >= 18 {
            last_tick = ticks();
            app.clock = Some(clock());
        }
        while let Some(ev) = queue.pop_front() {
            app.handle(ev);
        }
    }
}

fn leave() -> ! {
    clear_screen();
    bios_cursor_home();
    dos_exit(0)
}

/// What is waiting: a key from the BIOS, the mouse. F12 is the test's
/// key, as in OWLOSUI: the frame into SHOT.BIN and SHOT.TXT.
fn poll(mouse: &mut MouseState, out: &mut VecDeque<Event>, frame: &[u8]) {
    while let Some((ascii, scan)) = bios_key() {
        if let Some(k) = decode(ascii, scan) {
            if k.code == KeyCode::F(12) && !k.mods.alt && !k.mods.ctrl {
                shot(frame);
                continue;
            }
            out.push_back(Event::Key(k));
        }
    }
    mouse.poll(out);
}

/// What System Info says of the machine: the DOS version (30h), and the
/// largest block of conventional memory left (48h asked for too much).
fn system_lines() -> Vec<String> {
    let mut r = RealRegs { eax: 0x3000, ..Default::default() };
    real_int(0x21, &mut r);
    let (major, minor) = (r.eax & 0xFF, (r.eax >> 8) & 0xFF);
    let mut m = RealRegs { eax: 0x4800, ebx: 0xFFFF, ..Default::default() };
    real_int(0x21, &mut m);
    let free = (m.ebx & 0xFFFF) as u64 * 16; // paragraphs * 16 bytes
    alloc::vec![alloc::format!("DOS {}.{:02}", major, minor), alloc::format!("Conventional memory free: {}", kc::names::human(free))]
}

/// The time of day, from DOS.
fn clock() -> (u8, u8) {
    let mut r = RealRegs { eax: 0x2C00, ..Default::default() };
    real_int(0x21, &mut r);
    ((r.ecx >> 8) as u8, r.ecx as u8)
}

/// The program's environment block. The PSP's word at 2Ch names it - a
/// segment in real mode, but once a DPMI host has switched the program to
/// protected mode it is a selector there (the DPMI 0.9 rule; CWSDPMI
/// does it), so its base is asked of DPMI (0006h). A word DPMI does not
/// know as a selector is taken as the segment it was.
fn env_block(psp: u16) -> Option<Vec<u8>> {
    let mut w = [0u8; 2];
    read_real(real(psp, 0x2C), &mut w);
    let env = u16::from_le_bytes(w);
    if env == 0 {
        return None;
    }
    let base = selector_base(env).unwrap_or_else(|| real(env, 0));
    let mut block = alloc::vec![0u8; 4096];
    read_real(base, &mut block);
    Some(block)
}

/// A selector's linear base address, through DPMI 0006h (CX:DX), or None
/// if DPMI says it is no selector.
fn selector_base(sel: u16) -> Option<u32> {
    let hi: u16;
    let lo: u16;
    let failed: u16;
    unsafe {
        core::arch::asm!(
            "int 0x31",
            "setc {f:l}",
            "movzx {f:e}, {f:l}",
            inout("ax") 0x0006u16 => _,
            in("bx") sel,
            out("cx") hi,
            out("dx") lo,
            f = lateout(reg_abcd) failed,
            options(nostack),
        );
    }
    if failed != 0 {
        None
    } else {
        Some((hi as u32) << 16 | lo as u32) // CX the high word, DX the low
    }
}

/// COMSPEC from the program's environment.
fn comspec(psp: u16) -> Option<String> {
    let block = env_block(psp)?;
    for var in block.split(|&b| b == 0) {
        if var.is_empty() {
            break;
        }
        if var.len() > 8 && var[..8].eq_ignore_ascii_case(b"COMSPEC=") {
            return Some(var[8..].iter().map(|&b| b as char).collect());
        }
    }
    None
}

/// The program's own full path, `C:\KC\KC.COM`: DOS 3 and later put it in
/// the environment block, after the variables' closing empty string and a
/// word that counts the strings that follow.
fn program_path(psp: u16) -> Option<String> {
    let block = env_block(psp)?;
    // The empty string that closes the variables: at once, for an empty
    // environment, or where a NUL follows the last variable's NUL.
    let close = if block[0] == 0 { 0 } else { block.windows(2).position(|w| w == [0, 0])? + 1 };
    let path = &block[close + 3..]; // past that NUL and the count word
    let len = path.iter().position(|&b| b == 0)?;
    if len == 0 {
        return None;
    }
    Some(path[..len].iter().map(|&b| b as char).collect())
}

/// A command line through COMMAND.COM, on the DOS screen; what it left
/// there comes back as the terminal's lines.
fn run(fs: &mut dosfs::DosFs, comspec: &str, dir: &str, line: &str) -> Vec<String> {
    prompt(fs, dir, line);
    if let Err(e) = fs.exec(comspec, dir, line) {
        print(fs, &alloc::format!("{}.\r\n$", e.kind.text()));
    }
    screen_lines()
}

/// Out of memory for RUNPROG.BAT, with the screen as `run` leaves it for
/// the line - the program writes under the prompt, and the next start
/// reads it back (screen_lines).
fn step_out(fs: &mut dosfs::DosFs, dir: &str, line: &str) -> ! {
    prompt(fs, dir, line);
    dos_exit(0)
}

/// The screen clear and `C:\WORK>line` on its first row, the cursor under
/// it. DOS writes where the BIOS cursor is, not the CRT's: both go home
/// first, or the prompt lands wherever DOS last wrote.
fn prompt(fs: &mut dosfs::DosFs, dir: &str, line: &str) {
    clear_screen();
    bios_cursor_home();
    print(fs, &alloc::format!("{}>{}\r\n$", dir, line));
    cursor(Some((0, 1)));
}

/// The BIOS cursor on the top left (INT 10h, 02h, page 0).
fn bios_cursor_home() {
    let mut c = RealRegs { eax: 0x0200, ebx: 0, edx: 0, ..Default::default() };
    real_int(0x10, &mut c);
}

/// The command tail: `/STEP`, and the panels' folders (`KC /STEP A:\ C:\`).
/// The loader's own words, /SHOT and the like, are passed over.
fn arguments(psp: u16) -> (bool, Vec<String>) {
    let mut tail = [0u8; 128];
    read_real((psp as u32) << 4 | 0x80, &mut tail);
    let len = (tail[0] as usize).min(127);
    let text: String = tail[1..=len].iter().map(|&b| b as char).collect();
    let mut step = false;
    let mut dirs = Vec::new();
    for w in text.split_whitespace() {
        if w.eq_ignore_ascii_case("/STEP") {
            step = true;
        } else if !w.starts_with('/') {
            dirs.push(w.to_ascii_uppercase());
        }
    }
    (step, dirs)
}

/// The text screen as it is now, line by line, the empty lines at its end
/// left out.
fn screen_lines() -> Vec<String> {
    let mut video = alloc::vec![0u8; 80 * 25 * 2];
    read_real(0xB8000, &mut video);
    // A cleared cell holds 0, not a blank (clear_screen): both are nothing
    // to keep, or a screen of empty rows hides the lines above them.
    let cell = |b: u8| if b == 0 { ' ' } else { b as char };
    let mut lines: Vec<String> = (0..25).map(|y| (0..80).map(|x| cell(video[(y * 80 + x) * 2])).collect::<String>().trim_end().into()).collect();
    while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }
    lines
}

/// Text through DOS (09h, `$`-ended).
fn print(fs: &mut dosfs::DosFs, s: &str) {
    let seg = fs.segment();
    let b: Vec<u8> = s.chars().map(|c| c as u32 as u8).collect();
    write_real(real(seg, 0x400), &b);
    let mut r = RealRegs { eax: 0x0900, edx: 0x400, ds: seg, ..Default::default() };
    real_int(0x21, &mut r);
}
