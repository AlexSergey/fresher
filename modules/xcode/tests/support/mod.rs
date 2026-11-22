//! The fixture homes of the xcode module's tests and one run of the module on each. The running apps are given to the
//! module as fakes; facts the disk cannot be made to show (a socket, the user's lock) are stated by the home.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_module_xcode::XcodeModule;
use fresher_testing::{FIXTURE_NOW, FakeRunningApps, FixtureTree, ModuleRun, Ports, run_module, running_app};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn xcode() -> XcodeModule {
    XcodeModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml` scanned with `running` as the running apps, the clock at `FIXTURE_NOW`.
fn scan(name: &str, running: FakeRunningApps) -> (FixtureTree, ModuleRun) {
    let tree = FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap();
    let ports = Ports { running: Arc::new(running), ..Ports::at(FIXTURE_NOW) };
    let run = run_module(&xcode(), &tree, &ports).unwrap();
    (tree, run)
}

/// No app running.
fn nothing_running() -> FakeRunningApps {
    FakeRunningApps::default()
}

/// Names, shapes and folders nothing may claim.
pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    scan("xcode-not-claimed", nothing_running())
}

/// One folder per reason to keep it, with `running` as the running apps.
pub(crate) fn kept_with(running: FakeRunningApps) -> (FixtureTree, ModuleRun) {
    scan("xcode-kept", running)
}

/// The kept home with Xcode running.
pub(crate) fn kept_xcode_open() -> (FixtureTree, ModuleRun) {
    kept_with(FakeRunningApps::new(vec![running_app("Xcode", &["com.apple.dt.Xcode"]).unwrap()]))
}

/// The kept home with only the Simulator running.
pub(crate) fn kept_simulator_open() -> (FixtureTree, ModuleRun) {
    kept_with(FakeRunningApps::new(vec![running_app("Simulator", &["com.apple.iphonesimulator"]).unwrap()]))
}

/// The kept home with the running apps failing.
pub(crate) fn kept_no_system() -> (FixtureTree, ModuleRun) {
    kept_with(FakeRunningApps::failing("no process list"))
}

/// Every kind, nothing running.
pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    scan("xcode-offered", nothing_running())
}
