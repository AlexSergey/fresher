//! The whole path on the Rust side: a command reaches the engine, the `demo` module runs, the events come back, and
//! the settings live in a real settings file that survives a restart.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e` rewrites `fixtures/golden/demo.json`; the diff is reviewed.

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
mod support;

#[cfg(test)]
mod app {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::mpsc::Receiver;
    use std::thread;
    use std::time::Duration;

    use fresher_core::{
        Access, AccessProbe, AccessStatus, CancelToken, ClaimedPath, DiskState, DiskWalk, Event, Match, StateId,
        Timestamp, WalkError, WalkProgress, WalkRequest, WalkSummary,
    };
    use fresher_engine::{Engine, Ports, Registry};
    use fresher_module_demo::DemoModule;
    use fresher_store::{DataFolder, FileSettingsStore};
    use fresher_testing::{FakeClock, FakeCommandRunner, FixedDiskState};

    /// The state both walks commit: nothing was walked.
    fn empty_state() -> Arc<dyn DiskState> {
        Arc::new(FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), []))
    }

    /// The walk of the app until plan 004: it finds nothing.
    struct NoWalk;

    impl DiskWalk for NoWalk {
        fn walk(
            &self,
            _request: &WalkRequest<'_>,
            _cancel: &CancelToken,
            _on_match: &mut dyn FnMut(Match),
            _on_progress: &mut dyn FnMut(WalkProgress),
        ) -> Result<WalkSummary, WalkError> {
            Ok(WalkSummary::default())
        }

        fn commit(&self, _claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
            Ok(empty_state())
        }
    }

    /// A walk that holds the scan until it is cancelled.
    pub(crate) struct HeldWalk;

    impl DiskWalk for HeldWalk {
        fn walk(
            &self,
            _request: &WalkRequest<'_>,
            cancel: &CancelToken,
            _on_match: &mut dyn FnMut(Match),
            _on_progress: &mut dyn FnMut(WalkProgress),
        ) -> Result<WalkSummary, WalkError> {
            while !cancel.is_cancelled() {
                thread::yield_now();
            }
            Err(WalkError::Cancelled)
        }

        fn commit(&self, _claims: &[ClaimedPath]) -> Result<Arc<dyn DiskState>, WalkError> {
            Ok(empty_state())
        }
    }

    /// The access probe of the app until plan 006: it cannot tell.
    struct UnknownAccess;

    impl AccessProbe for UnknownAccess {
        fn status(&self) -> AccessStatus {
            AccessStatus { full_disk_access: Access::Unknown }
        }
    }

    fn engine_with(folder: &Path, walk: Arc<dyn DiskWalk>) -> Engine {
        let mut registry = Registry::new().unwrap();
        registry.register(Box::new(DemoModule::new().unwrap())).unwrap();
        let ports = Ports {
            walk,
            settings: Arc::new(FileSettingsStore::new(DataFolder::open(folder).unwrap())),
            clock: Arc::new(FakeClock::at(1_763_200_000)),
            commands: Arc::new(FakeCommandRunner::default()),
            access: Arc::new(UnknownAccess),
        };
        Engine::new(registry, ports).unwrap()
    }

    /// The app's engine on the data folder `folder`, with the demo module and a walk that finds nothing.
    pub(crate) fn engine(folder: &Path) -> Engine {
        engine_with(folder, Arc::new(NoWalk))
    }

    /// The same, with a scan that runs until it is cancelled.
    pub(crate) fn held_engine(folder: &Path) -> Engine {
        engine_with(folder, Arc::new(HeldWalk))
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
    use fresher_core::{CommandError, Operation, SettingUpdate, SettingValue, SettingsFile};

    use crate::app::{engine, held_engine, until_finished};

    #[test]
    fn a_second_scan_during_the_first_is_busy() {
        let folder = tempfile::tempdir().unwrap();
        let engine = held_engine(folder.path());
        let events = engine.subscribe();
        engine.start_scan(fresher_engine::ScanMode::Auto).unwrap();

        assert_eq!(
            engine.start_scan(fresher_engine::ScanMode::Auto),
            Err(CommandError::Busy { running: Operation::Scan })
        );

        engine.cancel_scan().unwrap();
        until_finished(&events);
    }

    #[test]
    fn a_settings_file_from_a_newer_version_is_not_overwritten() {
        let folder = tempfile::tempdir().unwrap();
        let file = folder.path().join("settings.json");
        let newer = "{ \"format\": \"settings\", \"version\": 2, \"values\": { \"demo.count\": 7 } }\n";
        std::fs::write(&file, newer).unwrap();
        let engine = engine(folder.path());

        let update = SettingUpdate { key: "demo.count".into(), value: SettingValue::Number(3) };
        let error = engine.update_setting(update);

        assert_eq!(error, Err(CommandError::SettingsReadOnly));
        assert_eq!(engine.settings().unwrap().file, SettingsFile::NewerVersion);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), newer);
    }
}

mod positive {
    use fresher_core::{FindingOrder, FindingsQuery, SettingKey, SettingUpdate, SettingValue, SettingsFile};
    use serde_json::json;

    use crate::app::{engine, until_finished};
    use crate::support::check_golden;

    #[test]
    fn the_demo_scan_matches_its_golden_report() {
        let folder = tempfile::tempdir().unwrap();
        let engine = engine(folder.path());
        let events = engine.subscribe();

        engine.start_scan(fresher_engine::ScanMode::Auto).unwrap();

        let events = until_finished(&events);
        let report = json!({ "events": events, "overview": engine.overview().unwrap() });
        check_golden("demo", &report);
    }

    #[test]
    fn a_setting_survives_a_restart() {
        let folder = tempfile::tempdir().unwrap();
        let count = SettingKey::new("demo.count").unwrap();
        let first = engine(folder.path());
        first.update_setting(SettingUpdate { key: "demo.count".into(), value: SettingValue::Number(3) }).unwrap();
        drop(first);

        let restarted = engine(folder.path());
        let events = restarted.subscribe();
        restarted.start_scan(fresher_engine::ScanMode::Auto).unwrap();
        until_finished(&events);

        let view = restarted.settings().unwrap();
        assert_eq!(view.file, SettingsFile::Loaded);
        assert_eq!(view.values.get(&count), Some(&SettingValue::Number(3)));
        let page = restarted.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 100)).unwrap();
        assert_eq!(page.total, 3);
    }
}
