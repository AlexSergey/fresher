use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE};
use tempfile::TempDir;

use crate::facts_lens::StatedFacts;
use crate::tree_description::{FIXTURE_NOW, TreeDescription, resolve};

/// A fake home folder built from a TOML description in a temporary folder, removed when dropped.
///
/// Files get their declared logical size as sparse files, so a "10 GB cache" costs no disk. Modification times come
/// from the description, never from the wall clock. The facts it states instead of making them (an old status change,
/// an owner, a flag, a socket) are kept for [`crate::FactsLens`] and the reference walk, at the root as given and at
/// its canonical path.
#[derive(Debug)]
pub struct FixtureTree {
    root: TempDir,
    stated: StatedFacts,
}

#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    #[error("cannot read the fixture {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("the fixture {path} is not valid: {source}")]
    Parse { path: PathBuf, source: Box<toml::de::Error> },
    #[error("the fixture path {path} must be relative and stay inside the root")]
    PathOutsideRoot { path: PathBuf },
    #[error("cannot create {path} in the fixture tree: {source}")]
    Create { path: PathBuf, source: std::io::Error },
    #[error("the tree spec is not valid: {reason}")]
    InvalidSpec { reason: &'static str },
    #[error("the fixture path {path} is written twice")]
    DuplicatePath { path: PathBuf },
    #[error("the file {path} has both `age` and `modified`")]
    AgeWithModified { path: PathBuf },
    #[error("a time of {path} is not \"<n>d\", \"<n>h\" or \"<n>s\" (\"-\" before it: after now) within the clock")]
    InvalidAge { path: PathBuf },
    #[error("the file {path} is filled over the cap of 64 MiB a file or 512 MiB a tree")]
    FillOverCap { path: PathBuf },
    #[error("the fixture path {path} passes through a link")]
    PathThroughLink { path: PathBuf },
    #[error("the app {path} must end in `.app`")]
    AppNotABundle { path: PathBuf },
    #[error("the app {path} has `{id}`, which is not a bundle id")]
    InvalidBundleId { path: PathBuf, id: String },
    #[error("the app {path} has a control character in its name, version or build")]
    AppTextControl { path: PathBuf },
    #[error("the app {path} has a `last-used` that is not \"<n>d\", \"<n>h\" or \"<n>s\"")]
    AppLastUsed { path: PathBuf },
    #[error("the app {path} has `installed-by = {value:?}`; it is \"homebrew:<token>\", the token of [a-z0-9@._+-]")]
    AppInstalledBy { path: PathBuf, value: String },
    #[error("the fixture path {path} is inside the app {app}, which writes its own bundle")]
    InsideApp { path: PathBuf, app: PathBuf },
    #[error("the file {path} states a `kind` and has a `fill`: a socket or a pipe holds no data")]
    KindWithFill { path: PathBuf },
    #[error("the file {path} is a cache directory tag and has a `size`: the tag's content sets it")]
    TagWithSize { path: PathBuf },
    #[error("the file {path} has `written` bytes and a `fill` other than sparse: only a sparse file has a head")]
    WrittenWithFill { path: PathBuf },
    #[error("the file {path} has more `written` bytes than its `size`")]
    WrittenOverSize { path: PathBuf },
    #[error("the file {path} has a `text` and a `{field}`: the text sets the size and the content")]
    TextWith { path: PathBuf, field: &'static str },
    #[error("the file {path} has a `text` of more than 4096 bytes: a small file is read whole")]
    TextTooLong { path: PathBuf },
}

impl FixtureTree {
    /// Builds the tree described by a TOML file (the format: docs/testing.md, "Fixture homes").
    pub fn from_toml(path: &Path) -> Result<Self, FixtureError> {
        let text =
            fs::read_to_string(path).map_err(|source| FixtureError::Read { path: path.to_path_buf(), source })?;
        Self::from_toml_str(&text).map_err(|error| match error {
            FixtureError::Parse { source, .. } => FixtureError::Parse { path: path.to_path_buf(), source },
            other => other,
        })
    }

    /// Builds the tree described by a TOML string.
    pub fn from_toml_str(text: &str) -> Result<Self, FixtureError> {
        let description = TreeDescription::from_toml_str(text)?;
        let root = TempDir::new().map_err(|source| FixtureError::Create { path: PathBuf::from("<temp>"), source })?;
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW);
        description.build_at(root.path(), now)?;
        let stated = description.stated(now)?;
        let canonical = fs::canonicalize(root.path())
            .map_err(|source| FixtureError::Create { path: root.path().to_path_buf(), source })?;
        let entries = || stated.iter().map(|(path, facts)| (path.as_path(), facts));
        let stated = StatedFacts::under(root.path(), entries()).joined(StatedFacts::under(&canonical, entries()));
        Ok(Self { root, stated })
    }

    /// The tree with the sandbox marker at its root: the only kind of tree a development build or a test of the
    /// cleaner may remove anything from. A tree without it is safe by default.
    pub fn with_sandbox(self) -> Result<Self, FixtureError> {
        write_marker(self.root())?;
        Ok(self)
    }

    /// Writes the sandbox marker into `folder`, a folder of the tree given relative to its root.
    pub fn mark_sandbox(&self, folder: &Path) -> Result<(), FixtureError> {
        let folder = resolve(self.root(), folder)?;
        fs::create_dir_all(&folder).map_err(|source| FixtureError::Create { path: folder.clone(), source })?;
        write_marker(&folder)
    }

    /// The root of the tree: the only folder a test may scan or clean.
    pub fn root(&self) -> &Path {
        self.root.path()
    }

    /// The facts the tree states instead of making them; empty when it states none.
    pub const fn stated(&self) -> &StatedFacts {
        &self.stated
    }
}

/// Writes the sandbox marker, its signature as its first line, into `folder`.
pub(crate) fn write_marker(folder: &Path) -> Result<(), FixtureError> {
    let marker = folder.join(SANDBOX_MARKER);
    fs::write(&marker, format!("{SANDBOX_SIGNATURE}\n")).map_err(|source| FixtureError::Create { path: marker, source })
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{FixtureError, FixtureTree};

        #[test]
        fn rejects_a_path_that_leaves_the_root() {
            let error = FixtureTree::from_toml_str("[[file]]\npath = \"../escape\"\n").unwrap_err();

            assert!(matches!(error, FixtureError::PathOutsideRoot { .. }));
        }

        #[test]
        fn rejects_an_absolute_path() {
            let error = FixtureTree::from_toml_str("[[dir]]\npath = \"/Users\"\n").unwrap_err();

            assert!(matches!(error, FixtureError::PathOutsideRoot { .. }));
        }

        #[test]
        fn rejects_an_unknown_field() {
            let error = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\ncolour = \"red\"\n").unwrap_err();

            assert!(matches!(error, FixtureError::Parse { .. }));
        }

        #[test]
        fn rejects_a_hard_link_to_a_file_outside_the_root() {
            let error =
                FixtureTree::from_toml_str("[[hardlink]]\npath = \"a\"\ntarget = \"/etc/hosts\"\n").unwrap_err();

            assert!(matches!(error, FixtureError::PathOutsideRoot { .. }));
        }

        #[test]
        fn a_fixture_tree_without_sandbox_has_no_marker() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a\"\n").unwrap();

            assert!(!tree.root().join(fresher_core::SANDBOX_MARKER).exists());
            assert!(matches!(
                tree.mark_sandbox(std::path::Path::new("../out")),
                Err(FixtureError::PathOutsideRoot { .. })
            ));
        }

        #[test]
        fn reports_a_missing_fixture_file() {
            let error = FixtureTree::from_toml(std::path::Path::new("/nonexistent/fixture.toml")).unwrap_err();

            assert!(matches!(error, FixtureError::Read { .. }));
        }
    }

    mod positive {
        use std::os::unix::fs::MetadataExt;
        use std::time::{Duration, SystemTime};

        use crate::FixtureTree;

        #[test]
        fn a_fixture_tree_holds_the_sandbox_marker() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"cache\"\n").unwrap().with_sandbox().unwrap();
            tree.mark_sandbox(std::path::Path::new("cache/inner")).unwrap();

            for folder in [tree.root().to_path_buf(), tree.root().join("cache/inner")] {
                let marker = std::fs::read_to_string(folder.join(fresher_core::SANDBOX_MARKER)).unwrap();
                assert_eq!(marker.lines().next(), Some(fresher_core::SANDBOX_SIGNATURE));
            }
        }

        #[test]
        fn creates_a_large_file_that_costs_no_disk() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"Library/Caches/big.bin\"\nsize = 10737418240\n")
                .unwrap();
            let metadata = std::fs::metadata(tree.root().join("Library/Caches/big.bin")).unwrap();

            assert_eq!(metadata.len(), 10_737_418_240);
            assert!(metadata.blocks() * 512 < 1024 * 1024, "the file must be sparse");
        }

        #[test]
        fn sets_the_modification_time_from_the_description() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"old.txt\"\nmodified = 1700000000\n").unwrap();
            let modified = std::fs::metadata(tree.root().join("old.txt")).unwrap().modified().unwrap();

            assert_eq!(modified, SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000));
        }

        #[test]
        fn creates_folders_and_symlinks() {
            let tree = FixtureTree::from_toml_str(
                "[[dir]]\npath = \"Projects/empty\"\n\n[[symlink]]\npath = \"Projects/out\"\ntarget = \"/tmp\"\n",
            )
            .unwrap();

            assert!(tree.root().join("Projects/empty").is_dir());
            assert!(tree.root().join("Projects/out").symlink_metadata().unwrap().is_symlink());
        }

        #[test]
        fn creates_hard_links() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"a\"\nsize = 3\n[[hardlink]]\npath = \"b/a\"\ntarget = \"a\"\n",
            )
            .unwrap();

            let (a, b) = (
                std::fs::metadata(tree.root().join("a")).unwrap(),
                std::fs::metadata(tree.root().join("b/a")).unwrap(),
            );
            assert_eq!((a.ino(), a.nlink()), (b.ino(), 2));
        }

        #[test]
        fn removes_the_tree_when_dropped() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a.txt\"\n").unwrap();
            let root = tree.root().to_path_buf();
            drop(tree);

            assert!(!root.exists());
        }
    }
}
