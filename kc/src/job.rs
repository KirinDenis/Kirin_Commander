//! Copy, move and delete, done a step at a time (docs/05-file-operations.md).
//!
//! A job never runs to its end in one call: `step` does one small thing -
//! a folder made, 32 K copied, a name removed - and returns, so the screen
//! stays alive and the progress box moves (rule 3). When something goes
//! wrong, or a name is taken, the job stops in `Asking` with the question
//! in `ask`, and waits for `answer`; the step that failed is kept, so
//! Retry is the same step again (rule 4).

use crate::fs::*;
use crate::path;
use alloc::string::String;
use alloc::vec::Vec;

/// How much of a file one step copies.
pub const CHUNK: usize = 32 * 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Copy,
    Move,
    /// `trash`: to the Recycle Bin / Trash, where the system has one.
    Delete { trash: bool },
}

/// "Already existing files" in the Copy dialog.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Existing {
    Ask,
    Overwrite,
    Skip,
    OlderOnly,
    Rename,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ask {
    /// `doing` - "Cannot copy the file" - and the path and why.
    Error { doing: &'static str, path: String, err: FsError },
    /// The target is there: both, for the sizes and dates.
    Overwrite { src: Entry, dst: Entry, path: String },
    /// A read-only file to delete, or to overwrite.
    ReadOnly { path: String, overwrite: bool },
    /// A folder with something in it, to delete.
    NotEmpty { path: String },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    Retry,
    Skip,
    SkipAll,
    Cancel,
    /// Overwrite, delete: this one.
    Yes,
    /// Overwrite, delete: this one and every one after it.
    All,
    Append,
    Rename,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Working,
    Asking,
    Done,
}

#[derive(Clone, Debug)]
enum Step {
    /// One name: a file or a folder, from `src` to `dst` (`dst` empty for
    /// a delete). `top` is the name in the panel it came from, so the
    /// panel can unselect what was done.
    Item { src: String, dst: String, top: String, checked: bool, ro_ok: bool },
    /// A source to remove after its copy was made whole: a move across drives.
    RemoveFile { path: String, top: String },
    /// A folder whose contents are done: removed (move, delete).
    RemoveDir { path: String, top: String },
    /// A top-level name finished.
    Finished { top: String },
    /// The target folder, with every level above it that is not there:
    /// a name typed with `\` at its end, or several files to a new name.
    MakeDirs { path: String },
}

struct Copying {
    src: String,
    dst: String,
    from: Handle,
    to: Handle,
    size: u64,
    done: u64,
    stamp: Stamp,
    attrs: Attrs,
    /// A move: the source is removed after the copy is complete.
    then_remove: bool,
    top: String,
}

pub struct Job {
    pub op: Op,
    pub existing: Existing,
    pub copy_attrs: bool,
    pub ask: Option<Ask>,
    /// The file or folder being worked on, for the progress box.
    pub current: String,
    pub current_to: String,
    pub file_done: u64,
    pub file_size: u64,
    pub files_done: usize,
    pub files_total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
    /// The top-level names that were done whole.
    pub done: Vec<String>,
    /// The paths that were skipped, for the summary box.
    pub skipped: Vec<String>,
    pub cancelled: bool,
    steps: Vec<Step>,
    copying: Option<Copying>,
    /// The step the question is about, kept for Retry or Yes.
    held: Option<Step>,
    skip_all: Vec<ErrorKind>,
    /// A folder with files in it is deleted without asking: All was
    /// answered, or the question is turned off (Confirmations).
    pub(crate) yes_all: bool,
    /// A read-only file is deleted or overwritten without asking: the
    /// same, for that question.
    pub(crate) read_only_all: bool,
    /// A top-level name had a part skipped: not done whole.
    spoiled: Vec<String>,
    sep: char,
    case_sensitive: bool,
    buf: Vec<u8>,
}

impl Job {
    /// `items`: the names in `src_dir`. `target`: what the person typed -
    /// a folder, a new name for one file, or a mask for several (5.2).
    /// Nothing for a delete.
    pub fn new(fs: &mut dyn Fs, op: Op, src_dir: &str, items: &[Entry], target: &str) -> Result<Job, (ErrorKind, String)> {
        let sep = fs.sep();
        let mut job = Job {
            op,
            existing: Existing::Ask,
            copy_attrs: true,
            ask: None,
            current: String::new(),
            current_to: String::new(),
            file_done: 0,
            file_size: 0,
            files_done: 0,
            files_total: 0,
            bytes_done: 0,
            bytes_total: 0,
            done: Vec::new(),
            skipped: Vec::new(),
            cancelled: false,
            steps: Vec::new(),
            copying: None,
            held: None,
            skip_all: Vec::new(),
            yes_all: false,
            read_only_all: false,
            spoiled: Vec::new(),
            sep,
            case_sensitive: fs.case_sensitive(),
            buf: alloc::vec![0u8; CHUNK],
        };
        let mut plan: Vec<Step> = Vec::new();
        for e in items {
            let src = path::join(src_dir, &e.name, sep);
            let dst = match op {
                Op::Delete { .. } => String::new(),
                _ => resolve_target(fs, src_dir, target, &e.name, items.len()),
            };
            if !dst.is_empty() {
                if same_path(&src, &dst, job.case_sensitive) {
                    return Err((ErrorKind::Exists, String::from("Cannot copy a file onto itself")));
                }
                if e.dir && path::is_inside(&dst, &src, sep, job.case_sensitive) {
                    return Err((ErrorKind::IntoItself, String::from("Cannot copy a folder into itself")));
                }
            }
            plan.push(Step::Item { src, dst, top: e.name.clone(), checked: false, ro_ok: false });
            plan.push(Step::Finished { top: e.name.clone() });
            job.files_total += 1;
            job.bytes_total += e.size;
        }
        // The folders the targets go into, made first if they are not there.
        let mut dirs: Vec<String> = Vec::new();
        for s in &plan {
            if let Step::Item { dst, .. } = s {
                if let Some((parent, _)) = path::split(dst, sep) {
                    if fs.stat(&parent).is_err() && !dirs.contains(&parent) {
                        dirs.push(parent);
                    }
                }
            }
        }
        for d in dirs.into_iter().rev() {
            plan.insert(0, Step::MakeDirs { path: d });
        }
        plan.reverse();
        job.steps = plan;
        Ok(job)
    }

    /// Count what is inside the folders too, for the total bar. Can be
    /// skipped: then only the file bar is right.
    pub fn count(&mut self, fs: &mut dyn Fs) {
        let mut files = 0usize;
        let mut bytes = 0u64;
        let mut stack: Vec<String> = self.steps.iter().filter_map(|s| match s { Step::Item { src, .. } => Some(src.clone()), _ => None }).collect();
        while let Some(p) = stack.pop() {
            match fs.stat(&p) {
                Ok(e) if e.dir => {
                    if let Ok(list) = fs.read_dir(&p) {
                        for c in list {
                            stack.push(path::join(&p, &c.name, self.sep));
                        }
                    }
                }
                Ok(e) => {
                    files += 1;
                    bytes += e.size;
                }
                Err(_) => {}
            }
        }
        self.files_total = files;
        self.bytes_total = bytes;
    }

    pub fn status(&self) -> Status {
        if self.ask.is_some() {
            Status::Asking
        } else if self.steps.is_empty() && self.copying.is_none() {
            Status::Done
        } else {
            Status::Working
        }
    }

    /// One small piece of work.
    pub fn step(&mut self, fs: &mut dyn Fs) -> Status {
        if self.ask.is_some() {
            return Status::Asking;
        }
        if self.copying.is_some() {
            self.copy_chunk(fs);
            return self.status();
        }
        let Some(step) = self.steps.pop() else { return Status::Done };
        match step.clone() {
            Step::Finished { top } => {
                if !self.spoiled.iter().any(|s| *s == top) {
                    self.done.push(top);
                }
            }
            Step::RemoveDir { path, .. } => {
                self.current = path.clone();
                if let Err(e) = fs.remove_dir(&path) {
                    self.fail(step, "Cannot delete the folder", &path, e);
                }
            }
            Step::MakeDirs { path } => {
                let mut cur = String::from(path::root(&path, self.sep));
                for part in path[cur.len()..].split(self.sep).filter(|p| !p.is_empty()) {
                    cur = path::join(&cur, part, self.sep);
                    if fs.stat(&cur).is_ok() {
                        continue;
                    }
                    if let Err(e) = fs.make_dir(&cur) {
                        return {
                            self.fail(step, "Cannot make the folder", &cur, e);
                            self.status()
                        };
                    }
                }
            }
            Step::RemoveFile { path, .. } => {
                if let Err(e) = fs.remove_file(&path) {
                    self.fail(step, "Copied, but cannot delete the source", &path, e);
                }
            }
            Step::Item { src, dst, top, checked, ro_ok } => {
                self.current = src.clone();
                self.current_to = dst.clone();
                match self.op {
                    Op::Delete { trash } => self.delete_item(fs, step, &src, &top, trash, checked, ro_ok),
                    Op::Copy | Op::Move => self.copy_item(fs, step, &src, &dst, &top, checked, ro_ok),
                }
            }
        }
        self.status()
    }

    /// The answer to `ask`.
    pub fn answer(&mut self, fs: &mut dyn Fs, a: Answer) {
        let Some(ask) = self.ask.take() else { return };
        let held = self.held.take();
        match a {
            Answer::Cancel => {
                if let Some(s) = held {
                    self.skip(s);
                }
                self.cancel(fs);
            }
            Answer::Retry => {
                if let Some(s) = held {
                    self.steps.push(s);
                }
            }
            Answer::Skip | Answer::SkipAll => {
                if a == Answer::SkipAll {
                    match &ask {
                        Ask::Error { err, .. } => self.skip_all.push(err.kind),
                        Ask::Overwrite { .. } => self.existing = Existing::Skip,
                        _ => {}
                    }
                }
                if let Some(s) = held {
                    self.skip(s);
                }
            }
            Answer::Yes | Answer::All | Answer::Append | Answer::Rename => {
                if a == Answer::All {
                    match &ask {
                        Ask::Overwrite { .. } => self.existing = Existing::Overwrite,
                        Ask::ReadOnly { .. } => self.read_only_all = true,
                        Ask::NotEmpty { .. } => self.yes_all = true,
                        Ask::Error { .. } => {}
                    }
                }
                let Some(held) = held else { return };
                let Step::Item { src, dst, top, ro_ok, .. } = held else {
                    self.steps.push(held);
                    return;
                };
                match (&ask, a) {
                    (Ask::Overwrite { .. }, Answer::Rename) => {
                        let free = free_name(fs, &dst, self.sep);
                        self.steps.push(Step::Item { src, dst: free, top, checked: true, ro_ok });
                    }
                    (Ask::Overwrite { .. }, Answer::Append) => {
                        self.held = Some(Step::Item { src: src.clone(), dst: dst.clone(), top: top.clone(), checked: true, ro_ok });
                        self.start_copy(fs, &src, &dst, &top, true);
                    }
                    (Ask::ReadOnly { .. }, _) => self.steps.push(Step::Item { src, dst, top, checked: true, ro_ok: true }),
                    _ => self.steps.push(Step::Item { src, dst, top, checked: true, ro_ok }),
                }
            }
        }
    }

    /// Esc and Stop: the rest not done, the half-copied file taken away.
    pub fn stop(&mut self, fs: &mut dyn Fs) {
        self.ask = None;
        if let Some(s) = self.held.take() {
            self.skip(s);
        }
        self.cancel(fs);
    }

    fn cancel(&mut self, fs: &mut dyn Fs) {
        // A half-copied file is taken away, so nothing short is left
        // looking like a whole one; its source was not touched.
        if let Some(c) = self.copying.take() {
            let _ = fs.close(c.from);
            let _ = fs.close(c.to);
            let _ = fs.remove_file(&c.dst);
            self.spoil(&c.top);
        }
        // What was not reached is not done either.
        for s in core::mem::take(&mut self.steps) {
            if let Step::Item { top, .. } = s {
                self.spoil(&top);
            }
        }
        self.cancelled = true;
    }

    fn skip(&mut self, s: Step) {
        let (p, top) = match &s {
            Step::Item { src, top, .. } => (src.clone(), top.clone()),
            Step::RemoveDir { path, top } | Step::RemoveFile { path, top } => (path.clone(), top.clone()),
            Step::Finished { .. } => return,
            Step::MakeDirs { path } => {
                // Nothing can go into a folder that could not be made.
                self.skipped.push(path.clone());
                self.cancel_silently();
                return;
            }
        };
        self.skipped.push(p.clone());
        self.spoil(&top);
        // The folders this was in cannot be removed now: they would not be empty.
        let (sep, cs) = (self.sep, self.case_sensitive);
        self.steps.retain(|st| match st {
            Step::RemoveDir { path: d, .. } => !path::is_inside(&p, d, sep, cs),
            _ => true,
        });
    }

    fn cancel_silently(&mut self) {
        for s in core::mem::take(&mut self.steps) {
            if let Step::Item { top, .. } = s {
                self.spoil(&top);
            }
        }
    }

    fn spoil(&mut self, top: &str) {
        if !self.spoiled.iter().any(|s| s == top) {
            self.spoiled.push(String::from(top));
        }
    }

    fn fail(&mut self, step: Step, doing: &'static str, path: &str, err: FsError) {
        if self.skip_all.contains(&err.kind) {
            self.skip(step);
            return;
        }
        self.held = Some(step);
        self.ask = Some(Ask::Error { doing, path: String::from(path), err });
    }

    fn ask(&mut self, step: Step, ask: Ask) {
        self.held = Some(step);
        self.ask = Some(ask);
    }

    #[allow(clippy::too_many_arguments)]
    fn delete_item(&mut self, fs: &mut dyn Fs, step: Step, src: &str, top: &str, trash: bool, checked: bool, ro_ok: bool) {
        let e = match fs.stat(src) {
            Ok(e) => e,
            Err(err) => return self.fail(step, "Cannot delete", src, err),
        };
        if trash {
            match fs.trash(src) {
                Ok(()) => self.files_done += 1,
                Err(err) => self.fail(step, "Cannot move to the Recycle Bin", src, err),
            }
            return;
        }
        if e.dir {
            let list = match fs.read_dir(src) {
                Ok(l) => l,
                Err(err) => return self.fail(step, "Cannot read the folder", src, err),
            };
            // A top-level folder with something in it: asked a second
            // time, in the error colour (5.4).
            if !list.is_empty() && !checked && !self.yes_all {
                return self.ask(step, Ask::NotEmpty { path: String::from(src) });
            }
            self.steps.push(Step::RemoveDir { path: String::from(src), top: String::from(top) });
            for c in list.iter().rev() {
                let child = path::join(src, &c.name, self.sep);
                self.steps.push(Step::Item { src: child, dst: String::new(), top: String::from(top), checked: true, ro_ok });
            }
            return;
        }
        if e.attrs.read_only {
            if !ro_ok && !self.read_only_all {
                return self.ask(step, Ask::ReadOnly { path: String::from(src), overwrite: false });
            }
            let mut a = e.attrs;
            a.read_only = false;
            if let Err(err) = fs.set_attrs(src, a) {
                return self.fail(step, "Cannot delete the file", src, err);
            }
        }
        match fs.remove_file(src) {
            Ok(()) => {
                self.files_done += 1;
                self.bytes_done += e.size;
            }
            Err(err) => self.fail(step, "Cannot delete the file", src, err),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn copy_item(&mut self, fs: &mut dyn Fs, step: Step, src: &str, dst: &str, top: &str, checked: bool, ro_ok: bool) {
        let moving = self.op == Op::Move;
        let doing = if moving { "Cannot move" } else { "Cannot copy" };
        let e = match fs.stat(src) {
            Ok(e) => e,
            Err(err) => return self.fail(step, doing, src, err),
        };
        let there = fs.stat(dst).ok();
        if e.dir {
            // A move on one drive is a rename: nothing copied (5.2).
            if moving && there.is_none() {
                match fs.rename(src, dst) {
                    Ok(()) => return,
                    Err(err) if err.kind == ErrorKind::CrossDevice => {}
                    Err(err) => return self.fail(step, "Cannot move the folder", src, err),
                }
            }
            match &there {
                Some(t) if t.dir => {}
                Some(_) => return self.fail(step, "Cannot make the folder", dst, FsError::new(ErrorKind::Exists)),
                None => {
                    if let Err(err) = fs.make_dir(dst) {
                        return self.fail(step, "Cannot make the folder", dst, err);
                    }
                }
            }
            let list = match fs.read_dir(src) {
                Ok(l) => l,
                Err(err) => return self.fail(step, "Cannot read the folder", src, err),
            };
            if moving {
                self.steps.push(Step::RemoveDir { path: String::from(src), top: String::from(top) });
            }
            for c in list.iter().rev() {
                self.steps.push(Step::Item {
                    src: path::join(src, &c.name, self.sep),
                    dst: path::join(dst, &c.name, self.sep),
                    top: String::from(top),
                    checked: false,
                    ro_ok: false,
                });
            }
            return;
        }
        if let Some(t) = &there {
            if t.dir {
                return self.fail(step, doing, dst, FsError::new(ErrorKind::Exists));
            }
            if !checked {
                match self.existing {
                    Existing::Ask => {
                        return self.ask(step, Ask::Overwrite { src: e, dst: t.clone(), path: String::from(dst) });
                    }
                    Existing::Overwrite => {}
                    Existing::Skip => return self.skip(step),
                    Existing::OlderOnly => {
                        if t.stamp >= e.stamp {
                            return self.skip(step);
                        }
                    }
                    Existing::Rename => {
                        let free = free_name(fs, dst, self.sep);
                        self.steps.push(Step::Item { src: String::from(src), dst: free, top: String::from(top), checked: true, ro_ok });
                        return;
                    }
                }
            }
            // A read-only target is asked about even after All (5.3).
            if t.attrs.read_only {
                if !ro_ok && !self.read_only_all {
                    return self.ask(step, Ask::ReadOnly { path: String::from(dst), overwrite: true });
                }
                let mut a = t.attrs;
                a.read_only = false;
                if let Err(err) = fs.set_attrs(dst, a) {
                    return self.fail(step, doing, dst, err);
                }
            }
            if moving {
                // A rename onto a taken name: the old one goes first.
                if let Err(err) = fs.remove_file(dst) {
                    return self.fail(step, doing, dst, err);
                }
            }
        }
        if moving {
            match fs.rename(src, dst) {
                Ok(()) => {
                    self.files_done += 1;
                    self.bytes_done += e.size;
                    return;
                }
                Err(err) if err.kind == ErrorKind::CrossDevice => {}
                Err(err) => return self.fail(step, "Cannot move the file", src, err),
            }
        }
        self.held = Some(step);
        self.start_copy(fs, src, dst, top, false);
    }

    /// Open both ends; the bytes go over in `copy_chunk`. `self.held` is
    /// the step, for Retry if this fails.
    fn start_copy(&mut self, fs: &mut dyn Fs, src: &str, dst: &str, top: &str, append: bool) {
        let held = self.held.take().unwrap_or(Step::Item {
            src: String::from(src),
            dst: String::from(dst),
            top: String::from(top),
            checked: true,
            ro_ok: true,
        });
        let e = match fs.stat(src) {
            Ok(e) => e,
            Err(err) => return self.fail(held, "Cannot copy the file", src, err),
        };
        let from = match fs.open_read(src) {
            Ok(h) => h,
            Err(err) => return self.fail(held, "Cannot read the file", src, err),
        };
        let mut prefix: Vec<u8> = Vec::new();
        if append {
            // Append: what is there, then this file after it.
            if let Ok(h) = fs.open_read(dst) {
                loop {
                    match fs.read(h, &mut self.buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => prefix.extend_from_slice(&self.buf[..n]),
                    }
                }
                let _ = fs.close(h);
            }
        }
        let to = match fs.create(dst) {
            Ok(h) => h,
            Err(err) => {
                let _ = fs.close(from);
                return self.fail(held, "Cannot copy the file", dst, err);
            }
        };
        if !prefix.is_empty() {
            if let Err(err) = fs.write(to, &prefix) {
                let _ = fs.close(from);
                let _ = fs.close(to);
                return self.fail(held, "Cannot copy the file", dst, err);
            }
        }
        self.file_size = e.size;
        self.file_done = 0;
        self.copying = Some(Copying {
            src: String::from(src),
            dst: String::from(dst),
            from,
            to,
            size: e.size + prefix.len() as u64,
            done: 0,
            stamp: e.stamp,
            attrs: e.attrs,
            then_remove: self.op == Op::Move,
            top: String::from(top),
        });
    }

    fn copy_chunk(&mut self, fs: &mut dyn Fs) {
        let Some(c) = self.copying.as_mut() else { return };
        let n = match fs.read(c.from, &mut self.buf) {
            Ok(n) => n,
            Err(err) => return self.chunk_failed(fs, "Cannot read the file", true, err),
        };
        if n > 0 {
            if let Err(err) = fs.write(c.to, &self.buf[..n]) {
                return self.chunk_failed(fs, "Cannot copy the file", false, err);
            }
            c.done += n as u64;
            self.file_done = c.done;
            self.bytes_done += n as u64;
            return;
        }
        // The end: closed, dated, and - a move - the source removed only
        // now that the copy is whole (5.2).
        let c = self.copying.take().unwrap();
        let _ = fs.close(c.from);
        if let Err(err) = fs.close(c.to) {
            let _ = fs.remove_file(&c.dst);
            let step = Step::Item { src: c.src.clone(), dst: c.dst.clone(), top: c.top.clone(), checked: true, ro_ok: true };
            return self.fail(step, "Cannot copy the file", &c.dst, err);
        }
        if self.copy_attrs {
            let _ = fs.set_stamp(&c.dst, c.stamp);
            let mut a = c.attrs;
            a.archive = true;
            let _ = fs.set_attrs(&c.dst, a);
        }
        self.files_done += 1;
        self.file_done = 0;
        if c.then_remove {
            let whole = fs.stat(&c.dst).map(|e| e.size >= c.size).unwrap_or(false);
            if !whole {
                let step = Step::Item { src: c.src.clone(), dst: c.dst.clone(), top: c.top.clone(), checked: true, ro_ok: true };
                return self.fail(step, "The copy is short; the source was kept", &c.dst, FsError::new(ErrorKind::Io));
            }
            if c.attrs.read_only {
                let mut a = c.attrs;
                a.read_only = false;
                let _ = fs.set_attrs(&c.src, a);
            }
            self.steps.push(Step::RemoveFile { path: c.src, top: c.top });
        }
    }

    fn chunk_failed(&mut self, fs: &mut dyn Fs, doing: &'static str, reading: bool, err: FsError) {
        let c = self.copying.take().unwrap();
        let _ = fs.close(c.from);
        let _ = fs.close(c.to);
        // The short file is taken away; Retry starts the file again.
        let _ = fs.remove_file(&c.dst);
        self.bytes_done = self.bytes_done.saturating_sub(c.done);
        self.file_done = 0;
        let p = if reading { c.src.clone() } else { c.dst.clone() };
        let step = Step::Item { src: c.src, dst: c.dst, top: c.top, checked: true, ro_ok: true };
        self.fail(step, doing, &p, err);
    }
}

fn same_path(a: &str, b: &str, case_sensitive: bool) -> bool {
    if case_sensitive {
        a == b
    } else {
        a.to_lowercase() == b.to_lowercase()
    }
}

/// Where one name goes, from what was typed (5.2): into an existing folder;
/// into a folder made for it when the text ends in a separator; else the
/// new name for one item, or a mask (`*.bak`) for several. Relative to the
/// source folder.
pub fn resolve_target(fs: &mut dyn Fs, src_dir: &str, target: &str, name: &str, count: usize) -> String {
    let sep = fs.sep();
    let t = target.trim();
    let full = if path::root_len(t, sep) > 0 { String::from(t) } else { path::join(src_dir, t, sep) };
    if t.ends_with(sep) || t.ends_with('/') {
        let dir = String::from(full.trim_end_matches(|c| c == sep || c == '/'));
        let dir = if dir.is_empty() || dir.ends_with(':') { full.clone() } else { dir };
        return path::join(&dir, name, sep);
    }
    if let Ok(e) = fs.stat(&full) {
        if e.dir {
            return path::join(&full, name, sep);
        }
    }
    let last = path::name(&full, sep);
    if last.contains('*') || last.contains('?') {
        let (dir, _) = path::split(&full, sep).unwrap_or((String::from(src_dir), String::new()));
        return path::join(&dir, &apply_mask(name, &last), sep);
    }
    if count == 1 {
        return full;
    }
    // Several names and a name that is not a folder: a folder to make.
    path::join(&full, name, sep)
}

/// `readme.txt` through `*.bak` is `readme.bak`; through `new_*.*`, `new_readme.txt`.
pub fn apply_mask(name: &str, mask: &str) -> String {
    let (n, e) = path::split_ext(name);
    let (mn, me) = match mask.rfind('.') {
        Some(i) => (&mask[..i], Some(&mask[i + 1..])),
        None => (mask, None),
    };
    let part = |m: &str, src: &str| -> String {
        let mut out = String::new();
        let mut chars = src.chars();
        for c in m.chars() {
            match c {
                '*' => {
                    out.extend(chars.by_ref());
                }
                '?' => {
                    if let Some(x) = chars.next() {
                        out.push(x);
                    }
                }
                // A letter of the mask is put in, the name's letters kept (Far):
                // `new_*` over `readme` is `new_readme`.
                c => out.push(c),
            }
        }
        out
    };
    let mut out = part(mn, n);
    match me {
        Some(me) => {
            let ext = part(me, e);
            if !ext.is_empty() {
                out.push('.');
                out.push_str(&ext);
            }
        }
        None => {
            if !e.is_empty() && mn.ends_with('*') {
                out.push('.');
                out.push_str(e);
            }
        }
    }
    out
}

/// `name (2).ext`, or on 8.3 `NAME~1.EXT`: the first that is free.
pub fn free_name(fs: &mut dyn Fs, path_: &str, sep: char) -> String {
    let (dir, name) = path::split(path_, sep).unwrap_or((String::new(), String::from(path_)));
    let (n, e) = path::split_ext(&name);
    for k in 1..1000u32 {
        let candidate = if fs.short_names() {
            let num = alloc::format!("~{}", k);
            let keep = 8usize.saturating_sub(num.len()).min(n.len());
            let mut c = String::from(&n[..keep]);
            c.push_str(&num);
            if !e.is_empty() {
                c.push('.');
                c.push_str(e);
            }
            c
        } else {
            let mut c = alloc::format!("{} ({})", n, k + 1);
            if !e.is_empty() {
                c.push('.');
                c.push_str(e);
            }
            c
        };
        let p = path::join(&dir, &candidate, sep);
        if fs.stat(&p).is_err() {
            return p;
        }
    }
    String::from(path_)
}
