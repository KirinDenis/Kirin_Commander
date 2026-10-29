//! The user menu (F2) and the extension table (Enter on a file), the
//! two files a person writes commands into (docs/03-keys.md 3.5,
//! docs/04-menus.md). Both speak NC's language for the file under the
//! cursor: `!.!` its name, `!` the name without the extension, `!:` the
//! drive, `!\` the folder with its `\` at the end, `!!` a `!`.
//!
//! `KC.MNU` - an item is a line at the left edge, its key, `:` and its
//! words; the commands under it are the lines that start with a space:
//!
//! ```text
//! A: Assemble the file
//!     fasm !.! !.com
//! R: Run it
//!     !.com
//! ```
//!
//! `KC.EXT` - an extension (or several), `:` and the command:
//!
//! ```text
//! asm inc: edit !.!
//! zip: unzip -l !.!
//! ```

use crate::path;
use alloc::string::String;
use alloc::vec::Vec;

pub const MENU_FILE: &str = "KC.MNU";
pub const EXT_FILE: &str = "KC.EXT";

/// One item of a user menu.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// The key that chooses it, if its line gave one.
    pub key: Option<char>,
    pub title: String,
    pub commands: Vec<String>,
}

/// A menu file's items, in its order.
pub fn parse_menu(text: &str) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() || line.trim_start().starts_with(';') {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(it) = items.last_mut() {
                it.commands.push(String::from(line.trim()));
            }
            continue;
        }
        let (key, title) = match line.split_once(':') {
            // `A: Assemble`: one letter before the colon is its key.
            Some((k, t)) if k.trim().chars().count() == 1 => (k.trim().chars().next(), t.trim()),
            Some((k, t)) if k.trim().is_empty() => (None, t.trim()),
            _ => (None, line.trim()),
        };
        items.push(Item { key, title: String::from(title), commands: Vec::new() });
    }
    items
}

/// The command the extension table gives a file, if it names its
/// extension.
pub fn ext_command(text: &str, name: &str) -> Option<String> {
    let ext = path::split_ext(name).1.to_lowercase();
    if ext.is_empty() {
        return None;
    }
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let Some((exts, cmd)) = line.split_once(':') else { continue };
        if exts.split_whitespace().any(|e| e.trim_start_matches('.').eq_ignore_ascii_case(&ext)) {
            return Some(String::from(cmd.trim()));
        }
    }
    None
}

/// A command with NC's `!` words put in, for the file `name` in `dir`.
pub fn expand(cmd: &str, dir: &str, name: &str, sep: char) -> String {
    let (stem, _) = path::split_ext(name);
    let drive = path::root(dir, sep).trim_end_matches(sep);
    let mut folder = String::from(dir);
    if !folder.ends_with(sep) {
        folder.push(sep);
    }
    let mut out = String::new();
    let mut rest = cmd;
    while let Some(at) = rest.find('!') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        if let Some(r) = after.strip_prefix(".!") {
            out.push_str(name);
            rest = r;
        } else if let Some(r) = after.strip_prefix('!') {
            out.push('!');
            rest = r;
        } else if let Some(r) = after.strip_prefix(':') {
            out.push_str(drive);
            rest = r;
        } else if let Some(r) = after.strip_prefix(sep) {
            out.push_str(&folder);
            rest = r;
        } else {
            out.push_str(stem);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}
