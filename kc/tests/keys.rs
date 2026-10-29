//! What the keys do, through `App` as a host drives it (docs/03-keys.md).

use kc::app::Request;
use kc::dialog::Purpose;
use kc::{App, Fs, MemFs};
use owlosui_core::{Event, Key, KeyCode, Mods};

fn app() -> App {
    let mut fs = MemFs::new(&['C', 'D']);
    fs.add_file("C:\\WORK\\HELLO.ASM", b"org 100h");
    fs.add_file("C:\\WORK\\HELLO.COM", b"\xB4\x09");
    fs.add_file("C:\\WORK\\README.TXT", b"read me");
    fs.add_dir("C:\\WORK\\OLD");
    fs.add_dir("D:\\BACKUP");
    fs.set_cwd("C:\\WORK");
    let mut app = App::new(Box::new(fs), 80, 25);
    // The right panel on D:.
    app.handle(key(KeyCode::Tab));
    app.handle(alt(KeyCode::F(2)));
    app.handle(key(KeyCode::Char('d')));
    app.handle(key(KeyCode::Tab));
    assert_eq!(app.panels[1].dir, "D:\\");
    app
}

fn key(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods::default() })
}
fn alt(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods::alt() })
}
fn ctrl(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods::ctrl() })
}
fn shift(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods { shift: true, ..Mods::default() } })
}
fn typed(app: &mut App, s: &str) {
    for c in s.chars() {
        app.handle(key(KeyCode::Char(c)));
    }
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

#[test]
fn f5_copies_the_selection_to_the_other_panel() {
    let mut app = app();
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::Insert));
    app.handle(key(KeyCode::Insert));
    assert_eq!(app.panels[0].selected_names(), ["HELLO.ASM", "HELLO.COM"]);
    app.handle(key(KeyCode::F(5)));
    let d = app.dialog.as_ref().unwrap();
    assert_eq!(d.purpose, Purpose::Transfer { op: kc::job::Op::Copy, here: false });
    assert_eq!(d.text(), "D:\\", "the other panel's folder");
    assert_eq!(d.lines[1].0, "Copy 2 files to");
    app.handle(key(KeyCode::Enter));
    finish(&mut app);
    let names: Vec<&str> = app.panels[1].entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["BACKUP", "HELLO.ASM", "HELLO.COM"], "the target panel re-read; a root has no `..`");
    assert!(!app.panels[0].has_selection(), "done: the selection cleared");
}

#[test]
fn typing_replaces_the_target_and_an_arrow_edits_it() {
    let mut app = app();
    on(&mut app, "README.TXT");
    app.handle(key(KeyCode::F(5)));
    typed(&mut app, "COPY.TXT");
    assert_eq!(app.dialog.as_ref().unwrap().text(), "COPY.TXT");
    app.handle(key(KeyCode::Esc));
    app.handle(key(KeyCode::F(5)));
    app.handle(key(KeyCode::End));
    typed(&mut app, "BACKUP");
    assert_eq!(app.dialog.as_ref().unwrap().text(), "D:\\BACKUP");
    app.handle(key(KeyCode::Enter));
    finish(&mut app);
    assert!(app.fs.stat("D:\\BACKUP\\README.TXT").is_ok());
}

#[test]
fn shift_f6_renames_in_place_and_the_cursor_follows() {
    let mut app = app();
    on(&mut app, "README.TXT");
    app.handle(shift(KeyCode::F(6)));
    assert_eq!(app.dialog.as_ref().unwrap().text(), "README.TXT");
    typed(&mut app, "NOTES.TXT");
    app.handle(key(KeyCode::Enter));
    finish(&mut app);
    assert!(app.fs.stat("C:\\WORK\\NOTES.TXT").is_ok());
    assert!(app.fs.stat("C:\\WORK\\README.TXT").is_err());
}

#[test]
fn f7_makes_a_folder_and_the_cursor_lands_on_it() {
    let mut app = app();
    app.handle(key(KeyCode::F(7)));
    typed(&mut app, "NEW\\DEEP");
    app.handle(key(KeyCode::Enter));
    assert!(app.fs.stat("C:\\WORK\\NEW\\DEEP").is_ok());
    assert_eq!(app.panels[0].current().unwrap().name, "NEW");
}

#[test]
fn f7_checks_an_eight_three_name_first() {
    let mut app = app();
    app.handle(key(KeyCode::F(7)));
    typed(&mut app, "TOOLONGNAME");
    app.handle(key(KeyCode::Enter));
    let d = app.dialog.as_ref().unwrap();
    assert!(d.lines.iter().any(|(l, _)| l.contains("at most 8")));
    assert!(app.fs.stat("C:\\WORK\\TOOLONGNAME").is_err());
}

#[test]
fn f8_deletes_and_the_cursor_goes_to_the_next() {
    let mut app = app();
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::F(8)));
    assert_eq!(app.dialog.as_ref().unwrap().lines[2].0, "\"HELLO.ASM\"");
    app.handle(key(KeyCode::Enter));
    finish(&mut app);
    assert!(app.fs.stat("C:\\WORK\\HELLO.ASM").is_err());
    assert_eq!(app.panels[0].current().unwrap().name, "HELLO.COM");
}

#[test]
fn esc_in_a_question_answers_no() {
    let mut app = app();
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::F(8)));
    app.handle(key(KeyCode::Esc));
    assert!(app.dialog.is_none());
    assert!(app.fs.stat("C:\\WORK\\HELLO.ASM").is_ok());
}

#[test]
fn enter_runs_a_program_and_views_anything_else() {
    let mut app = app();
    on(&mut app, "HELLO.COM");
    app.handle(key(KeyCode::Enter));
    on(&mut app, "README.TXT");
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.take_requests(), [Request::Run { dir: "C:\\WORK".into(), line: "HELLO.COM".into() }]);
    let viewer = app.editor.as_ref().map(|e| (e.path.as_str(), e.readonly));
    assert_eq!(viewer, Some(("C:\\WORK\\README.TXT", true)), "anything else: the viewer");
}

#[test]
fn f4_edits_f2_saves_and_f10_goes_back() {
    let mut app = app();
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::F(4)));
    assert!(app.editor.as_ref().is_some_and(|e| !e.readonly));
    app.handle(key(KeyCode::End));
    typed(&mut app, " ; com");
    // F10 with changes asks first; Esc on the question stays.
    app.handle(key(KeyCode::F(10)));
    app.handle(key(KeyCode::Esc));
    assert!(app.editor.is_some());
    app.handle(key(KeyCode::F(2)));
    assert_eq!(app.fs.stat("C:\\WORK\\HELLO.ASM").unwrap().size, 14);
    app.handle(key(KeyCode::F(10)));
    assert!(app.editor.is_none(), "saved: no question");
}

#[test]
fn the_editor_menu_is_hidden_until_f9() {
    let mut app = app();
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::F(4)));
    let mut buf = owlosui_core::Buffer::new(80, 25);
    app.draw(&mut buf, &kc::screen::cp437);
    let row0: String = (0..80).map(|x| buf.get(x, 0).unwrap().ch as u8 as char).collect();
    assert!(row0.starts_with(" C:\\WORK\\HELLO.ASM"), "the status line: {}", row0);
    app.handle(key(KeyCode::F(9)));
    app.draw(&mut buf, &kc::screen::cp437);
    let row0: String = (0..80).map(|x| buf.get(x, 0).unwrap().ch as u8 as char).collect();
    assert!(row0.contains("File"), "the menu bar: {}", row0);
    let row24: String = (0..80).map(|x| buf.get(x, 24).unwrap().ch as u8 as char).collect();
    // OWLOSUI's status line says what the item under the cursor does.
    assert!(row24.contains("Write the file"), "{}", row24);
}

#[test]
fn the_command_line_takes_typing_and_ctrl_enter_puts_the_name() {
    let mut app = app();
    on(&mut app, "HELLO.ASM");
    typed(&mut app, "fasm ");
    app.handle(ctrl(KeyCode::Enter));
    assert_eq!(app.cmd.text(), "fasm HELLO.ASM ");
    // While the line has text, Home and End are the line's.
    app.handle(key(KeyCode::Home));
    assert_eq!(app.cmd.caret, 0);
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.take_requests(), [Request::Run { dir: "C:\\WORK".into(), line: "fasm HELLO.ASM ".into() }]);
    assert!(app.cmd.is_empty());
}

#[test]
fn cd_moves_the_panel() {
    let mut app = app();
    typed(&mut app, "cd old");
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.panels[0].dir, "C:\\WORK\\old");
    typed(&mut app, "cd ..");
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.panels[0].dir, "C:\\WORK");
    assert!(app.take_requests().is_empty());
}

#[test]
fn alt_letter_is_quick_search() {
    let mut app = app();
    app.handle(alt(KeyCode::Char('r')));
    assert_eq!(app.panels[0].current().unwrap().name, "README.TXT");
    assert!(app.search.is_some());
    typed(&mut app, "q");
    assert_eq!(app.search.as_ref().unwrap().line.text(), "r", "no name starts with rq: the letter is not taken");
    app.handle(key(KeyCode::Esc));
    assert!(app.search.is_none());
    assert_eq!(app.panels[0].current().unwrap().name, "README.TXT");
}

#[test]
fn gray_plus_selects_by_mask() {
    let mut app = app();
    typed(&mut app, "+");
    assert!(matches!(app.dialog.as_ref().unwrap().purpose, Purpose::Select { on: true }));
    typed(&mut app, "hello.*");
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.panels[0].selected_names(), ["HELLO.ASM", "HELLO.COM"]);
}

#[test]
fn backspace_on_an_empty_line_goes_up_and_tab_keeps_both_cursors() {
    let mut app = app();
    on(&mut app, "README.TXT");
    app.handle(key(KeyCode::Tab));
    app.handle(key(KeyCode::Tab));
    assert_eq!(app.panels[0].current().unwrap().name, "README.TXT");
    app.handle(key(KeyCode::Backspace));
    assert_eq!(app.panels[0].dir, "C:\\");
    assert_eq!(app.panels[0].current().unwrap().name, "WORK");
}

#[test]
fn f10_asks_and_yes_quits() {
    let mut app = app();
    app.handle(key(KeyCode::F(10)));
    assert!(matches!(app.dialog.as_ref().unwrap().purpose, Purpose::Quit));
    app.handle(key(KeyCode::Char('y')));
    assert_eq!(app.take_requests(), [Request::Quit]);
}
