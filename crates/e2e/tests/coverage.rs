//! The coverage of a scan on a fixture home whose Desktop the system guards: the reference walk, the demo module and
//! a fake access probe. The Desktop is not allowed and not opted into, so it is never tried nor entered, and the
//! result names it as not entered.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e` rewrites `fixtures/golden/coverage.json`; the diff is reviewed.

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
mod support;

#[cfg(test)]
mod app {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::mpsc::Receiver;
    use std::time::Duration;

    use fresher_core::{Access, Capability, Event, GuardedFolder, SettingUpdate, SettingValue};
    use fresher_engine::{Engine, Ports, Registry};
    use fresher_module_demo::DemoModule;
    use fresher_store::{DataFolder, FileSettingsStore};
    use fresher_testing::{FakeAccess, FakeClock, FakeCommandRunner, FakeSystemUi, FixtureTree, NaiveWalk};

    use crate::support::fixtures;

    pub(crate) const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);

    /// The guarded fixture home, its root as the system names it (no link in it, so the root is an allowed folder).
    pub(crate) fn home() -> FixtureTree {
        FixtureTree::from_toml(&fixtures().join("homes/guarded.toml")).unwrap()
    }

    /// The engine over `home` as its only scan root, with the Desktop guarded and not asked; the access probe is
    /// returned to see what was tried.
    pub(crate) fn engine(folder: &Path, home: &Path) -> (Engine, Arc<FakeAccess>) {
        let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (DESKTOP, Access::NotAsked)])
            .home(home)
            .guard("Desktop", DESKTOP)
            .answering("Desktop", Access::Granted);
        let access = Arc::new(access);
        let mut registry = Registry::new().unwrap();
        registry.register(Box::new(DemoModule::new().unwrap())).unwrap();
        let ports = Ports {
            walk: Arc::new(NaiveWalk::new(home)),
            settings: Arc::new(FileSettingsStore::new(DataFolder::open(folder).unwrap())),
            clock: Arc::new(FakeClock::at(1_763_200_000)),
            commands: Arc::new(FakeCommandRunner::default()),
            access: Arc::clone(&access) as Arc<dyn fresher_core::AccessProbe>,
            system_ui: Arc::new(FakeSystemUi::default()),
            system: fresher_testing::fake_system(home),
        };
        let engine = Engine::new(registry, ports).unwrap();
        let roots = SettingValue::Folders(vec![home.to_path_buf()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
        (engine, access)
    }

    /// Every event up to `ScanFinished`; fails after 10 s instead of hanging.
    pub(crate) fn until_finished(events: &Receiver<Event>) -> Vec<Event> {
        let mut seen = Vec::new();
        loop {
            let event = events.recv_timeout(Duration::from_secs(10)).unwrap();
            let done = matches!(event, Event::ScanFinished { .. });
            seen.push(event);
            if done {
                return seen;
            }
        }
    }
}

mod negative {
    use fresher_core::{Event, NotScanned};
    use serde_json::json;

    use crate::app::{engine, home, until_finished};
    use crate::support::check_golden;

    #[test]
    fn a_guarded_desktop_that_is_not_allowed_is_not_entered() {
        let folder = tempfile::tempdir().unwrap();
        let tree = home();
        let root = tree.root().canonicalize().unwrap();
        let (engine, access) = engine(folder.path(), &root);
        let events = engine.subscribe();

        engine.start_scan(fresher_engine::ScanMode::Auto).unwrap();

        let events = until_finished(&events);
        assert_eq!(access.tried(), []);
        let state = engine.disk_state().unwrap().unwrap();
        let avoided: Vec<_> = state.unreadable(10).into_iter().map(|place| place.reason).collect();
        assert_eq!(avoided, [NotScanned::Avoided]);
        let coverage = events.iter().find(|event| matches!(event, Event::ScanCoverage { .. }));
        let report = json!({ "coverage": coverage, "lastScan": engine.overview().unwrap().last_scan });
        check_golden("coverage", &report);
    }
}

mod positive {
    use fresher_core::{ScanOutcome, SettingUpdate, SettingValue};

    use crate::app::{DESKTOP, engine, home, until_finished};

    #[test]
    fn an_opted_in_desktop_that_is_granted_is_entered() {
        let folder = tempfile::tempdir().unwrap();
        let tree = home();
        let root = tree.root().canonicalize().unwrap();
        let (engine, access) = engine(folder.path(), &root);
        let opt_in = SettingUpdate { key: "core.access.desktop".into(), value: SettingValue::Switch(true) };
        engine.update_setting(opt_in).unwrap();
        let events = engine.subscribe();

        engine.start_scan(fresher_engine::ScanMode::Auto).unwrap();

        until_finished(&events);
        assert_eq!(access.tried().iter().map(|place| place.capability).collect::<Vec<_>>(), [DESKTOP]);
        assert_eq!(engine.disk_state().unwrap().unwrap().unreadable(10), []);
        let last = engine.overview().unwrap().last_scan.unwrap();
        assert!(last.coverage.is_complete());
        assert_eq!(last.outcome, ScanOutcome::Completed);
    }
}
