//! What the Ollama flows of local-ai-models share: a fixture sandbox (its marker at the top, the home `home/`) scanned
//! by the real scanner, its reader seen through the facts lens, with the `local-ai-models` module; the real settings file,
//! scan cache and journal in the sandbox's data folder; the real `DiskCleaner` reaching only inside the sandbox folder,
//! with a fake Trash at `home/.Trash` (the shared flow home of `support/home*.rs`). Nothing is open, and no change is
//! named to a rescan; a flow sets the files held open.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
mod home;
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[path = "../support/home_rescan.rs"]
#[expect(dead_code, reason = "the flows of other modules rescan")]
mod home_rescan;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[path = "../support/report.rs"]
mod report;

use std::sync::Arc;

use fresher_core::{CleanupSelection, Event, LeftOutCount, ModuleId};
use fresher_module_local_ai_models::LocalAiModelsModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree};
use serde_json::{Value, json};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Parts, no_installs, unknown_access};

impl Flow {
    /// The sandbox holding `fixtures/homes/local-ai-models-ollama-flow.toml`, its engine with nothing open.
    pub(crate) fn new() -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("local-ai-models-ollama-flow.toml")).unwrap();
        Self::open(tree, parts)
    }

    /// A new engine on the same files: the app started again.
    pub(crate) fn restart(&mut self) {
        let parts = parts(&self.sandbox);
        self.restart_with(parts);
    }
}

/// The selection of `categories`, whole.
pub(crate) fn select(categories: &[&str]) -> CleanupSelection {
    let categories: Vec<Value> =
        categories.iter().map(|category| json!({ "category": category, "except": [] })).collect();
    serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
}

/// What the `local-ai-models` module left out in the events' `ModuleFinished`, as category, reason key and item
/// count, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32)> {
    let module = ModuleId::new("local-ai-models").unwrap();
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
fn parts(sandbox: &Sandbox) -> Parts {
    Parts {
        modules: vec![Box::new(LocalAiModelsModule::new().unwrap())],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: home_rescan::journal(sandbox, &[]),
        installed: FakeInstalledApps::default(),
        running: FakeRunningApps::default(),
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
