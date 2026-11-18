//! What the temp flows share: a fixture sandbox (its marker at the top, the home `home/`, the two temporary places
//! beside it, outside the scan root) scanned by the real scanner, its reader seen through the facts lens, with the
//! `temp` module; the real settings file, scan cache and journal in the sandbox's data folder; the real `DiskCleaner`
//! reaching only inside the sandbox folder, as sandbox mode composes it, with the sandbox's places protection and a
//! fake Trash at `home/.Trash` (the shared flow home of `support/home*.rs`). `restart` builds a new engine on the same
//! files, as when the app starts again.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
mod home;
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[path = "../support/report.rs"]
mod report;

use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{CleanupPlanView, CleanupSelection, Event, LeftOutCount, Module, ModuleId};
use fresher_module_temp::TempModule;
use fresher_testing::{FIXTURE_NOW, FakeChangeJournal, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree};
use serde_json::json;

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::Parts;

/// The user's temporary folder and the shared one, relative to the sandbox.
pub(crate) const USER_TEMP: &str = "private/var/folders/xx/sandbox/T";
pub(crate) const SHARED_TEMP: &str = "private/tmp";
pub(crate) const CATEGORY: &str = "temp.old-files";
pub(crate) const DAY: u64 = 24 * 60 * 60;

/// What a sandbox holds besides its layout.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Home {
    /// `fixtures/homes/temp-flow.toml`: old and fresh items with stated times.
    Flow,
    /// Nothing: a flow makes its own entries on the real disk, and nothing is stated.
    Bare,
}

/// The system an engine runs with: the clock's moment, and whether the `temp` module is registered.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Setup {
    pub(crate) now: u64,
    pub(crate) temp: bool,
}

impl Setup {
    /// The clock at `now`, the module registered.
    pub(crate) const fn at(now: u64) -> Self {
        Self { now, temp: true }
    }

    /// The clock at `FIXTURE_NOW`, no module registered: no module names a place.
    pub(crate) const fn without_temp() -> Self {
        Self { now: FIXTURE_NOW, temp: false }
    }
}

impl Flow {
    /// The sandbox holding `home`, its engine at `FIXTURE_NOW` with the module.
    pub(crate) fn new(home: Home) -> Self {
        let tree = match home {
            Home::Flow => FixtureTree::from_toml(&fixtures().join("homes").join("temp-flow.toml")).unwrap(),
            Home::Bare => FixtureTree::from_toml_str("").unwrap(),
        };
        Self::open(tree, |_| parts(Setup::at(FIXTURE_NOW)))
    }

    /// A new engine on the same files with `setup`: the app started again.
    pub(crate) fn restart(&mut self, setup: Setup) {
        self.restart_with(parts(setup));
    }

    /// The paths of `view`'s items relative to the sandbox, sorted.
    pub(crate) fn planned(&self, view: &CleanupPlanView) -> Vec<String> {
        let mut paths: Vec<String> = view
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .filter_map(|item| Some(self.relative_path(item.path.as_ref()?)))
            .collect();
        paths.sort();
        paths
    }
}

/// The selection of the module's one category, whole.
pub(crate) fn select_all() -> CleanupSelection {
    serde_json::from_value(json!({ "findings": [], "categories": [{ "category": CATEGORY, "except": [] }] })).unwrap()
}

/// What the `temp` module left out in the events' `ModuleFinished`, as reason key and item count, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, u32)> {
    let temp = ModuleId::new("temp").unwrap();
    let mut counts: Vec<(String, u32)> = events
        .iter()
        .filter_map(|event| match event {
            Event::ModuleFinished { module, left_out, .. } if *module == temp => Some(left_out),
            _ => None,
        })
        .flatten()
        .map(|count: &LeftOutCount| {
            assert_eq!(count.category.as_str(), CATEGORY);
            (count.reason.key().as_str().to_owned(), count.items)
        })
        .collect();
    counts.sort();
    counts
}

/// The newest status change of `paths` (the real disk's), in seconds.
pub(crate) fn newest_change(paths: &[PathBuf]) -> u64 {
    paths.iter().map(|path| u64::try_from(fs::symlink_metadata(path).unwrap().ctime()).unwrap()).max().unwrap()
}

/// The engine's parts in `setup`: no app installed or open, a change journal that knows nothing, one thread.
fn parts(setup: Setup) -> Parts {
    let mut modules: Vec<Box<dyn Module>> = Vec::new();
    if setup.temp {
        modules.push(Box::new(TempModule::new().unwrap()));
    }
    Parts {
        modules,
        clock: Arc::new(FakeClock::at(setup.now)),
        journal: FakeChangeJournal::default(),
        installed: FakeInstalledApps::new(Vec::new()),
        running: FakeRunningApps::new(Vec::new()),
        threads: 1.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
