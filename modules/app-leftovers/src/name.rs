//! The bundle id a leftover is named after, and when two ids are one app's. Nothing here reads the disk.

use std::ffi::OsStr;

use fresher_core::BundleId;

/// The id `name` is named after: `name` without `suffix` (which it must end with, ASCII case ignored, as the file
/// system compares names), when what is left is a bundle id with at least one dot. `Cookies.binarycookies` gives
/// `Cookies`, which is no id: `None`.
pub(crate) fn app_id(name: &OsStr, suffix: Option<&str>) -> Option<BundleId> {
    let name = name.to_str()?;
    let id = match suffix {
        None => name,
        Some(suffix) => {
            let (id, end) = name.split_at_checked(name.len().checked_sub(suffix.len())?)?;
            if !end.eq_ignore_ascii_case(suffix) {
                return None;
            }
            id
        }
    };
    if !id.contains('.') {
        return None;
    }
    BundleId::new(id).ok()
}

/// Whether `a` and `b` are one app's: the same id, or one is the other followed by a dot and more
/// (`com.example.App.Helper` and `com.example.App`), case ignored. `com.example.Appx` is not `com.example.App`'s.
pub(crate) fn related(a: &BundleId, b: &BundleId) -> bool {
    let (a, b) = (a.folded(), b.folded());
    a == b || extends(a, b) || extends(b, a)
}

/// What Spotlight is asked for an id: the id, then each shorter id it extends that still has a dot
/// (`com.example.App.Helper` → also `com.example.App` and `com.example`), longest first, so a helper of an app kept
/// outside the apps folders finds its app.
pub(crate) fn search_ids(id: &BundleId) -> Vec<BundleId> {
    let mut ids = vec![id.clone()];
    let mut rest = id.as_str();
    while let Some((head, _)) = rest.rsplit_once('.') {
        if !head.contains('.') {
            break;
        }
        if let Ok(shorter) = BundleId::new(head) {
            ids.push(shorter);
        }
        rest = head;
    }
    ids
}

/// `long` is `short` followed by a dot and more.
fn extends(long: &str, short: &str) -> bool {
    long.strip_prefix(short).is_some_and(|rest| rest.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use fresher_core::BundleId;

    fn id(id: &str) -> BundleId {
        BundleId::new(id).unwrap()
    }

    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;

        use super::id;
        use crate::name::{app_id, related, search_ids};

        #[test]
        fn a_name_that_gives_no_id_with_a_dot_is_refused() {
            for (name, suffix) in [
                ("Notes", None),
                ("Cookies.binarycookies", Some(".binarycookies")),
                ("com.example.App.plistx", Some(".plist")),
                ("com.example.App", Some(".plist")),
                (".plist", Some(".plist")),
                ("com.exämple.App", None),
                ("com..example", None),
            ] {
                assert_eq!(app_id(OsStr::new(name), suffix), None, "{name}");
            }
        }

        #[test]
        fn a_name_that_is_not_utf8_is_refused() {
            assert_eq!(app_id(OsStr::from_bytes(b"com.example.\xff"), None), None);
        }

        #[test]
        fn ids_that_only_share_a_start_are_not_related() {
            assert!(!related(&id("com.example.Appx"), &id("com.example.App")));
            assert!(!related(&id("com.example.App"), &id("com.example.Appx")));
            assert!(!related(&id("com.example"), &id("org.example")));
        }

        #[test]
        fn an_id_of_one_dot_asks_spotlight_for_itself_alone() {
            assert_eq!(search_ids(&id("com.example")), [id("com.example")]);
        }
    }

    mod positive {
        use std::ffi::OsStr;

        use super::id;
        use crate::name::{app_id, related, search_ids};

        #[test]
        fn a_name_without_a_suffix_is_the_id() {
            assert_eq!(app_id(OsStr::new("com.example.App"), None), Some(id("com.example.App")));
        }

        #[test]
        fn a_name_with_its_suffix_gives_the_id_as_written_case_ignored() {
            assert_eq!(app_id(OsStr::new("com.example.App.plist"), Some(".plist")), Some(id("com.example.App")));
            let upper = app_id(OsStr::new("COM.EXAMPLE.APP.PLIST"), Some(".plist")).unwrap();
            assert_eq!(upper.as_str(), "COM.EXAMPLE.APP");
        }

        #[test]
        fn a_version_like_name_with_a_dot_is_shaped_as_an_id() {
            for name in ["IntelliJIdea2019.3", "Python3.11", "1.2.3"] {
                assert_eq!(app_id(OsStr::new(name), None), Some(id(name)), "{name}");
            }
        }

        #[test]
        fn the_same_id_in_another_case_and_a_helper_of_an_app_are_related() {
            assert!(related(&id("COM.example.app"), &id("com.example.App")));
            assert!(related(&id("com.example.App.Helper"), &id("com.example.App")));
            assert!(related(&id("com.example.App"), &id("com.example.App.Helper")));
        }

        #[test]
        fn spotlight_is_asked_for_the_id_and_every_shorter_id_it_extends() {
            assert_eq!(
                search_ids(&id("com.example.App.Helper")),
                [id("com.example.App.Helper"), id("com.example.App"), id("com.example")]
            );
            assert_eq!(search_ids(&id("com.example.App")), [id("com.example.App"), id("com.example")]);
        }
    }
}
