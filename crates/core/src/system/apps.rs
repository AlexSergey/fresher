//! The apps installed on the system.

use std::path::PathBuf;

use crate::{BundleId, SystemError, TeamId};

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

/// The apps installed on the system, read from the apps folders.
pub trait InstalledApps: Send + Sync {
    /// Every installed app, by path. A bundle that cannot be read is left out; only a failure to read every apps
    /// folder is an error.
    fn list(&self) -> Result<Vec<InstalledApp>, SystemError>;
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
        use crate::BundleId;

        #[test]
        fn an_app_does_not_own_another_id() {
            assert!(!super::app().owns(&BundleId::new("com.example.Other").unwrap()));
        }
    }

    mod positive {
        use crate::BundleId;

        #[test]
        fn an_app_owns_its_id_and_its_helpers() {
            let app = super::app();

            assert!(app.owns(&BundleId::new("COM.EXAMPLE.APP").unwrap()));
            assert!(app.owns(&BundleId::new("com.example.App.Helper").unwrap()));
        }
    }
}
