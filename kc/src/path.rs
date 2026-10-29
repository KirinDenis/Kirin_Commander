//! Paths as strings in the machine's own form, with its separator.
//! A root is `C:\` (DOS, Windows, the browser's drives) or `/` (Linux).

use alloc::string::String;

/// The length of the root at the start of a path: 3 for `C:\`, 1 for `/`,
/// 0 for a path with no root.
pub fn root_len(path: &str, sep: char) -> usize {
    let b = path.as_bytes();
    if b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
        if b.len() >= 3 && b[2] as char == sep {
            return 3;
        }
        return 2;
    }
    if path.starts_with(sep) {
        return sep.len_utf8();
    }
    0
}

pub fn is_root(path: &str, sep: char) -> bool {
    let r = root_len(path, sep);
    r > 0 && r == path.len()
}

pub fn root(path: &str, sep: char) -> &str {
    &path[..root_len(path, sep)]
}

/// A name in a folder.
pub fn join(dir: &str, name: &str, sep: char) -> String {
    let mut s = String::from(dir);
    if !s.ends_with(sep) && !s.is_empty() {
        s.push(sep);
    }
    s.push_str(name);
    s
}

/// The folder a path is in, and the path's last name: `C:\A\B` gives
/// `C:\A` and `B`; `C:\A` gives `C:\` and `A`; a root gives nothing.
pub fn split(path: &str, sep: char) -> Option<(String, String)> {
    let r = root_len(path, sep);
    let body = path[r..].trim_end_matches(sep);
    if body.is_empty() {
        return None;
    }
    let full = &path[..r + body.len()];
    match full[r..].rfind(sep) {
        Some(i) => Some((String::from(&full[..r + i]), String::from(&full[r + i + sep.len_utf8()..]))),
        None => Some((String::from(&full[..r]), String::from(&full[r..]))),
    }
}

/// The last name of a path; a root is its own name.
pub fn name(path: &str, sep: char) -> String {
    match split(path, sep) {
        Some((_, n)) => n,
        None => String::from(path),
    }
}

/// The name before the last dot and the extension after it. A name that
/// starts with its only dot (`.profile`) has no extension.
pub fn split_ext(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i + 1..]),
        _ => (name, ""),
    }
}

/// Whether `inner` is `outer` or lies inside it: copying a folder there
/// would never end.
pub fn is_inside(inner: &str, outer: &str, sep: char, case_sensitive: bool) -> bool {
    let eq = |a: &str, b: &str| if case_sensitive { a == b } else { a.eq_ignore_ascii_case(b) };
    let outer = outer.trim_end_matches(sep);
    if inner.len() < outer.len() || !eq(&inner[..outer.len()], outer) {
        return false;
    }
    inner.len() == outer.len() || inner[outer.len()..].starts_with(sep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dos_paths() {
        assert_eq!(split("C:\\A\\B", '\\'), Some(("C:\\A".into(), "B".into())));
        assert_eq!(split("C:\\A", '\\'), Some(("C:\\".into(), "A".into())));
        assert_eq!(split("C:\\", '\\'), None);
        assert!(is_root("C:\\", '\\'));
        assert_eq!(join("C:\\", "A", '\\'), "C:\\A");
        assert_eq!(join("C:\\A", "B", '\\'), "C:\\A\\B");
    }

    #[test]
    fn linux_paths() {
        assert_eq!(split("/home/k", '/'), Some(("/home".into(), "k".into())));
        assert_eq!(split("/home", '/'), Some(("/".into(), "home".into())));
        assert_eq!(split("/", '/'), None);
    }

    #[test]
    fn inside() {
        assert!(is_inside("C:\\A\\B", "C:\\A", '\\', false));
        assert!(is_inside("c:\\a", "C:\\A", '\\', false));
        assert!(!is_inside("C:\\AB", "C:\\A", '\\', false));
    }

    #[test]
    fn extensions() {
        assert_eq!(split_ext("HELLO.ASM"), ("HELLO", "ASM"));
        assert_eq!(split_ext(".profile"), (".profile", ""));
        assert_eq!(split_ext("a.tar.gz"), ("a.tar", "gz"));
    }
}
