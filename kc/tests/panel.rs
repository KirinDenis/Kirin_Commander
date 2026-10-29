//! The cursor rules of docs/02-panels.md 2.4, and the selection of 2.5,
//! one test each.

use kc::panel::Focus;
use kc::{Fs, MemFs, Mode, Panel, Sort};

fn tree() -> MemFs {
    let mut fs = MemFs::new(&['C', 'D']);
    for d in ["C:\\GAMES\\CITY", "C:\\GAMES\\OWLFLY2", "C:\\GAMES\\OWLFLY3", "C:\\GAMES\\SOKO", "D:\\WORK"] {
        fs.add_dir(d);
    }
    for f in ["C:\\GAMES\\GO.BAT", "C:\\GAMES\\README.TXT", "C:\\GAMES\\A.TXT", "C:\\GAMES\\Z.TXT", "C:\\GAMES\\CITY\\CITY.COM"] {
        fs.add_file(f, b"12345");
    }
    fs.set_cwd("C:\\GAMES");
    fs
}

fn panel(fs: &mut MemFs, dir: &str) -> Panel {
    let mut p = Panel::new(fs);
    p.set_rows(18);
    p.load(fs, dir, Focus::First).unwrap();
    p
}

fn names(p: &Panel) -> Vec<&str> {
    p.entries.iter().map(|e| e.name.as_str()).collect()
}

#[test]
fn up_first_then_folders_then_files() {
    let mut fs = tree();
    let p = panel(&mut fs, "C:\\GAMES");
    assert_eq!(names(&p), ["..", "CITY", "OWLFLY2", "OWLFLY3", "SOKO", "A.TXT", "GO.BAT", "README.TXT", "Z.TXT"]);
}

#[test]
fn rule_2_into_a_folder_the_cursor_is_on_up() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    p.set_cursor(1);
    assert!(p.enter(&mut fs).unwrap());
    assert_eq!(p.dir, "C:\\GAMES\\CITY");
    assert_eq!(p.cursor, 0);
}

#[test]
fn rule_3_out_of_a_folder_the_cursor_is_on_the_folder_left() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES\\OWLFLY3");
    p.set_cursor(0);
    assert!(p.enter(&mut fs).unwrap());
    assert_eq!(p.dir, "C:\\GAMES");
    assert_eq!(p.current().unwrap().name, "OWLFLY3");
}

#[test]
fn rule_4_to_the_root_the_cursor_is_on_the_first_level_folder() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES\\CITY");
    p.go_root(&mut fs).unwrap();
    assert_eq!(p.dir, "C:\\");
    assert_eq!(p.current().unwrap().name, "GAMES");
}

#[test]
fn rule_5_a_drive_comes_back_to_where_the_panel_was_on_it() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES\\CITY");
    p.go_drive(&mut fs, "D:\\").unwrap();
    assert_eq!(p.dir, "D:\\");
    p.go_drive(&mut fs, "C:\\").unwrap();
    assert_eq!(p.dir, "C:\\GAMES\\CITY");
}

#[test]
fn rule_6_a_name_gone_puts_the_cursor_on_the_next() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    let i = p.index_of("GO.BAT").unwrap();
    p.set_cursor(i);
    fs.remove_file("C:\\GAMES\\GO.BAT").unwrap();
    p.reread(&mut fs).unwrap();
    assert_eq!(p.current().unwrap().name, "README.TXT", "the next name, not the top");
    // The last one gone: the one before it.
    p.end();
    fs.remove_file("C:\\GAMES\\Z.TXT").unwrap();
    p.reread(&mut fs).unwrap();
    assert_eq!(p.current().unwrap().name, "README.TXT");
}

#[test]
fn rule_6_a_reread_keeps_the_name_when_others_come() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    let i = p.index_of("README.TXT").unwrap();
    p.set_cursor(i);
    fs.add_file("C:\\GAMES\\B.TXT", b"x");
    p.reread(&mut fs).unwrap();
    assert_eq!(p.current().unwrap().name, "README.TXT");
}

#[test]
fn rule_7_the_cursor_onto_a_new_name() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    fs.make_dir("C:\\GAMES\\NEW").unwrap();
    p.reread_onto(&mut fs, "NEW").unwrap();
    assert_eq!(p.current().unwrap().name, "NEW");
}

#[test]
fn rule_8_left_and_right_move_a_column_in_brief() {
    let mut fs = MemFs::new(&['C']);
    for i in 0..40 {
        fs.add_file(&format!("C:\\F{:02}.TXT", i), b"");
    }
    let mut p = Panel::new(&fs);
    p.set_rows(10);
    p.set_mode(Mode::Brief);
    p.load(&mut fs, "C:\\", Focus::First).unwrap();
    p.set_cursor(3);
    p.right();
    assert_eq!(p.cursor, 13);
    p.left();
    assert_eq!(p.cursor, 3);
    p.left();
    assert_eq!(p.cursor, 0, "from the first column: the first name");
    // Past the last column on the screen, the names scroll a whole column.
    p.set_cursor(29);
    assert_eq!(p.top, 0);
    p.down();
    assert_eq!(p.top, 10, "a column at a time, so the columns keep their places");
}

#[test]
fn rule_9_a_page_keeps_the_bar_on_its_row() {
    let mut fs = MemFs::new(&['C']);
    for i in 0..100 {
        fs.add_file(&format!("C:\\F{:03}.TXT", i), b"");
    }
    let mut p = Panel::new(&fs);
    p.set_rows(10);
    p.set_mode(Mode::Full);
    p.load(&mut fs, "C:\\", Focus::First).unwrap();
    p.set_cursor(4);
    p.page_down();
    assert_eq!(p.cursor - p.top, 4);
    assert_eq!(p.cursor, 14);
    p.end();
    p.page_down();
    assert_eq!(p.cursor, 99, "a root has no `..`: 100 names, 0 to 99");
    p.home();
    p.page_up();
    assert_eq!(p.cursor, 0);
}

#[test]
fn going_up_centres_the_folder_left() {
    let mut fs = MemFs::new(&['C']);
    for i in 0..60 {
        fs.add_dir(&format!("C:\\D{:02}", i));
    }
    let mut p = Panel::new(&fs);
    p.set_rows(10);
    p.set_mode(Mode::Full);
    p.load(&mut fs, "C:\\D40", Focus::First).unwrap();
    p.go_up(&mut fs).unwrap();
    assert_eq!(p.current().unwrap().name, "D40");
    assert_eq!(p.cursor - p.top, 5, "in the middle of the panel");
}

#[test]
fn up_is_never_selected_and_a_mask_takes_files_only() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    p.set_cursor(0);
    p.toggle();
    assert!(!p.has_selection());
    assert_eq!(p.select_mask("*.*", true, false), 4);
    assert_eq!(p.selected_names(), ["A.TXT", "GO.BAT", "README.TXT", "Z.TXT"]);
    assert_eq!(p.select_mask("*.txt", false, false), 3);
    assert_eq!(p.selected_names(), ["GO.BAT"]);
}

#[test]
fn the_targets_are_the_selection_else_the_cursor() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    p.set_cursor(0);
    assert!(p.targets().is_empty(), "never `..`");
    p.set_cursor(1);
    assert_eq!(p.targets()[0].name, "CITY");
    p.select_mask("z*", true, false);
    assert_eq!(p.targets().len(), 1);
    assert_eq!(p.targets()[0].name, "Z.TXT");
}

#[test]
fn the_selection_stays_through_a_reread_and_goes_with_the_folder() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    p.select_mask("*.txt", true, false);
    p.reread(&mut fs).unwrap();
    assert_eq!(p.selected_names().len(), 3);
    p.set_cursor(1);
    p.enter(&mut fs).unwrap();
    assert!(!p.has_selection());
}

#[test]
fn sorting_twice_reverses_and_keeps_the_cursor() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    let i = p.index_of("GO.BAT").unwrap();
    p.set_cursor(i);
    p.sort_by(Sort::Name);
    assert!(p.reverse);
    assert_eq!(p.entries[0].name, "..");
    assert_eq!(p.entries[1].name, "SOKO", "folders still first");
    assert_eq!(p.current().unwrap().name, "GO.BAT");
    p.sort_by(Sort::Ext);
    assert!(!p.reverse);
}

#[test]
fn quick_search_finds_by_the_start_and_wraps() {
    let mut fs = tree();
    let p = panel(&mut fs, "C:\\GAMES");
    assert_eq!(p.find_prefix("ow", 0).map(|i| p.entries[i].name.as_str()), Some("OWLFLY2"));
    let next = p.find_prefix("ow", 3).unwrap();
    assert_eq!(p.entries[next].name, "OWLFLY3");
    let wrapped = p.find_prefix("ow", 4).unwrap();
    assert_eq!(p.entries[wrapped].name, "OWLFLY2");
    assert_eq!(p.find_prefix("*fly3", 0).map(|i| p.entries[i].name.as_str()), Some("OWLFLY3"));
}

#[test]
fn natural_order_on_a_desktop() {
    assert_eq!(kc::panel::compare_names("file2", "file10", true), core::cmp::Ordering::Less);
    assert_eq!(kc::panel::compare_names("file2", "file10", false), core::cmp::Ordering::Greater);
}

#[test]
fn an_unreadable_folder_leaves_the_panel_where_it_was() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    fs.set_not_ready('D', true);
    assert!(p.go_drive(&mut fs, "D:\\").is_err());
    assert_eq!(p.dir, "C:\\GAMES");
    assert!(p.error.is_some());
}

#[test]
fn a_root_has_no_up() {
    let mut fs = tree();
    let mut p = panel(&mut fs, "C:\\GAMES");
    p.go_root(&mut fs).unwrap();
    assert_eq!(names(&p), ["GAMES"]);
    assert!(!p.go_up(&mut fs).unwrap(), "nowhere above a root");
    assert_eq!(p.dir, "C:\\");
    let mut empty = MemFs::new(&['E']);
    let mut q = Panel::new(&empty);
    q.load(&mut empty, "E:\\", Focus::First).unwrap();
    assert!(q.entries.is_empty() && q.current().is_none());
    q.reread(&mut empty).unwrap();
}
