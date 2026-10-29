//! A dialog: what it asks, in data - lines of text, perhaps one input
//! line, the choice for existing files, check boxes, and buttons (the
//! hotkey of a button the letter between `~`). OWLOSUI shows it as its
//! grey modal window and the person answers there (`chrome::open_form`,
//! D15); what they changed is read back into it when a button is pressed
//! (docs/05-file-operations.md).

use crate::job::{Answer, Existing, Op};
use crate::line::Line;
use alloc::string::String;
use alloc::vec::Vec;

/// What the answer is for.
#[derive(Clone, Debug, PartialEq)]
pub enum Purpose {
    MakeDir,
    Delete { trash: bool },
    /// Copy, or Rename/Move.
    Transfer { op: Op, here: bool },
    Select { on: bool },
    Filter,
    Colours,
    /// An Options dialog: its switches, in the order the page lists them.
    Options(crate::options::Page),
    EditNew,
    Quit,
    /// A question from the running job; the answers by button.
    Ask(Vec<Answer>),
    /// Esc while a job runs.
    StopJob,
    /// Words and an OK.
    Message,
    /// Menu file edit: Main, Local or Cancel.
    MenuFile,
    /// File attributes, for these paths.
    Attributes(Vec<String>),
    /// Find file.
    Find,
    /// The volume label of this drive.
    Label(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Normal,
    /// A name, the thing asked about.
    Bright,
    /// Right-aligned and quiet: `DOS error 27h`.
    Code,
}

#[derive(Clone)]
pub struct Dialog {
    pub purpose: Purpose,
    pub title: String,
    pub lines: Vec<(String, Tone)>,
    pub input: Option<Line>,
    /// How many of `lines` come before the input line.
    pub input_at: usize,
    /// "Already existing files": the Copy dialog's choice.
    pub existing: Option<Existing>,
    /// "Copy attributes and dates".
    pub check: Option<bool>,
    /// An Options dialog's switches: the words and whether it is on.
    pub switches: Vec<(String, bool)>,
    pub buttons: Vec<String>,
    pub error: bool,
}

pub enum Outcome {
    /// A button: its index.
    Button(usize),
    Cancel,
}

pub const EXISTING: [(Existing, &str); 5] = [
    (Existing::Ask, "Ask"),
    (Existing::Overwrite, "Overwrite"),
    (Existing::Skip, "Skip"),
    (Existing::OlderOnly, "Overwrite older"),
    (Existing::Rename, "Rename the new"),
];

impl Dialog {
    pub fn new(purpose: Purpose, title: &str) -> Dialog {
        Dialog {
            purpose,
            title: String::from(title),
            lines: Vec::new(),
            input: None,
            input_at: 0,
            existing: None,
            check: None,
            switches: Vec::new(),
            buttons: Vec::new(),
            error: false,
        }
    }

    /// The switches, after the lines written so far.
    pub fn switches(mut self, s: Vec<(String, bool)>) -> Dialog {
        if self.input.is_none() {
            // The lines before them, as for an input line.
            self.input_at = self.lines.len();
        }
        self.switches = s;
        self
    }

    pub fn line(mut self, s: &str, tone: Tone) -> Dialog {
        self.lines.push((String::from(s), tone));
        self
    }

    pub fn input(mut self, line: Line) -> Dialog {
        self.input_at = self.lines.len();
        self.input = Some(line);
        self
    }

    pub fn buttons(mut self, b: &[&str]) -> Dialog {
        self.buttons = b.iter().map(|s| String::from(*s)).collect();
        self
    }

    pub fn error(mut self) -> Dialog {
        self.error = true;
        self
    }

    pub fn text(&self) -> String {
        self.input.as_ref().map(|l| l.text()).unwrap_or_default()
    }

    /// The button whose hotkey is `c`, as `lang` shows the buttons.
    pub fn hotkey(&self, c: char, lang: &crate::lang::Lang) -> Option<usize> {
        let c: String = c.to_lowercase().collect();
        self.buttons.iter().position(|b| hot_letter(&lang.tr(b)).map(|h| h.to_lowercase().collect::<String>()) == Some(c.clone()))
    }
}

/// The letter between `~` in a button's text.
pub fn hot_letter(s: &str) -> Option<char> {
    let i = s.find('~')?;
    s[i + 1..].chars().next()
}

/// A button's text without the `~`.
pub fn plain(s: &str) -> String {
    s.chars().filter(|&c| c != '~').collect()
}
