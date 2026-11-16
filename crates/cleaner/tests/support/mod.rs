//! What the source tests share: the workspace's Rust files.

#![cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every `.rs` file under the workspace's code folders, relative to `root`, with its text.
pub(crate) fn rust_files(root: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    for folder in ["crates", "modules", "app"] {
        collect(root, &root.join(folder), &mut files);
    }
    files
}

fn collect(root: &Path, folder: &Path, files: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(folder) else { return };
    for entry in entries.map(Result::unwrap) {
        let path = entry.path();
        let name = entry.file_name();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() && !matches!(name.to_str(), Some("target" | "node_modules" | "gen" | "dist")) {
            collect(root, &path, files);
        } else if kind.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
            let relative = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            files.push((relative, fs::read_to_string(&path).unwrap()));
        }
    }
}
