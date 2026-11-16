//! Which subfolders the walk enters: not another volume, not an avoided place, never a protected path.

use std::fs;
use std::path::{Component, Path, PathBuf};

use fresher_core::NotScanned;

/// The system's folders no scan enters (docs/restrictions/deletion-safety.md); `/usr/local` is the one exception.
const PROTECTED: [&str; 5] = ["/System", "/usr", "/bin", "/sbin", "/private/var/db"];
const USR_LOCAL: &str = "/usr/local";
const USERS: &str = "/Users";

/// What the walk does with one subfolder of a listing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decision {
    Enter,
    /// Listed, not opened, reported.
    NotScanned(NotScanned),
    /// Not opened and not reported: a protected path. The settings check keeps roots away from them; this is the
    /// second guard.
    Protected,
}

/// The boundaries of one walk.
#[derive(Clone, Debug, Default)]
pub(crate) struct Boundary {
    avoid: Vec<PathBuf>,
    roots: Vec<PathBuf>,
}

impl Boundary {
    /// The boundaries of a walk of `roots` (canonical) avoiding `avoid`. Each avoided place is compared in its
    /// canonical form when it exists, as given otherwise; always by components, never as a string prefix.
    pub(crate) fn new(roots: &[PathBuf], avoid: &[PathBuf]) -> Self {
        let avoid = avoid.iter().map(|place| fs::canonicalize(place).unwrap_or_else(|_| place.clone())).collect();
        Self { avoid, roots: roots.to_vec() }
    }

    /// Whether the folder at `path`, on `device`, is entered by a walk whose root is on `root_device`.
    pub(crate) fn decide(&self, path: &Path, device: u64, root_device: u64) -> Decision {
        if self.is_protected(path) {
            Decision::Protected
        } else if device != root_device {
            Decision::NotScanned(NotScanned::OtherVolume)
        } else if self.is_avoided(path) {
            Decision::NotScanned(NotScanned::Avoided)
        } else {
            Decision::Enter
        }
    }

    /// Whether `path` itself is in the avoid list.
    pub(crate) fn is_avoided(&self, path: &Path) -> bool {
        self.avoid.iter().any(|place| place == path)
    }

    fn is_protected(&self, path: &Path) -> bool {
        let system = PROTECTED.iter().any(|protected| path.starts_with(protected)) && !path.starts_with(USR_LOCAL);
        system || self.is_another_home(path)
    }

    /// A folder right under `/Users` that holds no root: another user's home.
    fn is_another_home(&self, path: &Path) -> bool {
        let Ok(inside) = path.strip_prefix(USERS) else { return false };
        let mut names = inside.components();
        matches!((names.next(), names.next()), (Some(Component::Normal(_)), None))
            && !self.roots.iter().any(|root| root.starts_with(path))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::walk::boundary::Boundary;

    fn boundary(roots: &[&str], avoid: &[&str]) -> Boundary {
        let paths = |list: &[&str]| list.iter().map(PathBuf::from).collect::<Vec<_>>();
        Boundary::new(&paths(roots), &paths(avoid))
    }

    mod negative {
        use fresher_core::NotScanned;

        use super::{Path, boundary};
        use crate::walk::boundary::Decision;

        #[test]
        fn an_avoided_folder_is_not_entered() {
            let walk = boundary(&["/home/me"], &["/home/me/Documents"]);

            let decision = walk.decide(Path::new("/home/me/Documents"), 1, 1);

            assert_eq!(decision, Decision::NotScanned(NotScanned::Avoided));
        }

        #[test]
        fn a_folder_on_another_device_is_not_entered() {
            let walk = boundary(&["/home/me"], &[]);

            assert_eq!(walk.decide(Path::new("/home/me/mnt"), 2, 1), Decision::NotScanned(NotScanned::OtherVolume));
        }

        #[test]
        fn an_avoid_entry_that_is_a_prefix_string_does_not_avoid_its_neighbour() {
            let walk = boundary(&["/a"], &["/a/Desk"]);

            assert_eq!(walk.decide(Path::new("/a/Desktop"), 1, 1), Decision::Enter);
            assert!(!walk.is_avoided(Path::new("/a/Desk/inner")));
        }

        #[test]
        fn a_protected_path_is_never_entered_nor_reported() {
            let walk = boundary(&["/Users/me"], &[]);

            for path in ["/System", "/System/Library", "/usr", "/usr/lib", "/bin", "/sbin", "/private/var/db/x"] {
                assert_eq!(walk.decide(Path::new(path), 1, 1), Decision::Protected, "{path}");
            }
            assert_eq!(walk.decide(Path::new("/Users/someone-else"), 1, 1), Decision::Protected);
        }
    }

    mod positive {
        use super::{Path, boundary};
        use crate::walk::boundary::Decision;

        #[test]
        fn an_ordinary_folder_is_entered() {
            let walk = boundary(&["/Users/me"], &["/Users/me/Documents"]);

            for path in ["/Users/me/Library", "/usr/local/lib", "/Users/me/Documents/inner", "/private/var/folders"] {
                assert_eq!(walk.decide(Path::new(path), 1, 1), Decision::Enter, "{path}");
            }
        }

        #[test]
        fn the_home_of_a_root_is_not_another_users_home() {
            let walk = boundary(&["/Users/me/Projects"], &[]);

            assert_eq!(walk.decide(Path::new("/Users/me"), 1, 1), Decision::Enter);
            assert_eq!(walk.decide(Path::new("/Users/me/Library"), 1, 1), Decision::Enter);
        }
    }
}
