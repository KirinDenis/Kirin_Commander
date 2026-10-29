//! The colour schemes (docs/01-screen.md, 1.5): Volkov Commander 4.05's
//! attribute bytes, and the Black & White and Laptop schemes.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scheme {
    pub panel: u8,
    pub header: u8,
    pub selected: u8,
    pub cursor: u8,
    pub cursor_selected: u8,
    pub active_title: u8,
    pub menu_bar: u8,
    pub menu_bar_current: u8,
    pub menu: u8,
    pub menu_hot: u8,
    pub menu_current: u8,
    pub menu_current_hot: u8,
    pub menu_grey: u8,
    pub dialog: u8,
    pub dialog_bright: u8,
    pub dialog_button: u8,
    pub dialog_input: u8,
    pub error: u8,
    pub error_bright: u8,
    pub key_digit: u8,
    pub key_label: u8,
    pub clock: u8,
    pub command: u8,
    /// Laptop: a `√` before a selected name, since brightness does not show.
    pub mark_selected: bool,
    /// Names coloured by what they are, as OWLOSUI's file panels colour
    /// them: folders, programs, archives, scratch files, hidden ones.
    pub file_dir: u8,
    pub file_exe: u8,
    pub file_archive: u8,
    pub file_temp: u8,
    pub file_dim: u8,
}

pub const COLOR: Scheme = Scheme {
    panel: 0x1B,
    header: 0x1E,
    selected: 0x1E,
    cursor: 0x30,
    cursor_selected: 0x3E,
    active_title: 0x30,
    menu_bar: 0x30,
    menu_bar_current: 0x0F,
    menu: 0x3F,
    menu_hot: 0x3E,
    menu_current: 0x0F,
    menu_current_hot: 0x0E,
    menu_grey: 0x38,
    dialog: 0x70,
    dialog_bright: 0x7E,
    dialog_button: 0x30,
    dialog_input: 0x30,
    error: 0x4F,
    error_bright: 0x4E,
    key_digit: 0x07,
    key_label: 0x30,
    clock: 0x30,
    command: 0x07,
    mark_selected: false,
    file_dir: 0x1F,
    file_exe: 0x1A,
    file_archive: 0x1D,
    file_temp: 0x16,
    file_dim: 0x18,
};

pub const BLACK_WHITE: Scheme = Scheme {
    panel: 0x07,
    header: 0x0F,
    selected: 0x0F,
    cursor: 0x70,
    cursor_selected: 0x7F,
    active_title: 0x70,
    menu_bar: 0x70,
    menu_bar_current: 0x07,
    menu: 0x70,
    menu_hot: 0x7F,
    menu_current: 0x07,
    menu_current_hot: 0x0F,
    menu_grey: 0x78,
    dialog: 0x70,
    dialog_bright: 0x7F,
    dialog_button: 0x07,
    dialog_input: 0x07,
    error: 0x70,
    error_bright: 0x7F,
    key_digit: 0x07,
    key_label: 0x70,
    clock: 0x70,
    command: 0x07,
    mark_selected: false,
    file_dir: 0x0F,
    file_exe: 0x0F,
    file_archive: 0x07,
    file_temp: 0x07,
    file_dim: 0x07,
};

pub const LAPTOP: Scheme = Scheme { mark_selected: true, ..BLACK_WHITE };

/// A name's colour when it is neither under the cursor nor selected. The
/// order is OWLOSUI's (and Far's): hidden and system first, so a hidden
/// folder reads as hidden.
pub fn of_entry(s: &Scheme, e: &crate::fs::Entry) -> u8 {
    if e.is_up() {
        return s.panel;
    }
    if e.attrs.hidden || e.attrs.system {
        return s.file_dim;
    }
    if e.dir {
        return s.file_dir;
    }
    let ext = crate::path::split_ext(&e.name).1.to_ascii_lowercase();
    match ext.as_str() {
        "com" | "exe" | "bat" | "btm" | "cmd" => s.file_exe,
        "zip" | "rar" | "arj" | "lzh" | "lha" | "7z" | "gz" | "tar" => s.file_archive,
        "tmp" | "bak" | "old" | "$$$" | "syd" | "chk" => s.file_temp,
        _ => s.panel,
    }
}
