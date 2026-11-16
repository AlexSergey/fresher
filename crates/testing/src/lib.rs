//! Test helpers of Fresher: fixture trees, fake clock, fake Trash, fake external commands, fake volumes and change
//! journal, an in-memory scan cache, a fixed disk state, the reference walk, and the module harness and contract
//! checks.
//!
//! A test never touches the real home folder: it builds a [`FixtureTree`] in a temporary folder and gives the code
//! under test that folder as its only root (docs/testing.md). A module's tests run it with [`run_module`] and check it
//! with [`assert_module_contract`] and [`assert_texts_used`].

#![forbid(unsafe_code)]

mod contract;
mod fakes;
mod fixed_disk_state;
mod fixture_tree;
mod large_tree;
mod naive_walk;
mod run_module;
mod tree_report;

pub use contract::{
    Catalogue, ContractProblem, assert_module_contract, assert_texts_used, check_module_contract, check_texts_used,
};
pub use fakes::{FakeChangeJournal, FakeClock, FakeCommandRunner, FakeTrash, FakeVolumes, MemoryScanCache};
pub use fixed_disk_state::FixedDiskState;
pub use fixture_tree::{FixtureError, FixtureTree};
pub use large_tree::{Expected, LargeTree, TreeSpec};
pub use naive_walk::NaiveWalk;
pub use run_module::{HarnessError, ModuleRun, Ports, run_module};
pub use tree_report::{EntryKind, TreeEntry, TreeReport};
