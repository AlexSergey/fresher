//! The apps installed on the system.

use std::path::PathBuf;

use crate::{BundleId, CancelToken, SystemError, TeamId};

/// An installed app, as its own bundle says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledApp {
    pub bundle_id: BundleId,
    /// The team id of its code signature; `None` when it is unsigned, signed ad hoc, or the system's own.
    pub team_id: Option<TeamId>,
    /// The name it shows.
    pub name: String,
    /// The version the user sees (`2026.3`).
    pub version: Option<String>,
    /// The build number.
    pub build: Option<String>,
    /// The bundle.
    pub path: PathBuf,
    /// The bundle ids of the helpers inside it (login items, services, extensions).
    pub helpers: Vec<BundleId>,
}

impl InstalledApp {
    /// Whether `id` is the app's own or one of its helpers'.
    pub fn owns(&self, id: &BundleId) -> bool {
        self.bundle_id == *id || self.helpers.contains(id)
    }
}

/// The apps a listing found, and what it could not read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstalledList {
    /// Every app read, by path.
    pub apps: Vec<InstalledApp>,
    /// The apps folders, the folders in them and the app bundles that could not be read, by path: an app may be
    /// installed there that the list does not hold.
    pub unread: Vec<PathBuf>,
}

impl InstalledList {
    /// A list with nothing left unread.
    pub const fn complete(apps: Vec<InstalledApp>) -> Self {
        Self { apps, unread: Vec::new() }
    }

    /// Whether every folder and bundle was read: no installed app can be missing from `apps`.
    pub fn is_complete(&self) -> bool {
        self.unread.is_empty()
    }
}

/// The apps installed on the system, read from the apps folders.
pub trait InstalledApps: Send + Sync {
    /// Every installed app, by path, and what could not be read: a bundle or a folder that cannot be read is named in
    /// `unread`, never silently left out; only a failure to read every apps folder is an error.
    fn list(&self) -> Result<InstalledList, SystemError>;

    /// The bundles with one of `ids` anywhere the system's index knows, beyond the apps folders (an IDE on another
    /// drive, in a download folder): for a module that must not take an app installed elsewhere for one that is not
    /// installed. An error when the index could not be asked; then nothing is known.
    fn find(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError>;

    /// Whether `find` can see every app on the Mac: Spotlight's index is on and every mounted volume is indexed. An
    /// error names what it cannot see.
    fn search_complete(&self, cancel: &CancelToken) -> Result<(), SystemError>;
}

#[cfg(test)]
mod tests {
    use crate::{BundleId, InstalledApp};

    fn app() -> InstalledApp {
        InstalledApp {
            bundle_id: BundleId::new("com.example.App").unwrap(),
            team_id: None,
            name: "App".into(),
            version: Some("1.0".into()),
            build: None,
            path: "/Applications/App.app".into(),
            helpers: vec![BundleId::new("com.example.App.Helper").unwrap()],
        }
    }

    mod negative {
        use crate::{BundleId, InstalledList};

        #[test]
        fn an_app_does_not_own_another_id() {
            assert!(!super::app().owns(&BundleId::new("com.example.Other").unwrap()));
        }

        #[test]
        fn a_list_with_something_unread_is_not_complete() {
            let list = InstalledList { apps: vec![super::app()], unread: vec!["/Applications/Broken.app".into()] };

            assert!(!list.is_complete());
        }
    }

    mod positive {
        use crate::{BundleId, InstalledList};

        #[test]
        fn a_list_with_nothing_unread_is_complete() {
            assert!(InstalledList::complete(vec![super::app()]).is_complete());
            assert!(InstalledList::default().is_complete());
        }

        #[test]
        fn an_app_owns_its_id_and_its_helpers() {
            let app = super::app();

            assert!(app.owns(&BundleId::new("COM.EXAMPLE.APP").unwrap()));
            assert!(app.owns(&BundleId::new("com.example.App.Helper").unwrap()));
        }
    }
}
