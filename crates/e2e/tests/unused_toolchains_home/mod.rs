//! What the unused-toolchains flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the
//! real scanner, its reader seen through the facts lens, with the `unused-toolchains` module; the real settings file,
//! scan cache and journal in the sandbox's data folder; the real `DiskCleaner` reaching only inside the sandbox folder,
//! with a fake Trash at `home/.Trash` (the shared flow home of `support/home*.rs`). Nothing is open; the Node switch is
//! on. A flow sets the files held open and the folders the change journal names at a rescan.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
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

use fresher_core::{CleanupSelection, Event, LeftOutCount, ModuleId};
use fresher_module_unused_toolchains::UnusedToolchainsModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree};
use serde_json::{Value, json};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Parts, no_installs, unknown_access};

impl Flow {
    /// The sandbox holding `fixtures/homes/unused-toolchains-flow.toml`, its engine with nothing open and the Node
    /// switch on.
    pub(crate) fn new() -> Self {
        Self::with(&[])
    }

    /// The same sandbox, the change journal naming at each rescan, in order, the folders `changed` (relative to the
    /// sandbox). A rescan past the last one meets a gap, and the scan reads every folder.
    pub(crate) fn with(changed: &[Vec<&'static str>]) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("unused-toolchains-flow.toml")).unwrap();
        let flow = Self::open(tree, |sandbox| parts(sandbox, changed));
        let switch = json!({ "key": "unused-toolchains.node-versions", "value": { "kind": "switch", "value": true } });
        flow.engine.update_setting(serde_json::from_value(switch).unwrap()).unwrap();
        flow
    }

    /// A new engine on the same files: the app started again.
    pub(crate) fn restart(&mut self) {
        let parts = parts(&self.sandbox, &[]);
        self.restart_with(parts);
    }
}

/// The selection of `categories`, whole.
pub(crate) fn select(categories: &[&str]) -> CleanupSelection {
    let categories: Vec<Value> =
        categories.iter().map(|category| json!({ "category": category, "except": [] })).collect();
    serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
}

/// What the `unused-toolchains` module left out in the events' `ModuleFinished`, as category, reason key and item
/// count, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32)> {
    let module = ModuleId::new("unused-toolchains").unwrap();
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

/// The engine's parts on `sandbox`: the module alone, nothing open, the clock at `FIXTURE_NOW`, two threads.
fn parts(sandbox: &Sandbox, changed: &[Vec<&str>]) -> Parts {
    Parts {
        modules: vec![Box::new(UnusedToolchainsModule::new().unwrap())],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: home_rescan::journal(sandbox, changed),
        installed: FakeInstalledApps::default(),
        running: FakeRunningApps::default(),
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
