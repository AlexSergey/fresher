//! A module's flow home: a fixture sandbox (its marker at the top, the home `home/`, the rest of a sandbox's layout)
//! and the engine on it. Pulled in with `#[path]`, beside `home_engine.rs`, only by the homes that use it; a home
//! gives the parts of its engine (`Parts`) and keeps what is its module's own.

use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_engine::Engine;
use fresher_sandbox::Sandbox;
use fresher_testing::{FakeOpenFiles, FixtureTree, StatedFacts};

use super::home_engine::{Parts, engine};

/// The folders of a sandbox's layout besides what the fixture holds (`crates/sandbox/src/layout.rs`, checked by
/// `Sandbox::open`).
const LAYOUT: [&str; 9] = [
    "home/.Trash",
    "Applications",
    "System/Applications",
    ".fresher-data",
    "private/var/folders/xx/sandbox/C",
    "private/var/folders/xx/sandbox/0",
    "private/var/folders/xx/sandbox/T",
    "private/var/folders/xx/sandbox/X",
    "private/tmp",
];

/// The fixture sandbox with its engine. `fixture` is the canonical sandbox folder; the files held open are the
/// cleaner's, the same through every restart.
pub(crate) struct Flow {
    _tree: FixtureTree,
    pub(crate) fixture: PathBuf,
    pub(crate) sandbox: Sandbox,
    pub(crate) stated: StatedFacts,
    pub(crate) open_files: Arc<FakeOpenFiles>,
    pub(crate) engine: Engine,
}

impl Flow {
    /// `tree` laid out as a sandbox, its engine of the parts `parts` gives for it.
    pub(crate) fn open(tree: FixtureTree, parts: impl FnOnce(&Sandbox) -> Parts) -> Self {
        for folder in LAYOUT {
            fs::create_dir_all(tree.root().join(folder)).unwrap();
        }
        let tree = tree.with_sandbox().unwrap();
        let fixture = fs::canonicalize(tree.root()).unwrap();
        let sandbox = Sandbox::open(&fixture, user(&fixture)).unwrap();
        let stated = tree.stated().clone();
        let open_files = Arc::new(FakeOpenFiles::default());
        let engine = engine(&sandbox, &stated, &open_files, parts(&sandbox));
        Self { _tree: tree, fixture, sandbox, stated, open_files, engine }
    }

    /// A new engine of `parts` on the same files: the app started again.
    pub(crate) fn restart_with(&mut self, parts: Parts) {
        self.engine = engine(&self.sandbox, &self.stated, &self.open_files, parts);
    }

    /// The path of `relative` in the sandbox.
    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.fixture.join(relative)
    }

    /// `path` relative to the sandbox; a path outside it as it is.
    pub(crate) fn relative_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.fixture).unwrap_or(path).display().to_string()
    }
}

/// The owner of `folder`: the user the sandbox and its cleaner run as.
pub(crate) fn user(folder: &Path) -> u32 {
    fs::metadata(folder).unwrap().uid()
}
