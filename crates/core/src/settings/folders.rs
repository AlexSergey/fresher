//! Which folders a `Folders` setting may hold: `~`, a folder under it, or an absolute path, each written plainly
//! (no `.` or `..`), and never a protected system folder (docs/restrictions/deletion-safety.md).

use std::path::Path;

/// A system folder no setting may name, with how far the refusal reaches. Compared without regard to ASCII case, as
/// the macOS file system does by default.
enum Protected {
    /// The folder, everything inside it and every folder above it.
    Whole(&'static [&'static str]),
    /// The folder and every folder above it; a folder inside it may be named.
    Itself(&'static [&'static str]),
}

/// The protected paths of docs/restrictions/deletion-safety.md (`/var/db` is `/private/var/db` through its link),
/// and `/Users`, which holds other users' homes. `/` is above all of them.
const PROTECTED: &[Protected] = &[
    Protected::Whole(&["System"]),
    Protected::Whole(&["bin"]),
    Protected::Whole(&["sbin"]),
    Protected::Whole(&["private", "var", "db"]),
    Protected::Whole(&["var", "db"]),
    Protected::Whole(&["usr"]),
    Protected::Itself(&["Users"]),
];

/// Folders inside a protected folder that may be named all the same.
const ALLOWED_INSIDE: &[&[&str]] = &[&["usr", "local"]];

/// `true` when `folder` is `~`, `~/…` or an absolute path of plain names that is not protected.
pub(crate) fn is_allowed(folder: &Path) -> bool {
    let Some(text) = folder.to_str() else { return false };
    let text = text.strip_suffix('/').filter(|rest| !rest.is_empty()).unwrap_or(text);
    if text == "~" {
        return true;
    }
    if let Some(under_home) = text.strip_prefix("~/") {
        return plain_names(under_home).is_some();
    }
    // `/` itself has one empty name, so it is refused here too.
    text.strip_prefix('/').and_then(plain_names).is_some_and(|names| !is_protected(&names))
}

/// `true` when `path` is an absolute path of plain names, wherever it is (`/` itself has no name and is refused).
pub(crate) fn is_plain_absolute(path: &Path) -> bool {
    path.to_str().and_then(|text| text.strip_prefix('/')).and_then(plain_names).is_some()
}

/// The names of `path`, split at `/`; `None` when one of them is empty, `.`, `..` or holds a NUL.
fn plain_names(path: &str) -> Option<Vec<&str>> {
    let names: Vec<&str> = path.split('/').collect();
    let plain = |name: &&str| !name.is_empty() && *name != "." && *name != ".." && !name.contains('\0');
    names.iter().all(plain).then_some(names)
}

fn is_protected(names: &[&str]) -> bool {
    let starts_with = |path: &[&str], prefix: &[&str]| {
        path.len() >= prefix.len() && path.iter().zip(prefix).all(|(name, other)| name.eq_ignore_ascii_case(other))
    };
    if ALLOWED_INSIDE.iter().any(|allowed| starts_with(names, allowed)) {
        return false;
    }
    PROTECTED.iter().any(|protected| match protected {
        Protected::Whole(path) => starts_with(names, path) || starts_with(path, names),
        Protected::Itself(path) => starts_with(path, names),
    })
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use crate::settings::folders::{is_allowed, is_plain_absolute};

        #[test]
        fn a_plain_absolute_path_has_no_tilde_dots_or_empty_names() {
            for path in ["", "/", "~", "~/a", "a/b", "/a/../b", "/a/./b", "/a//b", "/a/"] {
                assert!(!is_plain_absolute(Path::new(path)), "{path:?}");
            }
        }

        #[test]
        fn refuses_a_folder_that_is_not_home_based_or_absolute() {
            for folder in ["", "Projects", "./Projects", "~user", "~user/Projects", "~~", "-"] {
                assert!(!is_allowed(Path::new(folder)), "{folder:?}");
            }
        }

        #[test]
        fn refuses_dots_and_empty_names() {
            for folder in
                ["~/..", "~/../etc", "~/./Projects", "~/a//b", "/Volumes/../System", "/./Volumes", "/a/.", "~/a\0b"]
            {
                assert!(!is_allowed(Path::new(folder)), "{folder:?}");
            }
        }

        #[test]
        fn refuses_a_protected_folder_inside_it_or_above_it() {
            let protected = [
                "/",
                "//",
                "/System",
                "/System/Library/Caches",
                "/system",
                "/usr",
                "/usr/bin",
                "/USR/lib",
                "/bin",
                "/sbin/",
                "/private",
                "/private/var",
                "/private/var/db",
                "/private/var/db/uuidtext",
                "/var",
                "/var/db",
                "/Users",
            ];

            for folder in protected {
                assert!(!is_allowed(Path::new(folder)), "{folder:?}");
            }
        }
    }

    mod positive {
        use std::path::Path;

        use crate::settings::folders::{is_allowed, is_plain_absolute};

        #[test]
        fn a_plain_absolute_path_may_be_anywhere() {
            for path in ["/System", "/Users/ann/Library/Caches/keep", "/usr/bin/x"] {
                assert!(is_plain_absolute(Path::new(path)), "{path:?}");
            }
        }

        #[test]
        fn allows_the_home_folder_and_folders_under_it() {
            for folder in ["~", "~/", "~/Projects", "~/Library/Caches", "~/My Projects/app/"] {
                assert!(is_allowed(Path::new(folder)), "{folder:?}");
            }
        }

        #[test]
        fn allows_an_absolute_folder_outside_the_protected_ones() {
            for folder in [
                "/Volumes/Work",
                "/usr/local",
                "/usr/local/Cellar",
                "/Users/me",
                "/private/var/folders",
                "/opt/homebrew",
                "/Systems",
            ] {
                assert!(is_allowed(Path::new(folder)), "{folder:?}");
            }
        }
    }
}
