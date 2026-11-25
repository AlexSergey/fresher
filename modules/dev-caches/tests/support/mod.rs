//! The fixture homes of the dev-caches module's tests, the system each is scanned with, and one run of the module on
//! each.
//!
//! Each home holds only files, every place at its default spot under the home (`Library/Caches/…`,
//! `Library/Developer/Xcode/DerivedData/…`, the dot folders); the open apps are given to the module as fakes. Facts the
//! disk cannot be made to show (a socket, the system's owner, the user's lock) are stated, and the run reads them
//! through the facts lens.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_module_dev_caches::DevCachesModule;
use fresher_testing::{
    FIXTURE_NOW, FakeRunningApps, FixtureTree, MatchOrder, ModuleRun, Ports, run_module, running_app,
};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn dev_caches() -> DevCachesModule {
    DevCachesModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// Xcode open.
pub(crate) fn xcode_open() -> FakeRunningApps {
    FakeRunningApps::new(vec![running_app("Xcode", &["com.apple.dt.Xcode"]).unwrap()])
}

/// The system a home is scanned with: the open apps and the order the matches come in.
pub(crate) struct System {
    pub(crate) running: FakeRunningApps,
    pub(crate) order: MatchOrder,
}

impl System {
    /// Nothing open, the matches in the walk's order.
    pub(crate) fn plain() -> Self {
        Self { running: FakeRunningApps::default(), order: MatchOrder::Walked }
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
    let ports = Ports { running: Arc::new(system.running), order: system.order, ..Ports::at(FIXTURE_NOW) };
    let run = run_module(&dev_caches(), &tree, &ports).unwrap();
    (tree, run)
}

/// Folders named like a tool's cache, and a tool's other folders and files beside it.
pub(crate) fn similar_names() -> (FixtureTree, ModuleRun) {
    scan("dev-caches-similar-names", System::plain())
}

/// Every place without its sign.
pub(crate) fn unsigned() -> (FixtureTree, ModuleRun) {
    scan("dev-caches-unsigned", System::plain())
}

/// A file and links where a cache folder should be.
pub(crate) fn not_folders() -> (FixtureTree, ModuleRun) {
    scan("dev-caches-not-folders", System::plain())
}

/// Caches holding a socket, the system's file or the user's lock, and a derived data project, scanned with `running`.
pub(crate) fn kept_with(running: FakeRunningApps) -> (FixtureTree, ModuleRun) {
    scan("dev-caches-kept", System::plain().running(running))
}

/// The kept caches with Xcode open.
pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    kept_with(xcode_open())
}

/// Every place with its cache and its sign, Xcode closed.
pub(crate) fn caches() -> (FixtureTree, ModuleRun) {
    scan("dev-caches-caches", System::plain())
}
