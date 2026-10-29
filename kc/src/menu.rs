//! The pull-down menu, F9 (docs/04-menus.md): Left, Files, Commands,
//! Options, Right. Every item is the key it shows on its right, sent as
//! that key; an item with no key yet is there and grey, never missing.

use crate::options::{Page, O};
use crate::panel::View;
use alloc::string::String;
use alloc::vec::Vec;
use owlosui_core::{Key, KeyCode, Mods};

pub const TITLES: [&str; 5] = ["Left", "Files", "Commands", "Options", "Right"];

/// One line of a menu: `~h~otkey` in the label, the key's name, the key.
/// What an item does: a key the commander already has, or one of the few
/// things that have no key.
#[derive(Clone, Copy, Debug)]
pub enum Act {
    Key(Key),
    Compare,
    Colours,
    /// One of the Options dialogs.
    Options(Page),
    /// A switch the menu turns itself, ticked while it is on.
    Toggle(O),
    SaveSetup,
    /// Info panel, Tree panel, Quick view: in this title's own panel.
    View(View),
    MenuFile,
    ExtFile,
    Label,
    SystemInfo,
    Shortcuts,
    Languages,
}

pub struct Item {
    pub label: &'static str,
    pub keys: &'static str,
    pub key: Option<Act>,
}

pub const LINE: Item = Item { label: "-", keys: "", key: None };

const fn k(code: KeyCode, mods: Mods) -> Option<Act> {
    Some(Act::Key(Key { code, mods }))
}
const NONE: Mods = Mods { ctrl: false, alt: false, shift: false };
const CTRL: Mods = Mods { ctrl: true, alt: false, shift: false };
const ALT: Mods = Mods { ctrl: false, alt: true, shift: false };

const fn it(label: &'static str, keys: &'static str, key: Option<Act>) -> Item {
    Item { label, keys, key }
}

pub fn items(title: usize) -> Vec<Item> {
    match title {
        0 | 4 => alloc::vec![
            it("~B~rief", "Ctrl+1", k(KeyCode::Char('1'), CTRL)),
            it("~M~edium", "Ctrl+2", k(KeyCode::Char('2'), CTRL)),
            it("~F~ull", "Ctrl+3", k(KeyCode::Char('3'), CTRL)),
            it("~W~ide", "Ctrl+4", k(KeyCode::Char('4'), CTRL)),
            it("Det~a~iled", "Ctrl+5", k(KeyCode::Char('5'), CTRL)),
            LINE,
            it("~I~nfo panel", "Ctrl+L", Some(Act::View(View::Info))),
            it("~T~ree panel", "Ctrl+T", Some(Act::View(View::Tree))),
            it("~Q~uick view", "Ctrl+Q", Some(Act::View(View::Quick))),
            it("~O~n/off", if title == 0 { "Ctrl+F1" } else { "Ctrl+F2" }, k(KeyCode::F(if title == 0 { 1 } else { 2 }), CTRL)),
            LINE,
            it("~N~ame", "Ctrl+F3", k(KeyCode::F(3), CTRL)),
            it("e~X~tension", "Ctrl+F4", k(KeyCode::F(4), CTRL)),
            it("ti~M~e", "Ctrl+F5", k(KeyCode::F(5), CTRL)),
            it("~S~ize", "Ctrl+F6", k(KeyCode::F(6), CTRL)),
            it("~U~nsorted", "Ctrl+F7", k(KeyCode::F(7), CTRL)),
            LINE,
            it("~R~e-read", "Ctrl+R", k(KeyCode::Char('r'), CTRL)),
            it("fi~L~ter...", "Ctrl+F", k(KeyCode::Char('f'), CTRL)),
            it("~H~idden files", "Ctrl+H", k(KeyCode::Char('h'), CTRL)),
            it("~D~rive...", if title == 0 { "Alt+F1" } else { "Alt+F2" }, k(KeyCode::F(if title == 0 { 1 } else { 2 }), ALT)),
        ],
        1 => alloc::vec![
            it("~H~elp", "F1", k(KeyCode::F(1), NONE)),
            it("~U~ser menu", "F2", k(KeyCode::F(2), NONE)),
            it("~V~iew", "F3", k(KeyCode::F(3), NONE)),
            it("~E~dit", "F4", k(KeyCode::F(4), NONE)),
            it("~C~opy", "F5", k(KeyCode::F(5), NONE)),
            it("~R~ename or move", "F6", k(KeyCode::F(6), NONE)),
            it("~M~ake folder", "F7", k(KeyCode::F(7), NONE)),
            it("~D~elete", "F8", k(KeyCode::F(8), NONE)),
            LINE,
            it("File ~A~ttributes", "Ctrl+A", k(KeyCode::Char('a'), CTRL)),
            LINE,
            it("select ~G~roup...", "Gray +", k(KeyCode::Char('+'), NONE)),
            it("u~N~select group...", "Gray -", k(KeyCode::Char('-'), NONE)),
            it("~I~nvert selection", "Gray *", k(KeyCode::Char('*'), NONE)),
            it("re~S~tore selection", "Ctrl+M", k(KeyCode::Char('m'), CTRL)),
            LINE,
            it("~Q~uit", "F10", k(KeyCode::F(10), NONE)),
        ],
        2 => alloc::vec![
            it("~F~ind file", "Alt+F7", k(KeyCode::F(7), ALT)),
            it("~H~istory", "Alt+F8", k(KeyCode::F(8), ALT)),
            it("Folder ~T~ree", "Alt+F10", k(KeyCode::F(10), ALT)),
            it("~D~irectory sizes", "Alt+F6", k(KeyCode::F(6), ALT)),
            LINE,
            it("~S~wap panels", "Ctrl+U", k(KeyCode::Char('u'), CTRL)),
            it("~P~anels on/off", "Ctrl+O", k(KeyCode::Char('o'), CTRL)),
            it("~C~ompare folders", "", Some(Act::Compare)),
            LINE,
            it("~V~olume label", "", Some(Act::Label)),
            it("System ~I~nfo", "", Some(Act::SystemInfo)),
            LINE,
            it("~M~enu file edit", "", Some(Act::MenuFile)),
            it("e~X~tension file edit", "", Some(Act::ExtFile)),
            it("F~o~lder shortcuts", "", Some(Act::Shortcuts)),
        ],
        _ => alloc::vec![
            it("~C~onfiguration...", "", Some(Act::Options(Page::Configuration))),
            it("~P~anel settings...", "", Some(Act::Options(Page::Panels))),
            it("C~o~lours...", "", Some(Act::Colours)),
            it("Co~n~firmations...", "", Some(Act::Options(Page::Confirmations))),
            it("~V~iewer and editor...", "", Some(Act::Options(Page::Editor))),
            it("Lan~g~uages...", "", Some(Act::Languages)),
            LINE,
            it("~M~enu bar always visible", "", Some(Act::Toggle(O::MenuBar))),
            it("~K~ey bar", "Ctrl+B", k(KeyCode::Char('b'), CTRL)),
            it("C~l~ock", "", Some(Act::Toggle(O::Clock))),
            LINE,
            it("~S~ave setup", "Shift+F9", Some(Act::SaveSetup)),
        ],
    }
}

/// The open menu: which title, which item.
pub struct Menu {
    pub title: usize,
    pub item: usize,
    /// Where each title is on the bar, for the mouse.
    pub title_x: [i16; 5],
}

pub enum MenuOut {
    Stay,
    Close,
    /// An item chosen: this key, for this title's panel.
    Run(usize, Act),
}

impl Menu {
    pub fn new(title: usize) -> Menu {
        let mut m = Menu { title, item: 0, title_x: [0; 5] };
        let mut x = 2;
        for (i, t) in TITLES.iter().enumerate() {
            m.title_x[i] = x;
            x += t.len() as i16 + 4;
        }
        m
    }

    pub fn key(&mut self, k: Key) -> MenuOut {
        let list = items(self.title);
        let n = list.len();
        let step = |from: usize, down: bool| -> usize {
            let mut i = from;
            for _ in 0..n {
                i = if down { (i + 1) % n } else { (i + n - 1) % n };
                if list[i].label != "-" {
                    return i;
                }
            }
            from
        };
        match k.code {
            KeyCode::Esc | KeyCode::F(9) | KeyCode::F(10) => return MenuOut::Close,
            KeyCode::Left => self.open((self.title + 4) % 5),
            KeyCode::Right => self.open((self.title + 1) % 5),
            KeyCode::Up => self.item = step(self.item, false),
            KeyCode::Down => self.item = step(self.item, true),
            KeyCode::Home => self.item = 0,
            KeyCode::End => self.item = step(0, false),
            KeyCode::Enter => return self.choose(self.item),
            KeyCode::Char(c) if !k.mods.ctrl => {
                let c = c.to_ascii_lowercase();
                // A grey item is passed over.
                if let Some(i) = list.iter().position(|it| it.key.is_some() && hot(it.label) == Some(c)) {
                    return self.choose(i);
                }
            }
            _ => {}
        }
        MenuOut::Stay
    }

    fn open(&mut self, title: usize) {
        self.title = title;
        self.item = 0;
    }

    pub fn choose(&mut self, i: usize) -> MenuOut {
        let list = items(self.title);
        self.item = i;
        match list.get(i).and_then(|it| it.key) {
            Some(key) => MenuOut::Run(self.title, key),
            None => MenuOut::Stay,
        }
    }

    /// A click: a title on the bar, or an item in the open box.
    pub fn click(&mut self, x: i16, y: i16) -> MenuOut {
        if y == 0 {
            for i in (0..5).rev() {
                if x >= self.title_x[i] - 1 {
                    self.open(i);
                    return MenuOut::Stay;
                }
            }
            return MenuOut::Stay;
        }
        let (bx, w) = self.box_x();
        let row = y - 2;
        if x >= bx && x < bx + w && row >= 0 && (row as usize) < items(self.title).len() {
            return self.choose(row as usize);
        }
        MenuOut::Close
    }

    /// The box's left edge and width.
    pub fn box_x(&self) -> (i16, i16) {
        let width = items(self.title).iter().map(|it| plain(it.label).chars().count() + if it.keys.is_empty() { 0 } else { it.keys.len() + 2 }).max().unwrap_or(10);
        let w = width as i16 + 6;
        let x = (self.title_x[self.title] - 2).max(0);
        (x, w)
    }
}

pub fn hot(label: &str) -> Option<char> {
    let i = label.find('~')?;
    label[i + 1..].chars().next().map(|c| c.to_ascii_lowercase())
}

pub fn plain(label: &str) -> String {
    label.chars().filter(|&c| c != '~').collect()
}
