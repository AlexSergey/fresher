//! The names of a path under `~/Library/Logs`, read as bytes, never as UTF-8: which place it is in, whether it is
//! another module's, whether its name marks an app's state or a lock, its identity and its folder's text. Case is
//! ignored (ASCII), as the file system ignores it: a wider refusal only keeps more.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Component, Path};

/// The folder of crash and diagnostic reports, the first name below `Logs`.
const CRASH_REPORTS: &[u8] = b"DiagnosticReports";

/// The folder of the `jetbrains` module, the first name below `Logs`.
const JETBRAINS: &[u8] = b"JetBrains";

/// The suffixes of an app's state files and of lock files (design, section 4, row 3).
const STATE_OR_LOCK_SUFFIXES: [&[u8]; 7] =
    [b".plist", b".db", b".sqlite", b".sqlite-wal", b".sqlite-shm", b".lock", b".pid"];

/// The two places of the module's categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Place {
    /// Anywhere below `Logs` but the crash reports.
    AppLogs,
    /// Below `Logs/DiagnosticReports`.
    CrashReports,
}

/// A path's place and its names below the place's folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Located<'a> {
    pub(crate) place: Place,
    /// The names below `Logs` for app logs, below `Logs/DiagnosticReports` for crash reports; never empty.
    pub(crate) below: Vec<&'a OsStr>,
}

/// Where `path` is below `logs`: `None` when it is not strictly inside `logs` by components, or when its first name
/// below `logs` is another module's: `JetBrains`, or a folder shaped as a bundle id (a name with more names after it;
/// a top-level file shaped as one is not another module's).
pub(crate) fn locate<'a>(logs: &Path, path: &'a Path) -> Option<Located<'a>> {
    let below = path
        .strip_prefix(logs)
        .ok()?
        .components()
        .map(|component| match component {
            Component::Normal(name) => Some(name),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let first = below.first()?.as_bytes();
    if first.eq_ignore_ascii_case(JETBRAINS) || (below.len() > 1 && fresher_core::is_bundle_id_shaped(first)) {
        return None;
    }
    match below.split_first() {
        Some((_, rest)) if first.eq_ignore_ascii_case(CRASH_REPORTS) && !rest.is_empty() => {
            Some(Located { place: Place::CrashReports, below: rest.to_vec() })
        }
        _ => Some(Located { place: Place::AppLogs, below }),
    }
}

/// Whether `name` marks an app's state or a lock: it ends with one of the suffixes, ASCII case ignored, and is longer
/// than the suffix.
pub(crate) fn is_state_or_lock(name: &OsStr) -> bool {
    let name = name.as_bytes();
    STATE_OR_LOCK_SUFFIXES.iter().any(|suffix| {
        name.len() > suffix.len()
            && name.get(name.len() - suffix.len()..).is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
    })
}

/// The names as they go into a finding's identity: each name's UTF-8 as it is, a backslash doubled, every other byte
/// as `\xNN`, joined by `/`. Two paths never share an identity.
pub(crate) fn identity(names: &[&OsStr]) -> String {
    names.iter().map(|name| escaped(name)).collect::<Vec<_>>().join("/")
}

/// The names but the last, lossy UTF-8, joined by `/`: the folder a file is in, for a sentence. `None` for a file at
/// the top of its place.
pub(crate) fn folder_text(names: &[&OsStr]) -> Option<String> {
    let (_, folders) = names.split_last()?;
    if folders.is_empty() {
        return None;
    }
    Some(folders.iter().map(|name| name.to_string_lossy()).collect::<Vec<_>>().join("/"))
}

fn escaped(name: &OsStr) -> String {
    let mut escaped = String::new();
    for chunk in name.as_bytes().utf8_chunks() {
        escaped.push_str(&chunk.valid().replace('\\', "\\\\"));
        for &byte in chunk.invalid() {
            let digit = |nibble: u8| char::from_digit(u32::from(nibble), 16).unwrap_or('0').to_ascii_uppercase();
            escaped.extend(['\\', 'x', digit(byte >> 4), digit(byte & 0xF)]);
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    const LOGS: &str = "/home/Library/Logs";

    fn os(name: &str) -> &OsStr {
        OsStr::new(name)
    }

    fn logs() -> &'static Path {
        Path::new(LOGS)
    }

    mod negative {
        use std::collections::BTreeSet;
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;
        use std::path::{Path, PathBuf};

        use super::{logs, os};
        use crate::names::{identity, is_state_or_lock, locate};

        #[test]
        fn a_path_outside_the_logs_has_no_place() {
            for path in ["/home/Library/Logsx/a", "/home/Library/Logs", "/home/Documents/Logs/a.log", "/home/Library"] {
                assert_eq!(locate(logs(), Path::new(path)), None, "{path}");
            }
        }

        #[test]
        fn a_path_with_a_parent_step_below_the_logs_has_no_place() {
            assert_eq!(locate(logs(), Path::new("/home/Library/Logs/../Preferences/x")), None);
        }

        #[test]
        fn a_jetbrains_path_has_no_place() {
            for path in ["JetBrains/IdeaIC2025.1/idea.log", "jetbrains/x/y.log", "JETBRAINS", "JetBrains"] {
                assert_eq!(locate(logs(), &logs().join(path)), None, "{path}");
            }
        }

        #[test]
        fn a_file_in_a_bundle_id_top_folder_has_no_place() {
            for path in ["com.example.App/app.log", "com.example.App/a/b/c.log", "com.apple.Safari/x.log", "a.b/c"] {
                assert_eq!(locate(logs(), &logs().join(path)), None, "{path}");
            }
        }

        #[test]
        fn a_name_like_but_not_a_state_or_lock_is_not_one() {
            for name in ["plist", ".plist", "x.plist.log", "x.dbx", "lockfile", "x.sqlite-journal", "pid", ""] {
                assert!(!is_state_or_lock(os(name)), "{name}");
            }
        }

        #[test]
        fn two_paths_differing_only_in_bytes_that_are_not_utf8_never_share_an_identity() {
            let names: [&[u8]; 4] = [b"log\xff", b"log\xfe", b"log\\xFF", b"log\xef\xbf\xbd"];

            let identities: BTreeSet<String> = names
                .iter()
                .map(|name| {
                    let path: PathBuf = logs().join("Vendor").join(OsStr::from_bytes(name));
                    identity(&locate(logs(), &path).unwrap().below)
                })
                .collect();

            assert_eq!(identities.len(), names.len(), "{identities:?}");
        }
    }

    mod positive {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;

        use super::{logs, os};
        use crate::names::{Place, folder_text, identity, is_state_or_lock, locate};

        #[test]
        fn a_file_at_the_top_is_an_app_log_with_no_folder() {
            let path = logs().join("Install Log.txt");

            let located = locate(logs(), &path).unwrap();

            assert_eq!(located.place, Place::AppLogs);
            assert_eq!(located.below, [os("Install Log.txt")]);
            assert_eq!(folder_text(&located.below), None);
            assert_eq!(identity(&located.below), "Install Log.txt");
        }

        #[test]
        fn a_deeper_file_is_an_app_log_with_its_folder() {
            let path = logs().join("Vendor/Updater/1.0/updater.log");

            let located = locate(logs(), &path).unwrap();

            assert_eq!(located.place, Place::AppLogs);
            assert_eq!(folder_text(&located.below).as_deref(), Some("Vendor/Updater/1.0"));
            assert_eq!(identity(&located.below), "Vendor/Updater/1.0/updater.log");
        }

        #[test]
        fn a_diagnostic_report_is_a_crash_report() {
            for (path, below) in [
                ("DiagnosticReports/App-2026-11-01.ips", "App-2026-11-01.ips"),
                ("diagnosticreports/x.ips", "x.ips"),
                ("DiagnosticReports/Retired/x.ips", "Retired/x.ips"),
            ] {
                let path_buf = logs().join(path);
                let located = locate(logs(), &path_buf).unwrap();

                assert_eq!(located.place, Place::CrashReports, "{path}");
                assert_eq!(identity(&located.below), below, "{path}");
            }
        }

        #[test]
        fn a_top_level_file_shaped_as_a_bundle_id_is_an_app_log() {
            for name in ["com.example.App.log", "fsck_apfs.log", "system.log", "install.log"] {
                let path = logs().join(name);

                let located = locate(logs(), &path).unwrap();

                assert_eq!(located.place, Place::AppLogs, "{name}");
                assert_eq!(located.below, [os(name)], "{name}");
                assert_eq!(folder_text(&located.below), None, "{name}");
            }
        }

        #[test]
        fn the_diagnostic_reports_name_alone_is_an_app_log() {
            let path = logs().join("DiagnosticReports");
            let located = locate(logs(), &path).unwrap();

            assert_eq!(located.place, Place::AppLogs);
            assert_eq!(located.below, [os("DiagnosticReports")]);
        }

        #[test]
        fn every_state_and_lock_suffix_is_one() {
            for suffix in [".plist", ".db", ".sqlite", ".sqlite-wal", ".sqlite-shm", ".lock", ".pid"] {
                let lower = format!("x{suffix}");
                let upper = lower.to_ascii_uppercase();

                assert!(is_state_or_lock(os(&lower)), "{lower}");
                assert!(is_state_or_lock(os(&upper)), "{upper}");
            }
            assert!(is_state_or_lock(OsStr::from_bytes(b"\xff.plist")));
        }

        #[test]
        fn a_utf8_path_is_its_own_identity_and_other_bytes_are_escaped() {
            assert_eq!(identity(&[os("Vendor"), os("na\u{ef}ve \\x.log")]), "Vendor/na\u{ef}ve \\\\x.log");
            assert_eq!(identity(&[OsStr::from_bytes(b"a\xffb"), os("c")]), "a\\xFFb/c");
            assert_eq!(folder_text(&[OsStr::from_bytes(b"a\xffb"), os("c")]).as_deref(), Some("a\u{fffd}b"));
        }
    }
}
