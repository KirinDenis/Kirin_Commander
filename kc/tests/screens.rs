//! The program's screens against the documents' screens, cell for cell.
//!
//! docs/ draws the commander as text; these tests put the commander in the
//! same state and compare what it draws with what the documents show. A
//! change to either that makes them disagree fails here - so the
//! documents stay the program's picture, and the program stays the
//! documents'.

use kc::screen::cp437;
use kc::{App, Entry, MemFs, Stamp};
use owlosui_core::{Buffer, Event, Key, KeyCode, Mods};

fn stamp(d: u8, m: u8, y: u16, h: u8, mi: u8) -> Stamp {
    Stamp { year: y, month: m, day: d, hour: h, minute: mi, second: 0 }
}

/// C:\LESSONS as docs/01-screen.md has it.
fn lessons() -> MemFs {
    let mut fs = MemFs::new(&['A', 'B', 'C', 'D', 'E']);
    let lesson_day = stamp(24, 10, 2026, 15, 34);
    fs.add_dir("C:\\LESSONS");
    fs.set_stamp_of("C:\\LESSONS", stamp(28, 10, 2026, 9, 12));
    for i in 1..=31 {
        let p = format!("C:\\LESSONS\\L{:02}", i);
        fs.add_dir(&p);
        fs.set_stamp_of(&p, lesson_day);
    }
    let files: [(&str, usize, Stamp); 6] = [
        ("COURSE.TXT", 6211, lesson_day),
        ("GO.BAT", 38, lesson_day),
        ("LESSONS.INI", 2048, lesson_day),
        ("MAKE.BAT", 296, lesson_day),
        ("README.TXT", 3876, stamp(21, 10, 2026, 9, 40)),
        ("TOOLS.TXT", 911, stamp(21, 10, 2026, 9, 40)),
    ];
    for (n, size, s) in files {
        let p = format!("C:\\LESSONS\\{}", n);
        fs.add_file(&p, &vec![b'x'; size]);
        fs.set_stamp_of(&p, s);
    }
    fs.set_stamp_of("C:\\LESSONS\\L07", stamp(28, 10, 2026, 9, 12));
    let l07: [(&str, usize, Stamp); 5] = [
        ("BUILD.BAT", 156, lesson_day),
        ("BUILD.TXT", 211, stamp(28, 10, 2026, 9, 15)),
        ("GO.BAT", 38, lesson_day),
        ("HELLO.ASM", 2345, stamp(28, 10, 2026, 9, 14)),
        ("HELLO.COM", 412, stamp(28, 10, 2026, 9, 15)),
    ];
    for (n, size, s) in l07 {
        let p = format!("C:\\LESSONS\\L07\\{}", n);
        fs.add_file(&p, &vec![b'x'; size]);
        fs.set_stamp_of(&p, s);
    }
    fs.set_cwd("C:\\LESSONS\\L07");
    // The drive menu's numbers: two floppy drives, three disks.
    fs.set_removable('A');
    fs.set_removable('B');
    fs.set_capacity('C', 2 << 30);
    fs.set_capacity('D', 512 << 20);
    fs.set_capacity('E', 4 << 20);
    fs.add_file("E:\\DISK.IMG", &vec![0; 1 << 20]);
    fs
}

fn key(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods::default() })
}

fn with(code: KeyCode, mods: Mods) -> Event {
    Event::Key(Key { code, mods })
}

/// The first ```text block after `marker` in a document.
fn screen_in(doc: &str, marker: &str) -> Vec<String> {
    let path = format!("{}/../docs/{}", env!("CARGO_MANIFEST_DIR"), doc);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {}", path, e));
    let from = text.find(marker).unwrap_or_else(|| panic!("{} not in {}", marker, doc));
    let rest = &text[from..];
    let start = rest.find("```text\n").expect("a text block") + 8;
    let end = rest[start..].find("```").unwrap() + start;
    rest[start..end].lines().map(|l| l.to_string()).collect()
}

/// The buffer against a screen: every cell's glyph. The caret is `_` in
/// the documents, a blank cell on the screen.
fn compare(buf: &Buffer, want: &[String], what: &str) {
    let mut bad = Vec::new();
    for (y, line) in want.iter().enumerate() {
        let got: String = (0..buf.width()).map(|x| buf.get(x, y as i16).unwrap().ch).map(back).collect();
        let line: String = line.chars().collect();
        let mut wanted = String::new();
        for (x, c) in line.chars().enumerate() {
            let g = cp437(c);
            let gc = buf.get(x as i16, y as i16).map(|c| c.ch);
            if c == '_' && gc == Some(b' ' as _) {
                wanted.push(' ');
            } else {
                wanted.push(back(g));
            }
        }
        if got.trim_end() != wanted.trim_end() {
            bad.push(format!("line {:2}\n want {}\n  got {}", y + 1, wanted, got));
        }
    }
    assert!(bad.is_empty(), "{} differs from the document:\n{}", what, bad.join("\n"));
}

/// A glyph back to a char, for the message: the box characters by table.
fn back(g: owlosui_core::Glyph) -> char {
    for c in "►◄▲▼░▒▓│┤╢╗╝┐└┴┬├─┼╟╚╔╧╤═║┘┌█▄▀√·•".chars() {
        if cp437(c) == g {
            return c;
        }
    }
    char::from_u32(g as u32).unwrap_or('?')
}

/// The main screen's state: the left panel came out of L07, two files
/// selected; the right panel in L07 on hello.asm.
fn main_screen() -> App {
    main_screen_on(lessons())
}

fn main_screen_on(fs: MemFs) -> App {
    let mut app = App::new(Box::new(fs), 80, 25);
    app.clock = Some((12, 34));
    // The left panel: out of L07 with Enter on `..` - rule 3 puts the
    // cursor on L07.
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.panels[0].current().unwrap().name, "L07");
    // The right panel: Full, on hello.asm.
    app.handle(key(KeyCode::Tab));
    app.handle(with(KeyCode::Char('3'), Mods::ctrl()));
    for _ in 0..4 {
        app.handle(key(KeyCode::Down));
    }
    app.handle(key(KeyCode::Tab));
    let i = app.panels[0].index_of("GO.BAT").unwrap();
    app.panels[0].set_selected(i, true);
    let i = app.panels[0].index_of("README.TXT").unwrap();
    app.panels[0].set_selected(i, true);
    app
}

#[test]
fn the_main_screen_is_the_documents() {
    let mut app = main_screen();
    let mut buf = Buffer::new(80, 25);
    app.draw(&mut buf, &cp437);
    compare(&buf, &screen_in("01-screen.md", "## 1.1"), "01-screen.md, the main screen");
    // What the lines under it say is lit.
    let attr = |x: i16, y: i16| buf.get(x, y).unwrap().attr;
    assert_eq!(attr(14, 0), 0x30, "the active panel's title");
    assert_eq!(attr(1, 9), 0x30, "the cursor bar on L07");
    assert_eq!(attr(27, 2), 0x1E, "readme.txt selected");
    assert_eq!(attr(54, 6), 0x1B, "no bar in the passive panel");
    assert_eq!(attr(1, 3), 0x1F, "a folder, white");
    assert_eq!(attr(41, 3), 0x1A, "build.bat, a program, light green");
    assert_eq!(attr(41, 6), 0x1B, "hello.asm, plain, light cyan");
    assert_eq!(app.cursor(), Some((11, 23)), "the caret after the prompt");
}

#[test]
fn the_drive_menu_is_the_documents() {
    let mut app = main_screen();
    app.handle(with(KeyCode::F(1), Mods::alt()));
    let mut buf = Buffer::new(80, 25);
    app.draw(&mut buf, &cp437);
    compare(&buf, &screen_in("02-panels.md", "## 2.8"), "02-panels.md, the drive menu");
    assert_eq!(buf.get(6, 12).unwrap().attr, 0x30, "C, the panel's drive, under the cursor: {:?}", (9..16).map(|y| buf.get(6, y).unwrap().attr).collect::<Vec<_>>());
    // A letter chooses at once.
    app.handle(key(KeyCode::Char('d')));
    assert!(app.drive_menu.is_none());
    assert_eq!(app.panels[0].dir, "D:\\");
}

#[test]
fn a_drive_menu_longer_than_the_panel_scrolls() {
    let letters: Vec<char> = ('A'..='Z').collect();
    let mut app = App::new(Box::new(MemFs::new(&letters)), 80, 25);
    app.handle(with(KeyCode::F(1), Mods::alt()));
    app.handle(key(KeyCode::End));
    let mut buf = Buffer::new(80, 25);
    app.draw(&mut buf, &cp437);
    let screen: Vec<String> = (0..25).map(|y| (0..80).map(|x| char::from_u32(buf.get(x, y).unwrap().ch as u32).unwrap()).collect()).collect();
    let has = |s: &str| screen.iter().any(|l| l.contains(s));
    assert!(has(" Z:  memory"), "the cursor's drive is in view\n{}", screen.join("\n"));
    assert!(!has(" A:  memory"), "the first drives scrolled away\n{}", screen.join("\n"));
    assert!(buf.get(0, 23).unwrap().ch != b'\xC9' as _, "the box stays above the command line");
}

#[test]
fn the_terminal_is_behind_the_panels() {
    let mut app = main_screen();
    app.handle(with(KeyCode::Char('o'), Mods::ctrl()));
    let mut buf = Buffer::new(80, 25);
    app.draw(&mut buf, &cp437);
    // No panel on the screen; the command line and the key bar stay.
    let row = |y: i16| -> String { (0..80).map(|x| back(buf.get(x, y).unwrap().ch)).collect() };
    assert!(row(0).trim().is_empty());
    assert!(row(23).starts_with("C:\\LESSONS>"));
    assert!(row(24).starts_with("F1 Help F2 Menu"), "OWLOSUI's status line");
    app.handle(with(KeyCode::Char('o'), Mods::ctrl()));
    assert!(app.panels_on);
}

#[allow(dead_code)]
fn entry(n: &str) -> Entry {
    Entry::file(n, 0)
}

#[test]
fn the_menu_is_the_documents() {
    let mut app = main_screen();
    app.handle(key(KeyCode::F(9)));
    let mut buf = Buffer::new(80, 25);
    app.draw(&mut buf, &cp437);
    compare(&buf, &screen_in("04-menus.md", "# 4."), "04-menus.md, F9 with the left panel active");
    // OWLOSUI's menu colours (core/src/menu.rs): green under the cursor,
    // its hotkey red.
    assert_eq!(buf.get(2, 0).unwrap().attr, 0x24, "Left: the current title, its hotkey");
    assert_eq!(buf.get(4, 2).unwrap().attr, 0x20, "Brief, the current item");
    assert_eq!(buf.get(3, 2).unwrap().attr, 0x24, "its hotkey");
    // Right goes to Files; Esc closes; Enter on Full makes the left panel Full.
    app.handle(key(KeyCode::Down));
    app.handle(key(KeyCode::Down));
    app.handle(key(KeyCode::Enter));
    assert!(!app.chrome.menu_is_open());
    assert_eq!(app.panels[0].mode, kc::Mode::Full);
}

/// Not a test: the screens the documents show, as the program draws them
/// now, into target/screens.txt - for redrawing a document after the
/// program changed on purpose. `cargo test -p kc --test screens -- --ignored`.
#[test]
#[ignore]
fn write_the_screens() {
    let frame = |app: &mut App| -> String {
        let mut buf = Buffer::new(80, 25);
        app.draw(&mut buf, &cp437);
        (0..25).map(|y| (0..80).map(|x| back(buf.get(x, y).unwrap().ch)).collect::<String>().trim_end().to_string()).collect::<Vec<_>>().join("\n")
    };
    let mut out = String::new();
    let mut app = main_screen();
    out += &format!("== main\n{}\n", frame(&mut app));
    let mut app = main_screen();
    app.handle(with(KeyCode::F(1), Mods::alt()));
    out += &format!("== drive menu\n{}\n", frame(&mut app));
    let mut app = main_screen();
    app.handle(key(KeyCode::F(9)));
    out += &format!("== menu\n{}\n", frame(&mut app));
    // 05-file-operations.md: copy, the overwrite question, an error,
    // delete, a full folder, the progress.
    let mut app = main_screen();
    app.handle(key(KeyCode::F(5)));
    out += &format!("== copy\n{}\n", frame(&mut app));
    app.handle(key(KeyCode::Enter));
    while app.busy() {
        app.tick();
    }
    out += &format!("== overwrite\n{}\n", frame(&mut app));
    let mut fs = lessons();
    fs.fail_on("C:\\LESSONS\\L07\\README.TXT", kc::fs::ErrorKind::WriteProtected, 9);
    let mut app = main_screen_on(fs);
    app.handle(key(KeyCode::F(5)));
    app.handle(key(KeyCode::Enter));
    app.handle(key(KeyCode::Enter));
    while app.busy() {
        app.tick();
    }
    out += &format!("== error\n{}\n", frame(&mut app));
    let mut app = main_screen();
    app.handle(key(KeyCode::Tab));
    app.handle(key(KeyCode::F(8)));
    out += &format!("== delete\n{}\n", frame(&mut app));
    let mut app = App::new(Box::new(lessons()), 80, 25);
    app.handle(key(KeyCode::Enter));
    app.handle(key(KeyCode::F(8)));
    app.handle(key(KeyCode::Enter));
    out += &format!("== full folder\n{}\n", frame(&mut app));
    let mut fs = lessons();
    fs.add_file("C:\\LESSONS\\L07\\DISK.IMG", &vec![0; 3 << 20]);
    let mut app = App::new(Box::new(fs), 80, 25);
    let i = app.panels[0].index_of("DISK.IMG").unwrap();
    app.panels[0].set_cursor(i);
    app.handle(key(KeyCode::F(5)));
    for c in "D:\\".chars() {
        app.handle(key(KeyCode::Char(c)));
    }
    app.handle(key(KeyCode::Enter));
    for _ in 0..2 {
        app.tick();
    }
    out += &format!("== progress\n{}\n", frame(&mut app));
    let path = format!("{}/../target/screens.txt", env!("CARGO_MANIFEST_DIR"));
    std::fs::write(&path, out).unwrap();
}
