//! The fixture homes of the stale-projects module's tests and one run of the module on each.
//!
//! Every folder of a project is a dated `[[dir]]` in its home; the walk reads the tree as it is, and the projects are
//! found as the engine finds them.

use std::path::{Path, PathBuf};

use fresher_core::StoredSettings;
use fresher_module_stale_projects::StaleProjectsModule;
use fresher_testing::{FIXTURE_NOW, FixtureTree, MatchOrder, ModuleRun, Ports, run_module};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn stale_projects() -> StaleProjectsModule {
    StaleProjectsModule::new().unwrap()
}

/// How a home is scanned: whether the scan is restored from the cache, the order the matches come in and the folder
/// the walk does not open.
pub(crate) struct System {
    restored: bool,
    order: MatchOrder,
    avoid: Vec<PathBuf>,
}

impl System {
    /// A walked scan of every folder.
    pub(crate) fn plain() -> Self {
        Self { restored: false, order: MatchOrder::Walked, avoid: Vec::new() }
    }

    /// The kept home's system: `Code/hidden-part/private` is not opened.
    pub(crate) fn kept() -> Self {
        Self::plain().avoiding("Code/hidden-part/private")
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

/// The home `name` scanned with `system`, the clock at `FIXTURE_NOW` and `settings` stored.
pub(crate) fn scan_with(name: &str, system: System, settings: StoredSettings) -> (FixtureTree, ModuleRun) {
    let tree = FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap();
    let ports = Ports {
        settings,
        restored: system.restored,
        order: system.order,
        avoid: system.avoid,
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&stale_projects(), &tree, &ports).unwrap();
    (tree, run)
}

/// The home `name` scanned with `system`.
pub(crate) fn scan(name: &str, system: System) -> (FixtureTree, ModuleRun) {
    scan_with(name, system, StoredSettings::default())
}

/// Names that are not the module's to claim.
pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    scan("stale-projects-not-claimed", System::plain())
}

/// One project per reason to keep it, scanned with `system`.
pub(crate) fn kept_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("stale-projects-kept", system)
}

/// The kept home with its own system.
pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    kept_with(System::kept())
}

/// Every category offered, scanned with `system`.
pub(crate) fn offered_with(system: System) -> (FixtureTree, ModuleRun) {
    scan("stale-projects-offered", system)
}

/// The offered home with its own system.
pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    offered_with(System::plain())
}
