//! The menu's commands beyond the panels' keys (docs/04-menus.md): Help,
//! the user menu, the extension table, File Attributes, Find file,
//! History, Folder Tree, Volume label, System Info, Folder shortcuts,
//! Languages - each does what its item says.

use kc::app::Request;
use kc::commands::Pick;
use kc::dialog::Purpose;
use kc::panel::View;
use kc::screen::cp437;
use kc::{App, MemFs};
use owlosui_core::{Buffer, Event, Key, KeyCode, Mods};

fn fs() -> MemFs {
    let mut fs = MemFs::new(&['C', 'D']);
    fs.add_file("C:\\WORK\\HELLO.ASM", b"org 100h");
    fs.add_file("C:\\WORK\\README.TXT", b"read me");
    fs.add_dir("C:\\WORK\\OLD");
    fs.add_file("C:\\WORK\\OLD\\NOTE.TXT", b"kept");
    fs.add_dir("C:\\KC");
    fs.add_dir("D:\\BACKUP");
    fs.set_cwd("C:\\WORK");
    fs
}

fn start(fs: MemFs) -> App {
    let mut app = App::new(Box::new(fs), 80, 25);
    app.setup_path = Some(String::from("C:\\KC\\KC.INI"));
    app
}

fn app() -> App {
    start(fs())
}

fn key(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods::default() })
}
fn with(code: KeyCode, mods: Mods) -> Event {
    Event::Key(Key { code, mods })
}
fn typed(app: &mut App, s: &str) {
    for c in s.chars() {
        app.handle(key(KeyCode::Char(c)));
    }
}
fn on(app: &mut App, name: &str) {
    let i = app.panels[app.active].index_of(name).unwrap();
    app.panels[app.active].set_cursor(i);
}
fn write(app: &mut App, path: &str, text: &str) {
    let h = app.fs.create(path).unwrap();
    app.fs.write(h, text.as_bytes()).unwrap();
    app.fs.close(h).unwrap();
}
fn screen(app: &mut App) -> Vec<String> {
    let mut buf = Buffer::new(80, 25);
    app.draw(&mut buf, &cp437);
    (0..25).map(|y| (0..80).map(|x| char::from_u32(buf.get(x, y).unwrap().ch as u32).unwrap_or('?')).collect()).collect()
}
/// F9, a title of the bar (0 Left .. 4 Right), an item by its hotkey.
fn menu(app: &mut App, title: usize, hot: char) {
    app.handle(key(KeyCode::F(9)));
    for _ in 0..title {
        app.handle(key(KeyCode::Right));
    }
    app.handle(key(KeyCode::Char(hot)));
}
fn runs(app: &mut App) -> Vec<String> {
    app.take_requests()
        .into_iter()
        .filter_map(|r| match r {
            Request::Run { line, .. } => Some(line),
            _ => None,
        })
        .collect()
}

#[test]
fn f1_shows_the_keys_in_the_viewer() {
    let mut app = app();
    app.handle(key(KeyCode::F(1)));
    assert!(app.editor.is_some());
    assert!(screen(&mut app).iter().any(|l| l.contains("Kirin Commander - the keys")));
    // The viewer's F6 has no file to edit: it stays the help.
    app.handle(key(KeyCode::F(6)));
    assert!(app.editor.as_ref().map(|e| e.readonly).unwrap_or(false));
}

#[test]
fn f2_runs_an_item_of_the_folders_menu_with_the_name_put_in() {
    let mut app = app();
    write(&mut app, "C:\\WORK\\KC.MNU", "; the lessons\r\nA: Assemble the file\r\n    fasm !.! !.com\r\nR: Run it\r\n    !\r\n");
    app.reread_both();
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::F(2)));
    assert!(matches!(app.pick, Some(Pick::UserMenu(_))));
    assert!(screen(&mut app).iter().any(|l| l.contains("A  Assemble the file")));
    app.handle(key(KeyCode::Char('a')));
    assert!(app.pick.is_none());
    assert_eq!(runs(&mut app), ["fasm HELLO.ASM HELLO.com"]);
}

#[test]
fn f2_without_a_menu_here_opens_the_main_one_and_says_when_there_is_none() {
    let mut app = app();
    app.handle(key(KeyCode::F(2)));
    let d = app.dialog.as_ref().expect("no menu: a message");
    assert!(d.lines.iter().any(|(l, _)| l.contains("Menu file edit")));
    app.handle(key(KeyCode::Esc));
    write(&mut app, "C:\\KC\\KC.MNU", "D: Directory\r\n    dir\r\n");
    app.handle(key(KeyCode::F(2)));
    app.handle(key(KeyCode::Enter));
    assert_eq!(runs(&mut app), ["dir"]);
    // Shift+F2: the main menu, even with a folder's own.
    write(&mut app, "C:\\WORK\\KC.MNU", "L: Local\r\n    echo local\r\n");
    app.handle(with(KeyCode::F(2), Mods { shift: true, ctrl: false, alt: false }));
    app.handle(key(KeyCode::Enter));
    assert_eq!(runs(&mut app), ["dir"]);
}

#[test]
fn menu_file_edit_opens_the_local_menu_in_the_editor() {
    let mut app = app();
    menu(&mut app, 2, 'm');
    assert_eq!(app.dialog.as_ref().map(|d| d.purpose.clone()), Some(Purpose::MenuFile));
    app.handle(key(KeyCode::Char('l')));
    let e = app.editor.as_ref().expect("the editor");
    assert_eq!(e.path, "C:\\WORK\\KC.MNU");
    assert!(!e.readonly);
}

#[test]
fn enter_runs_what_the_extension_table_says_else_views() {
    let mut app = app();
    write(&mut app, "C:\\KC\\KC.EXT", "; Enter on a file\r\nasm inc: edit !.!\r\n");
    on(&mut app, "HELLO.ASM");
    app.handle(key(KeyCode::Enter));
    assert_eq!(runs(&mut app), ["edit HELLO.ASM"]);
    assert!(app.editor.is_none());
    on(&mut app, "README.TXT");
    app.handle(key(KeyCode::Enter));
    assert!(app.editor.as_ref().map(|e| e.readonly).unwrap_or(false), "no entry: the viewer");
    // The table is a file the menu opens.
    app.editor = None;
    menu(&mut app, 2, 'x');
    assert_eq!(app.editor.as_ref().map(|e| e.path.clone()), Some(String::from("C:\\KC\\KC.EXT")));
}

#[test]
fn ctrl_a_sets_the_attributes_and_the_date() {
    let mut app = app();
    on(&mut app, "README.TXT");
    app.handle(with(KeyCode::Char('a'), Mods::ctrl()));
    assert!(matches!(app.dialog.as_ref().map(|d| &d.purpose), Some(Purpose::Attributes(_))));
    typed(&mut app, "01-02-03 04:05");
    // To the check boxes: Read-only is the first.
    app.handle(key(KeyCode::Tab));
    app.handle(key(KeyCode::Char(' ')));
    app.handle(key(KeyCode::Enter));
    assert!(app.dialog.is_none(), "{:?}", app.dialog.as_ref().map(|d| d.lines.clone()));
    let e = app.fs.stat("C:\\WORK\\README.TXT").unwrap();
    assert!(e.attrs.read_only);
    assert_eq!((e.stamp.year, e.stamp.month, e.stamp.day, e.stamp.hour, e.stamp.minute), (2003, 2, 1, 4, 5));
}

#[test]
fn alt_f7_finds_in_the_folders_below_and_goes_to_one() {
    let mut app = app();
    app.handle(with(KeyCode::F(7), Mods::alt()));
    typed(&mut app, "*.TXT");
    app.handle(key(KeyCode::Enter));
    let Some(Pick::Found(list)) = &app.pick else { panic!("no list") };
    assert_eq!(list, &["C:\\WORK\\README.TXT", "C:\\WORK\\OLD\\NOTE.TXT"]);
    app.handle(key(KeyCode::Down));
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.panels[0].dir, "C:\\WORK\\OLD");
    assert_eq!(app.panels[0].current().unwrap().name, "NOTE.TXT");
}

#[test]
fn alt_f7_says_when_nothing_matches() {
    let mut app = app();
    app.handle(with(KeyCode::F(7), Mods::alt()));
    typed(&mut app, "*.ZIP");
    app.handle(key(KeyCode::Enter));
    assert!(app.pick.is_none());
    assert!(app.dialog.as_ref().unwrap().lines.iter().any(|(l, _)| l.contains("No file matches *.ZIP")));
}

#[test]
fn alt_f8_runs_a_line_of_the_history_again() {
    let mut app = app();
    typed(&mut app, "dir /w");
    app.handle(key(KeyCode::Enter));
    assert_eq!(runs(&mut app), ["dir /w"]);
    app.handle(with(KeyCode::F(8), Mods::alt()));
    assert!(matches!(app.pick, Some(Pick::History(_))));
    app.handle(key(KeyCode::Enter));
    assert_eq!(runs(&mut app), ["dir /w"]);
}

#[test]
fn alt_f10_is_the_folder_tree() {
    let mut app = app();
    app.handle(with(KeyCode::F(10), Mods::alt()));
    assert_eq!(app.panels[0].view, View::Tree);
}

#[test]
fn volume_label_writes_the_drives_label() {
    let mut app = app();
    menu(&mut app, 2, 'v');
    assert!(matches!(app.dialog.as_ref().map(|d| &d.purpose), Some(Purpose::Label(_))));
    typed(&mut app, "lessons");
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.fs.label("C:\\").as_deref(), Some("LESSONS"));
    // Twelve letters are one too many on DOS.
    menu(&mut app, 2, 'v');
    typed(&mut app, "TWELVELETTER");
    app.handle(key(KeyCode::Enter));
    assert!(app.dialog.as_ref().unwrap().lines.iter().any(|(l, _)| l.contains("at most 11")));
    assert_eq!(app.fs.label("C:\\").as_deref(), Some("LESSONS"));
}

#[test]
fn system_info_lists_the_machine_and_its_drives() {
    let mut app = app();
    app.system = vec![String::from("DOS 5.00")];
    menu(&mut app, 2, 'i');
    let d = app.dialog.as_ref().unwrap();
    assert_eq!(d.title, "System Info");
    assert!(d.lines.iter().any(|(l, _)| l == "DOS 5.00"));
    assert!(d.lines.iter().any(|(l, _)| l.trim_start().starts_with("C:")));
    assert!(d.lines.iter().any(|(l, _)| l.trim_start().starts_with("D:")));
}

#[test]
fn a_folder_shortcut_is_set_gone_to_and_saved() {
    let mut app = app();
    app.handle(with(KeyCode::Char('3'), Mods { ctrl: true, shift: true, alt: false }));
    assert_eq!(app.shortcuts[3].as_deref(), Some("C:\\WORK"));
    typed(&mut app, "cd OLD");
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.panels[0].dir, "C:\\WORK\\OLD");
    menu(&mut app, 2, 'o');
    assert!(matches!(app.pick, Some(Pick::Shortcuts)));
    app.handle(key(KeyCode::Char('3')));
    assert_eq!(app.panels[0].dir, "C:\\WORK");
    app.save_setup();
    let mut again = App::new(app.fs, 80, 25);
    again.setup_path = Some(String::from("C:\\KC\\KC.INI"));
    again.load_setup();
    assert_eq!(again.shortcuts[3].as_deref(), Some("C:\\WORK"));
}

const DEUTSCH: &str = "; Kirin Commander language file\r\n[language]\r\nname=Deutsch\r\n[text]\r\nHelp=Hilfe\r\nCopy=Kopie\r\nCopy {} to=Kopiere {} nach\r\n~C~opy=~K~opieren\r\n~D~elete=~L~oeschen\r\n";

#[test]
fn languages_lists_the_files_and_the_one_chosen_speaks() {
    let mut app = app();
    write(&mut app, "C:\\KC\\KCDEU.LNG", DEUTSCH);
    menu(&mut app, 3, 'g');
    let s = screen(&mut app);
    assert!(s.iter().any(|l| l.contains("English")));
    assert!(s.iter().any(|l| l.contains("Deutsch  (KCDEU.LNG)")));
    app.handle(key(KeyCode::Down));
    app.handle(key(KeyCode::Enter));
    assert_eq!(app.chrome.lang.name, "Deutsch");
    let s = screen(&mut app);
    assert!(s[24].starts_with("F1 Hilfe"), "{}", s[24]);
    // A dialog: its words in the language, its letters too.
    on(&mut app, "README.TXT");
    app.handle(key(KeyCode::F(5)));
    let s = screen(&mut app);
    assert!(s.iter().any(|l| l.contains("Kopiere \"README.TXT\" nach")), "{}", s.join("\n"));
    assert!(s.iter().any(|l| l.contains("Kopieren")));
    app.handle(key(KeyCode::Esc));
    app.handle(key(KeyCode::F(8)));
    app.handle(key(KeyCode::Char('l')));
    assert!(app.busy() || app.fs.stat("C:\\WORK\\README.TXT").is_err(), "L is Loeschen");
    // Saved, and read back at the next start.
    app.save_setup();
    let mut again = App::new(app.fs, 80, 25);
    again.setup_path = Some(String::from("C:\\KC\\KC.INI"));
    again.load_setup();
    assert_eq!(again.chrome.lang.name, "Deutsch");
}

#[test]
fn english_is_there_without_any_file() {
    let mut app = app();
    menu(&mut app, 3, 'g');
    let Some(Pick::Languages(list)) = &app.pick else { panic!("no list") };
    assert_eq!(list.len(), 1);
    app.handle(key(KeyCode::Enter));
    assert!(app.chrome.lang.is_english());
}

#[test]
fn every_item_of_the_menus_does_something() {
    for t in 0..5 {
        for it in kc::menu::items(t) {
            assert!(it.label == "-" || it.key.is_some(), "{} has nothing to do", it.label);
        }
    }
}
