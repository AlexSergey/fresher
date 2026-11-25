//! What the logs flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the real scanner,
//! its reader seen through the facts lens, with the modules of `Library/Logs` the app registers (`jetbrains`,
//! `app-leftovers`, `logs`, in its order); the real settings file, scan cache and journal in the sandbox's data
//! folder; the real `DiskCleaner` reaching only inside the sandbox folder, as sandbox mode composes it, with a fake
//! Trash at `home/.Trash` (the shared flow home of `support/home*.rs`). `WebStorm` 2025.3 is installed and nothing is
//! open; the clock stands at `FIXTURE_NOW`.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the shared flow home keeps what the flows of other modules restart")]
mod home;
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[path = "../support/report.rs"]
mod report;

use std::sync::Arc;

use fresher_core::{CleanupSelection, Event, LeftOutCount, ModuleId, SettingUpdate, SettingValue};
use fresher_module_app_leftovers::AppLeftoversModule;
use fresher_module_jetbrains::JetBrainsModule;
use fresher_module_logs::LogsModule;
use fresher_testing::{
    FIXTURE_NOW, FakeChangeJournal, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree, installed_app,
};
use serde_json::json;

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Parts, no_installs, unknown_access};

/// The module's two categories.
pub(crate) const APP_LOGS: &str = "logs.app-logs";
pub(crate) const CRASH_REPORTS: &str = "logs.crash-reports";

impl Flow {
    /// The sandbox holding `fixtures/homes/logs-flow.toml`, its engine on it.
    pub(crate) fn new() -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("logs-flow.toml")).unwrap();
        Self::open(tree, |_| parts())
    }

    /// Sets `logs.older-than-days` to `days` through the engine, into the real settings file.
    pub(crate) fn set_days(&self, days: i64) {
        let update = SettingUpdate { key: "logs.older-than-days".into(), value: SettingValue::Number(days) };
        self.engine.update_setting(update).unwrap();
    }
}

/// The selection of the module's two categories, whole.
pub(crate) fn select_logs() -> CleanupSelection {
    let categories = [APP_LOGS, CRASH_REPORTS].map(|category| json!({ "category": category, "except": [] }));
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

/// The engine's parts: the app's modules of `Library/Logs` in its order, `WebStorm` 2025.3.1 installed, nothing
/// open, a change journal that knows nothing, two threads.
fn parts() -> Parts {
    Parts {
        modules: vec![
            Box::new(JetBrainsModule::new().unwrap()),
            Box::new(AppLeftoversModule::new().unwrap()),
            Box::new(LogsModule::new().unwrap()),
        ],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: FakeChangeJournal::default(),
        installed: FakeInstalledApps::new(vec![
            installed_app("com.jetbrains.WebStorm", "WebStorm", "2025.3.1").unwrap(),
        ]),
        running: FakeRunningApps::default(),
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
