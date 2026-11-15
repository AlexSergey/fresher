//! Test helpers of Mac Fresher: fixture trees, fake clock, fake Trash, fake external commands.
//!
//! A test never touches the real home folder: it builds a [`FixtureTree`] in a temporary folder and gives the code
//! under test that folder as its only root (docs/testing.md).

mod fakes;
mod fixture_tree;
mod tree_report;

pub use fakes::{FakeClock, FakeCommandRunner, FakeTrash};
pub use fixture_tree::{FixtureError, FixtureTree};
pub use tree_report::{EntryKind, TreeEntry, TreeReport};
