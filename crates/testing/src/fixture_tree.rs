use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Deserialize;
use tempfile::TempDir;

/// A fake home folder built from a TOML description in a temporary folder, removed when dropped.
///
/// Files get their declared logical size as sparse files, so a "10 GB cache" costs no disk. Modification times come
/// from the description, never from the wall clock.
#[derive(Debug)]
pub struct FixtureTree {
    root: TempDir,
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
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Description {
    #[serde(default, rename = "dir")]
    dirs: Vec<DirSpec>,
    #[serde(default, rename = "file")]
    files: Vec<FileSpec>,
    #[serde(default, rename = "symlink")]
    symlinks: Vec<SymlinkSpec>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirSpec {
    path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSpec {
    path: PathBuf,
    #[serde(default)]
    size: u64,
    /// Seconds since the Unix epoch.
    modified: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SymlinkSpec {
    path: PathBuf,
    target: PathBuf,
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
        let description: Description = toml::from_str(text)
            .map_err(|source| FixtureError::Parse { path: PathBuf::from("<inline>"), source: Box::new(source) })?;
        let root = TempDir::new().map_err(|source| FixtureError::Create { path: PathBuf::from("<temp>"), source })?;
        let tree = Self { root };
        tree.build(&description)?;
        Ok(tree)
    }

    /// The root of the tree: the only folder a test may scan or clean.
    pub fn root(&self) -> &Path {
        self.root.path()
    }

    fn build(&self, description: &Description) -> Result<(), FixtureError> {
        for dir in &description.dirs {
            let path = self.resolve(&dir.path)?;
            fs::create_dir_all(&path).map_err(|source| FixtureError::Create { path, source })?;
        }
        for file in &description.files {
            self.create_file(file)?;
        }
        for link in &description.symlinks {
            let path = self.resolve(&link.path)?;
            create_parent(&path)?;
            symlink(&link.target, &path).map_err(|source| FixtureError::Create { path, source })?;
        }
        Ok(())
    }

    fn create_file(&self, spec: &FileSpec) -> Result<(), FixtureError> {
        let path = self.resolve(&spec.path)?;
        create_parent(&path)?;
        let create_error = |source| FixtureError::Create { path: path.clone(), source };
        let file = File::create(&path).map_err(create_error)?;
        file.set_len(spec.size).map_err(create_error)?;
        if let Some(seconds) = spec.modified {
            file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)).map_err(create_error)?;
        }
        Ok(())
    }

    fn resolve(&self, relative: &Path) -> Result<PathBuf, FixtureError> {
        let inside = relative.components().all(|component| matches!(component, Component::Normal(_)));
        if !inside || relative.as_os_str().is_empty() {
            return Err(FixtureError::PathOutsideRoot { path: relative.to_path_buf() });
        }
        Ok(self.root().join(relative))
    }
}

fn create_parent(path: &Path) -> Result<(), FixtureError> {
    match path.parent() {
        Some(parent) => {
            fs::create_dir_all(parent).map_err(|source| FixtureError::Create { path: parent.into(), source })
        }
        None => Ok(()),
    }
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
        fn removes_the_tree_when_dropped() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a.txt\"\n").unwrap();
            let root = tree.root().to_path_buf();
            drop(tree);

            assert!(!root.exists());
        }
    }
}
