//! One line of text being typed: the command line, a dialog's input.
//! Characters, not glyphs - the screen turns them into glyphs when it draws.

use alloc::string::String;
use alloc::vec::Vec;
use owlosui_core::{Key, KeyCode};

#[derive(Clone, Debug, Default)]
pub struct Line {
    text: Vec<char>,
    pub caret: usize,
    /// The whole text selected, as a dialog's line comes: typing replaces
    /// it, an arrow first lets it be edited (NC's rule).
    pub all: bool,
    /// The first character shown, when the line is longer than its place.
    pub scroll: usize,
    history: Vec<String>,
    /// Where Up/Down are in `history`; `None` while typing a new line.
    back: Option<usize>,
}

impl Line {
    pub fn new(s: &str) -> Line {
        let text: Vec<char> = s.chars().collect();
        Line { caret: text.len(), text, ..Line::default() }
    }

    /// Filled, all of it selected.
    pub fn selected(s: &str) -> Line {
        let mut l = Line::new(s);
        l.all = true;
        l
    }

    pub fn text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn set(&mut self, s: &str) {
        self.text = s.chars().collect();
        self.caret = self.text.len();
        self.all = false;
    }

    pub fn clear(&mut self) {
        self.set("");
        self.back = None;
    }

    /// Text put in at the caret: Ctrl+Enter's name, a paste.
    pub fn insert(&mut self, s: &str) {
        if self.all {
            self.text.clear();
            self.caret = 0;
            self.all = false;
        }
        for c in s.chars() {
            if c == '\r' || c == '\n' {
                continue;
            }
            self.text.insert(self.caret, c);
            self.caret += 1;
        }
    }

    /// Remembered for Up and Down (Alt+F8 lists them).
    pub fn remember(&mut self, s: &str) {
        if s.trim().is_empty() {
            return;
        }
        self.history.retain(|h| h != s);
        self.history.push(String::from(s));
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        self.back = None;
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// A key for the line; false if it was not one. `arrows`: Left, Right,
    /// Home and End belong to the line (a dialog's line; the command line
    /// only while it has text).
    pub fn key(&mut self, k: Key, arrows: bool) -> bool {
        let plain = !k.mods.ctrl && !k.mods.alt;
        match k.code {
            KeyCode::Char(c) if plain && !c.is_control() => {
                let mut b = [0u8; 4];
                self.insert(c.encode_utf8(&mut b));
                true
            }
            KeyCode::Backspace if plain => {
                if self.all {
                    self.clear();
                } else if self.caret > 0 {
                    self.caret -= 1;
                    self.text.remove(self.caret);
                }
                true
            }
            KeyCode::Delete if plain && arrows => {
                if self.all {
                    self.clear();
                } else if self.caret < self.text.len() {
                    self.text.remove(self.caret);
                }
                true
            }
            KeyCode::Left if arrows && plain => {
                self.all = false;
                self.caret = self.caret.saturating_sub(1);
                true
            }
            KeyCode::Right if arrows && plain => {
                self.all = false;
                self.caret = (self.caret + 1).min(self.text.len());
                true
            }
            KeyCode::Left if arrows && k.mods.ctrl => {
                self.all = false;
                while self.caret > 0 && self.text[self.caret - 1] == ' ' {
                    self.caret -= 1;
                }
                while self.caret > 0 && self.text[self.caret - 1] != ' ' {
                    self.caret -= 1;
                }
                true
            }
            KeyCode::Right if arrows && k.mods.ctrl => {
                self.all = false;
                while self.caret < self.text.len() && self.text[self.caret] != ' ' {
                    self.caret += 1;
                }
                while self.caret < self.text.len() && self.text[self.caret] == ' ' {
                    self.caret += 1;
                }
                true
            }
            KeyCode::Home if arrows => {
                self.all = false;
                self.caret = 0;
                true
            }
            KeyCode::End if arrows => {
                self.all = false;
                self.caret = self.text.len();
                true
            }
            // Ctrl+Y: the line emptied (NC and Far).
            KeyCode::Char('y') if k.mods.ctrl => {
                self.clear();
                true
            }
            // Ctrl+E / Ctrl+X: back and forth through what was typed before.
            KeyCode::Char('e') if k.mods.ctrl => {
                self.older();
                true
            }
            KeyCode::Char('x') if k.mods.ctrl => {
                self.newer();
                true
            }
            _ => false,
        }
    }

    pub fn older(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let i = match self.back {
            None => self.history.len() - 1,
            Some(0) => 0,
            Some(i) => i - 1,
        };
        self.back = Some(i);
        let h = self.history[i].clone();
        self.set(&h);
    }

    pub fn newer(&mut self) {
        match self.back {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.back = None;
                self.set("");
            }
            Some(i) => {
                self.back = Some(i + 1);
                let h = self.history[i + 1].clone();
                self.set(&h);
            }
        }
    }

    /// What `width` cells show, and where in them the caret is.
    pub fn view(&mut self, width: usize) -> (String, usize) {
        if width == 0 {
            return (String::new(), 0);
        }
        if self.caret < self.scroll {
            self.scroll = self.caret;
        }
        if self.caret >= self.scroll + width {
            self.scroll = self.caret + 1 - width;
        }
        let s: String = self.text.iter().skip(self.scroll).take(width).collect();
        (s, self.caret - self.scroll)
    }
}
