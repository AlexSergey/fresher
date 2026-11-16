//! The places where the system keeps things, by meaning: a module names a known folder, never an OS path.

use std::fmt;
use std::path::PathBuf;

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
}

impl KnownFolder {
    /// Every known folder, in a fixed order.
    pub const ALL: [Self; 14] = [
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
    ];

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

#[cfg(test)]
mod tests {
    mod negative {
        use crate::KnownFolder;

        #[test]
        fn an_unknown_folder_name_is_not_read() {
            for name in ["Home", "app_caches", "caches", ""] {
                assert!(serde_json::from_value::<KnownFolder>(name.into()).is_err(), "{name}");
            }
        }
    }

    mod positive {
        use std::collections::BTreeSet;

        use crate::KnownFolder;

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
