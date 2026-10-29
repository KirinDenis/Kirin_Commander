//! Masks (docs/02-panels.md, 2.6): `*` and `?`, several after `,` or `;`,
//! exclusions after `|`; case ignored everywhere; a mask with no wildcard
//! matches as if `*` followed it.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mask {
    take: Vec<String>,
    leave: Vec<String>,
    /// DOS's rule: `*.*` matches a name with no extension too.
    dos: bool,
}

impl Mask {
    pub fn parse(text: &str, dos: bool) -> Mask {
        let (take, leave) = match text.find('|') {
            Some(i) => (&text[..i], &text[i + 1..]),
            None => (text, ""),
        };
        let list = |s: &str| -> Vec<String> {
            s.split(|c| c == ',' || c == ';')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty())
                .map(|p| {
                    let mut p: String = p.chars().flat_map(|c| c.to_lowercase()).collect();
                    if !p.contains('*') && !p.contains('?') {
                        p.push('*');
                    }
                    p
                })
                .collect()
        };
        let mut take = list(take);
        if take.is_empty() {
            take.push(String::from("*"));
        }
        Mask { take, leave: list(leave), dos }
    }

    pub fn matches(&self, name: &str) -> bool {
        let name: String = name.chars().flat_map(|c| c.to_lowercase()).collect();
        let hit = |p: &String| self.one(p, &name);
        self.take.iter().any(hit) && !self.leave.iter().any(hit)
    }

    fn one(&self, pattern: &str, name: &str) -> bool {
        if self.dos && !name.contains('.') {
            // `*.*` and `NAME.*` reach a name that has no dot, as DOS's own
            // matching does: the name is taken to end in an empty extension.
            if let Some(stem) = pattern.strip_suffix(".*") {
                return wild(stem, name);
            }
        }
        wild(pattern, name)
    }
}

/// `*` any run, `?` any one character, compared by characters.
fn wild(p: &str, s: &str) -> bool {
    let p: Vec<char> = p.chars().collect();
    let s: Vec<char> = s.chars().collect();
    // The classic two-pointer match with one remembered star.
    let (mut i, mut j) = (0usize, 0usize);
    let mut star: Option<(usize, usize)> = None;
    while j < s.len() {
        if i < p.len() && (p[i] == '?' || p[i] == s[j]) {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == '*' {
            star = Some((i, j));
            i += 1;
        } else if let Some((si, sj)) = star {
            i = si + 1;
            j = sj + 1;
            star = Some((si, sj + 1));
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == '*' {
        i += 1;
    }
    i == p.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stars_and_marks() {
        let m = Mask::parse("*.asm", true);
        assert!(m.matches("HELLO.ASM"));
        assert!(!m.matches("HELLO.COM"));
        assert!(Mask::parse("h?llo.*", true).matches("hello.com"));
    }

    #[test]
    fn dos_star_dot_star_takes_a_name_without_extension() {
        assert!(Mask::parse("*.*", true).matches("LICENSE"));
        assert!(!Mask::parse("*.*", false).matches("LICENSE"));
        assert!(Mask::parse("*", false).matches("LICENSE"));
    }

    #[test]
    fn lists_and_exclusions() {
        let m = Mask::parse("*.asm, *.inc", true);
        assert!(m.matches("E_MATH.INC"));
        let m = Mask::parse("*.*|*.bak;*.tmp", true);
        assert!(m.matches("A.TXT"));
        assert!(!m.matches("A.BAK"));
        assert!(!m.matches("x.tmp"));
    }

    #[test]
    fn no_wildcard_is_a_prefix() {
        assert!(Mask::parse("read", false).matches("README.md"));
        assert!(!Mask::parse("read", false).matches("unread"));
    }
}
