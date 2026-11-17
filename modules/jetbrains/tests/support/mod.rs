//! The fixture homes of the jetbrains module's tests, the apps each is scanned with, and one run of the module on each.
//!
//! Each home holds only files; its apps are given to the module as fakes. The clock stands at `FIXTURE_NOW`, from
//! which the homes' folder ages count back.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{InstalledApp, RunningApp, StoredSettings};
use fresher_module_jetbrains::JetBrainsModule;
use fresher_testing::{
    FIXTURE_NOW, FakeInstalledApps, FakeRunningApps, FixtureTree, ModuleRun, Ports, installed_app, run_module,
    running_app,
};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn jetbrains() -> JetBrainsModule {
    JetBrainsModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// An app installed in the apps folder.
pub(crate) fn app(bundle_id: &str, name: &str, version: &str) -> InstalledApp {
    installed_app(bundle_id, name, version).unwrap()
}

/// An app open with one bundle id.
pub(crate) fn open(name: &str, bundle_id: &str) -> RunningApp {
    running_app(name, &[bundle_id]).unwrap()
}

pub(crate) fn webstorm() -> InstalledApp {
    app("com.jetbrains.WebStorm", "WebStorm", "2025.3.1")
}

/// The system a home is scanned with: the installed apps, the open ones and the stored settings (as JSON).
pub(crate) struct System {
    pub(crate) installed: FakeInstalledApps,
    pub(crate) running: Vec<RunningApp>,
    pub(crate) stored: &'static str,
}

impl System {
    pub(crate) fn installed(apps: Vec<InstalledApp>) -> Self {
        Self::with(FakeInstalledApps::new(apps))
    }

    pub(crate) fn with(installed: FakeInstalledApps) -> Self {
        Self { installed, running: Vec::new(), stored: "{}" }
    }

    pub(crate) fn running(self, running: Vec<RunningApp>) -> Self {
        Self { running, ..self }
    }

    pub(crate) fn stored(self, stored: &'static str) -> Self {
        Self { stored, ..self }
    }
}

/// The home `name` scanned with `system`, the clock at `FIXTURE_NOW`.
pub(crate) fn scan(name: &str, system: System) -> (FixtureTree, ModuleRun) {
    let tree = home(name);
    let settings: StoredSettings = serde_json::from_str(system.stored).unwrap();
    let ports = Ports {
        settings,
        apps: Arc::new(system.installed),
        running: Arc::new(FakeRunningApps::new(system.running)),
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&jetbrains(), &tree, &ports).unwrap();
    (tree, run)
}

/// Only the version in use has folders: `WebStorm` 2025.3.1 installed.
pub(crate) fn current() -> (FixtureTree, ModuleRun) {
    scan("jetbrains-current", System::installed(vec![webstorm()]))
}

/// Names that only look like version folders: `WebStorm` 2025.3.1 installed.
pub(crate) fn similar_names() -> (FixtureTree, ModuleRun) {
    scan("jetbrains-similar-names", System::installed(vec![webstorm()]))
}

/// `GoLand` 2025.2.1 installed and open; `CLion` open from no apps folder.
pub(crate) fn running() -> (FixtureTree, ModuleRun) {
    let goland = app("com.jetbrains.goland", "GoLand", "2025.2.1");
    let open = vec![open("GoLand", "com.jetbrains.goland"), open("CLion", "com.jetbrains.CLion")];
    scan("jetbrains-running", System::installed(vec![goland]).running(open))
}

/// The folders of IDEs that are not installed, one of each changed recently; `PyCharm` (the full edition) installed.
pub(crate) fn uninstalled_fresh() -> (FixtureTree, ModuleRun) {
    scan("jetbrains-uninstalled-fresh", System::installed(vec![app("com.jetbrains.pycharm", "PyCharm", "2025.2.3")]))
}

/// `WebStorm` 2025.3.1, `IntelliJ IDEA` 2025.2.4 and its early access build 2025.3 installed.
pub(crate) fn stale() -> (FixtureTree, ModuleRun) {
    let idea = app("com.jetbrains.intellij", "IntelliJ IDEA", "2025.2.4");
    let early = app("com.jetbrains.intellij-EAP", "IntelliJ IDEA EAP", "2025.3 EAP");
    scan("jetbrains-stale", System::installed(vec![webstorm(), idea, early]))
}

/// The data of `DataGrip` 2024.3, 40 days old, scanned with `system`.
pub(crate) fn uninstalled_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("jetbrains-uninstalled", system)
}

/// The data of `DataGrip` 2024.3, 40 days old, with no app installed or open.
pub(crate) fn uninstalled() -> (FixtureTree, ModuleRun) {
    uninstalled_with(System::installed(Vec::new()))
}
