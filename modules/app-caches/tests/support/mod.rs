//! What the app-caches module's tests share: the homes' loader, the system a home is scanned with, and one run of the
//! module on a home. The homes are in `plain.rs` and `tables.rs`, each used by the test files that need it. The installed and running apps are given to the module as fakes; facts the disk cannot be made to show (a
//! socket, the user's lock) are stated by the home.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{BundleId, InstalledApp};
use fresher_module_app_caches::AppCachesModule;
use fresher_testing::{
    FIXTURE_NOW, FakeInstalledApps, FakeRunningApps, FixtureTree, MatchOrder, ModuleRun, Ports, installed_app,
    run_module,
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
pub(crate) fn scan(name: &str, system: System) -> (FixtureTree, ModuleRun) {
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
