//! The engine with the real scanner and the scan cache in a temporary data folder: a scan, a scripted change, a rescan
//! that reads only what the journal names (and the check's sample), a restore in a new engine that reads no folder.
//! The journal is scripted: no test waits on the system's.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test rescan` rewrites `fixtures/golden/rescan.json`.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "app/mod.rs"]
mod app;

#[cfg(test)]
mod scripted {
    use std::collections::VecDeque;
    use std::fs::OpenOptions;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc::Receiver;
    use std::time::Duration;

    use fresher_core::{
        CancelToken, ChangeJournal, Changes, EntryAccess, Event, FolderIdentity, FolderListing, FolderReader,
        JournalGap, JournalPosition, ReadError, VolumeId,
    };
    use fresher_engine::Engine;
    use fresher_scanner::StdReader;

    /// A journal whose answers a test adds as it goes; it stands at the same position throughout.
    #[derive(Debug, Default)]
    pub(crate) struct Journal {
        answers: Mutex<VecDeque<Result<Vec<PathBuf>, JournalGap>>>,
        pub(crate) asked: AtomicUsize,
    }

    impl Journal {
        /// The next answer: these folders changed.
        pub(crate) fn changed(&self, folders: Vec<PathBuf>) {
            self.answers.lock().unwrap().push_back(Ok(folders));
        }

        pub(crate) fn gap(&self, gap: JournalGap) {
            self.answers.lock().unwrap().push_back(Err(gap));
        }
    }

    impl ChangeJournal for Journal {
        fn position(&self, _volume: &VolumeId) -> Option<JournalPosition> {
            Some(JournalPosition { journal: 1, cursor: 10 })
        }

        fn changes_since(
            &self,
            _volume: &VolumeId,
            since: &JournalPosition,
            _cancel: &CancelToken,
        ) -> Result<Changes, JournalGap> {
            self.asked.fetch_add(1, Ordering::Relaxed);
            let answer = self.answers.lock().unwrap().pop_front().unwrap_or(Err(JournalGap::Failed));
            answer.map(|folders| Changes { folders, subtrees: Vec::new(), end: *since })
        }
    }

    /// A reader that counts its calls and refuses every one: what a restore must never call.
    #[derive(Debug, Default)]
    pub(crate) struct Refusing {
        pub(crate) calls: AtomicUsize,
    }

    impl FolderReader for Refusing {
        fn identify(&self, _path: &Path) -> Result<FolderIdentity, ReadError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(ReadError::Failed)
        }

        fn access(&self, _path: &Path) -> Result<EntryAccess, ReadError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(ReadError::Failed)
        }

        fn read(&self, _path: &Path, _expected: FolderIdentity, _out: &mut FolderListing) -> Result<(), ReadError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(ReadError::Failed)
        }

        fn head(&self, _folder: &Path, _name: &str, _limit: usize) -> Result<Vec<u8>, ReadError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(ReadError::Failed)
        }
    }

    /// The portable reader behind a gate: while it is closed, a read waits and says so.
    #[derive(Debug)]
    pub(crate) struct Gated {
        pub(crate) open: AtomicBool,
        pub(crate) waiting: AtomicBool,
    }

    impl FolderReader for Gated {
        fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
            StdReader::new().identify(path)
        }

        fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
            StdReader::new().access(path)
        }

        fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
            while !self.open.load(Ordering::Acquire) {
                self.waiting.store(true, Ordering::Release);
                std::thread::yield_now();
            }
            StdReader::new().read(path, expected, out)
        }

        fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
            StdReader::new().head(folder, name, limit)
        }
    }

    /// Restores the last scan and gives its events; fails after 30 s instead of hanging.
    pub(crate) fn restore(engine: &Engine) -> Vec<Event> {
        let events: Receiver<Event> = engine.subscribe();
        engine.restore().unwrap();
        until_finished(&events)
    }

    pub(crate) fn until_finished(events: &Receiver<Event>) -> Vec<Event> {
        let mut seen = Vec::new();
        loop {
            let event = events.recv_timeout(Duration::from_secs(30)).unwrap();
            let done = matches!(event, Event::ScanFinished { .. });
            seen.push(event);
            if done {
                return seen;
            }
        }
    }

    /// Grows the file at `path` in place to `size` bytes.
    pub(crate) fn grow(path: &Path, size: u64) {
        OpenOptions::new().write(true).open(path).unwrap().set_len(size).unwrap();
    }

    /// The `ScanFinished` of `events`.
    pub(crate) fn finished(events: &[Event]) -> Option<&Event> {
        events.iter().find(|event| matches!(event, Event::ScanFinished { .. }))
    }
}

mod negative {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use fresher_core::{Event, JournalGap, ScanKind, ScanOutcome};

    use crate::app::{engine_on, home, probe_findings, scan};
    use crate::scripted::{Gated, Journal, Refusing, finished, grow, restore, until_finished};

    #[test]
    fn a_restore_reads_no_folder() {
        let home = home("scan-basic.toml", Arc::new(Journal::default()));
        scan(&home.engine);
        let reader = Arc::new(Refusing::default());
        let restarted = engine_on(&home.root, home.data.path(), Arc::new(Journal::default()), Arc::clone(&reader) as _);
        let before = probe_findings(&home);
        drop(home.engine);

        let events = restore(&restarted);

        assert_eq!(reader.calls.load(Ordering::Relaxed), 0);
        assert!(matches!(finished(&events), Some(Event::ScanFinished { kind: ScanKind::Restored { .. }, .. })));
        let restored = crate::app::Home { engine: restarted, ..home };
        assert_eq!(probe_findings(&restored), before);
    }

    #[test]
    fn nothing_to_restore_ends_idle_without_a_last_scan_and_says_so() {
        let home = home("scan-basic.toml", Arc::new(Journal::default()));
        let events = home.engine.subscribe();

        let scan = home.engine.restore().unwrap();

        let ended = events.recv_timeout(std::time::Duration::from_secs(30));
        assert_eq!(ended, Ok(Event::NothingRestored { scan }));
        assert_eq!(home.engine.status(), Ok(fresher_core::EngineStatus::Idle));
        assert_eq!(events.try_iter().count(), 0);
        assert_eq!(home.engine.overview().unwrap().last_scan, None);
    }

    #[test]
    fn a_journal_gap_means_a_full_scan() {
        let journal = Arc::new(Journal::default());
        let home = home("scan-basic.toml", Arc::clone(&journal) as _);
        scan(&home.engine);
        journal.gap(JournalGap::Dropped);

        let events = scan(&home.engine);

        assert!(matches!(finished(&events), Some(Event::ScanFinished { kind: ScanKind::Full, .. })));
        assert_eq!(journal.asked.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn a_cancelled_rescan_keeps_the_previous_cache() {
        let journal = Arc::new(Journal::default());
        let reader = Arc::new(Gated { open: AtomicBool::new(true), waiting: AtomicBool::new(false) });
        let home = home("scan-basic.toml", Arc::new(Journal::default()));
        let engine = engine_on(&home.root, home.data.path(), Arc::clone(&journal) as _, Arc::clone(&reader) as _);
        let home = crate::app::Home { engine, ..home };
        scan(&home.engine);
        let before = probe_findings(&home);
        grow(&home.root.join("Projects/app/build/out.o"), 900_000);
        journal.changed(vec![home.root.join("Projects/app/build")]);
        reader.open.store(false, Ordering::Release);
        let events = home.engine.subscribe();

        home.engine.start_scan(fresher_engine::ScanMode::Auto).unwrap();
        while !reader.waiting.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        home.engine.cancel_scan().unwrap();
        reader.open.store(true, Ordering::Release);

        let events = until_finished(&events);
        assert!(matches!(finished(&events), Some(Event::ScanFinished { outcome: ScanOutcome::Stopped, .. })));
        let restarted =
            engine_on(&home.root, home.data.path(), Arc::new(Journal::default()), Arc::new(Refusing::default()));
        restore(&restarted);
        assert_eq!(probe_findings(&crate::app::Home { engine: restarted, ..home }), before);
    }
}

mod positive {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use fresher_core::{
        Access, Capability, Coverage, Event, GuardedFolder, NotEntered, ScanKind, SettingUpdate, SettingValue,
    };
    use fresher_scanner::StdReader;
    use fresher_testing::FakeAccess;
    use serde_json::{Value, json};

    use crate::app::{Home, engine_on, engine_with_access, home, probe_findings, report, scan};
    use crate::scripted::{Journal, Refusing, finished, grow, restore};
    use crate::support::check_golden;

    const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);
    const DOWNLOADS: Capability = Capability::Folder(GuardedFolder::Downloads);

    /// The report without its progress events, whose count depends on the threads.
    fn steady(report: Value) -> Value {
        let mut report = report;
        if let Some(events) = report["events"].as_array_mut() {
            events.retain(|event| event["type"] != "scan-progress");
        }
        report
    }

    #[test]
    fn a_second_scan_is_a_rescan_and_a_restart_restores_it() {
        let journal = Arc::new(Journal::default());
        let home = home("scan-basic.toml", Arc::clone(&journal) as _);
        scan(&home.engine);
        let size_of = |home: &Home, path: &str| {
            probe_findings(home).into_iter().find(|(found, _)| found == path).map(|(_, size)| size).unwrap()
        };
        let build = size_of(&home, "Projects/app/build");
        grow(&home.tree.root().join("Projects/app/build/out.o"), 900_000);
        journal.changed(vec![home.root.join("Projects/app/build")]);

        let events = scan(&home.engine);

        assert!(matches!(finished(&events), Some(Event::ScanFinished { kind: ScanKind::Rescan { .. }, .. })));
        assert_eq!(size_of(&home, "Projects/app/build"), build + 900_000 - 70_000);
        let rescanned = steady(report(&home, &events));
        let findings = probe_findings(&home);
        let reader = Arc::new(Refusing::default());
        let restarted = engine_on(&home.root, home.data.path(), Arc::new(Journal::default()), Arc::clone(&reader) as _);
        drop(home.engine);
        let home = Home { engine: restarted, ..home };

        let events = restore(&home.engine);

        assert_eq!(reader.calls.load(Ordering::Relaxed), 0);
        assert_eq!(probe_findings(&home), findings);
        check_golden("rescan", &json!({ "rescan": rescanned, "restore": steady(report(&home, &events)) }));
    }

    /// Every file denied; `Projects` is the Desktop, opted in and answering `desktop`; `Downloads` is not opted in.
    fn guarded(root: &std::path::Path, desktop: Access) -> FakeAccess {
        FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (DESKTOP, desktop), (DOWNLOADS, Access::NotAsked)])
            .home(root)
            .guard("Projects", DESKTOP)
            .guard("Downloads", DOWNLOADS)
    }

    fn coverage(events: &[Event]) -> Option<&Coverage> {
        events.iter().find_map(|event| match event {
            Event::ScanCoverage { coverage, .. } => Some(coverage),
            _ => None,
        })
    }

    #[test]
    fn a_restart_restores_a_scan_that_avoided_places_with_its_coverage() {
        let home = home("scan-basic.toml", Arc::new(Journal::default()));
        let access = guarded(&home.root, Access::Granted);
        let engine = engine_with_access(
            &home.root,
            home.data.path(),
            Arc::new(Journal::default()),
            Arc::new(StdReader::new()),
            access,
        );
        // The guarded places are found under absolute roots.
        let roots =
            SettingUpdate { key: "core.scan.roots".into(), value: SettingValue::Folders(vec![home.root.clone()]) };
        engine.update_setting(roots).unwrap();
        let switch = SettingUpdate { key: "core.access.desktop".into(), value: SettingValue::Switch(true) };
        engine.update_setting(switch).unwrap();
        let home = Home { engine, ..home };
        let scanned = scan(&home.engine);
        let findings = probe_findings(&home);
        let reader = Arc::new(Refusing::default());
        let access = guarded(&home.root, Access::NotAsked);
        let restarted = engine_with_access(
            &home.root,
            home.data.path(),
            Arc::new(Journal::default()),
            Arc::clone(&reader) as _,
            access,
        );
        drop(home.engine);
        let home = Home { engine: restarted, ..home };

        let events = restore(&home.engine);

        assert_eq!(reader.calls.load(Ordering::Relaxed), 0);
        assert!(matches!(finished(&events), Some(Event::ScanFinished { kind: ScanKind::Restored { .. }, .. })));
        let not_entered = vec![NotEntered { capability: DOWNLOADS, access: Access::NotAsked }];
        assert_eq!(coverage(&scanned).map(|coverage| &coverage.not_entered), Some(&not_entered));
        assert_eq!(coverage(&events), coverage(&scanned));
        assert_eq!(probe_findings(&home), findings);
        assert!(findings.iter().any(|(path, _)| path.starts_with("Projects/")), "{findings:?}");
        assert!(!findings.iter().any(|(path, _)| path.starts_with("Downloads/")), "{findings:?}");
    }

    #[test]
    fn start_scan_full_ignores_the_cache() {
        let journal = Arc::new(Journal::default());
        let home = home("scan-basic.toml", Arc::clone(&journal) as _);
        scan(&home.engine);
        let events = home.engine.subscribe();

        home.engine.start_scan(fresher_engine::ScanMode::Full).unwrap();

        let events = crate::scripted::until_finished(&events);
        assert!(matches!(finished(&events), Some(Event::ScanFinished { kind: ScanKind::Full, .. })));
        assert_eq!(journal.asked.load(Ordering::Relaxed), 0);
    }
}
