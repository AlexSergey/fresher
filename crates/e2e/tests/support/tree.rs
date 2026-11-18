//! A file tree's contents, compared before and after a cleanup and its put back. Pulled in with `#[path]` only by the
//! tests that use it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Every file at or under `at` with its bytes, by path.
pub(crate) fn contents(at: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![at.to_path_buf()];
    while let Some(next) = pending.pop() {
        if !next.is_dir() {
            files.insert(next.clone(), fs::read(&next).unwrap());
            continue;
        }
        for entry in fs::read_dir(&next).unwrap() {
            pending.push(entry.unwrap().path());
        }
    }
    files
}
