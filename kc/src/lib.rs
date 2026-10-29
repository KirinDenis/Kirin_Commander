//! Kirin Commander - the part that is the same everywhere.
//!
//! The commander is a state (`App`), a file system it acts on (`Fs`), and
//! a picture of itself it draws into OWLOSUI's `Buffer`. A host - DOS, a
//! desktop terminal or window, the browser - gives it keys and mouse
//! events, draws the buffer, and does the few things only it can: run a
//! program, open the viewer or the editor. Those come back from `App` as
//! `Request`s.
//!
//! The design is docs/; each module names the section it follows.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod app;
pub mod chrome;
pub mod colors;
pub mod dialog;
pub mod editor;
pub mod fs;
pub mod help;
pub mod job;
pub mod lang;
pub mod line;
pub mod mask;
pub mod menu;
pub mod memfs;
pub mod names;
pub mod options;
pub mod panel;
pub mod path;
pub mod screen;
pub mod tree;
pub mod usermenu;
pub mod commands;
pub mod setup;
pub mod sides;
pub mod step;
#[cfg(feature = "std")]
pub mod stdfs;

pub use app::{App, Request};
pub use fs::{Attrs, Drive, Entry, ErrorKind, Fs, FsError, FsResult, Stamp};
pub use memfs::MemFs;
pub use panel::{Mode, Panel, Sort};
