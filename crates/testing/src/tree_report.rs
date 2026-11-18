use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A deterministic listing of a fixture tree: what a golden report compares against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeReport {
    pub entries: Vec<TreeEntry>,
    pub total_logical_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeEntry {
    /// The path relative to the root, with `/` separators.
    pub path: String,
    pub kind: EntryKind,
    /// The logical size of a file; 0 for folders, symlinks and other entries.
    pub logical_size: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    Dir,
    File,
    Symlink,
    /// A socket, a named pipe or a device.
    Other,
}

impl TreeReport {
    /// Lists every entry under `root`, sorted by path. Symlinks are listed, never followed.
    pub fn of(root: &Path) -> std::io::Result<Self> {
        let mut entries = Vec::new();
        collect(root, root, &mut entries)?;
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        let total_logical_size = entries.iter().map(|entry| entry.logical_size).sum();
        Ok(Self { entries, total_logical_size })
    }
}

fn collect(root: &Path, dir: &Path, entries: &mut Vec<TreeEntry>) -> std::io::Result<()> {
    for item in fs::read_dir(dir)? {
        let item = item?;
        let path = item.path();
        let metadata = fs::symlink_metadata(&path)?;
        let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        let kind = if metadata.is_symlink() {
            EntryKind::Symlink
        } else if metadata.is_dir() {
            EntryKind::Dir
        } else if metadata.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };
        let logical_size = if kind == EntryKind::File { metadata.len() } else { 0 };
        entries.push(TreeEntry { path: relative, kind, logical_size });
        if kind == EntryKind::Dir {
            collect(root, &path, entries)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{FixtureTree, TreeReport};

        #[test]
        fn does_not_follow_a_symlink_out_of_the_root() {
            let tree = FixtureTree::from_toml_str("[[symlink]]\npath = \"out\"\ntarget = \"/usr\"\n").unwrap();
            let report = TreeReport::of(tree.root()).unwrap();

            assert_eq!(report.entries.len(), 1);
            assert_eq!(report.total_logical_size, 0);
        }

        #[test]
        fn fails_on_a_missing_root() {
            assert!(TreeReport::of(std::path::Path::new("/nonexistent/root")).is_err());
        }
    }

    mod positive {
        use crate::{EntryKind, FixtureTree, TreeReport};

        #[test]
        fn lists_entries_sorted_with_their_sizes() {
            let tree = FixtureTree::from_toml_str(
                "[[file]]\npath = \"b/two.bin\"\nsize = 2\n\n[[file]]\npath = \"a.bin\"\nsize = 1\n",
            )
            .unwrap();
            let report = TreeReport::of(tree.root()).unwrap();
            let paths: Vec<&str> = report.entries.iter().map(|entry| entry.path.as_str()).collect();

            assert_eq!(paths, ["a.bin", "b", "b/two.bin"]);
            assert_eq!(report.entries[1].kind, EntryKind::Dir);
            assert_eq!(report.total_logical_size, 3);
        }
    }
}
