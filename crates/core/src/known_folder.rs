//! The places where the system keeps things, by meaning: a module names a known folder, never an OS path.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A place the system keeps things of a kind. Each operating system's adapter answers where it is; a module never
/// writes the path itself (the user's decision of 2026-11-15: macOS first, Windows not ruled out).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KnownFolder {
    /// The user's home folder.
    Home,
    /// The user's library of app data.
    Library,
    /// Where apps keep their caches.
    AppCaches,
    /// Where apps keep their support data.
    AppSupport,
    /// Where apps keep their settings.
    Preferences,
    /// Where apps keep their logs.
    Logs,
    /// The sandboxed apps' own folders.
    Containers,
    /// The folders apps of one developer share.
    GroupContainers,
    /// The windows apps reopen after a restart.
    SavedState,
    /// The user's Trash.
    Trash,
    /// The user's downloads.
    Downloads,
    /// The apps of every user.
    Applications,
    /// The apps of this user only.
    UserApplications,
    /// The apps that come with the system.
    SystemApplications,
    /// The user's own temporary folder.
    UserTemp,
    /// The temporary folder every user shares.
    SharedTemp,
}

impl KnownFolder {
    /// Every known folder, in a fixed order.
    pub const ALL: [Self; 16] = [
        Self::Home,
        Self::Library,
        Self::AppCaches,
        Self::AppSupport,
        Self::Preferences,
        Self::Logs,
        Self::Containers,
        Self::GroupContainers,
        Self::SavedState,
        Self::Trash,
        Self::Downloads,
        Self::Applications,
        Self::UserApplications,
        Self::SystemApplications,
        Self::UserTemp,
        Self::SharedTemp,
    ];

    /// Whether a scan walks it as a place of its own when it lies outside the scan roots and a module names it: the
    /// temporary folders only, so no other folder (the apps', say) ever becomes a place read in full on every scan.
    pub const fn is_walked_place(self) -> bool {
        matches!(self, Self::UserTemp | Self::SharedTemp)
    }

    /// The folder's name as rules write it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Library => "library",
            Self::AppCaches => "app-caches",
            Self::AppSupport => "app-support",
            Self::Preferences => "preferences",
            Self::Logs => "logs",
            Self::Containers => "containers",
            Self::GroupContainers => "group-containers",
            Self::SavedState => "saved-state",
            Self::Trash => "trash",
            Self::Downloads => "downloads",
            Self::Applications => "applications",
            Self::UserApplications => "user-applications",
            Self::SystemApplications => "system-applications",
            Self::UserTemp => "user-temp",
            Self::SharedTemp => "shared-temp",
        }
    }
}

impl fmt::Display for KnownFolder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Where this system keeps each known folder. Answers from what it was built with; opens nothing.
pub trait KnownFolders: Send + Sync {
    /// The absolute path of `folder`; `None` when this system has no such place.
    fn locate(&self, folder: KnownFolder) -> Option<PathBuf>;
}

/// `root` as the settings write it, its leading `~` component (`~` or `~/…`) filled in with `home`; any other root as
/// given (`~user` and a `~` further in are not expanded). Pure: it opens nothing.
pub fn expand_home(root: &Path, home: &Path) -> PathBuf {
    let mut parts = root.components();
    match parts.next() {
        Some(Component::Normal(first)) if first == OsStr::new("~") => home.join(parts.as_path()),
        _ => root.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use crate::{KnownFolder, expand_home};

        #[test]
        fn only_a_leading_tilde_component_is_expanded() {
            let home = Path::new("/Users/someone");

            for root in ["/Volumes/Data", "~user/x", "a/~", "/~", "~x"] {
                assert_eq!(expand_home(Path::new(root), home), Path::new(root), "{root}");
            }
        }

        #[test]
        fn only_the_temporary_folders_are_walked_places() {
            let walked: Vec<KnownFolder> =
                KnownFolder::ALL.into_iter().filter(|folder| folder.is_walked_place()).collect();

            assert_eq!(walked, [KnownFolder::UserTemp, KnownFolder::SharedTemp]);
        }

        #[test]
        fn an_unknown_folder_name_is_not_read() {
            for name in ["Home", "app_caches", "caches", ""] {
                assert!(serde_json::from_value::<KnownFolder>(name.into()).is_err(), "{name}");
            }
        }
    }

    mod positive {
        use std::collections::BTreeSet;
        use std::path::Path;

        use crate::{KnownFolder, expand_home};

        #[test]
        fn a_leading_tilde_is_the_home() {
            let home = Path::new("/Users/someone");

            assert_eq!(expand_home(Path::new("~"), home), home);
            assert_eq!(expand_home(Path::new("~/"), home), home);
            assert_eq!(expand_home(Path::new("~/Documents/Projects"), home), home.join("Documents/Projects"));
        }

        #[test]
        fn every_folder_has_one_name_and_reads_back_from_it() {
            let names: BTreeSet<_> = KnownFolder::ALL.iter().map(|folder| folder.as_str()).collect();

            assert_eq!(names.len(), KnownFolder::ALL.len());
            for folder in KnownFolder::ALL {
                assert_eq!(serde_json::to_value(folder).unwrap(), folder.as_str());
                assert_eq!(serde_json::from_value::<KnownFolder>(folder.as_str().into()).unwrap(), folder);
                assert_eq!(folder.to_string(), folder.as_str());
            }
        }
    }
}
