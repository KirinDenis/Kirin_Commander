//! Copy, move and delete on a file system that fails where the test says
//! (docs/05-file-operations.md).

use kc::fs::{Attrs, ErrorKind, Fs};
use kc::job::{apply_mask, Answer, Ask, Existing, Job, Op, Status};
use kc::{Entry, MemFs};

fn fs() -> MemFs {
    let mut fs = MemFs::new(&['C', 'D']);
    fs.add_file("C:\\SRC\\A.TXT", b"alpha");
    fs.add_file("C:\\SRC\\B.TXT", &vec![7u8; 100_000]);
    fs.add_file("C:\\SRC\\SUB\\C.TXT", b"charlie");
    fs.add_file("C:\\SRC\\SUB\\DEEP\\D.TXT", b"delta");
    fs.add_dir("D:\\DST");
    fs
}

fn item(fs: &mut MemFs, p: &str) -> Entry {
    fs.stat(p).unwrap()
}

/// To the end, answering every question with `answer`.
fn run(job: &mut Job, fs: &mut MemFs, answer: Answer) -> Vec<Ask> {
    let mut asked = Vec::new();
    for _ in 0..10_000 {
        match job.step(fs) {
            Status::Done => return asked,
            Status::Asking => {
                asked.push(job.ask.clone().unwrap());
                job.answer(fs, answer);
            }
            Status::Working => {}
        }
    }
    panic!("the job never ended");
}

#[test]
fn a_folder_copied_whole_across_drives() {
    let mut fs = fs();
    let items = [item(&mut fs, "C:\\SRC\\SUB"), item(&mut fs, "C:\\SRC\\B.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    job.count(&mut fs);
    assert_eq!(job.files_total, 3);
    let asked = run(&mut job, &mut fs, Answer::Cancel);
    assert!(asked.is_empty());
    assert_eq!(fs.data("D:\\DST\\SUB\\DEEP\\D.TXT"), Some(&b"delta"[..]));
    assert_eq!(fs.data("D:\\DST\\B.TXT").map(|d| d.len()), Some(100_000));
    assert_eq!(job.done, ["SUB", "B.TXT"]);
    assert!(fs.exists("C:\\SRC\\SUB"), "a copy leaves the source");
}

#[test]
fn a_big_file_goes_in_steps() {
    let mut fs = fs();
    let items = [item(&mut fs, "C:\\SRC\\B.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST\\").unwrap();
    let mut steps = 0;
    while job.step(&mut fs) != Status::Done {
        steps += 1;
    }
    assert!(steps >= 4, "100 000 bytes in 32 K pieces: {} steps", steps);
}

#[test]
fn move_on_one_drive_is_a_rename() {
    let mut fs = fs();
    let items = [item(&mut fs, "C:\\SRC\\SUB")];
    let mut job = Job::new(&mut fs, Op::Move, "C:\\SRC", &items, "C:\\").unwrap();
    run(&mut job, &mut fs, Answer::Cancel);
    assert!(fs.exists("C:\\SUB\\DEEP\\D.TXT"));
    assert!(!fs.exists("C:\\SRC\\SUB"));
    assert_eq!(job.files_done, 0, "nothing copied");
}

#[test]
fn move_across_drives_copies_then_deletes() {
    let mut fs = fs();
    let items = [item(&mut fs, "C:\\SRC\\SUB")];
    let mut job = Job::new(&mut fs, Op::Move, "C:\\SRC", &items, "D:\\DST").unwrap();
    run(&mut job, &mut fs, Answer::Cancel);
    assert!(fs.exists("D:\\DST\\SUB\\DEEP\\D.TXT"));
    assert!(!fs.exists("C:\\SRC\\SUB"), "the source folder gone once its files are across");
    assert_eq!(job.done, ["SUB"]);
}

#[test]
fn a_move_that_stops_leaves_every_file_somewhere() {
    let mut fs = fs();
    fs.set_capacity('D', 10);
    let items = [item(&mut fs, "C:\\SRC\\SUB")];
    let mut job = Job::new(&mut fs, Op::Move, "C:\\SRC", &items, "D:\\DST").unwrap();
    let asked = run(&mut job, &mut fs, Answer::Skip);
    assert!(asked.iter().any(|a| matches!(a, Ask::Error { err, .. } if err.kind == ErrorKind::DiskFull)));
    // C.TXT (7 bytes) fitted and moved; D.TXT did not, and is still at the source.
    assert!(fs.exists("D:\\DST\\SUB\\C.TXT"));
    assert!(!fs.exists("C:\\SRC\\SUB\\C.TXT"));
    assert!(fs.exists("C:\\SRC\\SUB\\DEEP\\D.TXT"));
    assert!(!fs.exists("D:\\DST\\SUB\\DEEP\\D.TXT"), "no short file left at the target");
    assert!(job.done.is_empty(), "SUB is not done whole: it stays selected");
}

#[test]
fn overwrite_is_asked_and_all_stops_asking() {
    let mut fs = fs();
    fs.add_file("D:\\DST\\A.TXT", b"old");
    fs.add_file("D:\\DST\\B.TXT", b"old");
    let items = [item(&mut fs, "C:\\SRC\\A.TXT"), item(&mut fs, "C:\\SRC\\B.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    let asked = run(&mut job, &mut fs, Answer::All);
    assert_eq!(asked.len(), 1);
    assert!(matches!(&asked[0], Ask::Overwrite { path, .. } if path == "D:\\DST\\A.TXT"));
    assert_eq!(fs.data("D:\\DST\\A.TXT"), Some(&b"alpha"[..]));
    assert_eq!(fs.data("D:\\DST\\B.TXT").map(|d| d.len()), Some(100_000));
}

#[test]
fn skip_keeps_the_old_file_and_the_name_selected() {
    let mut fs = fs();
    fs.add_file("D:\\DST\\A.TXT", b"old");
    let items = [item(&mut fs, "C:\\SRC\\A.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    run(&mut job, &mut fs, Answer::Skip);
    assert_eq!(fs.data("D:\\DST\\A.TXT"), Some(&b"old"[..]));
    assert!(job.done.is_empty());
}

#[test]
fn rename_and_append() {
    let mut fs = fs();
    fs.short_names = false;
    fs.add_file("D:\\DST\\A.TXT", b"old");
    let items = [item(&mut fs, "C:\\SRC\\A.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    run(&mut job, &mut fs, Answer::Rename);
    assert_eq!(fs.data("D:\\DST\\A (2).TXT"), Some(&b"alpha"[..]));
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    run(&mut job, &mut fs, Answer::Append);
    assert_eq!(fs.data("D:\\DST\\A.TXT"), Some(&b"oldalpha"[..]));
}

#[test]
fn eight_three_renames_with_a_tilde() {
    let mut fs = fs();
    fs.add_file("D:\\DST\\A.TXT", b"old");
    let items = [item(&mut fs, "C:\\SRC\\A.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    job.existing = Existing::Rename;
    run(&mut job, &mut fs, Answer::Cancel);
    assert!(fs.exists("D:\\DST\\A~1.TXT"));
}

#[test]
fn a_read_only_target_is_asked_even_after_all() {
    let mut fs = fs();
    fs.add_file("D:\\DST\\A.TXT", b"old");
    fs.set_attrs_of("D:\\DST\\A.TXT", Attrs { read_only: true, ..Attrs::default() });
    let items = [item(&mut fs, "C:\\SRC\\A.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    job.existing = Existing::Overwrite;
    let asked = run(&mut job, &mut fs, Answer::Yes);
    assert!(matches!(&asked[..], [Ask::ReadOnly { overwrite: true, .. }]));
    assert_eq!(fs.data("D:\\DST\\A.TXT"), Some(&b"alpha"[..]));
}

#[test]
fn retry_after_the_disk_came_back() {
    let mut fs = fs();
    fs.fail_on("D:\\DST\\A.TXT", ErrorKind::NotReady, 1);
    let items = [item(&mut fs, "C:\\SRC\\A.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    let asked = run(&mut job, &mut fs, Answer::Retry);
    assert_eq!(asked.len(), 1);
    assert_eq!(fs.data("D:\\DST\\A.TXT"), Some(&b"alpha"[..]));
    assert_eq!(job.done, ["A.TXT"]);
}

#[test]
fn skip_all_skips_the_same_kind_without_asking() {
    let mut fs = fs();
    fs.fail_on("D:\\DST\\A.TXT", ErrorKind::Denied, 0);
    fs.fail_on("D:\\DST\\B.TXT", ErrorKind::Denied, 0);
    let items = [item(&mut fs, "C:\\SRC\\A.TXT"), item(&mut fs, "C:\\SRC\\B.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    let asked = run(&mut job, &mut fs, Answer::SkipAll);
    assert_eq!(asked.len(), 1);
    assert_eq!(job.skipped.len(), 2);
}

#[test]
fn cancel_mid_file_leaves_no_short_file() {
    let mut fs = fs();
    let items = [item(&mut fs, "C:\\SRC\\B.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST").unwrap();
    job.step(&mut fs);
    job.step(&mut fs);
    assert!(fs.exists("D:\\DST\\B.TXT"));
    job.stop(&mut fs);
    assert_eq!(job.status(), Status::Done);
    assert!(!fs.exists("D:\\DST\\B.TXT"));
    assert!(fs.exists("C:\\SRC\\B.TXT"));
}

#[test]
fn onto_itself_and_into_itself_are_refused_before_anything() {
    let mut fs = fs();
    let items = [item(&mut fs, "C:\\SRC\\A.TXT")];
    assert!(Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "C:\\SRC").is_err());
    let items = [item(&mut fs, "C:\\SRC\\SUB")];
    assert!(Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "C:\\SRC\\SUB\\DEEP").is_err());
}

#[test]
fn delete_asks_about_a_full_folder_and_a_read_only_file() {
    let mut fs = fs();
    fs.set_attrs_of("C:\\SRC\\SUB\\C.TXT", Attrs { read_only: true, ..Attrs::default() });
    let items = [item(&mut fs, "C:\\SRC\\SUB")];
    let mut job = Job::new(&mut fs, Op::Delete { trash: false }, "C:\\SRC", &items, "").unwrap();
    let asked = run(&mut job, &mut fs, Answer::Yes);
    assert!(matches!(asked[0], Ask::NotEmpty { .. }));
    assert!(matches!(asked[1], Ask::ReadOnly { overwrite: false, .. }));
    assert_eq!(asked.len(), 2, "the folders inside are not asked about again");
    assert!(!fs.exists("C:\\SRC\\SUB"));
}

#[test]
fn a_skipped_file_keeps_its_folders() {
    let mut fs = fs();
    fs.fail_on("C:\\SRC\\SUB\\DEEP\\D.TXT", ErrorKind::InUse, 0);
    let items = [item(&mut fs, "C:\\SRC\\SUB")];
    let mut job = Job::new(&mut fs, Op::Delete { trash: false }, "C:\\SRC", &items, "").unwrap();
    let mut asked = 0;
    loop {
        match job.step(&mut fs) {
            Status::Done => break,
            Status::Asking => {
                asked += 1;
                let a = if matches!(job.ask, Some(Ask::NotEmpty { .. })) { Answer::Yes } else { Answer::Skip };
                job.answer(&mut fs, a);
            }
            Status::Working => {}
        }
    }
    assert_eq!(asked, 2, "the folder question, then the one error - not a NotEmpty for each folder above");
    assert!(fs.exists("C:\\SRC\\SUB\\DEEP\\D.TXT"));
    assert!(!fs.exists("C:\\SRC\\SUB\\C.TXT"));
    assert_eq!(job.skipped, ["C:\\SRC\\SUB\\DEEP\\D.TXT"]);
}

#[test]
fn targets_resolve_as_the_dialog_says() {
    let mut fs = fs();
    // One file and a new name: a copy under that name, here.
    let items = [item(&mut fs, "C:\\SRC\\A.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "A2.TXT").unwrap();
    run(&mut job, &mut fs, Answer::Cancel);
    assert_eq!(fs.data("C:\\SRC\\A2.TXT"), Some(&b"alpha"[..]));
    // A name ending in `\`: a folder, made.
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\NEW\\").unwrap();
    let asked = run(&mut job, &mut fs, Answer::Cancel);
    assert!(asked.is_empty() || fs.exists("D:\\NEW\\A.TXT"), "{:?}", asked);
    // Several and a mask.
    let items = [item(&mut fs, "C:\\SRC\\A.TXT"), item(&mut fs, "C:\\SRC\\B.TXT")];
    let mut job = Job::new(&mut fs, Op::Copy, "C:\\SRC", &items, "D:\\DST\\*.BAK").unwrap();
    run(&mut job, &mut fs, Answer::Cancel);
    assert!(fs.exists("D:\\DST\\A.BAK") && fs.exists("D:\\DST\\B.BAK"));
    assert_eq!(apply_mask("readme.txt", "new_*.*"), "new_readme.txt");
}
