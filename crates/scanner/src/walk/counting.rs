//! `CountingReader`: a reader that records every folder it reads, for the deterministic "folders read" bars.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use fresher_core::{EntryAccess, FolderIdentity, FolderListing, FolderReader, ReadError};

/// Wraps a reader and records each `read`, in the order they were made. `identify`, `access` and `head` are not
/// recorded.
#[derive(Debug, Default)]
pub struct CountingReader<R> {
    inner: R,
    log: Mutex<Vec<PathBuf>>,
}

impl<R> CountingReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, log: Mutex::new(Vec::new()) }
    }

    /// Every folder read, in order; a folder read twice is there twice.
    pub fn log(&self) -> Vec<PathBuf> {
        self.log.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// How many times each folder was read.
    pub fn reads(&self) -> HashMap<PathBuf, usize> {
        let mut reads = HashMap::new();
        for path in self.log() {
            *reads.entry(path).or_insert(0) += 1;
        }
        reads
    }

    /// How many reads were made in all.
    pub fn total(&self) -> usize {
        self.log.lock().unwrap_or_else(PoisonError::into_inner).len()
    }
}

impl<R: FolderReader> FolderReader for CountingReader<R> {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        self.inner.identify(path)
    }

    fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
        self.inner.access(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        self.log.lock().unwrap_or_else(PoisonError::into_inner).push(path.to_path_buf());
        self.inner.read(path, expected, out)
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.inner.head(folder, name, limit)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::walk::counting::CountingReader;
        use crate::walk::std_reader::StdReader;

        #[test]
        fn a_new_reader_counted_nothing() {
            let reader = CountingReader::new(StdReader::new());

            assert_eq!((reader.total(), reader.log().len(), reader.reads().len()), (0, 0, 0));
        }
    }

    mod positive {
        use fresher_core::{FolderListing, FolderReader};
        use fresher_testing::FixtureTree;

        use crate::walk::counting::CountingReader;
        use crate::walk::std_reader::StdReader;

        #[test]
        fn every_read_is_counted_in_order_and_identify_and_head_are_not() {
            let tree = FixtureTree::from_toml_str("[[file]]\npath = \"a/f\"\n[[dir]]\npath = \"b\"\n").unwrap();
            let reader = CountingReader::new(StdReader::new());
            let (a, b) = (tree.root().join("a"), tree.root().join("b"));
            let mut listing = FolderListing::new();

            for path in [&a, &b, &a] {
                let identity = reader.identify(path).unwrap();
                reader.read(path, identity, &mut listing).unwrap();
            }
            reader.head(&a, "f", 4).unwrap();

            assert_eq!(reader.log(), [a.clone(), b.clone(), a.clone()]);
            assert_eq!((reader.reads()[&a], reader.reads()[&b], reader.total()), (2, 1, 3));
        }
    }
}
