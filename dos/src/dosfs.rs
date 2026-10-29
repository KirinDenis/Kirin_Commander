//! The DOS file system: INT 21h, reached from protected mode through
//! DPMI's real-mode calls (docs/06-file-systems.md, 6.1).
//!
//! Everything DOS reads or writes has to be below a megabyte, so a path,
//! a second path, the DTA and an 8 K block of data live in one block DOS
//! gives us, asked for once. After a failure the extended error (59h) is
//! the number kept, the one the error box shows small: `DOS error 27h`.

use alloc::string::String;
use alloc::vec::Vec;
use kc::fs::*;
use owlosui_dos::{dos_alloc, read_real, real, real_int, write_real, RealRegs};

const PATH1: u16 = 0x000;
const PATH2: u16 = 0x100;
const DTA: u16 = 0x200;
const PARAMS: u16 = 0x240;
const TAIL: u16 = 0x260;
const DATA: u16 = 0x400;
pub const DATA_SIZE: usize = 8192;

pub struct DosFs {
    seg: u16,
}

fn carry(r: &RealRegs) -> bool {
    r.flags & 1 != 0
}

/// The last error, as DOS explains it (59h), in the commander's words.
fn last_error() -> FsError {
    let mut r = RealRegs { eax: 0x5900, ebx: 0, ..Default::default() };
    real_int(0x21, &mut r);
    let code = (r.eax & 0xFFFF) as i32;
    let kind = match code {
        2 | 3 | 15 | 18 => ErrorKind::NotFound,
        5 | 16 | 82 => ErrorKind::Denied,
        17 => ErrorKind::CrossDevice,
        19 => ErrorKind::WriteProtected,
        21 => ErrorKind::NotReady,
        32 | 33 | 36 => ErrorKind::InUse,
        39 | 112 => ErrorKind::DiskFull,
        80 => ErrorKind::Exists,
        123 => ErrorKind::BadName,
        _ => ErrorKind::Io,
    };
    FsError::with_code(kind, code)
}

fn int21(r: &mut RealRegs) -> FsResult<()> {
    real_int(0x21, r);
    if carry(r) {
        Err(last_error())
    } else {
        Ok(())
    }
}

/// What a drive number (1 = A:) is, by 4408h - "fixed", "removable", or
/// "remote" for a redirector's (a network drive, a CD-ROM), which 4408h
/// does not serve; None when DOS has no such drive. It asks the driver,
/// not the disk: an empty A: is not touched.
fn drive_kind(d: u8) -> Option<&'static str> {
    let mut r = RealRegs { eax: 0x4408, ebx: d as u32, ..Default::default() };
    real_int(0x21, &mut r);
    if !carry(&r) {
        Some(if r.eax & 0xFFFF == 0 { "removable" } else { "fixed" })
    } else if r.eax & 0xFFFF != 15 {
        Some("remote")
    } else {
        None
    }
}

/// A DOS name is bytes; a core string carries each byte as the char of
/// that number, so this is a cast both ways.
fn bytes_of(s: &str) -> Vec<u8> {
    s.chars().take(250).map(|c| c as u32 as u8).collect()
}

fn string_of(b: &[u8]) -> String {
    b.iter().take_while(|&&c| c != 0).map(|&c| c as char).collect()
}

impl DosFs {
    pub fn new() -> Option<DosFs> {
        let seg = dos_alloc(((DATA as usize + DATA_SIZE) / 16) as u16)?;
        Some(DosFs { seg })
    }

    pub fn segment(&self) -> u16 {
        self.seg
    }

    fn put(&self, at: u16, path: &str) {
        let mut b = bytes_of(path);
        b.push(0);
        write_real(real(self.seg, at), &b);
    }

    fn regs(&self, ax: u32, dx: u16) -> RealRegs {
        RealRegs { eax: ax, edx: dx as u32, ds: self.seg, es: self.seg, ..Default::default() }
    }

    /// One DTA entry as an `Entry`.
    fn dta_entry(&self) -> Entry {
        let mut d = [0u8; 43];
        read_real(real(self.seg, DTA), &mut d);
        let time = u16::from_le_bytes([d[22], d[23]]);
        let date = u16::from_le_bytes([d[24], d[25]]);
        Entry {
            name: string_of(&d[30..43]),
            dir: d[21] & 0x10 != 0,
            size: u32::from_le_bytes([d[26], d[27], d[28], d[29]]) as u64,
            stamp: Stamp {
                year: 1980 + (date >> 9),
                month: ((date >> 5) & 15) as u8,
                day: (date & 31) as u8,
                hour: (time >> 11) as u8,
                minute: ((time >> 5) & 63) as u8,
                second: ((time & 31) * 2) as u8,
            },
            attrs: Attrs::from_dos(d[21]),
        }
    }

    fn set_dta(&self) {
        let mut r = self.regs(0x1A00, DTA);
        real_int(0x21, &mut r);
    }

    /// Run a command line through COMMAND.COM in `dir`. The shell is the
    /// one COMSPEC names; `env` is the program's environment segment.
    pub fn exec(&mut self, comspec: &str, dir: &str, line: &str) -> FsResult<u8> {
        // The panel's drive and folder become DOS's own first.
        let b = dir.as_bytes();
        if b.len() >= 2 && b[1] == b':' {
            let mut r = RealRegs { eax: 0x0E00, edx: (b[0].to_ascii_uppercase() - b'A') as u32, ..Default::default() };
            real_int(0x21, &mut r);
        }
        self.put(PATH1, dir);
        int21(&mut self.regs(0x3B00, PATH1))?;
        self.put(PATH2, comspec);
        // The command tail: length, " /C line", CR.
        let mut tail = Vec::new();
        tail.extend_from_slice(b" /C ");
        tail.extend_from_slice(&bytes_of(line));
        tail.truncate(126);
        let mut t = Vec::with_capacity(tail.len() + 2);
        t.push(tail.len() as u8);
        t.extend_from_slice(&tail);
        t.push(13);
        write_real(real(self.seg, TAIL), &t);
        // The parameter block: the environment inherited (0), the tail, two
        // FCBs nobody uses (pointed at the tail's end, as is customary).
        let mut p = [0u8; 14];
        p[2..4].copy_from_slice(&TAIL.to_le_bytes());
        p[4..6].copy_from_slice(&self.seg.to_le_bytes());
        p[6..8].copy_from_slice(&(TAIL + 0x80).to_le_bytes());
        p[8..10].copy_from_slice(&self.seg.to_le_bytes());
        p[10..12].copy_from_slice(&(TAIL + 0x80).to_le_bytes());
        p[12..14].copy_from_slice(&self.seg.to_le_bytes());
        write_real(real(self.seg, PARAMS), &p);
        let mut r = RealRegs { eax: 0x4B00, edx: PATH2 as u32, ebx: PARAMS as u32, ds: self.seg, es: self.seg, ..Default::default() };
        int21(&mut r)?;
        let mut c = RealRegs { eax: 0x4D00, ..Default::default() };
        real_int(0x21, &mut c);
        Ok(c.eax as u8)
    }
}

impl Fs for DosFs {
    fn sep(&self) -> char {
        '\\'
    }
    fn short_names(&self) -> bool {
        true
    }
    fn case_sensitive(&self) -> bool {
        false
    }

    fn drives(&mut self) -> Vec<Drive> {
        let mut v = Vec::new();
        for d in 1..=26u8 {
            if let Some(what) = drive_kind(d) {
                let letter = (b'A' + d - 1) as char;
                v.push(Drive { root: alloc::format!("{}:\\", letter), letter, what: String::from(what) });
            }
        }
        v
    }

    fn disk_space(&mut self, root: &str) -> Option<(u64, u64)> {
        let d = root.as_bytes().first()?.to_ascii_uppercase().checked_sub(b'A')? + 1;
        // Only a fixed disk is asked: 36h on an empty floppy spins it and
        // stops at Abort, Retry, Fail; a redirector's drive may do the same.
        if drive_kind(d)? != "fixed" {
            return None;
        }
        let mut r = RealRegs { eax: 0x3600, edx: d as u32, ..Default::default() };
        real_int(0x21, &mut r);
        if r.eax & 0xFFFF == 0xFFFF {
            return None;
        }
        // AX sectors a cluster, BX clusters free, CX bytes a sector, DX clusters in all.
        let cluster = (r.eax & 0xFFFF) as u64 * (r.ecx & 0xFFFF) as u64; // bytes a cluster: sectors * bytes
        Some((cluster * (r.edx & 0xFFFF) as u64, cluster * (r.ebx & 0xFFFF) as u64)) // size, free: clusters * bytes
    }

    /// The label, by Find First with the volume attribute (08h) - asked of
    /// a fixed disk only, as `disk_space` is. DOS writes it as a name with
    /// a dot after eight letters; the dot is not the label's.
    fn label(&mut self, root: &str) -> Option<String> {
        let d = root.as_bytes().first()?.to_ascii_uppercase().checked_sub(b'A')? + 1;
        if drive_kind(d)? != "fixed" {
            return None;
        }
        self.put(PATH1, &alloc::format!("{}:\\*.*", (b'A' + d - 1) as char));
        self.set_dta();
        let mut r = self.regs(0x4E00, PATH1);
        r.ecx = 0x08;
        real_int(0x21, &mut r);
        if carry(&r) {
            return None;
        }
        let mut a = [0u8; 1];
        read_real(real(self.seg, DTA + 21), &mut a);
        if a[0] & 0x08 == 0 {
            return None;
        }
        let name: String = self.dta_entry().name.chars().filter(|&c| c != '.').collect();
        Some(name).filter(|n| !n.is_empty())
    }

    fn can_set_label(&self) -> bool {
        true
    }

    /// The label through an extended FCB, the only way DOS has: the old
    /// one deleted (13h with `???????????`), the new one created (16h) and
    /// closed (10h). The FCB is 44 bytes in the data block: FFh, five
    /// zeros, the attribute 08h, the drive (1 = A:), the 11 letters.
    fn set_label(&mut self, root: &str, label: &str) -> FsResult<()> {
        let d = root.as_bytes().first().map(|c| c.to_ascii_uppercase()).and_then(|c| c.checked_sub(b'A')).ok_or(FsError::new(ErrorKind::BadName))? + 1;
        let fcb = |name: &[u8]| -> [u8; 44] {
            let mut f = [0u8; 44];
            f[0] = 0xFF;
            f[6] = 0x08;
            f[7] = d;
            for i in 0..11 {
                f[8 + i] = *name.get(i).unwrap_or(&b' ');
            }
            f
        };
        write_real(real(self.seg, DATA), &fcb(b"???????????"));
        let mut r = self.regs(0x1300, DATA);
        real_int(0x21, &mut r);
        if label.is_empty() {
            return Ok(());
        }
        write_real(real(self.seg, DATA), &fcb(&bytes_of(label)));
        let mut c = self.regs(0x1600, DATA);
        real_int(0x21, &mut c);
        if c.eax & 0xFF != 0 {
            return Err(FsError::new(ErrorKind::Denied));
        }
        let mut z = self.regs(0x1000, DATA);
        real_int(0x21, &mut z);
        Ok(())
    }

    fn current_dir(&mut self) -> String {
        let mut r = RealRegs { eax: 0x1900, ..Default::default() };
        real_int(0x21, &mut r);
        let drive = b'A' + (r.eax & 0xFF) as u8;
        let mut dir = alloc::format!("{}:\\", drive as char);
        let mut q = RealRegs { eax: 0x4700, edx: 0, esi: PATH1 as u32, ds: self.seg, ..Default::default() };
        real_int(0x21, &mut q);
        if !carry(&q) {
            let mut b = [0u8; 64];
            read_real(real(self.seg, PATH1), &mut b);
            dir.push_str(&string_of(&b));
        }
        dir
    }

    fn read_dir(&mut self, dir: &str) -> FsResult<Vec<Entry>> {
        let mut pattern = String::from(dir);
        if !pattern.ends_with('\\') {
            pattern.push('\\');
        }
        pattern.push_str("*.*");
        self.put(PATH1, &pattern);
        self.set_dta();
        let mut r = self.regs(0x4E00, PATH1);
        r.ecx = 0x37;
        real_int(0x21, &mut r);
        let mut v = Vec::new();
        if carry(&r) {
            let e = last_error();
            // No more files: a folder with nothing in it is still a folder.
            return if e.code == 18 { Ok(v) } else { Err(e) };
        }
        loop {
            let e = self.dta_entry();
            if e.name != "." && e.name != ".." {
                v.push(e);
            }
            let mut n = RealRegs { eax: 0x4F00, ..Default::default() };
            real_int(0x21, &mut n);
            if carry(&n) {
                break;
            }
        }
        Ok(v)
    }

    fn stat(&mut self, path: &str) -> FsResult<Entry> {
        if kc::path::is_root(path, '\\') {
            // A root has no entry of its own; reading it says it is there.
            self.read_dir(path)?;
            return Ok(Entry::folder(path));
        }
        self.put(PATH1, path);
        self.set_dta();
        let mut r = self.regs(0x4E00, PATH1);
        r.ecx = 0x37;
        real_int(0x21, &mut r);
        if carry(&r) {
            let e = last_error();
            return Err(if e.code == 18 { FsError::with_code(ErrorKind::NotFound, 2) } else { e });
        }
        Ok(self.dta_entry())
    }

    fn free_space(&mut self, path: &str) -> Option<u64> {
        let d = path.as_bytes().first()?.to_ascii_uppercase();
        let mut r = RealRegs { eax: 0x3600, edx: (d - b'A' + 1) as u32, ..Default::default() };
        real_int(0x21, &mut r);
        if r.eax & 0xFFFF == 0xFFFF {
            return None;
        }
        Some((r.eax & 0xFFFF) as u64 * (r.ebx & 0xFFFF) as u64 * (r.ecx & 0xFFFF) as u64)
    }

    fn make_dir(&mut self, path: &str) -> FsResult<()> {
        self.put(PATH1, path);
        int21(&mut self.regs(0x3900, PATH1)).map_err(|e| if e.code == 5 && self.stat(path).is_ok() { FsError::with_code(ErrorKind::Exists, 5) } else { e })
    }

    fn remove_file(&mut self, path: &str) -> FsResult<()> {
        if let Ok(e) = self.stat(path) {
            if e.attrs.read_only {
                return Err(FsError::with_code(ErrorKind::ReadOnly, 5));
            }
        }
        self.put(PATH1, path);
        int21(&mut self.regs(0x4100, PATH1))
    }

    fn remove_dir(&mut self, path: &str) -> FsResult<()> {
        self.put(PATH1, path);
        int21(&mut self.regs(0x3A00, PATH1)).map_err(|e| {
            // DOS says "access denied" for a folder with something in it.
            if e.code == 5 && self.read_dir(path).map(|l| !l.is_empty()).unwrap_or(false) {
                FsError::with_code(ErrorKind::NotEmpty, 5)
            } else {
                e
            }
        })
    }

    fn rename(&mut self, from: &str, to: &str) -> FsResult<()> {
        let (a, b) = (from.as_bytes(), to.as_bytes());
        if a.len() > 1 && b.len() > 1 && !a[0].eq_ignore_ascii_case(&b[0]) {
            return Err(FsError::with_code(ErrorKind::CrossDevice, 17));
        }
        if !from.eq_ignore_ascii_case(to) && self.stat(to).is_ok() {
            return Err(FsError::with_code(ErrorKind::Exists, 5));
        }
        self.put(PATH1, from);
        self.put(PATH2, to);
        let mut r = self.regs(0x5600, PATH1);
        r.edi = PATH2 as u32;
        int21(&mut r)
    }

    fn set_attrs(&mut self, path: &str, attrs: Attrs) -> FsResult<()> {
        self.put(PATH1, path);
        let mut r = self.regs(0x4301, PATH1);
        r.ecx = attrs.to_dos() as u32;
        int21(&mut r)
    }

    fn set_stamp(&mut self, path: &str, s: Stamp) -> FsResult<()> {
        let h = {
            self.put(PATH1, path);
            let mut r = self.regs(0x3D02, PATH1);
            int21(&mut r)?;
            r.eax & 0xFFFF
        };
        let time = ((s.hour as u32) << 11) | ((s.minute as u32) << 5) | (s.second as u32 / 2);
        let date = ((s.year.saturating_sub(1980) as u32) << 9) | ((s.month as u32) << 5) | s.day as u32;
        let mut r = RealRegs { eax: 0x5701, ebx: h, ecx: time, edx: date, ..Default::default() };
        let res = int21(&mut r);
        let mut c = RealRegs { eax: 0x3E00, ebx: h, ..Default::default() };
        real_int(0x21, &mut c);
        res
    }

    fn open_read(&mut self, path: &str) -> FsResult<Handle> {
        self.put(PATH1, path);
        let mut r = self.regs(0x3D00, PATH1);
        int21(&mut r)?;
        Ok(r.eax & 0xFFFF)
    }

    fn create(&mut self, path: &str) -> FsResult<Handle> {
        if let Ok(e) = self.stat(path) {
            if e.attrs.read_only {
                return Err(FsError::with_code(ErrorKind::ReadOnly, 5));
            }
        }
        self.put(PATH1, path);
        let mut r = self.regs(0x3C00, PATH1);
        r.ecx = 0;
        int21(&mut r)?;
        Ok(r.eax & 0xFFFF)
    }

    fn read(&mut self, h: Handle, buf: &mut [u8]) -> FsResult<usize> {
        let want = buf.len().min(DATA_SIZE);
        let mut r = RealRegs { eax: 0x3F00, ebx: h, ecx: want as u32, edx: DATA as u32, ds: self.seg, ..Default::default() };
        int21(&mut r)?;
        let got = (r.eax & 0xFFFF) as usize;
        read_real(real(self.seg, DATA), &mut buf[..got]);
        Ok(got)
    }

    fn write(&mut self, h: Handle, buf: &[u8]) -> FsResult<()> {
        for chunk in buf.chunks(DATA_SIZE) {
            write_real(real(self.seg, DATA), chunk);
            let mut r = RealRegs { eax: 0x4000, ebx: h, ecx: chunk.len() as u32, edx: DATA as u32, ds: self.seg, ..Default::default() };
            int21(&mut r)?;
            // Fewer bytes than asked, and no error: the disk is full.
            if ((r.eax & 0xFFFF) as usize) < chunk.len() {
                return Err(FsError::with_code(ErrorKind::DiskFull, 39));
            }
        }
        Ok(())
    }

    fn close(&mut self, h: Handle) -> FsResult<()> {
        let mut r = RealRegs { eax: 0x3E00, ebx: h, ..Default::default() };
        int21(&mut r)
    }
}
