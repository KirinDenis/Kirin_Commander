//! Languages (docs/04-menus.md, Options > Languages): the commander's
//! words in another language, from a file beside the setup - `KCxxx.LNG`,
//! lines a person writes:
//!
//! ```text
//! ; Kirin Commander language file
//! [language]
//! name=Deutsch
//! [text]
//! ~C~opy=~K~opieren
//! Copy {} to=Kopiere {} nach
//! ```
//!
//! A line is the English words, `=`, the same words in the language. `{}`
//! stands for what changes - a name, a number - and the translation puts
//! it where its own grammar wants it. Words a file does not name stay
//! English, so a half-done translation is still a working commander.
//! English is no file: it is the words as they are written in the program.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Default)]
pub struct Lang {
    /// The language's name, as its own file says it: `Deutsch`. Empty for
    /// English.
    pub name: String,
    /// The file it came from, its name only: `KCDEU.LNG`.
    pub file: String,
    words: Vec<(String, String)>,
    /// The lines with `{}`: the English cut at each `{}`, and the
    /// translation cut the same way.
    patterns: Vec<(Vec<String>, Vec<String>)>,
}

impl Lang {
    /// A language file's text.
    pub fn parse(file: &str, text: &str) -> Lang {
        let mut lang = Lang { file: String::from(file), ..Lang::default() };
        let mut section = String::new();
        for line in text.lines() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() || line.trim_start().starts_with(';') {
                continue;
            }
            let t = line.trim();
            if t.starts_with('[') && t.ends_with(']') {
                section = t[1..t.len() - 1].trim().to_lowercase();
                continue;
            }
            let Some((from, to)) = line.split_once('=') else { continue };
            match section.as_str() {
                "language" if from.trim().eq_ignore_ascii_case("name") => lang.name = String::from(to.trim()),
                "text" => {
                    let (from, to) = (from.trim(), to.trim());
                    if from.contains("{}") {
                        let a = from.split("{}").map(String::from).collect::<Vec<String>>();
                        let b = to.split("{}").map(String::from).collect::<Vec<String>>();
                        // A translation keeps every `{}` of its English.
                        if a.len() == b.len() {
                            lang.patterns.push((a, b));
                        }
                    } else {
                        lang.words.push((String::from(from), String::from(to)));
                    }
                }
                _ => {}
            }
        }
        lang
    }

    /// The language's name for the Languages list.
    pub fn title(&self) -> &str {
        if self.name.is_empty() {
            "English"
        } else {
            &self.name
        }
    }

    pub fn is_english(&self) -> bool {
        self.words.is_empty() && self.patterns.is_empty()
    }

    /// The words in this language: as they are, if it has none for them.
    pub fn tr(&self, s: &str) -> String {
        if self.is_english() || s.trim().is_empty() {
            return String::from(s);
        }
        // Words with spaces around them - a dialog's centred line - keep
        // the spaces.
        let lead = s.len() - s.trim_start().len();
        let core = s.trim();
        let tail = s.len() - s.trim_end().len();
        let found = self.words.iter().find(|(f, _)| f == core).map(|(_, t)| t.clone()).or_else(|| self.pattern(core));
        match found {
            Some(t) => alloc::format!("{}{}{}", &s[..lead], t, &s[s.len() - tail..]),
            None => String::from(s),
        }
    }

    fn pattern(&self, s: &str) -> Option<String> {
        for (from, to) in &self.patterns {
            if let Some(parts) = capture(from, s) {
                let mut out = String::new();
                for (i, piece) in to.iter().enumerate() {
                    out.push_str(piece);
                    if let Some(p) = parts.get(i) {
                        out.push_str(p);
                    }
                }
                return Some(out);
            }
        }
        None
    }
}

/// What stands where each `{}` is, if `s` is `pieces` with something
/// between them: `["Copy ", " to"]` and `Copy 3 files to` give `3 files`.
fn capture(pieces: &[String], s: &str) -> Option<Vec<String>> {
    let first = pieces.first()?;
    let last = pieces.last()?;
    if !s.starts_with(first.as_str()) || s.len() < first.len() + last.len() || !s.ends_with(last.as_str()) {
        return None;
    }
    let mut rest = &s[first.len()..s.len() - last.len()];
    let mut out = Vec::new();
    for piece in &pieces[1..pieces.len() - 1] {
        let at = rest.find(piece.as_str())?;
        out.push(String::from(&rest[..at]));
        rest = &rest[at + piece.len()..];
    }
    out.push(String::from(rest));
    Some(out)
}

/// Whether a file's name is a language file's: `KC*.LNG`, any case.
pub fn is_lang_file(name: &str) -> bool {
    let n = name.to_ascii_uppercase();
    n.starts_with("KC") && n.ends_with(".LNG")
}

impl Lang {
    /// A dialog's words in this language, the rest of it as it is.
    pub fn dialog(&self, d: &crate::dialog::Dialog) -> crate::dialog::Dialog {
        let mut t = d.clone();
        t.title = self.tr(&d.title);
        for (s, _) in t.lines.iter_mut() {
            *s = self.tr(s);
        }
        for b in t.buttons.iter_mut() {
            *b = self.tr(b);
        }
        for (s, _) in t.switches.iter_mut() {
            *s = self.tr(s);
        }
        t
    }
}

impl Lang {
    /// A menu's words in this language: the items, their hints, their
    /// submenus.
    pub fn menu(&self, items: &mut [owlosui_core::MenuItem]) {
        for it in items.iter_mut() {
            it.text = self.tr(&it.text);
            it.hint = self.tr(&it.hint);
            self.menu(&mut it.items);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_and_patterns_turn_and_the_rest_stays() {
        let l = Lang::parse("KCDEU.LNG", "[language]\nname=Deutsch\n[text]\nHelp=Hilfe\nCopy {} to {}=Nach {1} {}\n{} bytes free={} Bytes frei\n");
        assert_eq!(l.title(), "Deutsch");
        assert_eq!(l.tr("Help"), "Hilfe");
        assert_eq!(l.tr("  Help  "), "  Hilfe  ");
        assert_eq!(l.tr(" 1 234 bytes free"), " 1 234 Bytes frei");
        assert_eq!(l.tr("Delete"), "Delete");
        // A translation that loses a `{}` is not taken.
        assert_eq!(l.tr("Copy a to b"), "Copy a to b");
    }
}
