//! The desktop's file system: Windows and Linux, through Rust's `std::fs`.
//! The errors are read from the system's own numbers, so a sharing
//! violation on Windows and EBUSY on Linux both come back as `InUse`
//! (docs/06-file-systems.md, 6.4).

use crate::fs::*;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;

pub struct StdFs {
    files: BTreeMap<Handle, std::fs::File>,
    next: Handle,
}

impl Default for StdFs {
    fn default() -> Self {
        StdFs { files: BTreeMap::new(), next: 1 }
    }
}

impl StdFs {
    pub fn new() -> StdFs {
        StdFs::default()
    }
}

/// The system's error, in the commander's words.
pub fn error_of(e: &std::io::Error) -> FsError {
    use std::io::ErrorKind as K;
    let code = e.raw_os_error().unwrap_or(0);
    let kind = if cfg!(windows) {
        match code {
            2 | 3 => ErrorKind::NotFound,
            5 => ErrorKind::Denied,
            19 => ErrorKind::WriteProtected,
            21 => ErrorKind::NotReady,
            32 | 33 => ErrorKind::InUse,
            39 | 112 => ErrorKind::DiskFull,
            80 | 183 => ErrorKind::Exists,
            123 | 161 => ErrorKind::BadName,
            145 => ErrorKind::NotEmpty,
            17 => ErrorKind::CrossDevice,
            206 => ErrorKind::BadName,
            _ => kind_of(e.kind()),
        }
    } else {
        match code {
            2 => ErrorKind::NotFound,
            1 | 13 => ErrorKind::Denied,
            16 | 26 => ErrorKind::InUse,
            17 => ErrorKind::Exists,
            18 => ErrorKind::CrossDevice,
            28 | 122 => ErrorKind::DiskFull,
            30 => ErrorKind::WriteProtected,
            36 => ErrorKind::BadName,
            39 => ErrorKind::NotEmpty,
            _ => kind_of(e.kind()),
        }
    };
    let _ = K::Other;
    FsError::with_code(kind, code)
}

fn kind_of(k: std::io::ErrorKind) -> ErrorKind {
    use std::io::ErrorKind as K;
    match k {
        K::NotFound => ErrorKind::NotFound,
        K::PermissionDenied => ErrorKind::Denied,
        K::AlreadyExists => ErrorKind::Exists,
        K::InvalidInput => ErrorKind::BadName,
        K::Unsupported => ErrorKind::Unsupported,
        _ => ErrorKind::Io,
    }
}

fn stamp_of(t: std::time::SystemTime) -> Stamp {
    // Local time without a time zone database: the offset the system
    // reports is not in `std`, so this is UTC on Linux. Windows hosts can
    // set TZ; the panels compare stamps, and UTC orders the same.
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) + offset();
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (y, m, d) = civil(days);
    Stamp { year: y as u16, month: m as u8, day: d as u8, hour: (rem / 3600) as u8, minute: (rem / 60 % 60) as u8, second: (rem % 60) as u8 }
}

/// The local offset from UTC, in seconds: `KC_TZ_MINUTES` if set, else
/// what the system says.
pub fn local_offset() -> i64 {
    offset()
}

fn offset() -> i64 {
    if let Some(m) = std::env::var("KC_TZ_MINUTES").ok().and_then(|v| v.parse::<i64>().ok()) {
        return m * 60;
    }
    system_offset()
}

#[cfg(windows)]
fn system_offset() -> i64 {
    #[repr(C)]
    struct TimeZoneInformation {
        bias: i32,
        rest: [u8; 168],
    }
    extern "system" {
        fn GetTimeZoneInformation(tz: *mut TimeZoneInformation) -> u32;
    }
    let mut tz = TimeZoneInformation { bias: 0, rest: [0; 168] };
    // SAFETY: the structure is the size Windows writes; nothing else is touched.
    let r = unsafe { GetTimeZoneInformation(&mut tz) };
    let bias_at = |i: usize| i32::from_le_bytes([tz.rest[i], tz.rest[i + 1], tz.rest[i + 2], tz.rest[i + 3]]);
    // The standard or the daylight bias on top: at bytes 84 and 168 of the 172.
    let extra = match r {
        1 => bias_at(80),
        2 => bias_at(164),
        _ => 0,
    };
    -((tz.bias + extra) as i64) * 60
}

#[cfg(not(windows))]
fn system_offset() -> i64 {
    0
}


/// Days since 1970-01-01 to a date (Howard Hinnant's algorithm).
fn civil(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn entry_of(name: String, md: &std::fs::Metadata) -> Entry {
    let mut attrs = Attrs { read_only: md.permissions().readonly(), ..Attrs::default() };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let a = md.file_attributes();
        attrs.hidden = a & 2 != 0;
        attrs.system = a & 4 != 0;
        attrs.archive = a & 0x20 != 0;
    }
    Entry {
        name,
        dir: md.is_dir(),
        size: if md.is_dir() { 0 } else { md.len() },
        stamp: md.modified().map(stamp_of).unwrap_or_default(),
        attrs,
    }
}

impl Fs for StdFs {
    fn sep(&self) -> char {
        std::path::MAIN_SEPARATOR
    }
    fn short_names(&self) -> bool {
        false
    }
    fn case_sensitive(&self) -> bool {
        !cfg!(windows)
    }

    fn drives(&mut self) -> Vec<Drive> {
        let mut v = Vec::new();
        if cfg!(windows) {
            for l in b'A'..=b'Z' {
                let root = format!("{}:\\", l as char);
                if Path::new(&root).exists() {
                    v.push(Drive { root, letter: l as char, what: String::from("drive") });
                }
            }
        } else {
            v.push(Drive { root: String::from("/"), letter: '/', what: String::from("root") });
        }
        if let Some(home) = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }) {
            v.push(Drive { root: home.to_string_lossy().into_owned(), letter: '~', what: String::from("home") });
        }
        v
    }

    fn current_dir(&mut self) -> String {
        std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_else(|_| String::from(if cfg!(windows) { "C:\\" } else { "/" }))
    }

    fn read_dir(&mut self, dir: &str) -> FsResult<Vec<Entry>> {
        let rd = std::fs::read_dir(dir).map_err(|e| error_of(&e))?;
        let mut v = Vec::new();
        for item in rd.flatten() {
            let name = item.file_name().to_string_lossy().into_owned();
            // A link is shown as what it points to; one that points nowhere, as itself.
            let md = std::fs::metadata(item.path()).or_else(|_| item.metadata());
            if let Ok(md) = md {
                v.push(entry_of(name, &md));
            }
        }
        Ok(v)
    }

    fn stat(&mut self, path: &str) -> FsResult<Entry> {
        let md = std::fs::metadata(path).map_err(|e| error_of(&e))?;
        Ok(entry_of(crate::path::name(path, self.sep()), &md))
    }

    fn make_dir(&mut self, path: &str) -> FsResult<()> {
        std::fs::create_dir(path).map_err(|e| error_of(&e))
    }

    fn remove_file(&mut self, path: &str) -> FsResult<()> {
        let md = std::fs::symlink_metadata(path).map_err(|e| error_of(&e))?;
        if md.permissions().readonly() && cfg!(windows) {
            return Err(FsError::new(ErrorKind::ReadOnly));
        }
        std::fs::remove_file(path).map_err(|e| error_of(&e))
    }

    fn remove_dir(&mut self, path: &str) -> FsResult<()> {
        std::fs::remove_dir(path).map_err(|e| error_of(&e))
    }

    fn rename(&mut self, from: &str, to: &str) -> FsResult<()> {
        // Never onto a name that is there: the job asks about that first.
        // (A change of case alone is the same name on Windows.)
        if Path::new(to).exists() && !from.eq_ignore_ascii_case(to) {
            return Err(FsError::new(ErrorKind::Exists));
        }
        std::fs::rename(from, to).map_err(|e| error_of(&e))
    }

    fn set_attrs(&mut self, path: &str, attrs: Attrs) -> FsResult<()> {
        let md = std::fs::metadata(path).map_err(|e| error_of(&e))?;
        let mut p = md.permissions();
        if p.readonly() != attrs.read_only {
            #[allow(clippy::permissions_set_readonly_false)]
            p.set_readonly(attrs.read_only);
            std::fs::set_permissions(path, p).map_err(|e| error_of(&e))?;
        }
        Ok(())
    }

    fn open_read(&mut self, path: &str) -> FsResult<Handle> {
        let f = std::fs::File::open(path).map_err(|e| error_of(&e))?;
        let h = self.next;
        self.next += 1;
        self.files.insert(h, f);
        Ok(h)
    }

    fn create(&mut self, path: &str) -> FsResult<Handle> {
        if let Ok(md) = std::fs::metadata(path) {
            if md.permissions().readonly() {
                return Err(FsError::new(ErrorKind::ReadOnly));
            }
        }
        let f = std::fs::File::create(path).map_err(|e| error_of(&e))?;
        let h = self.next;
        self.next += 1;
        self.files.insert(h, f);
        Ok(h)
    }

    fn read(&mut self, h: Handle, buf: &mut [u8]) -> FsResult<usize> {
        let f = self.files.get_mut(&h).ok_or(FsError::new(ErrorKind::Io))?;
        f.read(buf).map_err(|e| error_of(&e))
    }

    fn write(&mut self, h: Handle, buf: &[u8]) -> FsResult<()> {
        let f = self.files.get_mut(&h).ok_or(FsError::new(ErrorKind::Io))?;
        f.write_all(buf).map_err(|e| error_of(&e))
    }

    fn close(&mut self, h: Handle) -> FsResult<()> {
        let f = self.files.remove(&h).ok_or(FsError::new(ErrorKind::Io))?;
        // A full disk can show itself only when the data is pushed out.
        f.sync_all().map_err(|e| error_of(&e))
    }

    fn free_space(&mut self, _path: &str) -> Option<u64> {
        None
    }

    fn set_stamp(&mut self, path: &str, stamp: Stamp) -> FsResult<()> {
        let _ = (path, stamp);
        // `std` sets a modification time only from a SystemTime, and going
        // back from local fields needs the zone; copies keep the time the
        // system gave them until a host-specific call is added.
        Ok(())
    }
}
