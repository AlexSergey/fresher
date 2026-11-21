//! Test helpers of Fresher: fixture trees and app bundles, fake known folders, installed and running apps, fake clock,
//! fake Trash, fake external commands, fake volumes and change journal, an in-memory scan cache, fake open files, an
//! in-memory cleanup journal, a fake cleaner, a fixed disk state, fake access and system interface, the reference
//! walk, named pipes, a folder swapped for a link in a race, the facts a tree states and the lens that reports them
//! over a real reader, the small-file readers, and the module harness and contract checks.
//!
//! A test never touches the real home folder: it builds a [`FixtureTree`] in a temporary folder and gives the code
//! under test that folder as its only root (docs/testing.md). A module's tests run it with [`run_module`] and check it
//! with [`assert_module_contract`] and [`assert_texts_used`].

#![forbid(unsafe_code)]

mod access_time;
mod app_bundle;
mod cleanup_fakes;
mod contract;
mod entry_facts;
mod facts_lens;
mod fake_access;
mod fakes;
mod fixed_disk_state;
mod fixture_tree;
mod known_folders;
mod large_tree;
mod naive_places;
mod naive_walk;
mod run_module;
mod small_files;
mod special_files;
mod stated_apps;
mod swapper;
mod system_fakes;
mod tree_description;
mod tree_report;

pub use access_time::settle_access_time;
pub use app_bundle::AppBundle;
pub use cleanup_fakes::{ExecutedPlan, FakeCleaner, FakeOpenFiles, MemoryCleanupJournal};
pub use contract::{
    Catalogue, ContractProblem, assert_module_contract, assert_texts_used, check_module_contract,
    check_removals_matched, check_texts_used,
};
pub use facts_lens::{FactsLens, OTHER_USER, Stated, StatedFacts, StatedFlag, StatedKind, StatedOwner};
pub use fake_access::{FakeAccess, FakeSystemUi, SystemUiCall};
pub use fakes::{FakeChangeJournal, FakeClock, FakeCommandRunner, FakeTrash, FakeVolumes, MemoryScanCache};
pub use fixed_disk_state::FixedDiskState;
pub use fixture_tree::{FixtureError, FixtureTree};
pub use known_folders::FakeKnownFolders;
pub use large_tree::{Expected, LargeTree, TreeSpec};
pub use naive_walk::NaiveWalk;
pub use run_module::{HarnessError, MatchOrder, ModuleRun, Ports, run_module};
pub use small_files::{DiskFiles, NoFiles};
pub use special_files::make_pipe;
pub use stated_apps::StatedInstalledApps;
pub use swapper::Swapper;
pub use system_fakes::{
    FakeInstallRecords, FakeInstalledApps, FakeRunningApps, changed_at, fake_system, installed_app, running_app,
    signed_by,
};
pub use tree_description::{FIXTURE_NOW, TreeDescription, time_at_age};
pub use tree_report::{EntryKind, TreeEntry, TreeReport};
