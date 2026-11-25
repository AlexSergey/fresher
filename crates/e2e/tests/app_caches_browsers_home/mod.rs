//! What the app-caches browser flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the
//! real scanner, its reader seen through the facts lens, with the modules of `Library` the app registers
//! (`jetbrains`, `dev-caches`, `app-leftovers`, `app-caches`, in its order); the real settings file, scan cache and
//! journal in the sandbox's data folder; the real `DiskCleaner` reaching only inside the sandbox folder, as sandbox
//! mode composes it, with a fake Trash at `home/.Trash` (the shared flow home of `support/home*.rs`).
//! Google Chrome, Redis Insight, `com.example.Editor` and `WebStorm` 2025.3 are installed; the clock, the installed and
//! open apps and the folders the change journal names at each rescan are what a flow sets.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the shared flow home keeps what the flows of other modules restart or read")]
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

use fresher_core::{Event, InstalledApp, LeftOutCount, ModuleId};
use fresher_module_app_caches::AppCachesModule;
use fresher_module_app_leftovers::AppLeftoversModule;
use fresher_module_dev_caches::DevCachesModule;
use fresher_module_jetbrains::JetBrainsModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, FixtureTree, installed_app};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Parts, no_installs, unknown_access};

/// The module's id.
pub(crate) const MODULE: &str = "app-caches";
/// Google Chrome's bundle id.
pub(crate) const CHROME_ID: &str = "com.google.Chrome";
/// Redis Insight's bundle id.
pub(crate) const REDIS_INSIGHT_ID: &str = "org.RedisLabs.RedisInsight-V2";

/// The world an engine runs in besides the files.
#[derive(Debug)]
pub(crate) struct Setup {
    pub(crate) installed: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    /// At each rescan, in order, the folders the change journal says changed, relative to the sandbox. A rescan past
    /// the last one meets a gap, and the scan reads every folder.
    pub(crate) changed: Vec<Vec<&'static str>>,
}

impl Default for Setup {
    /// The apps of `installed_apps` installed, nothing open, no change named.
    fn default() -> Self {
        Self {
            installed: FakeInstalledApps::new(installed_apps()),
            running: FakeRunningApps::default(),
            changed: Vec::new(),
        }
    }
}

/// Google Chrome and Redis Insight, whose table caches the module offers, `com.example.Editor`, whose id-named cache
/// it offers, and `WebStorm` 2025.3.1, whose 2024.3 folder `jetbrains` offers.
pub(crate) fn installed_apps() -> Vec<InstalledApp> {
    vec![
        app(CHROME_ID, "Google Chrome", "131.0"),
        app(REDIS_INSIGHT_ID, "Redis Insight", "2.60.0"),
        app("com.example.Editor", "Editor", "1.0"),
        app("com.jetbrains.WebStorm", "WebStorm", "2025.3.1"),
    ]
}

/// An app installed in `/Applications`.
fn app(bundle_id: &str, name: &str, version: &str) -> InstalledApp {
    installed_app(bundle_id, name, version).unwrap()
}

impl Flow {
    /// The sandbox holding `fixtures/homes/app-caches-browsers-flow.toml`, its engine in the default setup.
    pub(crate) fn new() -> Self {
        Self::with(Setup::default())
    }

    /// The same sandbox, its engine in `setup`.
    pub(crate) fn with(setup: Setup) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("app-caches-browsers-flow.toml")).unwrap();
        Self::open(tree, |sandbox| parts(sandbox, setup))
    }
}

/// What the module left out in the events' `ModuleFinished`, as category and reason (both without the module's
/// prefix) and item count, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32)> {
    let module = ModuleId::new(MODULE).unwrap();
    let short = |key: &str| key.trim_start_matches("app-caches.").trim_start_matches("left-out.").to_owned();
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

/// The engine's parts in `setup` on `sandbox`: the app's modules in its order, two threads, the clock at
/// `FIXTURE_NOW`.
fn parts(sandbox: &Sandbox, setup: Setup) -> Parts {
    Parts {
        modules: vec![
            Box::new(JetBrainsModule::new().unwrap()),
            Box::new(DevCachesModule::new().unwrap()),
            Box::new(AppLeftoversModule::new().unwrap()),
            Box::new(AppCachesModule::new().unwrap()),
        ],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: home_rescan::journal(sandbox, &setup.changed),
        installed: setup.installed,
        running: setup.running,
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
