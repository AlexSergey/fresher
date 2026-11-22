use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use fresher_core::{
    BundleFacts, BundleId, CancelToken, InstalledApp, InstalledApps, InstalledList, NodeKind, SystemError,
};
use fresher_testing::FakeInstalledApps;

use super::super::fixture::{app, facts, state};

const PATIENCE: Duration = Duration::from_secs(10);

/// Two measurable apps.
fn two_apps() -> FakeInstalledApps {
    let (editor, viewer) = (app("com.example.Editor", "Editor"), app("com.example.Viewer", "Viewer"));
    FakeInstalledApps::new(vec![editor.clone(), viewer.clone()])
        .with_bundle(editor.path, Ok(facts(2, None)))
        .with_bundle(viewer.path, Ok(facts(1, None)))
}

/// Installed apps whose bundles' facts wait until the test opens them: a slow measure.
struct Slow {
    inner: FakeInstalledApps,
    entered: AtomicUsize,
    open: AtomicBool,
}

impl Slow {
    fn new(inner: FakeInstalledApps) -> Self {
        Self { inner, entered: AtomicUsize::new(0), open: AtomicBool::new(false) }
    }

    fn wait_until_measuring(&self) {
        let deadline = Instant::now() + PATIENCE;
        while self.entered.load(Ordering::SeqCst) == 0 {
            assert!(Instant::now() < deadline, "the measure did not start");
            thread::yield_now();
        }
    }

    fn open(&self) {
        self.open.store(true, Ordering::SeqCst);
    }
}

impl InstalledApps for Slow {
    fn list(&self) -> Result<InstalledList, SystemError> {
        self.inner.list()
    }

    fn find(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        self.inner.find(ids, cancel)
    }

    fn search_complete(&self, cancel: &CancelToken) -> Result<(), SystemError> {
        self.inner.search_complete(cancel)
    }

    fn bundle(&self, app: &InstalledApp, cancel: &CancelToken) -> Result<BundleFacts, SystemError> {
        self.entered.fetch_add(1, Ordering::SeqCst);
        let deadline = Instant::now() + PATIENCE;
        while !self.open.load(Ordering::SeqCst) && Instant::now() < deadline {
            if cancel.is_cancelled() {
                return Err(SystemError { reason: "the bundle's measure was cancelled".into() });
            }
            thread::yield_now();
        }
        self.inner.bundle(app, cancel)
    }
}

/// Two states: the editor's cache in the first, nothing in the second.
fn states() -> Vec<fresher_testing::FixedDiskState> {
    vec![state(1, &[("/home/Library/Caches/com.example.Editor", NodeKind::Folder, 1)]), state(2, &[])]
}

mod negative {
    use std::sync::Arc;
    use std::thread;

    use fresher_core::{AppsQuery, CommandError, EngineStatus, Operation, StateId};

    use super::super::super::fixture::{fixture, idle, rows};
    use super::{Slow, states, two_apps};
    use crate::testing::{ScriptedWalk, scan, wait_for_finish};

    const BUSY: CommandError = CommandError::Busy { running: Operation::Scan };

    #[test]
    fn a_result_of_an_older_state_is_dropped() {
        let slow = Arc::new(Slow::new(two_apps()));
        let engine = fixture(states(), Arc::clone(&slow) as _, idle()).engine();
        scan(&engine);

        let older = thread::scope(|scope| {
            let listing = scope.spawn(|| engine.apps(AppsQuery::default()));
            slow.wait_until_measuring();
            scan(&engine);
            listing.join().unwrap()
        });

        assert_eq!(older, Err(BUSY), "the scan cancelled the measure of the state it replaced");
        slow.open();
        let measured = slow.inner.bundle_calls();
        let newer = rows(&engine);
        assert!(newer.iter().all(|row| row.app.state() == StateId::new(2)));
        assert_eq!(slow.inner.bundle_calls(), measured + 2, "the older result was not kept");
    }

    #[test]
    fn a_scan_that_starts_cancels_the_measure() {
        let slow = Arc::new(Slow::new(two_apps()));
        let engine = fixture(states(), Arc::clone(&slow) as _, idle()).engine();
        scan(&engine);

        thread::scope(|scope| {
            let listing = scope.spawn(|| engine.apps(AppsQuery::default()));
            slow.wait_until_measuring();
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            assert_eq!(listing.join().unwrap(), Err(BUSY), "ended without the bundle ever being opened");
            wait_for_finish(&events);
        });
    }

    #[test]
    fn the_measure_holds_no_engine_lock() {
        let slow = Arc::new(Slow::new(two_apps()));
        let engine = fixture(states(), Arc::clone(&slow) as _, idle()).engine();
        scan(&engine);

        thread::scope(|scope| {
            let listing = scope.spawn(|| engine.apps(AppsQuery::default()));
            slow.wait_until_measuring();

            assert!(engine.overview().is_ok());
            assert!(engine.settings().is_ok());
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();
            wait_for_finish(&events);
            assert_eq!(engine.status(), Ok(EngineStatus::Idle));
            listing.join().unwrap().unwrap_err();
        });
    }

    #[test]
    fn two_listings_at_once_measure_once() {
        let slow = Arc::new(Slow::new(two_apps()));
        let engine = fixture(states(), Arc::clone(&slow) as _, idle()).engine();
        scan(&engine);

        let (first, second) = thread::scope(|scope| {
            let first = scope.spawn(|| engine.apps(AppsQuery::default()));
            slow.wait_until_measuring();
            let second = scope.spawn(|| engine.apps(AppsQuery::default()));
            slow.open();
            (first.join().unwrap().unwrap(), second.join().unwrap().unwrap())
        });

        assert_eq!(first, second);
        assert_eq!(slow.inner.bundle_calls(), 2, "one measure of the two bundles");
    }

    #[test]
    fn nothing_is_measured_during_a_scan() {
        let (walk, gate) = ScriptedWalk::held();
        let apps = Arc::new(two_apps());
        let system = fixture(Vec::new(), Arc::clone(&apps) as _, idle()).system;
        let engine = crate::testing::Fixture::new(walk).system(system).engine();
        gate.release();
        scan(&engine);
        gate.close();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();

        assert_eq!(engine.apps(AppsQuery::default()), Err(BUSY));
        assert_eq!(engine.app(fresher_core::AppId::new(StateId::new(1), 0)), Err(BUSY));
        assert_eq!(apps.bundle_calls(), 0);

        gate.release();
        wait_for_finish(&events);
        assert_eq!(rows(&engine).len(), 2);
    }
}

mod positive {
    use std::sync::Arc;

    use fresher_core::{AppId, AppsQuery, StateId};
    use fresher_testing::FakeRunningApps;

    use super::super::super::fixture::{fixture, idle, rows};
    use super::{states, two_apps};
    use crate::testing::{ScriptedWalk, scan};

    #[test]
    fn the_bundles_are_measured_once_per_state() {
        let apps = Arc::new(two_apps());
        let engine = fixture(states(), Arc::clone(&apps) as _, Arc::new(FakeRunningApps::default())).engine();
        scan(&engine);

        rows(&engine);
        rows(&engine);
        engine.app(AppId::new(StateId::new(1), 0)).unwrap();

        assert_eq!(apps.bundle_calls(), 2);
    }

    #[test]
    fn a_new_commit_measures_again() {
        let apps = Arc::new(two_apps());
        let engine = fixture(states(), Arc::clone(&apps) as _, Arc::new(FakeRunningApps::default())).engine();
        scan(&engine);
        rows(&engine);

        scan(&engine);
        let rows = rows(&engine);

        assert_eq!(apps.bundle_calls(), 4);
        assert!(rows.iter().all(|row| row.app.state() == StateId::new(2)));
    }

    #[test]
    fn the_scan_during_a_measured_state_answers_the_kept_facts() {
        let (walk, gate) = ScriptedWalk::held();
        let apps = Arc::new(two_apps());
        let system = fixture(Vec::new(), Arc::clone(&apps) as _, idle()).system;
        let engine = crate::testing::Fixture::new(walk).system(system).engine();
        gate.release();
        scan(&engine);
        rows(&engine);
        gate.close();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();

        let during = engine.apps(AppsQuery::default()).unwrap();

        assert_eq!(during.apps.len(), 2);
        assert_eq!(apps.bundle_calls(), 2);
        gate.release();
        crate::testing::wait_for_finish(&events);
    }
}
