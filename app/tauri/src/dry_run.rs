//! The Trash of a development build outside a sandbox: it refuses every call and touches nothing. The dry run never
//! reaches it (the cleaner's `Reach::DryRun` stops every act); it is the second wall.

use std::fs::File;
use std::path::Path;

use fresher_core::{Landed, Trash, TrashError};

/// A Trash that refuses to move anything and to put anything back.
#[derive(Debug)]
pub(crate) struct DryRunTrash;

impl Trash for DryRunTrash {
    fn move_to_trash(&self, path: &Path, _held: &File) -> Result<Landed, TrashError> {
        tracing::error!(path = %path.display(), "a dry run reached the Trash");
        Err(TrashError::NotAllowed)
    }

    fn put_back(&self, landed: &Path, _original: &Path) -> Result<(), TrashError> {
        tracing::error!(path = %landed.display(), "a dry run reached the Trash");
        Err(TrashError::NotAllowed)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;
        use std::path::PathBuf;

        use fresher_core::{Trash, TrashError};

        use crate::dry_run::DryRunTrash;

        fn listing(folder: &std::path::Path) -> Vec<PathBuf> {
            let mut names: Vec<PathBuf> = fs::read_dir(folder).unwrap().map(|entry| entry.unwrap().path()).collect();
            names.sort();
            names
        }

        #[test]
        fn the_dry_run_trash_refuses_to_move_and_touches_nothing() {
            let folder = tempfile::tempdir().unwrap();
            let path = folder.path().join("blob.bin");
            fs::write(&path, b"kept bytes").unwrap();
            let held = fs::File::open(&path).unwrap();

            let moved = DryRunTrash.move_to_trash(&path, &held);

            assert_eq!(moved, Err(TrashError::NotAllowed));
            assert_eq!(fs::read(&path).unwrap(), b"kept bytes");
            assert_eq!(listing(folder.path()), [path]);
        }

        #[test]
        fn the_dry_run_trash_refuses_to_put_back() {
            let folder = tempfile::tempdir().unwrap();
            let landed = folder.path().join("landed.bin");
            let original = folder.path().join("original.bin");
            fs::write(&landed, b"in the trash").unwrap();

            let back = DryRunTrash.put_back(&landed, &original);

            assert_eq!(back, Err(TrashError::NotAllowed));
            assert_eq!(fs::read(&landed).unwrap(), b"in the trash");
            assert!(!original.exists());
            assert_eq!(listing(folder.path()), [landed]);
        }
    }
}
