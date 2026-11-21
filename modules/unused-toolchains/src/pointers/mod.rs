//! What points at a version: the parsers of a manager's pointer files and of the projects' pin files. Pure
//! functions on text and names; nothing here reads a file. Whatever cannot be read with certainty is `Unknown`,
//! which blocks offers, never "nothing points at it".

use std::collections::BTreeSet;

pub(crate) mod nvm;
pub(crate) mod rustup;
pub(crate) mod tool_versions;

/// The dot-separated numbers of `text`, none empty and all digits.
pub(crate) fn numbers(text: &str) -> Option<Vec<u32>> {
    text.split('.')
        .map(|part| if part.bytes().all(|byte| byte.is_ascii_digit()) { part.parse().ok() } else { None })
        .collect()
}

/// `text` without the byte order mark an editor may have put first.
pub(crate) fn without_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// What a reference keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Resolved {
    /// The installed versions it keeps, as indices into the installed versions of a row.
    Keeps(BTreeSet<usize>),
    /// It points at no version of the manager (`system`, a linked toolchain path).
    Nothing,
    /// The module cannot tell.
    Unknown,
}

/// The resolved pointers of one row: the manager's default and every other pointer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pointers {
    pub(crate) default: Resolved,
    pub(crate) others: Vec<Resolved>,
}

impl Pointers {
    /// Whether the default or another pointer keeps the version at `index`.
    pub(crate) fn keeps(&self, index: usize) -> bool {
        std::iter::once(&self.default)
            .chain(&self.others)
            .any(|resolved| matches!(resolved, Resolved::Keeps(kept) if kept.contains(&index)))
    }

    pub(crate) fn default_unknown(&self) -> bool {
        self.default == Resolved::Unknown
    }

    pub(crate) fn other_unknown(&self) -> bool {
        self.others.contains(&Resolved::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keeps(indices: &[usize]) -> Resolved {
        Resolved::Keeps(indices.iter().copied().collect())
    }

    mod negative {
        use super::*;

        #[test]
        fn nothing_and_unknown_keep_no_version() {
            let pointers = Pointers { default: Resolved::Nothing, others: vec![Resolved::Unknown] };

            assert!(!pointers.keeps(0));
        }

        #[test]
        fn a_version_no_pointer_names_is_not_kept() {
            let pointers = Pointers { default: keeps(&[1]), others: vec![keeps(&[2])] };

            assert!(!pointers.keeps(0));
        }

        #[test]
        fn known_pointers_are_not_unknown() {
            let pointers = Pointers { default: Resolved::Nothing, others: vec![keeps(&[0])] };

            assert!(!pointers.default_unknown());
            assert!(!pointers.other_unknown());
        }
    }

    mod positive {
        use super::*;

        #[test]
        fn the_default_keeps_its_version() {
            let pointers = Pointers { default: keeps(&[3]), others: vec![] };

            assert!(pointers.keeps(3));
        }

        #[test]
        fn another_pointer_keeps_its_version() {
            let pointers = Pointers { default: Resolved::Nothing, others: vec![keeps(&[0]), keeps(&[2])] };

            assert!(pointers.keeps(2));
        }

        #[test]
        fn an_unknown_default_and_an_unknown_other_are_reported() {
            let pointers = Pointers { default: Resolved::Unknown, others: vec![Resolved::Nothing, Resolved::Unknown] };

            assert!(pointers.default_unknown());
            assert!(pointers.other_unknown());
        }
    }
}
