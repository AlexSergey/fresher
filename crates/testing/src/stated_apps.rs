use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{BundleId, CancelToken, InstalledApps, InstalledList, SystemError};

use crate::StatedFacts;

/// The installed apps of `inner`, each with the change time the sandbox states for its bundle: a sandbox's generator
/// writes every bundle now, so a case says "installed 10 days ago" by stating the bundle's `changed`.
pub struct StatedInstalledApps {
    inner: Arc<dyn InstalledApps>,
    stated: StatedFacts,
}

impl std::fmt::Debug for StatedInstalledApps {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("StatedInstalledApps").finish_non_exhaustive()
    }
}

impl StatedInstalledApps {
    /// `inner` seen through `stated`.
    pub fn new(inner: Arc<dyn InstalledApps>, stated: StatedFacts) -> Self {
        Self { inner, stated }
    }
}

impl InstalledApps for StatedInstalledApps {
    fn list(&self) -> Result<InstalledList, SystemError> {
        let mut list = self.inner.list()?;
        for app in &mut list.apps {
            if let Some(changed) = self.stated.get(&app.path).and_then(|stated| stated.changed) {
                app.changed = Some(changed);
            }
        }
        Ok(list)
    }

    fn find(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        self.inner.find(ids, cancel)
    }

    fn search_complete(&self, cancel: &CancelToken) -> Result<(), SystemError> {
        self.inner.search_complete(cancel)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_core::{InstalledApp, Timestamp};

    use crate::{FakeInstalledApps, Stated, StatedFacts, changed_at, installed_app};

    fn app(path: &str, changed: u64) -> InstalledApp {
        let app = installed_app("com.example.Tool", "Tool", "1.0").unwrap();
        changed_at(InstalledApp { path: path.into(), ..app }, changed)
    }

    fn stating(path: &str, changed: u64) -> StatedFacts {
        let stated = Stated { changed: Some(Timestamp::from_secs(changed)), ..Stated::default() };
        StatedFacts::under(Path::new("/box"), [(Path::new(path), &stated)])
    }

    mod negative {
        use std::sync::Arc;

        use fresher_core::{InstalledApps, Timestamp};

        use super::{FakeInstalledApps, StatedFacts, app, stating};
        use crate::StatedInstalledApps;

        #[test]
        fn an_app_without_a_stated_change_keeps_its_own() {
            for stated in [stating("Applications/Other.app", 7), StatedFacts::new()] {
                let inner = FakeInstalledApps::new(vec![app("/box/Applications/Tool.app", 500)]);
                let apps = StatedInstalledApps::new(Arc::new(inner), stated);

                assert_eq!(apps.list().unwrap().apps[0].changed, Some(Timestamp::from_secs(500)));
            }
        }
    }

    mod positive {
        use std::path::PathBuf;
        use std::sync::Arc;

        use fresher_core::{BundleId, CancelToken, InstalledApps, Timestamp};

        use super::{FakeInstalledApps, app, stating};
        use crate::StatedInstalledApps;

        #[test]
        fn a_stated_change_replaces_the_apps_change() {
            let inner = FakeInstalledApps::new(vec![app("/box/Applications/Tool.app", 500)]);
            let apps = StatedInstalledApps::new(Arc::new(inner), stating("Applications/Tool.app", 7));

            assert_eq!(apps.list().unwrap().apps[0].changed, Some(Timestamp::from_secs(7)));
        }

        #[test]
        fn find_and_search_pass_through() {
            let id = BundleId::new("com.example.Tool").unwrap();
            let listed = app("/box/Applications/Tool.app", 500);
            let inner = FakeInstalledApps::new(vec![listed]).failing_search("index off");
            let apps = StatedInstalledApps::new(Arc::new(inner), stating("Applications/Tool.app", 7));
            let cancel = CancelToken::new();

            assert_eq!(apps.find(&[id], &cancel).unwrap(), [PathBuf::from("/box/Applications/Tool.app")]);
            assert!(apps.search_complete(&cancel).is_err());
        }
    }
}
