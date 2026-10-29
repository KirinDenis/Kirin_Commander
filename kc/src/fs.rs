//! The file system, as the commander sees it (docs/06-file-systems.md).
//!
//! One interface, four machines under it: DOS through INT 21h, Windows,
//! Linux, and the browser's own storage. Everything the panels and the file
//! operations do goes through `Fs`, and every failure comes back as an
//! `FsError` whose `kind` means the same thing on all four - so the error
//! box asks the same question whether DOS said "write protect error" or
//! Linux said EROFS.

use alloc::string::String;
use alloc::vec::Vec;

/// What went wrong, in the words the error box uses. The machine's own
/// number travels in `FsError::code` for the line under it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    NotFound,
    /// The name is taken: a folder made where one is, a rename onto a name.
    Exists,
    Denied,
    /// The file is read-only: asked about, then forced if the person says so.
    ReadOnly,
    /// A folder still has something in it.
    NotEmpty,
    DiskFull,
    /// No disk in the drive, a network drive gone.
    NotReady,
    WriteProtected,
    /// Another program has it open (sharing violation, EBUSY).
    InUse,
    /// A name the system does not allow: `*` in it, too long for 8.3.
    BadName,
    /// A rename to another drive: a move has to copy and delete instead.
    CrossDevice,
    /// Copying a folder into itself.
    IntoItself,
    Unsupported,
    Io,
}

impl ErrorKind {
    /// The first line of the error box.
    pub fn text(self) -> &'static str {
        match self {
            ErrorKind::NotFound => "File not found",
            ErrorKind::Exists => "The name already exists",
            ErrorKind::Denied => "Access denied",
            ErrorKind::ReadOnly => "The file is read-only",
            ErrorKind::NotEmpty => "The folder is not empty",
            ErrorKind::DiskFull => "The disk is full",
            ErrorKind::NotReady => "The drive is not ready",
            ErrorKind::WriteProtected => "The disk is write-protected",
            ErrorKind::InUse => "The file is in use by another program",
            ErrorKind::BadName => "The name is not allowed here",
            ErrorKind::CrossDevice => "Not on the same drive",
            ErrorKind::IntoItself => "Cannot copy a folder into itself",
            ErrorKind::Unsupported => "Not possible on this drive",
            ErrorKind::Io => "Read or write error",
        }
    }

    /// Whether trying again can help: the box offers Retry only then.
    pub fn retry_helps(self) -> bool {
        matches!(
            self,
            ErrorKind::NotReady | ErrorKind::WriteProtected | ErrorKind::InUse | ErrorKind::DiskFull | ErrorKind::Io | ErrorKind::Denied
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FsError {
    pub kind: ErrorKind,
    /// The machine's own number: the DOS extended error, errno, GetLastError.
    pub code: i32,
}

impl FsError {
    pub const fn new(kind: ErrorKind) -> Self {
        FsError { kind, code: 0 }
    }
    pub const fn with_code(kind: ErrorKind, code: i32) -> Self {
        FsError { kind, code }
    }
}

pub type FsResult<T> = Result<T, FsError>;

/// When a file was written, in local time. Seconds are kept for comparing;
/// the panel shows minutes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Stamp {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Attrs {
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub archive: bool,
}

impl Attrs {
    /// DOS's attribute byte.
    pub fn from_dos(b: u8) -> Attrs {
        Attrs { read_only: b & 1 != 0, hidden: b & 2 != 0, system: b & 4 != 0, archive: b & 0x20 != 0 }
    }
    pub fn to_dos(self) -> u8 {
        (self.read_only as u8) | (self.hidden as u8) << 1 | (self.system as u8) << 2 | (self.archive as u8) << 5
    }
}

/// One name in a folder.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entry {
    pub name: String,
    pub dir: bool,
    pub size: u64,
    pub stamp: Stamp,
    pub attrs: Attrs,
}

impl Entry {
    pub fn file(name: &str, size: u64) -> Entry {
        Entry { name: String::from(name), dir: false, size, stamp: Stamp::default(), attrs: Attrs::default() }
    }
    pub fn folder(name: &str) -> Entry {
        Entry { name: String::from(name), dir: true, size: 0, stamp: Stamp::default(), attrs: Attrs::default() }
    }
    pub fn is_up(&self) -> bool {
        self.name == ".."
    }
}

/// A drive the drive menu lists.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drive {
    /// What a panel opens: `C:\`, `/`, `/home/kirin`.
    pub root: String,
    /// What the menu shows and what a key chooses it by: `C`, or for a
    /// place that is not a letter, its first letter.
    pub letter: char,
    /// A word for the desktop's list: "fixed", "network", "home".
    pub what: String,
}

/// An open file, numbered by the file system.
pub type Handle = u32;

/// The machine. Paths are strings in the machine's own form: `C:\GAMES` on
/// DOS, Windows and the browser's drives; `/home/kirin` on Linux.
pub trait Fs {
    /// The folder separator: `\` or `/`.
    fn sep(&self) -> char;
    /// Names are 8.3, upper case on the disk: the panel splits name and
    /// extension into columns and shows folders upper, files lower case.
    fn short_names(&self) -> bool;
    /// Two names that differ only in case are two files (Linux).
    fn case_sensitive(&self) -> bool;
    /// A deleted file goes to the Recycle Bin / Trash (D9).
    fn has_trash(&self) -> bool {
        false
    }
    /// A file keeps its attributes and date when it is written: the copy
    /// dialog offers "Copy attributes and dates" only then (a browser's
    /// storage keeps neither, docs/05-file-operations.md 5.1).
    fn keeps_attrs(&self) -> bool {
        true
    }

    fn drives(&mut self) -> Vec<Drive>;
    /// Where a panel starts: the current folder of the program.
    fn current_dir(&mut self) -> String;
    /// Every name in a folder, `.` and `..` left out.
    fn read_dir(&mut self, dir: &str) -> FsResult<Vec<Entry>>;
    fn stat(&mut self, path: &str) -> FsResult<Entry>;
    /// Free bytes on the drive the path is on, if the machine says.
    fn free_space(&mut self, _path: &str) -> Option<u64> {
        None
    }
    /// The size of a drive and its free bytes, for the drive menu - if the
    /// machine says, and if asking touches no disk that may not be there:
    /// a removable drive answers None rather than spin an empty floppy.
    fn disk_space(&mut self, _root: &str) -> Option<(u64, u64)> {
        None
    }

    /// The drive's volume label, for the Info panel, if it has one - and
    /// as `disk_space`, never asked of a drive that may have no disk.
    fn label(&mut self, _root: &str) -> Option<String> {
        None
    }

    /// Whether `set_label` can change a drive's label: DOS can.
    fn can_set_label(&self) -> bool {
        false
    }

    /// A drive's volume label written, or taken away when empty.
    fn set_label(&mut self, _root: &str, _label: &str) -> FsResult<()> {
        Err(FsError::new(ErrorKind::Unsupported))
    }

    fn make_dir(&mut self, path: &str) -> FsResult<()>;
    fn remove_file(&mut self, path: &str) -> FsResult<()>;
    /// An empty folder.
    fn remove_dir(&mut self, path: &str) -> FsResult<()>;
    /// To the Recycle Bin / Trash, where `has_trash`.
    fn trash(&mut self, _path: &str) -> FsResult<()> {
        Err(FsError::new(ErrorKind::Unsupported))
    }
    /// A rename or a move on the same drive. `CrossDevice` when it is not.
    fn rename(&mut self, from: &str, to: &str) -> FsResult<()>;
    fn set_attrs(&mut self, path: &str, attrs: Attrs) -> FsResult<()>;
    fn set_stamp(&mut self, _path: &str, _stamp: Stamp) -> FsResult<()> {
        Ok(())
    }

    fn open_read(&mut self, path: &str) -> FsResult<Handle>;
    /// A new file, or an existing one emptied.
    fn create(&mut self, path: &str) -> FsResult<Handle>;
    /// Up to `buf.len()` bytes; 0 at the end.
    fn read(&mut self, h: Handle, buf: &mut [u8]) -> FsResult<usize>;
    /// All of `buf`, or an error (DiskFull when the disk took less).
    fn write(&mut self, h: Handle, buf: &[u8]) -> FsResult<()>;
    fn close(&mut self, h: Handle) -> FsResult<()>;
}
