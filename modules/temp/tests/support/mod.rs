//! The fixture homes of the temp module's tests, the system each is scanned with, and one run of the module on each.
//!
//! Each home lays out the two temporary places as the fixture known folders do (`private/var/folders/xx/fixture/T`,
//! `private/tmp`); its apps are given to the module as fakes. The clock stands at `FIXTURE_NOW`, from which the homes'
//! ages and stated times count back.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{InstalledApp, RunningApp, StoredSettings};
use fresher_module_temp::TempModule;
use fresher_testing::{
    FIXTURE_NOW, FakeInstalledApps, FakeRunningApps, FixtureTree, MatchOrder, ModuleRun, Ports, installed_app,
    run_module, running_app,
};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn temp() -> TempModule {
    TempModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// `com.example.Viewer`, installed and not open.
pub(crate) fn viewer() -> InstalledApp {
    installed_app("com.example.Viewer", "Viewer", "1.0").unwrap()
}

/// `com.example.Editor`, open.
pub(crate) fn editor() -> RunningApp {
    running_app("Editor", &["com.example.Editor"]).unwrap()
}

/// The system a home is scanned with: the installed and the open apps, the stored settings (as JSON), whether the
/// scan is restored from the cache, and the order the matches come in.
pub(crate) struct System {
    pub(crate) installed: Vec<InstalledApp>,
    pub(crate) running: FakeRunningApps,
    pub(crate) stored: &'static str,
    pub(crate) restored: bool,
    pub(crate) order: MatchOrder,
}

impl System {
    /// `com.example.Viewer` installed, nothing open, every setting at its default, a walked scan.
    pub(crate) fn plain() -> Self {
        Self {
            installed: vec![viewer()],
            running: FakeRunningApps::default(),
            stored: "{}",
            restored: false,
            order: MatchOrder::Walked,
        }
    }

    pub(crate) fn running(self, running: FakeRunningApps) -> Self {
        Self { running, ..self }
    }

    pub(crate) fn stored(self, stored: &'static str) -> Self {
        Self { stored, ..self }
    }

    pub(crate) fn restored(self) -> Self {
        Self { restored: true, ..self }
    }
}

/// The home `name` scanned with `system`, the clock at `FIXTURE_NOW`.
pub(crate) fn scan(name: &str, system: System) -> (FixtureTree, ModuleRun) {
    let tree = home(name);
    let settings: StoredSettings = serde_json::from_str(system.stored).unwrap();
    let ports = Ports {
        settings,
        apps: Arc::new(FakeInstalledApps::new(system.installed)),
        running: Arc::new(system.running),
        restored: system.restored,
        order: system.order,
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&temp(), &tree, &ports).unwrap();
    (tree, run)
}

/// Everything changed recently.
pub(crate) fn recent() -> (FixtureTree, ModuleRun) {
    scan("temp-recent", System::plain())
}

/// The system's, another user's and a locked item.
pub(crate) fn system() -> (FixtureTree, ModuleRun) {
    scan("temp-system", System::plain())
}

/// Sockets, a pipe and locks.
pub(crate) fn special() -> (FixtureTree, ModuleRun) {
    scan("temp-special", System::plain())
}

/// The items of `com.example.Editor`, open.
pub(crate) fn running() -> (FixtureTree, ModuleRun) {
    scan("temp-running", System::plain().running(FakeRunningApps::new(vec![editor()])))
}

/// Five old items.
pub(crate) fn old() -> (FixtureTree, ModuleRun) {
    scan("temp-old", System::plain())
}
