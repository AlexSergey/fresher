//! What the stale-projects flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the real
//! scanner, its reader seen through the facts lens, with the modules the app registers (`jetbrains`, `temp`,
//! `dev-caches`, `app-leftovers`, `downloads`, `stale-projects`, in its order); the real settings file, scan cache and
//! journal in the sandbox's data folder; the real `DiskCleaner` reaching only inside the sandbox folder, with a fake
//! Trash at `home/.Trash` (the shared flow home of `support/home*.rs`). Every folder of the home is dated. The clock,
//! the open apps and the folders the change journal names at each rescan are what a flow sets.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the shared flow home keeps what the flows of other modules restart or read")]
mod home;
#[path = "../support/home_cleanup.rs"]
#[expect(dead_code, reason = "the flows of other modules read the report it also writes")]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[path = "../support/home_rescan.rs"]
mod home_rescan;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[path = "../support/report.rs"]
#[expect(dead_code, reason = "the flows of other modules sort and unnumber their reports")]
mod report;

use std::sync::Arc;

use fresher_core::{CleanupSelection, Event, LeftOutCount, ModuleId, SettingUpdate};
use fresher_module_app_leftovers::AppLeftoversModule;
use fresher_module_dev_caches::DevCachesModule;
use fresher_module_downloads::DownloadsModule;
use fresher_module_jetbrains::JetBrainsModule;
use fresher_module_stale_projects::StaleProjectsModule;
use fresher_module_temp::TempModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree};
use serde_json::{Value, json};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Parts, no_installs, unknown_access};

/// The module's id.
pub(crate) const MODULE: &str = "stale-projects";

/// The world an engine runs in besides the files.
#[derive(Debug, Default)]
pub(crate) struct Setup {
    /// At each rescan, in order, the folders the change journal says changed, relative to the sandbox. A rescan past
    /// the last one meets a gap, and the scan reads every folder.
    pub(crate) changed: Vec<Vec<&'static str>>,
}

impl Flow {
    /// The sandbox holding `fixtures/homes/stale-projects-flow.toml`, its engine with no change named.
    pub(crate) fn new() -> Self {
        Self::with(&Setup::default())
    }

    /// The same sandbox, its engine in `setup`, scanning the home.
    pub(crate) fn with(setup: &Setup) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("stale-projects-flow.toml")).unwrap();
        let flow = Self::open(tree, |sandbox| parts(sandbox, setup));
        let roots = json!({ "key": "core.scan.roots", "value": { "kind": "folders", "value": [flow.sandbox.home()] } });
        flow.engine.update_setting(serde_json::from_value(roots).unwrap()).unwrap();
        flow
    }

    /// The selection of every category of the module, whole.
    pub(crate) fn everything(&self) -> CleanupSelection {
        let module = self.engine.modules().into_iter().find(|module| module.id.as_str() == MODULE).unwrap();
        let categories: Vec<Value> = module
            .categories
            .iter()
            .map(|category| json!({ "category": category.descriptor.id(), "except": [] }))
            .collect();
        serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
    }

    /// The setting `key` set to `days`.
    pub(crate) fn set_days(&self, key: &str, days: i64) {
        let update = json!({ "key": key, "value": { "kind": "number", "value": days } });
        let update: SettingUpdate = serde_json::from_value(update).unwrap();
        self.engine.update_setting(update).unwrap();
    }
}

/// What the module left out in the events' `ModuleFinished`, as category and reason (both without the module's
/// prefix), item count and logical bytes, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32, u64)> {
    let module = ModuleId::new(MODULE).unwrap();
    let short = |key: &str| key.trim_start_matches("stale-projects.").trim_start_matches("left-out.").to_owned();
    let mut counts: Vec<(String, String, u32, u64)> = events
        .iter()
        .filter_map(|event| match event {
            Event::ModuleFinished { module: finished, left_out, .. } if *finished == module => Some(left_out),
            _ => None,
        })
        .flatten()
        .map(|count: &LeftOutCount| {
            (
                short(count.category.as_str()),
                short(count.reason.key().as_str()),
                count.items,
                count.sizes.logical.bytes(),
            )
        })
        .collect();
    counts.sort();
    counts
}

/// The engine's parts on `sandbox`: the app's modules in its order, the clock at `FIXTURE_NOW`, two threads.
fn parts(sandbox: &Sandbox, setup: &Setup) -> Parts {
    Parts {
        modules: vec![
            Box::new(JetBrainsModule::new().unwrap()),
            Box::new(TempModule::new().unwrap()),
            Box::new(DevCachesModule::new().unwrap()),
            Box::new(AppLeftoversModule::new().unwrap()),
            Box::new(DownloadsModule::new().unwrap()),
            Box::new(StaleProjectsModule::new().unwrap()),
        ],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: home_rescan::journal(sandbox, &setup.changed),
        installed: FakeInstalledApps::default(),
        running: FakeRunningApps::default(),
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
