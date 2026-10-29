//! Stepping out on DOS (step.rs, D18): the command lines go into
//! RUNPROG.BAT for the batch file that started the commander, where it was
//! goes into KC.SAV, and the next start is where it was.

use kc::app::Request;
use kc::step::{batch, RUN_FILE, SAVE_FILE};
use kc::{App, Fs, MemFs};
use owlosui_core::{Event, Key, KeyCode, Mods};

fn fs() -> MemFs {
    let mut fs = MemFs::new(&['A', 'C']);
    fs.add_dir("A:\\LESSONS\\L01");
    fs.add_file("A:\\LESSONS\\L01\\LINE.COM", b"\xC3");
    fs.add_file("A:\\LESSONS\\L01\\BUILD.BAT", b"@echo off");
    fs.add_dir("C:\\GAMES");
    fs.set_cwd("C:\\");
    fs
}

fn key(code: KeyCode) -> Event {
    Event::Key(Key { code, mods: Mods::default() })
}

fn read(fs: &mut dyn Fs, path: &str) -> Option<String> {
    let h = fs.open_read(path).ok()?;
    let mut buf = vec![0u8; 4096];
    let n = fs.read(h, &mut buf).ok()?;
    let _ = fs.close(h);
    Some(String::from_utf8_lossy(&buf[..n]).into_owned())
}

#[test]
fn the_batch_runs_each_line_in_its_folder_and_comes_home() {
    let runs = vec![(String::from("A:\\LESSONS\\L01"), String::from("BUILD.BAT")), (String::from("C:\\"), String::from("dir"))];
    assert_eq!(
        batch(&runs, "C:\\", '\\'),
        "@echo off\r\nA:\r\ncd \\LESSONS\\L01\r\ncall BUILD.BAT\r\nC:\r\ncd \\\r\ncall dir\r\nC:\r\ncd \\\r\n"
    );
}

#[test]
fn the_folders_on_the_command_line_are_the_panels() {
    let mut app = App::new(Box::new(fs()), 80, 25);
    app.start_in(&[String::from("A:\\"), String::from("C:\\GAMES")]);
    assert_eq!((app.panels[0].dir.as_str(), app.panels[1].dir.as_str()), ("A:\\", "C:\\GAMES"));
    app.start_in(&[String::from("A:\\LESSONS"), String::from("Q:\\NOWHERE")]);
    assert_eq!((app.panels[0].dir.as_str(), app.panels[1].dir.as_str()), ("A:\\LESSONS", "C:\\GAMES"));
}

#[test]
fn a_step_out_leaves_the_lines_and_the_next_start_is_where_it_was() {
    let mut app = App::new(Box::new(fs()), 80, 25);
    app.start_in(&[String::from("A:\\LESSONS\\L01"), String::from("C:\\GAMES")]);
    let i = app.panels[0].index_of("LINE.COM").unwrap();
    app.panels[0].set_cursor(i);
    app.handle(key(KeyCode::Enter));
    let mut requests = app.take_requests();
    let runs = App::take_runs(&mut requests);
    assert!(requests.is_empty());
    assert_eq!(runs, vec![(String::from("A:\\LESSONS\\L01"), String::from("LINE.COM"))]);
    assert!(app.step_out("C:\\", &runs));
    let bat = read(&mut *app.fs, &format!("C:\\{RUN_FILE}")).unwrap();
    assert!(bat.contains("A:\r\ncd \\LESSONS\\L01\r\ncall LINE.COM\r\nC:\r\ncd \\\r\n"), "{bat}");
    let save = read(&mut *app.fs, &format!("C:\\{SAVE_FILE}")).unwrap();

    // The commander started again, on another folder: KC.SAV puts it back.
    let mut fs2 = fs();
    fs2.add_file(&format!("C:\\{SAVE_FILE}"), save.as_bytes());
    let mut again = App::new(Box::new(fs2), 80, 25);
    assert!(again.resume("C:\\"));
    assert_eq!(again.active, 0);
    assert_eq!(again.panels[0].dir, "A:\\LESSONS\\L01");
    assert_eq!(again.panels[1].dir, "C:\\GAMES");
    assert_eq!(again.panels[0].current().map(|e| e.name.as_str()), Some("LINE.COM"));
    assert_eq!(again.cmd.history(), &[String::from("LINE.COM")]);
    // Read once: the start after it is a start of its own.
    assert!(read(&mut *again.fs, &format!("C:\\{SAVE_FILE}")).is_none());
    assert!(!again.resume("C:\\"));
}

#[test]
fn requests_other_than_runs_stay() {
    let mut requests = vec![Request::Quit, Request::Run { dir: String::from("C:\\"), line: String::from("ver") }];
    let runs = App::take_runs(&mut requests);
    assert_eq!(runs.len(), 1);
    assert!(matches!(requests.as_slice(), [Request::Quit]));
}
