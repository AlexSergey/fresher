//! What the Trash flows share: a fixture sandbox (its marker at the top, the home `home/`, a marker next to each trial
//! cache) scanned by the real scanner, its reader seen through the facts lens, with the `trial` module; the real
//! settings file, scan cache and journal in the sandbox's data folder; the real `DiskCleaner` reaching only inside the
//! sandbox folder, as sandbox mode composes it, with a fake Trash at `home/.Trash` (the shared flow home of
//! `support/home*.rs`), or at a second Trash a flow names. The clock stands at `FIXTURE_NOW` until a flow moves it;
//! the system's interface records what was opened; Finder's Trash setting is what a flow says.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the shared flow home keeps what the flows of other modules restart")]
mod home;
#[expect(dead_code, reason = "the flows of other modules read the report it also writes")]
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[expect(dead_code, reason = "the flows of other modules read the findings")]
#[path = "../support/home_scan.rs"]
mod home_scan;
#[expect(dead_code, reason = "the flows of other modules sort and unnumber their reports")]
#[path = "../support/report.rs"]
mod report;

use std::fs;
use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    CleanupId, CleanupPlanView, Confirmation, DangerLevel, Event, FindingOrder, FindingsQuery, Sizes, Subject,
    TrashState,
};
use fresher_module_trial::TrialModule;
use fresher_testing::{
    FIXTURE_NOW, FakeChangeJournal, FakeClock, FakeInstalledApps, FakeRunningApps, FakeSystemUi, FixtureTree,
};
use serde_json::json;

use crate::support::fixtures;
use events::until;
pub(crate) use home::Flow;
pub(crate) use home_engine::Extras;
use home_engine::{Parts, engine_with, no_installs, unknown_access};

/// The two trial caches the module claims, each next to a sandbox marker.
pub(crate) const CACHE_A: &str = "home/work-a/Sample app cache";
pub(crate) const CACHE_B: &str = "home/work-b/Sample app cache";
/// The file the user put in the home's Trash before any scan.
pub(crate) const USERS_FILE: &str = "home/.Trash/old-notes.txt";
/// The file no one claims.
const PLAIN: &str = "home/plain/keep.txt";

/// The flow home with the handles a Trash flow reads back.
pub(crate) struct Home {
    pub(crate) flow: Flow,
    pub(crate) clock: Arc<FakeClock>,
    pub(crate) system_ui: Arc<FakeSystemUi>,
    /// The bytes of the files no flow may change, as the fixture made them.
    kept: Vec<(String, Vec<u8>)>,
}

impl Home {
    /// The sandbox holding `fixtures/homes/trash.toml`, its engine with the default extras.
    pub(crate) fn new() -> Self {
        Self::with(Extras::default())
    }

    /// The same sandbox, its engine with `extras`.
    pub(crate) fn with(extras: Extras) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("trash.toml")).unwrap();
        for cache in [CACHE_A, CACHE_B] {
            tree.mark_sandbox(Path::new(cache).parent().unwrap()).unwrap();
        }
        let clock = Arc::new(FakeClock::at(FIXTURE_NOW));
        let system_ui = Arc::clone(&extras.system_ui);
        let mut flow = Flow::open(tree, |_| parts(&clock));
        // `Flow::open` builds the engine of the defaults; the flow's own takes its place before anything runs.
        flow.engine = engine_with(&flow.sandbox, &flow.stated, &flow.open_files, parts(&clock), extras);
        let kept = [PLAIN, USERS_FILE].map(|path| (path.to_owned(), fs::read(flow.path(path)).unwrap())).to_vec();
        Self { flow, clock, system_ui, kept }
    }

    /// The plan of the trial caches at `paths`, picked one by one.
    pub(crate) fn prepare(&self, paths: &[&str]) -> CleanupPlanView {
        let page = self.flow.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let ids: Vec<_> = paths
            .iter()
            .map(|path| {
                let path = self.flow.path(path);
                let finding = page
                    .findings
                    .iter()
                    .find(|finding| matches!(finding.finding().subject(), Subject::Path(found) if *found == path))
                    .unwrap();
                serde_json::to_value(finding.id()).unwrap()
            })
            .collect();
        let selection = serde_json::from_value(json!({ "findings": ids, "categories": [] })).unwrap();
        self.flow.engine.prepare_cleanup(&selection).unwrap()
    }

    /// Scans, then cleans the caches at `paths` to the Trash; the cleanup.
    pub(crate) fn scan_and_clean(&self, paths: &[&str]) -> CleanupId {
        self.flow.scan();
        let view = self.prepare(paths);
        self.flow.clean(&view).0
    }

    /// Confirms `view` with the permanent method, as the dialog gives it, and waits for the end.
    pub(crate) fn clean_permanently(&self, view: &CleanupPlanView) -> CleanupId {
        assert!(view.permanent_available);
        let events = self.flow.engine.subscribe();
        let summary = view.summary;
        let echo = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                           "highest": summary.highest, "method": "permanent",
                           "highRiskUnderstood": summary.highest == DangerLevel::High });
        let confirmation: Confirmation = serde_json::from_value(echo).unwrap();
        let cleanup = self.flow.engine.confirm_cleanup(&confirmation).unwrap();
        until(&events, |event| matches!(event, Event::CleanupFinished { .. }));
        cleanup
    }

    /// The items of `cleanup` the history finds in the Trash, their sizes summed.
    pub(crate) fn in_trash(&self, cleanup: CleanupId) -> Sizes {
        let detail = self.flow.engine.cleanup(cleanup).unwrap();
        detail
            .items
            .iter()
            .filter(|item| item.trash == Some(TrashState::InTrash))
            .fold(Sizes::ZERO, |sum, item| sum.checked_add(item.sizes).unwrap())
    }

    /// The scan's measure of the user's file in the Trash.
    pub(crate) fn users_file(&self) -> Sizes {
        self.flow.engine.trash().trashes[0].waiting.unwrap()
    }

    /// The files outside every cleanup, the plain file and the user's file in the Trash, are as the fixture made
    /// them.
    pub(crate) fn assert_untouched(&self) {
        for (path, bytes) in &self.kept {
            assert_eq!(fs::read(self.flow.path(path)).ok().as_ref(), Some(bytes), "{path} changed");
        }
    }
}

/// The engine's parts: the `trial` module, nothing installed or open, a change journal that knows nothing, two
/// threads, the clock `clock`.
fn parts(clock: &Arc<FakeClock>) -> Parts {
    Parts {
        modules: vec![Box::new(TrialModule::new().unwrap())],
        clock: Arc::clone(clock),
        journal: FakeChangeJournal::default(),
        installed: FakeInstalledApps::default(),
        running: FakeRunningApps::default(),
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
