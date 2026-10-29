//! How a name, a size and a date are written in a panel (docs/02-panels.md, 2.2).

use crate::fs::{Entry, Stamp};
use crate::path;
use alloc::string::String;

/// A name in a column `width` cells wide.
///
/// 8.3: name and extension in two sub-columns, no dot, folders upper and
/// files lower case (`case`; off: as the disk has them), `░` between them
/// for a hidden or system file. A long name: as it is, cut with `}` in the
/// last cell.
pub fn panel_name(e: &Entry, short: bool, case: bool, width: usize) -> String {
    if e.is_up() {
        return pad("..", width);
    }
    if short && width >= 12 {
        let (n, x) = path::split_ext(&e.name);
        if n.chars().count() <= 8 && x.chars().count() <= 3 {
            let up = |s: &str| {
                if !case {
                    String::from(s)
                } else if e.dir {
                    s.to_uppercase()
                } else {
                    s.to_lowercase()
                }
            };
            let mut s = pad(&up(n), 8);
            s.push(if e.attrs.hidden || e.attrs.system { '░' } else { ' ' });
            s.push_str(&pad(&up(x), 3));
            return pad(&s, width);
        }
    }
    cut(&e.name, width)
}

/// `s` in `width` cells: padded, or cut with `}` in the last cell.
pub fn cut(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n <= width {
        return pad(s, width);
    }
    let mut out: String = s.chars().take(width.saturating_sub(1)).collect();
    if width > 0 {
        out.push('}');
    }
    out
}

pub fn pad(s: &str, width: usize) -> String {
    let mut out: String = s.chars().take(width).collect();
    let n = out.chars().count();
    for _ in n..width {
        out.push(' ');
    }
    out
}

pub fn pad_left(s: &str, width: usize) -> String {
    let n = s.chars().count();
    let mut out = String::new();
    for _ in n..width {
        out.push(' ');
    }
    out.push_str(s);
    out
}

/// `1 234 567`.
pub fn thousands(n: u64) -> String {
    let digits = alloc::format!("{}", n);
    let mut out = String::new();
    let len = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

/// The size column, nine cells: `►UP--DIR◄`, `►SUB-DIR◄`, or the size -
/// in K, M, G when the bytes do not fit.
pub fn size_text(e: &Entry) -> String {
    if e.is_up() {
        return String::from("►UP--DIR◄");
    }
    if e.dir {
        return String::from("►SUB-DIR◄");
    }
    fit_size(e.size, 9)
}

pub fn fit_size(n: u64, width: usize) -> String {
    let s = thousands(n);
    if s.chars().count() <= width {
        return s;
    }
    for (unit, div) in [("K", 1u64 << 10), ("M", 1 << 20), ("G", 1 << 30), ("T", 1 << 40)] {
        let s = alloc::format!("{} {}", thousands(n / div), unit);
        if s.chars().count() <= width {
            return s;
        }
    }
    String::from("?")
}

/// A short size for messages: `120 K`, `3.1 M`.
pub fn human(n: u64) -> String {
    if n < 1024 {
        return alloc::format!("{} bytes", n);
    }
    if n < 1 << 20 {
        return alloc::format!("{} K", n.div_ceil(1024));
    }
    let tenths = n * 10 / (1 << 20);
    alloc::format!("{}.{} M", tenths / 10, tenths % 10)
}

/// `dd-mm-yy`.
pub fn date(s: &Stamp) -> String {
    alloc::format!("{:02}-{:02}-{:02}", s.day, s.month, s.year % 100)
}

/// `hh:mm`.
pub fn time(s: &Stamp) -> String {
    alloc::format!("{:02}:{:02}", s.hour, s.minute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_three() {
        assert_eq!(panel_name(&Entry::file("HELLO.ASM", 0), true, true, 12), "hello    asm");
        assert_eq!(panel_name(&Entry::folder("L07"), true, true, 12), "L07         ");
        assert_eq!(panel_name(&Entry::folder(".."), true, true, 12), "..          ");
        // The case option off: the name as the disk has it.
        assert_eq!(panel_name(&Entry::file("HELLO.ASM", 0), true, false, 12), "HELLO    ASM");
    }

    #[test]
    fn long_names_are_cut() {
        assert_eq!(panel_name(&Entry::file("a-very-long-name.txt", 0), false, true, 10), "a-very-lo}");
    }

    #[test]
    fn sizes() {
        assert_eq!(thousands(1234567), "1 234 567");
        assert_eq!(thousands(412), "412");
        assert_eq!(fit_size(12_345_678_901, 9), "11 773 M");
    }
}
