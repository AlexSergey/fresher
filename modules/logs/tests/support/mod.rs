//! The fixture homes of the logs module's tests, the system each is scanned with, and one run of the module on each.
//!
//! Each home lays out `Library/Logs` as the fixture known folders do. The clock stands at `FIXTURE_NOW`, from which the
//! homes' ages and stated times count back. The module asks for no installed or running apps.

use std::path::{Path, PathBuf};

use fresher_core::{NewFinding, StoredSettings, Subject};
use fresher_module_logs::LogsModule;
use fresher_testing::{FIXTURE_NOW, FixtureTree, MatchOrder, ModuleRun, Ports, run_module};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn logs() -> LogsModule {
    LogsModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// The system a home is scanned with: the stored settings (as JSON), whether the scan is restored from the cache, and
/// the order the matches come in.
#[derive(Clone, Copy)]
pub(crate) struct System {
    pub(crate) stored: &'static str,
    pub(crate) restored: bool,
    pub(crate) order: MatchOrder,
}

impl System {
    /// Every setting at its default, a walked scan.
    pub(crate) fn plain() -> Self {
        Self { stored: "{}", restored: false, order: MatchOrder::Walked }
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
    let ports = Ports { settings, restored: system.restored, order: system.order, ..Ports::at(FIXTURE_NOW) };
    let run = run_module(&logs(), &tree, &ports).unwrap();
    (tree, run)
}

/// Nothing is the module's to claim.
pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    scan("logs-not-claimed", System::plain())
}

/// Every file is kept, each with its reason.
pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    scan("logs-kept", System::plain())
}

/// Eight old files offered, one kept.
pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    scan("logs-offered", System::plain())
}

/// The path a finding is about; every finding of this module has one.
pub(crate) fn subject_path(finding: &NewFinding) -> &Path {
    match finding.subject() {
        Subject::Path(path) => Some(path.as_path()),
        Subject::External { .. } => None,
    }
    .unwrap()
}
