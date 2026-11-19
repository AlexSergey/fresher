//! What the downloads flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the real
//! scanner, its reader seen through the facts lens, with the modules the app registers (`jetbrains`, `temp`,
//! `dev-caches`, `app-leftovers`, `downloads`, in its order); the real settings file, scan cache and journal in the
//! sandbox's data folder; the real `DiskCleaner` reaching only inside the sandbox folder, with a fake Trash at
//! `home/.Trash` (the shared flow home of `support/home*.rs`). The system's permissions are a `FakeAccess` guarding
//! `home/Downloads` with the Downloads folder's capability; Firefox 133.0 and `Example` 1.0 are installed, a receipt
//! proves `Tool-3.1.pkg`, macOS is 15.4. The clock, the installed and open apps, the access and the folders the
//! change journal names at each rescan are what a flow sets.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the shared flow home keeps what the flows of other modules restart or read")]
mod home;
#[path = "../support/home_cleanup.rs"]
#[expect(dead_code, reason = "the flows of other modules read the report it also writes")]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
#[expect(dead_code, reason = "the helpers serve the homes that need no access or installs")]
mod home_engine;
#[path = "../support/home_rescan.rs"]
mod home_rescan;
#[path = "../support/home_scan.rs"]
#[expect(dead_code, reason = "the flows of other modules restore")]
mod home_scan;
#[path = "../support/report.rs"]
#[expect(dead_code, reason = "the flows of other modules sort and unnumber their reports")]
mod report;

use std::sync::Arc;

use fresher_core::{
    Access, Capability, CleanupSelection, Event, GuardedFolder, InstalledApp, LeftOutCount, ModuleId, ModuleOutcome,
    Receipt, SettingUpdate, Timestamp,
};
use fresher_module_app_leftovers::AppLeftoversModule;
use fresher_module_dev_caches::DevCachesModule;
use fresher_module_downloads::DownloadsModule;
use fresher_module_jetbrains::JetBrainsModule;
use fresher_module_temp::TempModule;
use fresher_sandbox::Sandbox;
use fresher_testing::{
    FIXTURE_NOW, FakeAccess, FakeClock, FakeInstallRecords, FakeInstalledApps, FakeRunningApps, FixtureTree,
    changed_at, installed_app,
};
use serde_json::{Value, json};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::Parts;

/// The module's id.
pub(crate) const MODULE: &str = "downloads";
/// The capability that guards `home/Downloads`.
pub(crate) const DOWNLOADS: Capability = Capability::Folder(GuardedFolder::Downloads);
/// The switch that lets the scan try the folder.
pub(crate) const SWITCH: &str = "core.access.downloads";
/// Seconds in a day.
pub(crate) const DAY: u64 = 24 * 60 * 60;

/// The world an engine runs in besides the files.
#[derive(Debug)]
pub(crate) struct Setup {
    /// The clock; a flow that moves it on keeps a handle.
    pub(crate) clock: Arc<FakeClock>,
    pub(crate) installed: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    /// The state of Full Disk Access.
    pub(crate) full_disk: Access,
    /// The state of the Downloads folder's permission before the first try.
    pub(crate) folder: Access,
    /// What trying the folder answers.
    pub(crate) answer: Access,
    /// At each rescan, in order, the folders the change journal says changed, relative to the sandbox. A rescan past
    /// the last one meets a gap, and the scan reads every folder.
    pub(crate) changed: Vec<Vec<&'static str>>,
}

impl Default for Setup {
    /// The clock at `FIXTURE_NOW`, the apps of `installed_apps` installed, nothing open, Full Disk Access denied,
    /// Downloads not asked, and a try that is granted.
    fn default() -> Self {
        Self {
            clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
            installed: FakeInstalledApps::new(installed_apps(Example::Old)),
            running: FakeRunningApps::default(),
            full_disk: Access::Denied,
            folder: Access::NotAsked,
            answer: Access::Granted,
            changed: Vec::new(),
        }
    }
}

/// Which `Example` is installed.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Example {
    /// 1.0, changed 10 days ago: older than the installer of 2.0, which is 5 days old.
    Old,
    /// 2.0, changed 2 days ago: after the installer.
    Updated,
}

/// Firefox 133.0 changed 10 days ago (after its installer, 20 days old) and `Example`.
pub(crate) fn installed_apps(example: Example) -> Vec<InstalledApp> {
    let (version, days) = match example {
        Example::Old => ("1.0", 10),
        Example::Updated => ("2.0", 2),
    };
    vec![app("org.mozilla.firefox", "Firefox", "133.0", 10), app("com.example.Example", "Example", version, days)]
}

/// An installed app changed `days` days ago.
fn app(bundle_id: &str, name: &str, version: &str, days: u64) -> InstalledApp {
    changed_at(installed_app(bundle_id, name, version).unwrap(), FIXTURE_NOW - days * DAY)
}

/// The fixture sandbox, its engine, and the access it was given.
pub(crate) struct Home {
    pub(crate) flow: Flow,
    pub(crate) access: Arc<FakeAccess>,
}

impl Home {
    /// The sandbox holding `fixtures/homes/downloads-flow.toml`, its engine in the default setup.
    pub(crate) fn new() -> Self {
        Self::with(Setup::default())
    }

    /// The same sandbox, its engine in `setup`.
    pub(crate) fn with(setup: Setup) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("downloads-flow.toml")).unwrap();
        let mut handle = None;
        let flow = Flow::open(tree, |sandbox| {
            let access = Arc::new(access(sandbox, &setup));
            handle = Some(Arc::clone(&access));
            parts(sandbox, setup, access)
        });
        let roots = json!({ "key": "core.scan.roots", "value": { "kind": "folders", "value": [flow.sandbox.home()] } });
        flow.engine.update_setting(serde_json::from_value(roots).unwrap()).unwrap();
        Self { flow, access: handle.unwrap() }
    }

    /// The Downloads switch of the Settings turned on.
    pub(crate) fn opt_in(&self) {
        self.switch(true);
    }

    /// The Downloads switch of the Settings turned off again.
    pub(crate) fn opt_out(&self) {
        self.switch(false);
    }

    fn switch(&self, on: bool) {
        let update = json!({ "key": SWITCH, "value": { "kind": "switch", "value": on } });
        let update: SettingUpdate = serde_json::from_value(update).unwrap();
        self.flow.engine.update_setting(update).unwrap();
    }

    /// The selection of every category of the module, whole.
    pub(crate) fn everything(&self) -> CleanupSelection {
        let module = self.flow.engine.modules().into_iter().find(|module| module.id.as_str() == MODULE).unwrap();
        let categories: Vec<Value> = module
            .categories
            .iter()
            .map(|category| json!({ "category": category.descriptor.id(), "except": [] }))
            .collect();
        serde_json::from_value(json!({ "findings": [], "categories": categories })).unwrap()
    }
}

/// The access of `setup`: Downloads guarded under the sandbox's home.
fn access(sandbox: &Sandbox, setup: &Setup) -> FakeAccess {
    FakeAccess::with(&[(Capability::AllFiles, setup.full_disk), (DOWNLOADS, setup.folder)])
        .home(sandbox.home())
        .guard("Downloads", DOWNLOADS)
        .answering("Downloads", setup.answer)
}

/// The module's outcome in the events' `ModuleFinished`.
pub(crate) fn outcome(events: &[Event]) -> Option<ModuleOutcome> {
    let module = ModuleId::new(MODULE).unwrap();
    events.iter().find_map(|event| match event {
        Event::ModuleFinished { module: finished, outcome, .. } if *finished == module => Some(outcome.clone()),
        _ => None,
    })
}

/// What the module left out in the events' `ModuleFinished`, as category and reason (both without the module's
/// prefix) and item count, sorted.
pub(crate) fn left_out(events: &[Event]) -> Vec<(String, String, u32)> {
    let module = ModuleId::new(MODULE).unwrap();
    let short = |key: &str| key.trim_start_matches("downloads.").trim_start_matches("left-out.").to_owned();
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
fn parts(sandbox: &Sandbox, setup: Setup, access: Arc<FakeAccess>) -> Parts {
    let receipt =
        Receipt { package_file: "Tool-3.1.pkg".into(), installed: Timestamp::from_secs(FIXTURE_NOW - 10 * DAY) };
    Parts {
        modules: vec![
            Box::new(JetBrainsModule::new().unwrap()),
            Box::new(TempModule::new().unwrap()),
            Box::new(DevCachesModule::new().unwrap()),
            Box::new(AppLeftoversModule::new().unwrap()),
            Box::new(DownloadsModule::new().unwrap()),
        ],
        clock: setup.clock,
        journal: home_rescan::journal(sandbox, &setup.changed),
        installed: setup.installed,
        running: setup.running,
        access,
        installs: Arc::new(FakeInstallRecords::default().with_receipts(vec![receipt]).with_version(15, 4)),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
