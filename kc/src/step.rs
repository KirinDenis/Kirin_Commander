//! Stepping out (docs/07-command-line.md, D18): on DOS a program run from
//! the commander has only the memory the commander leaves it. A host
//! started with `/STEP` - by a batch file that says it will run what the
//! commander leaves behind - does not run a command line itself: it writes
//! the lines into `RUNPROG.BAT`, where it is, and what the commander was
//! into `KC.SAV` beside it, and leaves. The batch file runs `RUNPROG.BAT`
//! and starts the commander again, which reads `KC.SAV` once and is where
//! it was. The program has all of DOS's memory, as from the prompt.
//!
//! ```text
//! :again
//! KC.COM /STEP A:\ C:\
//! if not exist RUNPROG.BAT goto done
//! call RUNPROG.BAT
//! del RUNPROG.BAT
//! goto again
//! :done
//! ```

use crate::app::{App, Request};
use crate::path;
use alloc::string::String;
use alloc::vec::Vec;

/// What the batch file runs next.
pub const RUN_FILE: &str = "RUNPROG.BAT";
/// Where the commander was, read once by the next start.
pub const SAVE_FILE: &str = "KC.SAV";

/// `RUNPROG.BAT`'s text: each command line in its folder, through `call`
/// so a batch file returns, and back to `home` at the end.
pub fn batch(runs: &[(String, String)], home: &str, sep: char) -> String {
    let mut out = String::from("@echo off\r\n");
    let go = |out: &mut String, dir: &str| {
        let root = path::root(dir, sep);
        out.push_str(root.trim_end_matches(sep));
        out.push_str("\r\n");
        let rest = &dir[root.len()..];
        out.push_str(&alloc::format!("cd {}{}\r\n", sep, rest.trim_end_matches(sep)));
    };
    for (dir, line) in runs {
        go(&mut out, dir);
        out.push_str(&alloc::format!("call {}\r\n", line.trim()));
    }
    go(&mut out, home);
    out
}

impl App {
    /// The command lines the host was asked to run, out of the requests:
    /// `(folder, line)`. The other requests stay.
    pub fn take_runs(requests: &mut Vec<Request>) -> Vec<(String, String)> {
        let mut runs = Vec::new();
        requests.retain(|r| match r {
            Request::Run { dir, line } => {
                runs.push((dir.clone(), line.clone()));
                false
            }
            _ => true,
        });
        runs
    }

    /// Step out: `RUNPROG.BAT` and `KC.SAV` in `home`. False when either
    /// could not be written - then nothing is left behind, and the host
    /// runs the lines itself.
    pub fn step_out(&mut self, home: &str, runs: &[(String, String)]) -> bool {
        let sep = self.fs.sep();
        let run = path::join(home, RUN_FILE, sep);
        let save = path::join(home, SAVE_FILE, sep);
        let bat = crate::commands::encode(&*self.fs, &batch(runs, home, sep));
        let state = crate::commands::encode(&*self.fs, &crate::setup::session(self));
        if self.write_file(&save, &state) && self.write_file(&run, &bat) {
            return true;
        }
        let _ = self.fs.remove_file(&save);
        let _ = self.fs.remove_file(&run);
        false
    }

    /// Started again after a step: `KC.SAV` in `home`, read and taken away.
    /// False when there is none - a start of its own.
    pub fn resume(&mut self, home: &str) -> bool {
        let save = path::join(home, SAVE_FILE, self.fs.sep());
        let Some(text) = self.read_text(&save) else { return false };
        let _ = self.fs.remove_file(&save);
        crate::setup::read(self, &text);
        self.apply_options();
        true
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> bool {
        let fs = &mut *self.fs;
        fs.create(path)
            .and_then(|h| {
                let r = fs.write(h, data);
                let c = fs.close(h);
                r.and(c)
            })
            .is_ok()
    }
}

impl App {
    /// The panels on the folders the host was started with (`KC A:\ C:\`);
    /// one folder is both panels'. A folder that is not there is passed over.
    pub fn start_in(&mut self, dirs: &[String]) {
        for i in 0..2 {
            let Some(dir) = dirs.get(i).or(dirs.first()) else { return };
            let fs = &mut *self.fs;
            let _ = self.panels[i].load(fs, dir, crate::panel::Focus::First);
        }
    }
}
