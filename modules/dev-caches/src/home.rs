//! The home the unit tests use, at `/home`, and where each row's folder is in it.

use std::path::{Path, PathBuf};

use fresher_core::KnownFolders as _;
use fresher_testing::FakeKnownFolders;

use crate::tools::Cache;

/// 28 lowercase letters: the hash of a derived data project's name.
pub(crate) const HASH: &str = "abcdefghijklmnopqrstuvwxyzab";

/// The folder of `row` in the home, a derived data project named `Demo-<hash>`.
pub(crate) fn place(row: &Cache) -> PathBuf {
    let base = FakeKnownFolders::home_layout(Path::new("/home")).locate(row.folder).unwrap();
    base.join(row.path.replace('*', &format!("Demo-{HASH}")))
}
