//! The fixture homes of the app-leftovers module's tests, the system each is scanned with, and one run of the module
//! on each.
//!
//! Each home holds only files under its `Library`; the installed, running and elsewhere apps are given to the module
//! as fakes. Facts the disk cannot be made to show (a status change 40 days back, a socket, the system's owner) are
//! stated, and the run reads them through the facts lens.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{BundleId, InstalledApp};
use fresher_module_app_leftovers::AppLeftoversModule;
use fresher_testing::{
    FIXTURE_NOW, FakeInstalledApps, FakeRunningApps, FixtureTree, MatchOrder, ModuleRun, Ports, installed_app,
    run_module, running_app,
};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn app_leftovers() -> AppLeftoversModule {
    AppLeftoversModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// An installed app with the id `id` and the helpers `helpers`.
pub(crate) fn app(id: &str, helpers: &[&str]) -> InstalledApp {
    let helpers = helpers.iter().map(|&helper| BundleId::new(helper).unwrap()).collect();
    InstalledApp { helpers, ..installed_app(id, "App", "1").unwrap() }
}

/// The app the kept home's `com.example.Elsewhere` belongs to, outside the apps folders.
pub(crate) fn elsewhere() -> Vec<InstalledApp> {
    vec![app("com.example.Elsewhere", &[])]
}

/// The system a home is scanned with: the installed apps (and the ones Spotlight finds elsewhere), the running apps,
/// whether the scan is restored from the cache, the order the matches come in and the folders the walk avoids.
pub(crate) struct System {
    pub(crate) apps: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    pub(crate) restored: bool,
    pub(crate) order: MatchOrder,
    pub(crate) avoid: Vec<PathBuf>,
}

impl System {
    /// Nothing installed or running, a walked scan of every folder.
    pub(crate) fn plain() -> Self {
        Self {
            apps: FakeInstalledApps::default(),
            running: FakeRunningApps::default(),
            restored: false,
            order: MatchOrder::Walked,
            avoid: Vec::new(),
        }
    }

    /// The kept home's system: `com.example.Running` open, `com.example.Elsewhere` found by Spotlight.
    pub(crate) fn kept() -> Self {
        let running = FakeRunningApps::new(vec![running_app("Running", &["com.example.Running"]).unwrap()]);
        Self::plain().apps(FakeInstalledApps::default().with_elsewhere(elsewhere())).running(running)
    }

    pub(crate) fn apps(self, apps: FakeInstalledApps) -> Self {
        Self { apps, ..self }
    }

    pub(crate) fn running(self, running: FakeRunningApps) -> Self {
        Self { running, ..self }
    }

    pub(crate) fn restored(self) -> Self {
        Self { restored: true, ..self }
    }

    pub(crate) fn reversed(self) -> Self {
        Self { order: MatchOrder::Reversed, ..self }
    }

    /// The walk does not open `folder`, relative to the home.
    pub(crate) fn avoiding(self, folder: &str) -> Self {
        Self { avoid: vec![PathBuf::from(folder)], ..self }
    }
}

/// The home `name` scanned with `system`, the clock at `FIXTURE_NOW`.
pub(crate) fn scan(name: &str, system: System) -> (FixtureTree, ModuleRun) {
    let tree = home(name);
    let ports = Ports {
        apps: Arc::new(system.apps),
        running: Arc::new(system.running),
        restored: system.restored,
        order: system.order,
        avoid: system.avoid,
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&app_leftovers(), &tree, &ports).unwrap();
    (tree, run)
}

/// Names at the module's places that are not an app's id.
pub(crate) fn not_ids() -> (FixtureTree, ModuleRun) {
    scan("app-leftovers-not-ids", System::plain())
}

/// Apple's own apps' files in every place.
pub(crate) fn apple() -> (FixtureTree, ModuleRun) {
    scan("app-leftovers-apple", System::plain())
}

/// Ids with no proof of an app, version-like names among them.
pub(crate) fn unproven() -> (FixtureTree, ModuleRun) {
    scan("app-leftovers-unproven", System::plain())
}

/// An id's leftovers in a home with neither proof place.
pub(crate) fn proof_absent() -> (FixtureTree, ModuleRun) {
    scan("app-leftovers-proof-absent", System::plain())
}

/// An installed app's, its helper's and its related ids' files, `com.example.Editor` and `com.example.App` installed.
pub(crate) fn installed() -> (FixtureTree, ModuleRun) {
    let apps = vec![app("com.example.Editor", &["com.example.Editor.Helper"]), app("com.example.App", &[])];
    scan("app-leftovers-installed", System::plain().apps(FakeInstalledApps::new(apps)))
}

/// One proven id per reason to keep it, scanned with `system`.
pub(crate) fn kept_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-leftovers-kept", system)
}

/// The kept home with its own system.
pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    kept_with(System::kept())
}

/// The leftovers of apps that are gone, `com.example.App` installed, scanned with `system`'s other parts.
pub(crate) fn leftovers_with(system: System) -> (FixtureTree, ModuleRun) {
    let apps = FakeInstalledApps::new(vec![app("com.example.App", &[])]);
    scan("app-leftovers-leftovers", system.apps(apps))
}

/// The leftovers home, every folder walked.
pub(crate) fn leftovers() -> (FixtureTree, ModuleRun) {
    leftovers_with(System::plain())
}
