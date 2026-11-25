//! The fixture homes of the unused-toolchains module's tests and one run of the module on each.
//!
//! Each home holds the files of rustup and nvm at their default places, the pins in the folders of projects, and the
//! facts the disk cannot be made to show (a socket, the user's lock) as stated. The Node switch is on unless a run
//! says otherwise.

use std::path::{Path, PathBuf};

use fresher_module_unused_toolchains::UnusedToolchainsModule;
use fresher_testing::{FIXTURE_NOW, FixtureTree, MatchOrder, ModuleRun, Ports, run_module};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

pub(crate) fn unused_toolchains() -> UnusedToolchainsModule {
    UnusedToolchainsModule::new().unwrap()
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

/// How a run differs from the default one: walked in order, the Node switch on.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Variant {
    pub(crate) node_versions: bool,
    pub(crate) order: MatchOrder,
}

impl Variant {
    pub(crate) const DEFAULT: Self = Self { node_versions: true, order: MatchOrder::Walked };
}

/// The home `name` scanned as `variant` says, the clock at `FIXTURE_NOW`.
pub(crate) fn scan_as(name: &str, variant: Variant) -> (FixtureTree, ModuleRun) {
    let tree = home(name);
    let mut ports = Ports::at(FIXTURE_NOW);
    ports.settings.values.insert("unused-toolchains.node-versions".into(), variant.node_versions.into());
    ports.order = variant.order;
    let run = run_module(&unused_toolchains(), &tree, &ports).unwrap();
    (tree, run)
}

/// The home `name` scanned with the Node switch on, walked.
pub(crate) fn scan(name: &str) -> (FixtureTree, ModuleRun) {
    scan_as(name, Variant::DEFAULT)
}

/// Versions every pointer keeps, links, moving channels and names off the shape.
pub(crate) fn not_claimed() -> (FixtureTree, ModuleRun) {
    scan("unused-toolchains-not-claimed")
}

/// Versions each blocked by a socket or the user's lock, the defaults known.
pub(crate) fn kept() -> (FixtureTree, ModuleRun) {
    scan("unused-toolchains-kept")
}

/// Versions kept because the default and a pin cannot be read.
pub(crate) fn kept_unknown() -> (FixtureTree, ModuleRun) {
    scan("unused-toolchains-kept-unknown")
}

/// Every unused version offered.
pub(crate) fn offered() -> (FixtureTree, ModuleRun) {
    scan("unused-toolchains-offered")
}

/// The offered home with the Node switch off.
pub(crate) fn switched_off() -> (FixtureTree, ModuleRun) {
    scan_as("unused-toolchains-offered", Variant { node_versions: false, ..Variant::DEFAULT })
}
