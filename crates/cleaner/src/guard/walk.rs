//! Check 11's walk: what a folder holds now, measured beneath its held handle — each folder opened below the one
//! above it through no link and checked against its listing, no link followed (a link counts as itself), a hard link
//! counted once, and never past its root's volume.

use std::collections::HashSet;
use std::fmt;
use std::fs::{File, Metadata};
use std::os::unix::fs::MetadataExt as _;

use fresher_core::{CancelToken, EntryKind, SkipReason};

use crate::beneath::{Listed, descend, entries};

/// Why a walk stopped before it measured everything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WalkStop {
    /// The item cannot be confirmed: another volume inside, or something that could not be read.
    Skip(SkipReason),
    /// The cleanup was cancelled between two folders.
    Cancelled,
}

impl From<SkipReason> for WalkStop {
    fn from(reason: SkipReason) -> Self {
        Self::Skip(reason)
    }
}

/// Measures what an item allocates on the disk now.
pub(crate) trait SizeWalk: fmt::Debug + Send + Sync {
    /// The allocated bytes of the item held open as `item`, whose metadata is `metadata`, and of everything inside
    /// it; an entry not on `device` stops the walk.
    fn measure(&self, item: &File, metadata: &Metadata, device: u64, cancel: &CancelToken) -> Result<u64, WalkStop>;
}

/// The walk of the app: depth first beneath the item's handle, one open folder per level, with the bulk read.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HeldWalk;

impl SizeWalk for HeldWalk {
    fn measure(&self, item: &File, metadata: &Metadata, device: u64, cancel: &CancelToken) -> Result<u64, WalkStop> {
        let mut seen = HashSet::new();
        let mut total = blocks(metadata);
        if !metadata.is_dir() {
            return Ok(total);
        }
        let root = item.try_clone().map_err(|_| SkipReason::CheckFailed)?;
        let pending = read(&root, device, cancel, &mut total, &mut seen)?;
        let mut levels = vec![(root, pending)];
        while let Some((folder, pending)) = levels.last_mut() {
            let Some(next) = pending.next() else {
                levels.pop();
                continue;
            };
            let identity = next.identity.ok_or(SkipReason::CheckFailed)?;
            let child = descend(folder, &next.name, identity, device)?;
            total = total.saturating_add(blocks(&child.metadata().map_err(|_| SkipReason::CheckFailed)?));
            let pending = read(&child, device, cancel, &mut total, &mut seen)?;
            levels.push((child, pending));
        }
        Ok(total)
    }
}

/// Reads the held folder `folder`, adds what its files and links allocate to `total`, and gives its folders, still to
/// be read.
fn read(
    folder: &File,
    device: u64,
    cancel: &CancelToken,
    total: &mut u64,
    seen: &mut HashSet<(u64, u64)>,
) -> Result<std::vec::IntoIter<Listed>, WalkStop> {
    if cancel.is_cancelled() {
        return Err(WalkStop::Cancelled);
    }
    let mut folders = Vec::new();
    for entry in entries(folder)? {
        let identity = entry.identity.ok_or(SkipReason::CheckFailed)?;
        if identity.device() != device {
            return Err(SkipReason::OtherVolume.into());
        }
        if identity.kind() == EntryKind::Folder {
            folders.push(entry);
        } else if entry.links <= 1 || seen.insert((identity.device(), identity.file())) {
            *total = total.saturating_add(entry.allocated);
        }
    }
    Ok(folders.into_iter())
}

/// The bytes the item itself allocates, read from its handle.
fn blocks(metadata: &Metadata) -> u64 {
    metadata.blocks().saturating_mul(512)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::Path;

    use fresher_core::CancelToken;
    use fresher_testing::FixtureTree;

    use super::{HeldWalk, SizeWalk as _, WalkStop};
    use crate::paths::{LastLink, open_no_link};

    fn tree() -> FixtureTree {
        FixtureTree::from_toml_str("[[dir]]\npath = \"cache/inner\"\n\n[[dir]]\npath = \"outside\"\n").unwrap()
    }

    fn measure(item: &Path, cancel: &CancelToken) -> Result<u64, WalkStop> {
        let handle = open_no_link(&fs::canonicalize(item).unwrap(), LastLink::AsItself).unwrap();
        let metadata = handle.metadata().unwrap();
        HeldWalk.measure(&handle, &metadata, metadata.dev(), cancel)
    }

    fn blocks(path: &Path) -> u64 {
        fs::symlink_metadata(path).unwrap().blocks() * 512
    }

    mod negative {
        use std::fs;

        use fresher_core::{CancelToken, SkipReason};

        use super::{measure, tree};
        use crate::guard::walk::WalkStop;
        use crate::test_support::{DiskImage, Mode};

        #[test]
        fn a_volume_mounted_inside_is_another_volume() {
            let tree = tree();
            let Some(image) = DiskImage::mount_at(&tree.root().join("cache/inner/mounted")) else { return };
            fs::write(image.mount_point().join("on-the-image"), vec![1; 4096]).unwrap();

            let answer = measure(&tree.root().join("cache"), &CancelToken::new());

            assert_eq!(answer, Err(WalkStop::Skip(SkipReason::OtherVolume)));
            assert!(image.mount_point().join("on-the-image").exists());
        }

        #[test]
        fn an_unreadable_folder_inside_fails_the_check() {
            let tree = tree();
            let _mode = Mode::set(&tree.root().join("cache/inner"), 0o000);

            let answer = measure(&tree.root().join("cache"), &CancelToken::new());

            assert_eq!(answer, Err(WalkStop::Skip(SkipReason::CheckFailed)));
        }

        #[test]
        fn a_cancelled_walk_stops() {
            let tree = tree();
            let cancel = CancelToken::new();
            cancel.cancel();

            assert_eq!(measure(&tree.root().join("cache"), &cancel), Err(WalkStop::Cancelled));
        }
    }

    mod positive {
        use std::fs;
        use std::os::unix::fs::symlink;

        use fresher_core::CancelToken;

        use super::{blocks, measure, tree};

        #[test]
        fn a_hard_link_inside_counts_once() {
            let tree = tree();
            let cache = tree.root().join("cache");
            fs::write(cache.join("data.bin"), vec![7; 64 * 1024]).unwrap();
            fs::hard_link(cache.join("data.bin"), cache.join("inner/same.bin")).unwrap();

            let measured = measure(&cache, &CancelToken::new()).unwrap();

            let once = blocks(&cache) + blocks(&cache.join("inner")) + blocks(&cache.join("data.bin"));
            assert_eq!(measured, once);
            assert!(measured >= 64 * 1024);
        }

        #[test]
        fn a_link_inside_is_counted_as_itself_not_its_target() {
            let tree = tree();
            let cache = tree.root().join("cache");
            fs::write(tree.root().join("outside/big.bin"), vec![7; 1024 * 1024]).unwrap();
            symlink(tree.root().join("outside"), cache.join("inner/to-outside")).unwrap();
            symlink(tree.root().join("outside/big.bin"), cache.join("to-big")).unwrap();

            let measured = measure(&cache, &CancelToken::new()).unwrap();

            assert!(measured < 1024 * 1024, "{measured}");
            assert_eq!(
                measured,
                blocks(&cache)
                    + blocks(&cache.join("inner"))
                    + blocks(&cache.join("to-big"))
                    + blocks(&cache.join("inner/to-outside"))
            );
        }

        #[test]
        fn a_file_is_measured_without_a_walk() {
            let tree = tree();
            let file = tree.root().join("cache/data.bin");
            fs::write(&file, vec![7; 8192]).unwrap();

            assert_eq!(measure(&file, &CancelToken::new()).unwrap(), blocks(&file));
        }
    }
}
