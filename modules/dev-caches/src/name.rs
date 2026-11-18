//! The `project-hash` shape of a derived data folder's name: `<project>-<28 letters>`.

use std::ffi::OsStr;

/// How many letters follow the project's name.
const HASH_LENGTH: usize = 28;

/// The project of a `project-hash` name: the name split at its last `-` into a non-empty project and exactly 28 ASCII
/// lowercase letters. `None` for any other name, and for a name that is not UTF-8.
pub(crate) fn project(name: &OsStr) -> Option<&str> {
    let (project, hash) = name.to_str()?.rsplit_once('-')?;
    let hash_fits = hash.len() == HASH_LENGTH && hash.bytes().all(|byte| byte.is_ascii_lowercase());
    (!project.is_empty() && hash_fits).then_some(project)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    /// 28 lowercase letters.
    const HASH: &str = "abcdefghijklmnopqrstuvwxyzab";

    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;

        use super::HASH;
        use crate::name::project;

        #[test]
        fn a_short_hash_is_refused() {
            assert_eq!(project(OsStr::new("Demo-abc")), None);
        }

        #[test]
        fn a_hash_of_27_or_29_letters_is_refused() {
            assert_eq!(project(OsStr::new("Demo-bcdefghijklmnopqrstuvwxyzab")), None);
            assert_eq!(project(OsStr::new(&format!("Demo-{HASH}a"))), None);
        }

        #[test]
        fn a_hash_with_an_upper_case_letter_or_a_digit_is_refused() {
            assert_eq!(project(OsStr::new("Demo-Abcdefghijklmnopqrstuvwxyzab")), None);
            assert_eq!(project(OsStr::new("Demo-abcdefghijklmnopqrstuvwxy1ab")), None);
        }

        #[test]
        fn a_name_with_no_project_or_no_dash_is_refused() {
            assert_eq!(project(OsStr::new(&format!("-{HASH}"))), None);
            assert_eq!(project(OsStr::new("Demo")), None);
            assert_eq!(project(OsStr::new(HASH)), None);
        }

        #[test]
        fn a_name_that_is_not_utf8_is_refused() {
            let mut name = b"Demo\xff-".to_vec();
            name.extend_from_slice(HASH.as_bytes());

            assert_eq!(project(OsStr::from_bytes(&name)), None);
        }
    }

    mod positive {
        use super::{HASH, OsStr};
        use crate::name::project;

        #[test]
        fn the_project_is_the_name_before_the_hash() {
            assert_eq!(project(OsStr::new(&format!("Demo-{HASH}"))), Some("Demo"));
        }

        #[test]
        fn a_project_with_dashes_keeps_them() {
            assert_eq!(project(OsStr::new(&format!("My-App-{HASH}"))), Some("My-App"));
        }
    }
}
