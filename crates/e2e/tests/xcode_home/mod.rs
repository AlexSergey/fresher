//! What the xcode flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the real scanner,
//! its reader seen through the facts lens, with the two modules of `Library/Developer` the app registers
//! (`dev-caches`, `xcode`, in its order); the real settings file, scan cache and journal in the sandbox's data folder;
//! the real `DiskCleaner` reaching only inside the sandbox folder, as sandbox mode composes it, with a fake Trash at
//! `home/.Trash` (the shared flow home of `support/home*.rs`). Xcode is installed; the open apps at each scan, the
//! folders the change journal names at each rescan and the files held open are what a flow sets.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the shared flow home keeps what the flows of other modules restart")]
mod home;
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[path = "../support/home_rescan.rs"]
mod home_rescan;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[path = "../support/report.rs"]
mod report;

use std::sync::Arc;

use fresher_core::{CleanupSelection, Event, LeftOutCount, ModuleId, RunningApp};
use fresher_module_dev_caches::DevCachesModule;
use fresher_module_xcode::XcodeModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{
    FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree, installed_app, running_app,
};
use serde_json::{Value, json};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Parts, no_installs, unknown_access};

/// Xcode's bundle id: the app every finding of the module belongs to.
pub(crate) const XCODE: &str = "com.apple.dt.Xcode";
/// The Simulator's bundle id.
pub(crate) const SIMULATOR: &str = "com.apple.iphonesimulator";

/// The world an engine runs in besides the files.
#[derive(Debug, Default)]
pub(crate) struct Setup {
    /// The open apps at each scan, in order; the last one repeats. Nothing is open when empty.
    pub(crate) running: Vec<Vec<RunningApp>>,
    /// At each rescan, in order, the folders the change journal says changed, relative to the sandbox. A rescan past
    /// the last one meets a gap, and the scan reads every folder.
    pub(crate) changed: Vec<Vec<&'static str>>,
}

/// Xcode open.
pub(crate) fn xcode_open() -> Vec<RunningApp> {
    vec![running_app("Xcode", &[XCODE]).unwrap()]
}

/// The Simulator open, Xcode closed.
pub(crate) fn simulator_open() -> Vec<RunningApp> {
    vec![running_app("Simulator", &[SIMULATOR]).unwrap()]
}

impl Flow {
    /// The sandbox holding `fixtures/homes/xcode-flow.toml`, its engine with nothing open and no change named.
    pub(crate) fn new() -> Self {
        Self::with(Setup::default())
    }

    /// The same sandbox, its engine in `setup`.
    pub(crate) fn with(setup: Setup) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("xcode-flow.toml")).unwrap();
        Self::open(tree, |sandbox| parts(sandbox, setup))
    }
}

/// The selection of `categories`, whole.
pub(crate) fn select(categories: &[&str]) -> CleanupSelection {
    let categories: Vec<Value> =
        categories.iter().map(|category| json!({ "category": category, "except": [] })).collect();
    serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
}

/// What `module` left out in the events' `ModuleFinished`, as category, reason key and item count, sorted.
pub(crate) fn left_out(events: &[Event], module: &'static str) -> Vec<(String, String, u32)> {
    let module = ModuleId::new(module).unwrap();
    let mut counts: Vec<(String, String, u32)> = events
        .iter()
        .filter_map(|event| match event {
            Event::ModuleFinished { module: finished, left_out, .. } if *finished == module => Some(left_out),
            _ => None,
        })
        .flatten()
        .map(|count: &LeftOutCount| {
            (count.category.as_str().to_owned(), count.reason.key().as_str().to_owned(), count.items)
        })
        .collect();
    counts.sort();
    counts
}

/// The engine's parts in `setup` on `sandbox`: the app's modules of `Library/Developer` in its order, Xcode
/// installed, the clock at `FIXTURE_NOW`, two threads.
fn parts(sandbox: &Sandbox, setup: Setup) -> Parts {
    let mut answers = setup.running.into_iter();
    let first = FakeRunningApps::new(answers.next().unwrap_or_default());
    let xcode = installed_app(XCODE, "Xcode", "27.2").unwrap();
    Parts {
        modules: vec![Box::new(DevCachesModule::new().unwrap()), Box::new(XcodeModule::new().unwrap())],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: home_rescan::journal(sandbox, &setup.changed),
        installed: FakeInstalledApps::new(vec![xcode]),
        running: answers.fold(first, FakeRunningApps::then),
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
