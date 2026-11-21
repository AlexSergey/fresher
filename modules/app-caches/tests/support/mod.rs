//! The fixture homes of the app-caches module's tests, the system each is scanned with, and one run of the module on
//! each. The installed and running apps are given to the module as fakes; facts the disk cannot be made to show (a
//! socket, the user's lock) are stated by the home.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{BundleId, InstalledApp};
use fresher_module_app_caches::AppCachesModule;
use fresher_testing::{
    FIXTURE_NOW, FakeInstalledApps, FakeRunningApps, FixtureTree, MatchOrder, ModuleRun, Ports, installed_app,
    run_module, running_app,
};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn app_caches() -> AppCachesModule {
    AppCachesModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// An installed app named `name` with the id `id` and the helpers `helpers`.
pub(crate) fn app(name: &str, id: &str, helpers: &[&str]) -> InstalledApp {
    let helpers = helpers.iter().map(|&helper| BundleId::new(helper).unwrap()).collect();
    InstalledApp { helpers, ..installed_app(id, name, "1").unwrap() }
}

/// The system a home is scanned with: the installed apps, the running apps and the order the matches come in.
pub(crate) struct System {
    pub(crate) apps: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    pub(crate) order: MatchOrder,
}

impl System {
    /// The apps installed, nothing running.
    pub(crate) fn with(apps: Vec<InstalledApp>) -> Self {
        Self { apps: FakeInstalledApps::new(apps), running: FakeRunningApps::default(), order: MatchOrder::Walked }
    }

    pub(crate) fn apps(self, apps: FakeInstalledApps) -> Self {
        Self { apps, ..self }
    }

    pub(crate) fn running(self, running: FakeRunningApps) -> Self {
        Self { running, ..self }
    }

    pub(crate) fn reversed(self) -> Self {
        Self { order: MatchOrder::Reversed, ..self }
    }
}

/// The home `name` scanned with `system`, the clock at `FIXTURE_NOW`.
fn scan(name: &str, system: System) -> (FixtureTree, ModuleRun) {
    let tree = home(name);
    let ports = Ports {
        apps: Arc::new(system.apps),
        running: Arc::new(system.running),
        order: system.order,
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&app_caches(), &tree, &ports).unwrap();
    (tree, run)
}

/// The not-claimed home's system: `com.example.Editor` and its helper installed.
pub(crate) fn editor_system() -> System {
    System::with(vec![app("Editor", "com.example.Editor", &["com.example.Editor.Helper"])])
}

/// The kept home's system: its four apps installed, `com.example.Open` and `com.example.Agent`'s helper running.
pub(crate) fn kept_system() -> System {
    let apps = ["Busy", "Locked", "Open"].map(|name| app(name, &format!("com.example.{name}"), &[])).to_vec();
    let agent = app("Agent", "com.example.Agent", &["com.example.Agent.Menu"]);
    let running = FakeRunningApps::new(vec![
        running_app("Open", &["com.example.Open"]).unwrap(),
        running_app("Menu", &["com.example.Agent.Menu"]).unwrap(),
    ]);
    System::with([apps, vec![agent]].concat()).running(running)
}

/// The offered home's system: the editor with its helper, and the suite and its pro edition, installed.
pub(crate) fn offered_system() -> System {
    System::with(vec![
        app("Editor", "com.example.Editor", &["com.example.Editor.Helper"]),
        app("Suite", "com.example.Suite", &[]),
        app("Suite Pro", "com.example.Suite.Pro", &[]),
    ])
}

/// Names and ids nothing may claim, scanned with `system`.
pub(crate) fn not_claimed_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-not-claimed", system)
}

pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    not_claimed_with(editor_system())
}

/// One folder per reason to keep it, scanned with `system`.
pub(crate) fn kept_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-kept", system)
}

pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    kept_with(kept_system())
}

/// Every cache of an installed app, scanned with `system`.
pub(crate) fn offered_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("app-caches-offered", system)
}

pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    offered_with(offered_system())
}
