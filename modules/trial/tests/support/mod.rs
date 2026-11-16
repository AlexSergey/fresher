//! The fixture homes of the trial module's tests and one run of the module on each.

use std::path::{Path, PathBuf};

use fresher_core::CommandRunError;
use fresher_module_trial::TrialModule;
use fresher_testing::{FakeCommandRunner, FixtureTree, ModuleRun, Ports, run_module};

/// The command that lists the made-up tool's objects.
pub(crate) const LIST: &str = "fresher-trial-tool list";

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn trial() -> TrialModule {
    TrialModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`, with the sandbox marker written into each folder of `sandboxes`.
pub(crate) fn home(name: &str, sandboxes: &[&str]) -> FixtureTree {
    let tree = FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap();
    for folder in sandboxes {
        tree.mark_sandbox(Path::new(folder)).unwrap();
    }
    tree
}

/// The home where nothing may be claimed; the made-up tool is not installed.
pub(crate) fn unclaimed() -> (FixtureTree, ModuleRun) {
    let tree = home("trial-unclaimed", &["look-alike", "file", "link", "marked-above"]);
    let run = run_module(&trial(), &tree, &Ports::at(0)).unwrap();
    (tree, run)
}

/// The home with a sandbox of trial folders; the made-up tool lists one object.
pub(crate) fn sandbox() -> (FixtureTree, ModuleRun) {
    let tree = home("trial", &["sandbox"]);
    let ports =
        Ports { commands: FakeCommandRunner::default().answering(LIST, "trial-object 1048576\n"), ..Ports::at(0) };
    let run = run_module(&trial(), &tree, &ports).unwrap();
    (tree, run)
}

/// The sandbox home with a tool that cannot list its objects.
pub(crate) fn failed_listing() -> ModuleRun {
    let tree = home("trial", &["sandbox"]);
    let refused = CommandRunError::Failed { program: "fresher-trial-tool".into(), reason: "refused".into() };
    let ports = Ports { commands: FakeCommandRunner::default().failing(LIST, refused), ..Ports::at(0) };
    run_module(&trial(), &tree, &ports).unwrap()
}
