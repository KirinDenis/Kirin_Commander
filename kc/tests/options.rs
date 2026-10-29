//! The Options dialogs, the switches they set, and Save setup
//! (docs/04-menus.md, Options): each switch changes what the program does.

use kc::dialog::Purpose;
use kc::options::{Page, O};
use kc::screen::cp437;
use kc::{App, MemFs, Mode, Sort};
use owlosui_core::{Buffer, Event, Key, KeyCode, Mods, Mouse, MouseKind};

fn app() -> App {
    let mut fs = MemFs::new(&['C', 'D']);
    fs.add_file("C:\\WORK\\HELLO.ASM", b"org 100h");
    fs.add_file("C:\\WORK\\README.TXT", b"read me");
    fs.add_dir("C:\\WORK\\OLD");
    fs.add_file("C:\\WORK\\OLD\\NOTE.TXT", b"kept");
    fs.add_dir("D:\\BACKUP");
    fs.set_cwd("C:\\WORK");
    let mut app = App::new(Box::new(fs), 80, 25);
    app.handle(key(KeyCode::Tab));
    app.handle(with(KeyCode::F(2), Mods::alt()));
    app.handle(key(KeyCode::Char('d')));
    app.handle(key(KeyCode::Tab));
    app
}

fn key(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods::default() })
}
fn with(code: KeyCode, mods: Mods) -> Event {
    Event::Key(Key { code, mods })
}
fn finish(app: &mut App) {
    for _ in 0..1000 {
        if !app.busy() {
            return;
        }
        app.tick();
    }
    panic!("the job never ended");
}
fn on(app: &mut App, name: &str) {
    let i = app.panels[app.active].index_of(name).unwrap();
    app.panels[app.active].set_cursor(i);
}
fn screen(app: &mut App) -> Vec<String> {
    let mut buf = Buffer::new(80, 25);
    app.draw(&mut buf, &cp437);
    (0..25).map(|y| (0..80).map(|x| glyph_char(buf.get(x, y).unwrap().ch)).collect()).collect()
}
fn glyph_char(g: owlosui_core::Glyph) -> char {
    for c in "║═╔╗╚╝╟╢─│┴╤╧►◄▄▀".chars() {
        if cp437(c) == g {
            return c;
        }
    }
    char::from_u32(g as u32).unwrap_or('?')
}

/// F9, Options, an item by its hotkey.
fn open(app: &mut App, hot: char) {
    app.handle(key(KeyCode::F(9)));
    for _ in 0..3 {
        app.handle(key(KeyCode::Right));
    }
    app.handle(key(KeyCode::Char(hot)));
}

#[test]
fn the_options_items_open_their_dialogs() {
    for (hot, page) in [('c', Page::Configuration), ('p', Page::Panels), ('n', Page::Confirmations), ('v', Page::Editor)] {
        let mut app = app();
        open(&mut app, hot);
        let d = app.dialog.as_ref().unwrap_or_else(|| panic!("{:?} did not open", page));
        assert_eq!(d.purpose, Purpose::Options(page));
        assert_eq!(d.switches.len(), page.switches(true).len());
        app.handle(key(KeyCode::Esc));
        assert!(app.dialog.is_none());
    }
}

#[test]
fn the_confirmations_dialog_is_the_documents() {
    // The box in 04-menus.md, the block after its "Each ..." paragraph.
    let path = format!("{}/../docs/04-menus.md", env!("CARGO_MANIFEST_DIR"));
    let doc = std::fs::read_to_string(&path).unwrap();
    let from = doc.find("Each \"...\" item").expect("the paragraph");
    let start = doc[from..].find("```\n").unwrap() + from + 4;
    let end = doc[start..].find("```").unwrap() + start;
    let want: Vec<&str> = doc[start..end].lines().collect();

    let mut app = app();
    open(&mut app, 'n');
    let s = screen(&mut app);
    let top = s.iter().position(|l| l.contains(want[0])).unwrap_or_else(|| panic!("no box like the document's:\n{}", s.join("\n")));
    for (k, w) in want.iter().enumerate() {
        assert!(s[top + k].contains(w), "line {} differs:\n want {}\n  got {}", k + 1, w, s[top + k]);
    }
}

#[test]
fn space_turns_a_switch_and_ok_keeps_it_cancel_does_not() {
    let mut app = app();
    assert!(app.opt.on(O::InsertMovesDown));
    open(&mut app, 'c');
    app.handle(key(KeyCode::Char(' ')));
    app.handle(key(KeyCode::Esc));
    assert!(app.opt.on(O::InsertMovesDown), "Esc keeps the switches as they were");

    open(&mut app, 'c');
    app.handle(key(KeyCode::Char(' ')));
    app.handle(key(KeyCode::Enter));
    assert!(!app.opt.on(O::InsertMovesDown), "Enter is OK");
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::Insert));
    assert_eq!(app.panels[0].current().unwrap().name, "HELLO.ASM", "Insert no longer moves down");
}

#[test]
fn a_click_on_a_switch_turns_it() {
    let mut app = app();
    open(&mut app, 'p');
    screen(&mut app);
    // OWLOSUI's check box, found on the screen by its words.
    let s = screen(&mut app);
    let y = s.iter().position(|l| l.contains("Column titles")).expect("the switch on the screen") as i16;
    let x = s[y as usize].chars().position(|c| c == '[').expect("its box") as i16;
    assert_eq!(app.dialog.as_ref().unwrap().switches[3].0, "Column titles");
    app.handle(Event::Mouse(Mouse { x: x + 1, y, kind: MouseKind::Down(owlosui_core::Button::Left) }));
    assert!(!app.dialog.as_ref().unwrap().switches[3].1);
    app.handle(key(KeyCode::Enter));
    assert!(!app.opt.on(O::ColumnTitles));
}

#[test]
fn without_column_titles_and_mini_status_the_names_have_their_rows() {
    let mut app = app();
    let rows = app.panels[0].rows;
    app.opt.set(O::ColumnTitles, false);
    app.opt.set(O::MiniStatus, false);
    open(&mut app, 'p');
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.panels[0].rows, rows + 3, "a row from the titles, two from the mini status");
    let s = screen(&mut app);
    assert!(s[1].starts_with("║.."), "the names right under the top edge:\n{}", s.join("\n"));
    assert!(!s.iter().any(|l| l.contains("Name")), "no column titles");
    assert!(!s.iter().any(|l| l.contains('╟')), "no line over a mini status");
    assert!(s[22].contains('╧'), "the rules meet the bottom edge:\n{}", s.join("\n"));
}

#[test]
fn the_menu_bar_can_stay() {
    let mut app = app();
    open(&mut app, 'm');
    assert!(app.opt.on(O::MenuBar));
    assert!(!app.chrome.menu_is_open(), "a switch closes the menu");
    let s = screen(&mut app);
    assert!(s[0].contains("Left") && s[0].contains("Options"), "the bar stays:\n{}", s.join("\n"));
    assert!(s[1].starts_with('╔'), "the panels one line lower:\n{}", s.join("\n"));
    // A click on a name still finds it: a row lower than before.
    let i = app.panels[0].index_of("HELLO.ASM").unwrap();
    let y = 1 + 2 + i as i16;
    app.handle(Event::Mouse(Mouse { x: 3, y, kind: MouseKind::Down(owlosui_core::Button::Left) }));
    assert_eq!(app.panels[0].current().unwrap().name, "HELLO.ASM");
}

#[test]
fn l_turns_the_clock_and_g_is_languages() {
    let mut app = app();
    open(&mut app, 'l');
    assert!(!app.opt.on(O::Clock));
    open(&mut app, 'g');
    assert!(matches!(app.pick, Some(kc::commands::Pick::Languages(_))), "G opens the languages");
}

#[test]
fn delete_without_its_question_deletes_at_once() {
    let mut app = app();
    app.opt.set(O::ConfirmDelete, false);
    on(&mut app, "README.TXT");
    app.handle(key(KeyCode::F(8)));
    assert!(app.dialog.is_none(), "no question");
    finish(&mut app);
    assert!(app.fs.stat("C:\\WORK\\README.TXT").is_err());
}

#[test]
fn a_folder_with_files_is_asked_about_unless_turned_off() {
    let mut app = app();
    app.opt.set(O::ConfirmDelete, false);
    on(&mut app, "OLD");
    app.handle(key(KeyCode::F(8)));
    finish(&mut app);
    assert!(app.dialog.is_some(), "the not-empty question still comes");
    app.handle(key(KeyCode::Esc));
    finish(&mut app);

    app.opt.set(O::ConfirmNotEmpty, false);
    on(&mut app, "OLD");
    app.handle(key(KeyCode::F(8)));
    finish(&mut app);
    assert!(app.dialog.is_none());
    assert!(app.fs.stat("C:\\WORK\\OLD").is_err());
}

#[test]
fn copy_without_its_question_copies_to_the_other_panel() {
    let mut app = app();
    app.opt.set(O::ConfirmCopy, false);
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::F(5)));
    assert!(app.dialog.is_none());
    finish(&mut app);
    assert!(app.fs.stat("D:\\HELLO.ASM").is_ok());
    // Shift+F5 needs the new name: it asks whatever the switch says.
    app.handle(with(KeyCode::F(5), Mods { shift: true, ..Mods::default() }));
    assert!(app.dialog.is_some());
}

#[test]
fn overwrite_without_its_question_overwrites() {
    let mut app = app();
    app.opt.set(O::ConfirmCopy, false);
    app.opt.set(O::ConfirmOverwrite, false);
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::F(5)));
    finish(&mut app);
    app.handle(key(KeyCode::F(5)));
    finish(&mut app);
    assert!(app.dialog.is_none(), "no Overwrite question");
}

#[test]
fn quit_without_its_question_quits() {
    let mut app = app();
    app.opt.set(O::ConfirmQuit, false);
    app.handle(key(KeyCode::F(10)));
    assert_eq!(app.take_requests(), vec![kc::Request::Quit]);
}

#[test]
fn esc_on_an_empty_line_can_hide_the_panels() {
    let mut app = app();
    app.handle(key(KeyCode::Esc));
    assert!(app.panels_on, "off by default");
    app.opt.set(O::EscHidesPanels, true);
    app.handle(key(KeyCode::Esc));
    assert!(!app.panels_on);
    app.handle(key(KeyCode::Esc));
    assert!(app.panels_on, "and back");
}

#[test]
fn letters_can_search_without_alt() {
    let mut app = app();
    app.opt.set(O::SearchWithoutAlt, true);
    app.handle(key(KeyCode::Char('r')));
    assert!(app.search.is_some());
    assert_eq!(app.panels[0].current().unwrap().name, "README.TXT");
    assert!(app.cmd.is_empty(), "the letter did not go to the command line");
}

#[test]
fn the_editor_opens_as_its_switches_say() {
    // Syntax colours: the status line names the language, or names none.
    let mut status = Vec::new();
    for syntax in [true, false] {
        let mut app = app();
        app.opt.set(O::Syntax, syntax);
        on(&mut app, "HELLO.ASM");
        app.handle(key(KeyCode::F(4)));
        assert!(app.editor.is_some());
        status.push(screen(&mut app)[0].trim_end().to_string());
    }
    assert!(status[0].len() > status[1].len(), "the language named only with colours on:\n{}\n{}", status[0], status[1]);

    // Wrap: the viewer's F2 offers the other state.
    let mut app = app();
    app.opt.set(O::Wrap, true);
    on(&mut app, "README.TXT");
    app.handle(key(KeyCode::F(3)));
    assert!(screen(&mut app)[24].contains("F2 Unwrp"), "{}", screen(&mut app)[24]);
}

#[test]
fn names_keep_their_case_when_so_set() {
    let mut app = app();
    assert!(screen(&mut app).iter().any(|l| l.contains("hello    asm")));
    app.opt.set(O::NamesCase, false);
    assert!(screen(&mut app).iter().any(|l| l.contains("HELLO    ASM")));
}

#[test]
fn save_setup_writes_what_the_next_start_reads() {
    let mut app = app();
    app.setup_path = Some(String::from("C:\\KC\\KC.INI"));
    app.opt.set(O::Clock, false);
    app.opt.set(O::Wrap, true);
    app.scheme = kc::colors::LAPTOP;
    app.panels[1].set_mode(Mode::Full);
    app.panels[1].set_order(Sort::Size, true);
    app.handle(with(KeyCode::F(9), Mods { shift: true, ..Mods::default() }));
    assert!(app.dialog.is_none(), "saved without a word");
    let h = app.fs.open_read("C:\\KC\\KC.INI").expect("the file, its folder made");
    let mut buf = vec![0u8; 4096];
    let n = app.fs.read(h, &mut buf).unwrap();
    let _ = app.fs.close(h);
    let text = String::from_utf8_lossy(&buf[..n]).into_owned();
    assert!(text.contains("clock=0\r\n") && text.contains("wrap=1\r\n") && text.contains("colours=laptop"), "{}", text);

    // A second commander on the same disk reads it.
    let mut fresh = App::new(Box::new(MemFs::new(&['C'])), 80, 25);
    kc::setup::read(&mut fresh, &text);
    assert!(!fresh.opt.on(O::Clock));
    assert!(fresh.opt.on(O::Wrap));
    assert_eq!(fresh.scheme, kc::colors::LAPTOP);
    assert_eq!(fresh.panels[1].mode, Mode::Full);
    assert_eq!((fresh.panels[1].sort, fresh.panels[1].reverse), (Sort::Size, true));
}

#[test]
fn a_setup_line_it_does_not_know_is_passed_over() {
    let mut app = app();
    kc::setup::read(&mut app, "[options]\r\nno_such_thing=1\r\nclock=0\r\n[elsewhere]\r\nx=y\r\n");
    assert!(!app.opt.on(O::Clock));
    assert!(app.opt.on(O::KeyBar), "what the file does not name keeps its default");
}

#[test]
fn without_a_place_save_setup_says_so() {
    let mut app = app();
    app.handle(with(KeyCode::F(9), Mods { shift: true, ..Mods::default() }));
    assert!(app.dialog.is_some());
}
