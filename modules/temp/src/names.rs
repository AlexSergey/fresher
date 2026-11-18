//! The names of temporary items, read as bytes, never as UTF-8: the lock and marker names programs use to say
//! something is in use, the system's own prefix, the apps an item is named after, and the item's identity. Case is
//! ignored, as the file system ignores it: a wider match only keeps more.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;

use fresher_core::BundleId;

/// Names that are exactly a lock or a system marker.
const LOCK_NAMES: [&[u8]; 8] = [
    b".X11-unix",
    b".ICE-unix",
    b".font-unix",
    b".XIM-unix",
    b"quota.user",
    b"quota.group",
    b".AddressBookLocks",
    b".LINKS",
];

/// Prefixes of lock names.
const LOCK_PREFIXES: [&[u8]; 3] = [b".vfs_rsrc_streams_", b"LCK..", b".#"];

/// Suffixes of lock names.
const LOCK_SUFFIXES: [&[u8]; 2] = [b".lock", b".pid"];

/// The display server's locks: `.X*-lock`.
const DISPLAY_LOCK: (&[u8], &[u8]) = (b".X", b"-lock");

/// What a part of macOS names its items by.
const SYSTEM_PREFIX: &[u8] = b"com.apple.";

/// Whether `name` is a lock or a system marker (design, section 7, row 2).
pub(crate) fn is_lock(name: &OsStr) -> bool {
    let name = name.as_bytes();
    let (display_start, display_end) = DISPLAY_LOCK;
    LOCK_NAMES.iter().any(|lock| name.eq_ignore_ascii_case(lock))
        || LOCK_PREFIXES.iter().any(|prefix| starts_with(name, prefix))
        || LOCK_SUFFIXES.iter().any(|suffix| ends_with(name, suffix))
        || (starts_with(name, display_start) && ends_with(name, display_end))
}

/// Whether `name` is a part of macOS's: it starts with `com.apple.` (design, section 7, row 4).
pub(crate) fn is_system(name: &OsStr) -> bool {
    starts_with(name.as_bytes(), SYSTEM_PREFIX)
}

/// Whether `name`, or `name` without one leading dot (`.com.example.App.XXXXXX`), is named after `id`: the id as the
/// whole name or followed by a dot and more.
pub(crate) fn named_after(name: &OsStr, id: &BundleId) -> bool {
    let id = id.folded().as_bytes();
    let name = name.as_bytes();
    [Some(name), name.strip_prefix(b".")].into_iter().flatten().any(|name| {
        starts_with(name, id) && name.get(id.len()..).is_some_and(|rest| rest.is_empty() || rest.starts_with(b"."))
    })
}

/// The name as it goes into a finding's identity: its UTF-8 as it is, a backslash doubled, every other byte as
/// `\xNN`. Two names never share an identity.
pub(crate) fn identity(name: &OsStr) -> String {
    let mut identity = String::new();
    for chunk in name.as_bytes().utf8_chunks() {
        identity.push_str(&chunk.valid().replace('\\', "\\\\"));
        for &byte in chunk.invalid() {
            let digit = |nibble: u8| char::from_digit(u32::from(nibble), 16).unwrap_or('0').to_ascii_uppercase();
            identity.extend(['\\', 'x', digit(byte >> 4), digit(byte & 0xF)]);
        }
    }
    identity
}

fn starts_with(name: &[u8], prefix: &[u8]) -> bool {
    name.get(..prefix.len()).is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

fn ends_with(name: &[u8], suffix: &[u8]) -> bool {
    name.len()
        .checked_sub(suffix.len())
        .and_then(|start| name.get(start..))
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    fn os(name: &str) -> &OsStr {
        OsStr::new(name)
    }

    fn editor() -> fresher_core::BundleId {
        fresher_core::BundleId::new("com.example.Editor").unwrap()
    }

    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;

        use super::{editor, os};
        use crate::names::{identity, is_lock, is_system, named_after};

        #[test]
        fn a_name_like_but_not_after_an_app_is_not_named_after_it() {
            for name in ["com.example.Editorx", "..com.example.Editor", "x.com.example.Editor", "com.example", ""] {
                assert!(!named_after(os(name), &editor()), "{name}");
            }
            assert!(!named_after(OsStr::from_bytes(b"com.example.Edito\xff"), &editor()));
        }

        #[test]
        fn two_names_differing_only_in_bytes_that_are_not_utf8_never_share_an_identity() {
            let names: [&[u8]; 4] = [b"tmp\xff", b"tmp\xfe", b"tmp\\xFF", b"tmp\xef\xbf\xbd"];

            let identities: std::collections::BTreeSet<String> =
                names.iter().map(|name| identity(OsStr::from_bytes(name))).collect();

            assert_eq!(identities.len(), names.len(), "{identities:?}");
        }

        #[test]
        fn a_name_like_but_not_of_the_system_is_not_system() {
            for name in ["com.apple", "com.applex.y", "xcom.apple.y", "apple.com", ""] {
                assert!(!is_system(os(name)), "{name}");
            }
        }

        #[test]
        fn a_name_like_but_not_a_lock_is_not_a_lock() {
            for name in ["lockfile", "x.locked", ".X1", "LCK", "LCK.", "x.pid.bak", "X11-unix", ".X0-locks", "pid", ""]
            {
                assert!(!is_lock(os(name)), "{name}");
            }
        }

        #[test]
        fn a_name_that_is_not_utf8_is_neither() {
            let name = OsStr::from_bytes(b"tmp\xff\xfe");

            assert!(!is_lock(name));
            assert!(!is_system(name));
        }
    }

    mod positive {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;

        use super::{editor, os};
        use crate::names::{identity, is_lock, is_system, named_after};

        #[test]
        fn a_name_after_an_app_with_or_without_one_leading_dot_is_named_after_it() {
            for name in
                ["com.example.Editor", "com.example.editor.tmp", ".com.example.Editor.XXXXXX", ".COM.EXAMPLE.EDITOR"]
            {
                assert!(named_after(os(name), &editor()), "{name}");
            }
            assert!(named_after(OsStr::from_bytes(b".com.example.Editor.\xff"), &editor()));
        }

        #[test]
        fn a_utf8_name_is_its_own_identity_and_other_bytes_are_escaped() {
            assert_eq!(identity(os("tmp.IKKNWYIRnq")), "tmp.IKKNWYIRnq");
            assert_eq!(identity(os("na\u{ef}ve \\x")), "na\u{ef}ve \\\\x");
            assert_eq!(identity(OsStr::from_bytes(b"a\xffb")), "a\\xFFb");
        }

        #[test]
        fn a_name_starting_with_the_systems_prefix_is_system() {
            for name in ["com.apple.someagent", "com.apple.x", "COM.APPLE.Finder"] {
                assert!(is_system(os(name)), "{name}");
            }
        }

        #[test]
        fn every_listed_lock_name_is_a_lock() {
            for name in [
                ".X0-lock",
                ".X-lock",
                ".X1024-lock",
                ".X11-unix",
                ".ICE-unix",
                ".font-unix",
                ".XIM-unix",
                "quota.user",
                "quota.group",
                ".vfs_rsrc_streams_0x1234",
                ".AddressBookLocks",
                ".LINKS",
                "app.lock",
                ".lock",
                "server.pid",
                "LCK..ttyS0",
                ".#notes.txt",
                "APP.LOCK",
            ] {
                assert!(is_lock(os(name)), "{name}");
            }
        }

        #[test]
        fn a_lock_name_is_read_as_bytes_whatever_comes_before_it() {
            assert!(is_lock(OsStr::from_bytes(b"\xff.lock")));
        }
    }
}
