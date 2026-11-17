//! The apps running now, and whose a folder is.

use std::path::Path;

use crate::{BundleId, SystemError};

/// A running app: the name the user knows it by, and the bundle ids of the bundles its process runs from (a helper's
/// own and its app's).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunningApp {
    pub name: String,
    pub bundle_ids: Vec<BundleId>,
}

/// The apps running on the system.
pub trait RunningApps: Send + Sync {
    /// Every process that runs from a bundle, as its app. A process outside any bundle is left out: it has no id to
    /// own a folder by.
    fn list(&self) -> Result<Vec<RunningApp>, SystemError>;
}

/// The running apps of one moment, asked whose a path is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunningSet {
    apps: Vec<RunningApp>,
}

impl RunningSet {
    pub fn new(apps: Vec<RunningApp>) -> Self {
        Self { apps }
    }

    pub fn apps(&self) -> &[RunningApp] {
        &self.apps
    }

    /// The name of a running app that owns `path`: one of the path's names is one of the app's bundle ids, or starts
    /// with it and a dot (`com.example.App.savedState`), ignoring case. Too wide errs on the side of keeping.
    pub fn owner_of(&self, path: &Path) -> Option<&str> {
        let names: Vec<&str> = path.iter().filter_map(|name| name.to_str()).collect();
        self.apps
            .iter()
            .find(|app| app.bundle_ids.iter().any(|id| names.iter().any(|name| id.names(name))))
            .map(|app| app.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::{BundleId, RunningApp, RunningSet};

    fn running() -> RunningSet {
        let id = |id| BundleId::new(id).unwrap();
        RunningSet::new(vec![
            RunningApp { name: "Mail".into(), bundle_ids: vec![id("com.example.Mail")] },
            RunningApp {
                name: "Editor".into(),
                bundle_ids: vec![id("com.example.Editor.Helper"), id("com.example.Editor")],
            },
        ])
    }

    mod negative {
        use std::path::Path;

        use crate::RunningSet;

        #[test]
        fn a_path_named_like_but_not_after_a_running_app_has_no_owner() {
            let set = super::running();

            for path in [
                "/Users/a/Library/Caches/com.example.Mailer",
                "/Users/a/Library/Caches/x.com.example.Mail",
                "/Users/a/Library/Caches/com.example",
                "/Users/a/Library/Caches/com.example.Mai",
            ] {
                assert_eq!(set.owner_of(Path::new(path)), None, "{path}");
            }
        }

        #[test]
        fn nothing_running_owns_nothing() {
            let empty = RunningSet::default();

            assert_eq!(empty.owner_of(Path::new("/Users/a/Library/Caches/com.example.Mail")), None);
            assert_eq!(empty.apps(), []);
        }
    }

    mod positive {
        use std::path::Path;

        #[test]
        fn a_path_named_after_a_running_app_belongs_to_it() {
            let set = super::running();

            for (path, owner) in [
                ("/Users/a/Library/Caches/com.example.Mail", "Mail"),
                ("/Users/a/Library/Containers/COM.EXAMPLE.MAIL/Data/x", "Mail"),
                ("/Users/a/Library/Saved Application State/com.example.Mail.savedState", "Mail"),
                ("/Users/a/Library/Preferences/com.example.Editor.Helper.plist", "Editor"),
                ("/Users/a/Library/Caches/com.example.Editor", "Editor"),
            ] {
                assert_eq!(set.owner_of(Path::new(path)), Some(owner), "{path}");
            }
            assert_eq!(set.apps().len(), 2);
        }
    }
}
