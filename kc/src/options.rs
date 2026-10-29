//! The options (docs/04-menus.md, Options): one table that the Options
//! dialogs show, Save setup writes and the start reads back. Each switch is
//! named after what it does, not after a program that had it.

use alloc::string::String;
use alloc::vec::Vec;

/// A switch. The number is its place in `Options`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum O {
    // Configuration
    InsertMovesDown,
    SearchWithoutAlt,
    EscHidesPanels,
    // Panel settings
    Hidden,
    SelectFolders,
    NamesCase,
    ColumnTitles,
    MiniStatus,
    // Confirmations
    ConfirmCopy,
    ConfirmMove,
    ConfirmOverwrite,
    ConfirmDelete,
    ConfirmNotEmpty,
    ConfirmReadOnly,
    ConfirmQuit,
    // Viewer and editor
    Wrap,
    Syntax,
    Numbers,
    WordStar,
    OfferWords,
    // The Options menu's own switches
    MenuBar,
    KeyBar,
    Clock,
}

/// One switch as the table has it: the switch, its name in the setup file,
/// its words in a dialog, and how it starts.
pub struct Def {
    pub o: O,
    pub key: &'static str,
    pub label: &'static str,
    pub default: bool,
}

const fn d(o: O, key: &'static str, label: &'static str, default: bool) -> Def {
    Def { o, key, label, default }
}

/// Every switch, in `O`'s order.
pub const ALL: [Def; 23] = [
    d(O::InsertMovesDown, "insert_moves_down", "Insert moves the cursor down", true),
    d(O::SearchWithoutAlt, "search_without_alt", "Letters without Alt start quick search", false),
    d(O::EscHidesPanels, "esc_hides_panels", "Esc on an empty command line hides the panels", false),
    d(O::Hidden, "hidden", "Show hidden and system files", true),
    d(O::SelectFolders, "select_folders", "Gray + - * select folders too", false),
    d(O::NamesCase, "names_case", "Folders in upper case, files in lower case", true),
    d(O::ColumnTitles, "column_titles", "Column titles", true),
    d(O::MiniStatus, "mini_status", "Mini status", true),
    d(O::ConfirmCopy, "confirm_copy", "Copy", true),
    d(O::ConfirmMove, "confirm_move", "Rename or move", true),
    d(O::ConfirmOverwrite, "confirm_overwrite", "Overwrite", true),
    d(O::ConfirmDelete, "confirm_delete", "Delete", true),
    d(O::ConfirmNotEmpty, "confirm_not_empty", "Delete a folder with files in it", true),
    d(O::ConfirmReadOnly, "confirm_read_only", "Delete or overwrite a read-only file", true),
    d(O::ConfirmQuit, "confirm_quit", "Quit", true),
    d(O::Wrap, "wrap", "Wrap long lines", false),
    d(O::Syntax, "syntax", "Colour code by its language", true),
    d(O::Numbers, "line_numbers", "Line numbers", false),
    d(O::WordStar, "wordstar_keys", "WordStar keys (Ctrl+Y deletes a line)", false),
    d(O::OfferWords, "offer_words", "Offer words while typing code", true),
    d(O::MenuBar, "menu_bar", "Menu bar always visible", false),
    d(O::KeyBar, "key_bar", "Key bar", true),
    d(O::Clock, "clock", "Clock", true),
];

/// The Options dialogs, each a page of switches.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Configuration,
    Panels,
    Confirmations,
    Editor,
}

impl Page {
    pub fn title(self) -> &'static str {
        match self {
            Page::Configuration => "Configuration",
            Page::Panels => "Panel settings",
            Page::Confirmations => "Confirmations",
            Page::Editor => "Viewer and editor",
        }
    }

    /// What the dialog says above its switches.
    pub fn intro(self) -> &'static str {
        match self {
            Page::Configuration => "The keys",
            Page::Panels => "What the panels show",
            Page::Confirmations => "Ask before",
            Page::Editor => "How a file opens in the viewer and the editor",
        }
    }

    /// The switches on this page. Upper and lower case is DOS's alone:
    /// a long name is shown as it is.
    pub fn switches(self, short_names: bool) -> Vec<O> {
        match self {
            Page::Configuration => alloc::vec![O::InsertMovesDown, O::SearchWithoutAlt, O::EscHidesPanels],
            Page::Panels => {
                let mut v = alloc::vec![O::Hidden, O::SelectFolders];
                if short_names {
                    v.push(O::NamesCase);
                }
                v.push(O::ColumnTitles);
                v.push(O::MiniStatus);
                v
            }
            Page::Confirmations => alloc::vec![
                O::ConfirmCopy,
                O::ConfirmMove,
                O::ConfirmOverwrite,
                O::ConfirmDelete,
                O::ConfirmNotEmpty,
                O::ConfirmReadOnly,
                O::ConfirmQuit
            ],
            Page::Editor => alloc::vec![O::Wrap, O::Syntax, O::Numbers, O::WordStar, O::OfferWords],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Options {
    on: [bool; ALL.len()],
}

impl Default for Options {
    fn default() -> Options {
        let mut on = [false; ALL.len()];
        for (i, d) in ALL.iter().enumerate() {
            on[i] = d.default;
        }
        Options { on }
    }
}

impl Options {
    pub fn on(&self, o: O) -> bool {
        self.on[o as usize]
    }

    pub fn set(&mut self, o: O, v: bool) {
        self.on[o as usize] = v;
    }

    pub fn flip(&mut self, o: O) {
        self.on[o as usize] = !self.on[o as usize];
    }

    pub fn label(o: O) -> &'static str {
        ALL[o as usize].label
    }

    /// The switches as setup-file lines, `key=1` or `key=0`.
    pub fn lines(&self) -> Vec<String> {
        ALL.iter().map(|d| alloc::format!("{}={}", d.key, self.on(d.o) as u8)).collect()
    }

    /// One setup-file line, if it names a switch: true if it did.
    pub fn read_line(&mut self, key: &str, value: &str) -> bool {
        match ALL.iter().find(|d| d.key == key) {
            Some(d) => {
                self.set(d.o, value.trim() == "1");
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_in_the_switches_order() {
        for (i, d) in ALL.iter().enumerate() {
            assert_eq!(d.o as usize, i, "{} is out of place", d.key);
        }
    }
}
