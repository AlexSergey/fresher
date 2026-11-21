//! The fixture homes of the local-ai-models module's tests, the system each is scanned with, and one run of the module
//! on each.
//!
//! Each home holds only files and links at the tools' default places under the home; the open apps are given to the
//! module as fakes, and the facts the disk cannot be made to show (a socket, the user's lock) are stated.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_module_local_ai_models::LocalAiModelsModule;
use fresher_testing::{
    FIXTURE_NOW, FakeRunningApps, FixtureTree, MatchOrder, ModuleRun, Ports, run_module, running_app,
};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn local_ai_models() -> LocalAiModelsModule {
    LocalAiModelsModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// LM Studio open.
pub(crate) fn lm_studio_open() -> FakeRunningApps {
    FakeRunningApps::new(vec![running_app("LM Studio", &["ai.elementlabs.lmstudio"]).unwrap()])
}

/// The home `name` scanned with `running` open, the clock at `FIXTURE_NOW`.
pub(crate) fn scan_with(
    name: &str,
    running: FakeRunningApps,
    restored: bool,
    order: MatchOrder,
) -> (FixtureTree, ModuleRun) {
    let tree = home(name);
    let ports = Ports { running: Arc::new(running), restored, order, ..Ports::at(FIXTURE_NOW) };
    let run = run_module(&local_ai_models(), &tree, &ports).unwrap();
    (tree, run)
}

/// The home `name` scanned with nothing open, walked.
pub(crate) fn scan(name: &str) -> (FixtureTree, ModuleRun) {
    scan_with(name, FakeRunningApps::default(), false, MatchOrder::Walked)
}

/// Folders that are no model of a store.
pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    scan("local-ai-models-not-claimed")
}

/// Models kept, scanned with `running` open.
pub(crate) fn kept_with(running: FakeRunningApps) -> (FixtureTree, ModuleRun) {
    scan_with("local-ai-models-kept", running, false, MatchOrder::Walked)
}

/// The kept models with LM Studio open.
pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    kept_with(lm_studio_open())
}

/// Every model offered.
pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    scan("local-ai-models-offered")
}
