use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    BundleFacts, BundleId, CancelToken, InstalledApp, InstalledApps, InstalledList, SystemError, TeamId,
};

use crate::{StatedFacts, StatedFlag};

/// The installed apps of `inner`, each with the change time the sandbox states for its bundle: a sandbox's generator
/// writes every bundle now, so a case says "installed 10 days ago" by stating the bundle's `changed`. A bundle's facts
/// are `inner`'s with the stated owner, flags, last use and package manager of the bundle laid over them.
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

    fn is_as_listed(&self, path: &Path) -> bool {
        self.inner.is_as_listed(path)
    }

    /// A program inside a bundle `inner` lists passes as signed by any team: a sandbox's bundles are written unsigned,
    /// and in sandbox mode no program is ever run (the world's commands answer every call). Anything else is not.
    fn signed_by(&self, file: &Path, _team: &TeamId, _cancel: &CancelToken) -> bool {
        self.inner.list().is_ok_and(|list| list.apps.iter().any(|app| file.starts_with(&app.path) && file != app.path))
    }

    /// A stated owner is never the user (root or another user); stated flags replace the bundle's own.
    fn bundle(&self, app: &InstalledApp, cancel: &CancelToken) -> Result<BundleFacts, SystemError> {
        let mut facts = self.inner.bundle(app, cancel)?;
        let Some(stated) = self.stated.get(&app.path) else { return Ok(facts) };
        if stated.owner.is_some() {
            facts.entry.owned_by_user = false;
        }
        if !stated.flags.is_empty() {
            let has = |flag| stated.flags.contains(&flag);
            facts.entry.user_immutable = has(StatedFlag::UserLock);
            facts.entry.system_immutable = has(StatedFlag::SystemLock) || has(StatedFlag::NoUnlink);
            facts.entry.restricted = has(StatedFlag::Restricted);
        }
        if let Some(last_used) = stated.last_used {
            facts.last_used = Some(last_used);
        }
        if let Some(installed_by) = &stated.installed_by {
            facts.installed_by = Some(installed_by.clone());
        }
        Ok(facts)
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
        fn nothing_stated_keeps_the_inner_answer() {
            let tool = app("/box/Applications/Tool.app", 500);
            let unread = fresher_core::SystemError { reason: "the bundle cannot be read".into() };
            let inner = FakeInstalledApps::new(vec![tool.clone()]).with_bundle(&tool.path, Err(unread.clone()));
            let apps = StatedInstalledApps::new(Arc::new(inner), stating("Applications/Tool.app", 7));

            assert_eq!(apps.bundle(&tool, &fresher_core::CancelToken::new()), Err(unread));
            let inner = FakeInstalledApps::new(vec![tool.clone()]).with_bundle(&tool.path, Ok(crate::bundle_facts(3)));
            let apps = StatedInstalledApps::new(Arc::new(inner), stating("Applications/Tool.app", 7));
            assert_eq!(apps.bundle(&tool, &fresher_core::CancelToken::new()), Ok(crate::bundle_facts(3)));
        }

        #[test]
        fn a_program_outside_every_listed_bundle_is_signed_by_no_one() {
            let inner = FakeInstalledApps::new(vec![app("/box/Applications/Tool.app", 500)]);
            let apps = StatedInstalledApps::new(Arc::new(inner), StatedFacts::new());
            let team = fresher_core::TeamId::new("HUAQ24HBR6").unwrap();
            let cancel = fresher_core::CancelToken::new();

            for file in ["/box/Applications/.Tool.app/bin/tool", "/box/Applications/Tool.app", "/bin/tool"] {
                assert!(!apps.signed_by(std::path::Path::new(file), &team, &cancel), "{file}");
            }
        }

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
        use std::path::{Path, PathBuf};
        use std::sync::Arc;

        use fresher_core::{
            BundleId, CancelToken, EntryFacts, InstalledApps, InstalledBy, Manager, Removability, Timestamp,
        };

        use super::{FakeInstalledApps, app, stating};
        use crate::{Stated, StatedFacts, StatedFlag, StatedInstalledApps, StatedOwner, bundle_facts};

        #[test]
        fn a_program_inside_a_listed_bundle_passes_as_signed() {
            let inner = FakeInstalledApps::new(vec![app("/box/Applications/Tool.app", 500)]);
            let apps = StatedInstalledApps::new(Arc::new(inner), StatedFacts::new());
            let team = fresher_core::TeamId::new("HUAQ24HBR6").unwrap();

            let file = Path::new("/box/Applications/Tool.app/Contents/bin/tool");
            assert!(apps.signed_by(file, &team, &CancelToken::new()));
        }

        #[test]
        fn a_stated_owner_date_and_manager_overlay_the_measure() {
            let tool = app("/box/Applications/Tool.app", 500);
            let other = app("/box/Applications/Other.app", 500);
            let by = InstalledBy::new(Manager::Homebrew, "tool").unwrap();
            let stated = Stated {
                owner: Some(StatedOwner::System),
                flags: [StatedFlag::UserLock].into(),
                last_used: Some(Timestamp::from_secs(9)),
                installed_by: Some(by.clone()),
                ..Stated::default()
            };
            let inner = FakeInstalledApps::new(vec![tool.clone(), other.clone()])
                .with_bundle(&tool.path, Ok(bundle_facts(4096)))
                .with_bundle(&other.path, Ok(bundle_facts(10)));
            let stated = StatedFacts::under(Path::new("/box"), [(Path::new("Applications/Tool.app"), &stated)]);
            let apps = StatedInstalledApps::new(Arc::new(inner), stated);
            let cancel = CancelToken::new();

            let facts = apps.bundle(&tool, &cancel).unwrap();

            assert_eq!(facts.sizes, bundle_facts(4096).sizes);
            assert_eq!(facts.entry, EntryFacts { owned_by_user: false, user_immutable: true, ..EntryFacts::REMOVABLE });
            assert_eq!(facts.removability(), Removability::Locked);
            assert_eq!(facts.last_used, Some(Timestamp::from_secs(9)));
            assert_eq!(facts.installed_by, Some(by));
            assert_eq!(apps.bundle(&other, &cancel), Ok(bundle_facts(10)));
        }

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
