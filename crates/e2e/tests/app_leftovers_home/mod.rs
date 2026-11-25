//! What the app-leftovers flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the real
//! scanner, its reader seen through the facts lens, with the modules the app registers (`jetbrains`, `temp`,
//! `dev-caches`, `app-leftovers`, in its order); the real settings file, scan cache and journal in the sandbox's data
//! folder; the real `DiskCleaner` reaching only inside the sandbox folder, as sandbox mode composes it, with a fake
//! Trash at `home/.Trash` (the shared flow home of `support/home*.rs`). `com.example.Editor` and `WebStorm` 2025.3 are
//! installed; the clock, the installed and open apps, the folders the change journal names at each rescan and the
//! places no walk opens are what a flow sets. `restart` builds a new engine on the same files, as when the app starts
//! again.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
mod home;
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
pub(crate) mod home_engine;
#[path = "../support/home_rescan.rs"]
mod home_rescan;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[path = "../support/report.rs"]
mod report;

use std::sync::Arc;

use fresher_core::{CleanupSelection, Event, InstalledApp, LeftOutCount, ModuleId};
use fresher_macos::places_that_prompt;
use fresher_module_app_leftovers::AppLeftoversModule;
use fresher_module_dev_caches::DevCachesModule;
use fresher_module_jetbrains::JetBrainsModule;
use fresher_module_temp::TempModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree, installed_app};
use serde_json::{Value, json};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Parts, no_installs, unknown_access};

/// The module's id.
pub(crate) const MODULE: &str = "app-leftovers";

/// The places no walk opens.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Avoid {
    /// Every place is walked, `Library/Containers` included.
    #[default]
    Nothing,
    /// The places that may raise a system prompt, as the app composes them for the home.
    PlacesThatPrompt,
}

/// The world an engine runs in besides the files.
#[derive(Debug)]
pub(crate) struct Setup {
    /// The clock; a flow that moves it on keeps a handle.
    pub(crate) clock: Arc<FakeClock>,
    pub(crate) installed: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    /// At each rescan, in order, the folders the change journal says changed, relative to the sandbox. A rescan past
    /// the last one meets a gap, and the scan reads every folder.
    pub(crate) changed: Vec<Vec<&'static str>>,
    pub(crate) avoid: Avoid,
}

impl Default for Setup {
    /// The clock at `FIXTURE_NOW`, the apps of `installed_apps` installed, nothing open, no change named, every place
    /// walked.
    fn default() -> Self {
        Self {
            clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
            installed: FakeInstalledApps::new(installed_apps()),
            running: FakeRunningApps::default(),
            changed: Vec::new(),
            avoid: Avoid::Nothing,
        }
    }
}

/// `com.example.Editor`, whose files are never claimed, and `WebStorm` 2025.3.1, whose 2024.3 folder `jetbrains`
/// offers.
pub(crate) fn installed_apps() -> Vec<InstalledApp> {
    vec![app("com.example.Editor", "Editor", "1.0"), app("com.jetbrains.WebStorm", "WebStorm", "2025.3.1")]
}

/// An app installed in `/Applications`.
pub(crate) fn app(bundle_id: &str, name: &str, version: &str) -> InstalledApp {
    installed_app(bundle_id, name, version).unwrap()
}

impl Flow {
    /// The sandbox holding `fixtures/homes/app-leftovers-flow.toml`, its engine in the default setup.
    pub(crate) fn new() -> Self {
        Self::with(Setup::default())
    }

    /// The same sandbox, its engine in `setup`.
    pub(crate) fn with(setup: Setup) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("app-leftovers-flow.toml")).unwrap();
        Self::open(tree, |sandbox| parts(sandbox, setup))
    }

    /// A new engine on the same files in `setup`: the app started again.
    pub(crate) fn restart(&mut self, setup: Setup) {
        let parts = parts(&self.sandbox, setup);
        self.restart_with(parts);
    }

    /// The selection of every category of the module's choice `name`, whole, as its button makes it.
    pub(crate) fn choice(&self, name: &str) -> CleanupSelection {
        let module = self.engine.modules().into_iter().find(|module| module.id.as_str() == MODULE).unwrap();
        let choice = module.choices.into_iter().find(|choice| choice.name == name).unwrap();
        let categories: Vec<Value> =
            choice.categories.iter().map(|category| json!({ "category": category, "except": [] })).collect();
        serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
    }
}

/// What the module left out in the events' `ModuleFinished`, as category and reason (both without the module's
/// prefix) and item count, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32)> {
    let module = ModuleId::new(MODULE).unwrap();
    let short = |key: &str| key.trim_start_matches("app-leftovers.").trim_start_matches("left-out.").to_owned();
    let mut counts: Vec<(String, String, u32)> = events
        .iter()
        .filter_map(|event| match event {
            Event::ModuleFinished { module: finished, left_out, .. } if *finished == module => Some(left_out),
            _ => None,
        })
        .flatten()
        .map(|count: &LeftOutCount| (short(count.category.as_str()), short(count.reason.key().as_str()), count.items))
        .collect();
    counts.sort();
    counts
}

/// The engine's parts in `setup` on `sandbox`: the app's modules in its order, two threads.
pub(crate) fn parts(sandbox: &Sandbox, setup: Setup) -> Parts {
    let avoid = match setup.avoid {
        Avoid::Nothing => Vec::new(),
        Avoid::PlacesThatPrompt => places_that_prompt(&sandbox.home()),
    };
    Parts {
        modules: vec![
            Box::new(JetBrainsModule::new().unwrap()),
            Box::new(TempModule::new().unwrap()),
            Box::new(DevCachesModule::new().unwrap()),
            Box::new(AppLeftoversModule::new().unwrap()),
        ],
        clock: setup.clock,
        journal: home_rescan::journal(sandbox, &setup.changed),
        installed: setup.installed,
        running: setup.running,
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid,
    }
}
