//! A file system in memory, DOS-shaped: drives `A:`..`Z:`, `\`, names
//! compared without case. The tests run every file operation on it, with
//! failures put where a test wants them - a full disk, a read-only file,
//! a drive that is not ready - since a real disk cannot be made to fail
//! on demand.

use crate::fs::*;
use crate::path;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Clone, Debug)]
struct Node {
    /// The name as it was written, case and all.
    name: String,
    dir: bool,
    data: Vec<u8>,
    stamp: Stamp,
    attrs: Attrs,
}

struct Open {
    key: String,
    pos: usize,
    write: bool,
}

#[derive(Default)]
pub struct MemFs {
    /// Upper-cased full path to the node.
    nodes: BTreeMap<String, Node>,
    open: BTreeMap<Handle, Open>,
    next: Handle,
    cwd: String,
    /// Bytes a drive may hold in all, for the disk-full tests.
    capacity: BTreeMap<char, u64>,
    /// Paths (upper case) that fail with this error on any change to them.
    fail: BTreeMap<String, (ErrorKind, u32)>,
    /// Drives that are not ready.
    not_ready: Vec<char>,
    /// Drives that say "removable", as a floppy drive does.
    removable: Vec<char>,
    pub short_names: bool,
    /// Drives' volume labels, by letter.
    pub labels: BTreeMap<char, String>,
}

fn key(path: &str) -> String {
    path.trim_end_matches('\\').to_uppercase()
}

impl MemFs {
    /// Drives with nothing on them, the first one current.
    pub fn new(drives: &[char]) -> MemFs {
        let mut fs = MemFs { short_names: true, next: 1, ..MemFs::default() };
        for &d in drives {
            fs.nodes.insert(
                alloc::format!("{}:", d.to_ascii_uppercase()),
                Node { name: alloc::format!("{}:\\", d), dir: true, data: Vec::new(), stamp: Stamp::default(), attrs: Attrs::default() },
            );
        }
        fs.cwd = alloc::format!("{}:\\", drives.first().copied().unwrap_or('C').to_ascii_uppercase());
        fs
    }

    pub fn set_cwd(&mut self, dir: &str) {
        self.cwd = dir.to_string();
    }

    /// A folder and every folder above it.
    pub fn add_dir(&mut self, path: &str) {
        if let Some((parent, name)) = path::split(path, '\\') {
            if !self.nodes.contains_key(&key(&parent)) {
                self.add_dir(&parent);
            }
            self.nodes.entry(key(path)).or_insert(Node { name, dir: true, data: Vec::new(), stamp: Stamp::default(), attrs: Attrs::default() });
        }
    }

    /// A file with these bytes, its folders made as needed.
    pub fn add_file(&mut self, path: &str, data: &[u8]) {
        let (parent, name) = path::split(path, '\\').expect("a file has a folder");
        self.add_dir(&parent);
        self.nodes.insert(key(path), Node { name, dir: false, data: data.to_vec(), stamp: Stamp::default(), attrs: Attrs::default() });
    }

    pub fn set_stamp_of(&mut self, path: &str, stamp: Stamp) {
        if let Some(n) = self.nodes.get_mut(&key(path)) {
            n.stamp = stamp;
        }
    }

    pub fn set_attrs_of(&mut self, path: &str, attrs: Attrs) {
        if let Some(n) = self.nodes.get_mut(&key(path)) {
            n.attrs = attrs;
        }
    }

    /// The bytes of a file, for a test to look at.
    pub fn data(&self, path: &str) -> Option<&[u8]> {
        self.nodes.get(&key(path)).filter(|n| !n.dir).map(|n| n.data.as_slice())
    }

    pub fn exists(&self, path: &str) -> bool {
        self.nodes.contains_key(&key(path))
    }

    pub fn set_capacity(&mut self, drive: char, bytes: u64) {
        self.capacity.insert(drive.to_ascii_uppercase(), bytes);
    }

    /// A drive that says "removable" and so is never asked its space.
    pub fn set_removable(&mut self, drive: char) {
        self.removable.push(drive.to_ascii_uppercase());
    }

    /// Every change to this path fails with `kind`, `times` times (0: always).
    pub fn fail_on(&mut self, path: &str, kind: ErrorKind, times: u32) {
        self.fail.insert(key(path), (kind, times));
    }

    pub fn set_not_ready(&mut self, drive: char, not_ready: bool) {
        let d = drive.to_ascii_uppercase();
        self.not_ready.retain(|&x| x != d);
        if not_ready {
            self.not_ready.push(d);
        }
    }

    fn drive_of(path: &str) -> char {
        path.chars().next().unwrap_or('C').to_ascii_uppercase()
    }

    fn check(&mut self, path: &str) -> FsResult<()> {
        if self.not_ready.contains(&Self::drive_of(path)) {
            return Err(FsError::with_code(ErrorKind::NotReady, 21));
        }
        let k = key(path);
        if let Some((kind, times)) = self.fail.get_mut(&k) {
            let kind = *kind;
            if *times == 1 {
                self.fail.remove(&k);
            } else if *times > 1 {
                *times -= 1;
            }
            return Err(FsError::new(kind));
        }
        Ok(())
    }

    fn used(&self, drive: char) -> u64 {
        let prefix = alloc::format!("{}:", drive);
        self.nodes.iter().filter(|(k, n)| k.starts_with(&prefix) && !n.dir).map(|(_, n)| n.data.len() as u64).sum()
    }

    fn parent_is_dir(&self, path: &str) -> FsResult<()> {
        let (parent, _) = path::split(path, '\\').ok_or(FsError::new(ErrorKind::BadName))?;
        match self.nodes.get(&key(&parent)) {
            Some(n) if n.dir => Ok(()),
            _ => Err(FsError::new(ErrorKind::NotFound)),
        }
    }

    fn entry(n: &Node) -> Entry {
        Entry { name: n.name.clone(), dir: n.dir, size: n.data.len() as u64, stamp: n.stamp, attrs: n.attrs }
    }
}

impl Fs for MemFs {
    fn label(&mut self, root: &str) -> Option<String> {
        let d = root.chars().next()?.to_ascii_uppercase();
        self.labels.get(&d).cloned()
    }

    fn can_set_label(&self) -> bool {
        true
    }

    fn set_label(&mut self, root: &str, label: &str) -> FsResult<()> {
        let d = root.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or('C');
        if label.is_empty() {
            self.labels.remove(&d);
        } else {
            self.labels.insert(d, String::from(label));
        }
        Ok(())
    }

    fn sep(&self) -> char {
        '\\'
    }
    fn short_names(&self) -> bool {
        self.short_names
    }
    fn case_sensitive(&self) -> bool {
        false
    }

    fn drives(&mut self) -> Vec<Drive> {
        self.nodes
            .keys()
            .filter(|k| k.len() == 2)
            .map(|k| {
                let letter = k.chars().next().unwrap();
                let what = if self.removable.contains(&letter) { "removable" } else { "memory" };
                Drive { root: alloc::format!("{}:\\", letter), letter, what: String::from(what) }
            })
            .collect()
    }

    fn current_dir(&mut self) -> String {
        self.cwd.clone()
    }

    fn read_dir(&mut self, dir: &str) -> FsResult<Vec<Entry>> {
        if self.not_ready.contains(&Self::drive_of(dir)) {
            return Err(FsError::with_code(ErrorKind::NotReady, 21));
        }
        let k = key(dir);
        match self.nodes.get(&k) {
            Some(n) if n.dir => {}
            _ => return Err(FsError::new(ErrorKind::NotFound)),
        }
        let prefix = alloc::format!("{}\\", k);
        Ok(self
            .nodes
            .iter()
            .filter(|(nk, _)| nk.starts_with(&prefix) && !nk[prefix.len()..].contains('\\'))
            .map(|(_, n)| Self::entry(n))
            .collect())
    }

    fn stat(&mut self, path: &str) -> FsResult<Entry> {
        if self.not_ready.contains(&Self::drive_of(path)) {
            return Err(FsError::with_code(ErrorKind::NotReady, 21));
        }
        self.nodes.get(&key(path)).map(Self::entry).ok_or(FsError::new(ErrorKind::NotFound))
    }

    fn free_space(&mut self, path: &str) -> Option<u64> {
        let d = Self::drive_of(path);
        self.capacity.get(&d).map(|c| c.saturating_sub(self.used(d)))
    }

    fn disk_space(&mut self, root: &str) -> Option<(u64, u64)> {
        let d = Self::drive_of(root);
        if self.removable.contains(&d) {
            return None;
        }
        let size = *self.capacity.get(&d)?;
        Some((size, size.saturating_sub(self.used(d))))
    }

    fn make_dir(&mut self, path: &str) -> FsResult<()> {
        self.check(path)?;
        if self.nodes.contains_key(&key(path)) {
            return Err(FsError::new(ErrorKind::Exists));
        }
        self.parent_is_dir(path)?;
        let name = path::name(path, '\\');
        if name.contains(['*', '?', '<', '>', '|', '"']) {
            return Err(FsError::new(ErrorKind::BadName));
        }
        self.nodes.insert(key(path), Node { name, dir: true, data: Vec::new(), stamp: Stamp::default(), attrs: Attrs::default() });
        Ok(())
    }

    fn remove_file(&mut self, path: &str) -> FsResult<()> {
        self.check(path)?;
        match self.nodes.get(&key(path)) {
            None => Err(FsError::new(ErrorKind::NotFound)),
            Some(n) if n.dir => Err(FsError::new(ErrorKind::Denied)),
            Some(n) if n.attrs.read_only => Err(FsError::new(ErrorKind::ReadOnly)),
            Some(_) => {
                self.nodes.remove(&key(path));
                Ok(())
            }
        }
    }

    fn remove_dir(&mut self, path: &str) -> FsResult<()> {
        self.check(path)?;
        let k = key(path);
        match self.nodes.get(&k) {
            Some(n) if n.dir => {}
            _ => return Err(FsError::new(ErrorKind::NotFound)),
        }
        let prefix = alloc::format!("{}\\", k);
        if self.nodes.keys().any(|nk| nk.starts_with(&prefix)) {
            return Err(FsError::new(ErrorKind::NotEmpty));
        }
        self.nodes.remove(&k);
        Ok(())
    }

    fn rename(&mut self, from: &str, to: &str) -> FsResult<()> {
        self.check(from)?;
        self.check(to)?;
        if Self::drive_of(from) != Self::drive_of(to) {
            return Err(FsError::new(ErrorKind::CrossDevice));
        }
        let (kf, kt) = (key(from), key(to));
        if !self.nodes.contains_key(&kf) {
            return Err(FsError::new(ErrorKind::NotFound));
        }
        // Only a change of case may rename onto the same key.
        if kf != kt && self.nodes.contains_key(&kt) {
            return Err(FsError::new(ErrorKind::Exists));
        }
        self.parent_is_dir(to)?;
        let prefix = alloc::format!("{}\\", kf);
        let moving: Vec<String> = self.nodes.keys().filter(|k| **k == kf || k.starts_with(&prefix)).cloned().collect();
        for k in moving {
            let mut n = self.nodes.remove(&k).unwrap();
            let nk = alloc::format!("{}{}", kt, &k[kf.len()..]);
            if k == kf {
                n.name = path::name(to, '\\');
            }
            self.nodes.insert(nk, n);
        }
        Ok(())
    }

    fn set_attrs(&mut self, path: &str, attrs: Attrs) -> FsResult<()> {
        self.check(path)?;
        let n = self.nodes.get_mut(&key(path)).ok_or(FsError::new(ErrorKind::NotFound))?;
        n.attrs = attrs;
        Ok(())
    }

    fn set_stamp(&mut self, path: &str, stamp: Stamp) -> FsResult<()> {
        let n = self.nodes.get_mut(&key(path)).ok_or(FsError::new(ErrorKind::NotFound))?;
        n.stamp = stamp;
        Ok(())
    }

    fn open_read(&mut self, path: &str) -> FsResult<Handle> {
        if self.not_ready.contains(&Self::drive_of(path)) {
            return Err(FsError::with_code(ErrorKind::NotReady, 21));
        }
        match self.nodes.get(&key(path)) {
            Some(n) if !n.dir => {}
            Some(_) => return Err(FsError::new(ErrorKind::Denied)),
            None => return Err(FsError::new(ErrorKind::NotFound)),
        }
        let h = self.next;
        self.next += 1;
        self.open.insert(h, Open { key: key(path), pos: 0, write: false });
        Ok(h)
    }

    fn create(&mut self, path: &str) -> FsResult<Handle> {
        self.check(path)?;
        self.parent_is_dir(path)?;
        let k = key(path);
        match self.nodes.get_mut(&k) {
            Some(n) if n.dir => return Err(FsError::new(ErrorKind::Denied)),
            Some(n) if n.attrs.read_only => return Err(FsError::new(ErrorKind::ReadOnly)),
            Some(n) => n.data.clear(),
            None => {
                self.nodes.insert(
                    k.clone(),
                    Node { name: path::name(path, '\\'), dir: false, data: Vec::new(), stamp: Stamp::default(), attrs: Attrs::default() },
                );
            }
        }
        let h = self.next;
        self.next += 1;
        self.open.insert(h, Open { key: k, pos: 0, write: true });
        Ok(h)
    }

    fn read(&mut self, h: Handle, buf: &mut [u8]) -> FsResult<usize> {
        let o = self.open.get_mut(&h).ok_or(FsError::new(ErrorKind::Io))?;
        let n = self.nodes.get(&o.key).ok_or(FsError::new(ErrorKind::NotFound))?;
        let k = (n.data.len().saturating_sub(o.pos)).min(buf.len());
        buf[..k].copy_from_slice(&n.data[o.pos..o.pos + k]);
        o.pos += k;
        Ok(k)
    }

    fn write(&mut self, h: Handle, buf: &[u8]) -> FsResult<()> {
        let (k, write) = match self.open.get(&h) {
            Some(o) => (o.key.clone(), o.write),
            None => return Err(FsError::new(ErrorKind::Io)),
        };
        if !write {
            return Err(FsError::new(ErrorKind::Denied));
        }
        let d = k.chars().next().unwrap_or('C');
        if let Some(&cap) = self.capacity.get(&d) {
            if self.used(d) + buf.len() as u64 > cap {
                return Err(FsError::with_code(ErrorKind::DiskFull, 39));
            }
        }
        let n = self.nodes.get_mut(&k).ok_or(FsError::new(ErrorKind::NotFound))?;
        n.data.extend_from_slice(buf);
        Ok(())
    }

    fn close(&mut self, h: Handle) -> FsResult<()> {
        self.open.remove(&h).map(|_| ()).ok_or(FsError::new(ErrorKind::Io))
    }
}
