//! What a bundle takes on disk, walked with `lstat` only.

use std::collections::HashSet;
use std::fs::{self, Metadata};
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use fresher_core::{ByteSize, CancelToken, Completeness, Sizes};

/// The most entries one bundle is walked for; past it the sizes are `Partial`.
pub(crate) const MAX_BUNDLE_ENTRIES: usize = 2_000_000;

/// The sizes of the bundle at `bundle`: depth first, no link followed (a link counts as itself), nothing on another
/// device than the bundle entered or counted, a hard-linked file counted once (and reclaimable only when it has one
/// link). A folder that cannot be listed or an entry that cannot be read is skipped and makes the sizes `Partial`, as
/// does reaching [`MAX_BUNDLE_ENTRIES`]. An error when the bundle itself is not a folder or cannot be read, and
/// `Interrupted` once `cancel` is set (read between folders).
pub(crate) fn measure(bundle: &Path, cancel: &CancelToken) -> io::Result<Sizes> {
    measure_at_most(bundle, cancel, MAX_BUNDLE_ENTRIES)
}

fn measure_at_most(bundle: &Path, cancel: &CancelToken, cap: usize) -> io::Result<Sizes> {
    let root = fs::symlink_metadata(bundle)?;
    if !root.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the bundle is not a folder"));
    }
    let device = root.dev();
    let mut tally = Tally::default();
    let mut folders: Vec<PathBuf> = vec![bundle.to_path_buf()];
    let mut entries = 0_usize;
    while let Some(folder) = folders.pop() {
        if cancel.is_cancelled() {
            return Err(io::Error::from(io::ErrorKind::Interrupted));
        }
        // Checked again right before it is listed: a folder swapped for a link, or for another volume's folder, since
        // it was found is skipped. `read_dir` takes a path, so a swap between this check and the open is still
        // possible; it could only make the measure read another folder's metadata, never write or follow further.
        if !is_folder_on(&folder, device) {
            tally.partial = true;
            continue;
        }
        let Ok(listing) = fs::read_dir(&folder) else {
            tally.partial = true;
            continue;
        };
        for entry in listing {
            let Ok(entry) = entry else {
                tally.partial = true;
                continue;
            };
            entries += 1;
            if entries > cap {
                tally.partial = true;
                return Ok(tally.sizes());
            }
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                tally.partial = true;
                continue;
            };
            if metadata.dev() != device {
                continue;
            }
            if metadata.is_dir() {
                folders.push(path);
            } else {
                tally.add(&metadata);
            }
        }
    }
    Ok(tally.sizes())
}

/// Whether `folder` is, by its `lstat`, a real folder (not a link) on the volume `device`.
fn is_folder_on(folder: &Path, device: u64) -> bool {
    fs::symlink_metadata(folder).is_ok_and(|metadata| metadata.is_dir() && metadata.dev() == device)
}

/// The bytes counted so far.
#[derive(Debug, Default)]
struct Tally {
    logical: u64,
    allocated: u64,
    reclaimable: u64,
    /// The `(device, inode)` of every file with more than one link already counted.
    linked: HashSet<(u64, u64)>,
    partial: bool,
}

impl Tally {
    /// Counts a file, a link or another entry that is not a folder; a hard-linked file only the first time.
    fn add(&mut self, metadata: &Metadata) {
        let shared = metadata.nlink() > 1;
        if shared && !self.linked.insert((metadata.dev(), metadata.ino())) {
            return;
        }
        let allocated = metadata.blocks().saturating_mul(512);
        self.logical = self.logical.saturating_add(metadata.len());
        self.allocated = self.allocated.saturating_add(allocated);
        if !shared {
            self.reclaimable = self.reclaimable.saturating_add(allocated);
        }
    }

    fn sizes(&self) -> Sizes {
        Sizes {
            logical: ByteSize::new(self.logical),
            allocated: ByteSize::new(self.allocated),
            reclaimable: ByteSize::new(self.reclaimable),
            completeness: if self.partial { Completeness::Partial } else { Completeness::Complete },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::Path;

    /// The allocated bytes of the file at `path`, as the walk counts them.
    fn allocated(path: &Path) -> u64 {
        fs::symlink_metadata(path).unwrap().blocks() * 512
    }

    mod negative {
        use std::fs;
        use std::io;
        use std::os::unix::fs::PermissionsExt as _;

        use fresher_core::{ByteSize, CancelToken, Completeness};

        use super::allocated;
        use crate::bundle_facts::measure::{is_folder_on, measure, measure_at_most};

        #[test]
        fn a_link_inside_is_counted_as_itself_not_followed() {
            let root = tempfile::tempdir().unwrap();
            let outside = root.path().join("large");
            fs::write(&outside, vec![7_u8; 1 << 20]).unwrap();
            let bundle = root.path().join("Tool.app");
            fs::create_dir(&bundle).unwrap();
            let link = bundle.join("large");
            std::os::unix::fs::symlink(&outside, &link).unwrap();
            let folder_link = bundle.join("folder");
            std::os::unix::fs::symlink(root.path(), &folder_link).unwrap();

            let sizes = measure(&bundle, &CancelToken::new()).unwrap();

            let link_length =
                fs::symlink_metadata(&link).unwrap().len() + fs::symlink_metadata(&folder_link).unwrap().len();
            assert_eq!(sizes.logical, ByteSize::new(link_length));
            assert!(sizes.logical.bytes() < 4096);
            assert_eq!(sizes.allocated.bytes(), allocated(&link) + allocated(&folder_link));
            assert_eq!(sizes.completeness, Completeness::Complete);
        }

        #[test]
        fn a_hard_link_is_counted_once() {
            let root = tempfile::tempdir().unwrap();
            let bundle = root.path().join("Tool.app");
            fs::create_dir_all(bundle.join("Contents")).unwrap();
            let file = bundle.join("Contents/data");
            fs::write(&file, vec![1_u8; 10_000]).unwrap();
            fs::hard_link(&file, bundle.join("twin")).unwrap();

            let sizes = measure(&bundle, &CancelToken::new()).unwrap();

            assert_eq!(sizes.logical, ByteSize::new(10_000));
            assert_eq!(sizes.allocated.bytes(), allocated(&file));
            assert_eq!(sizes.reclaimable, ByteSize::ZERO);
        }

        #[test]
        fn an_unreadable_folder_makes_the_sizes_partial() {
            if fresher_macos::current_user() == 0 {
                return; // root reads a folder of mode 000 all the same
            }
            let root = tempfile::tempdir().unwrap();
            let bundle = root.path().join("Tool.app");
            let closed = bundle.join("Contents/Closed");
            fs::create_dir_all(&closed).unwrap();
            fs::write(closed.join("hidden"), vec![1_u8; 5_000]).unwrap();
            fs::write(bundle.join("Contents/seen"), b"seen").unwrap();
            fs::set_permissions(&closed, fs::Permissions::from_mode(0o000)).unwrap();

            let sizes = measure(&bundle, &CancelToken::new());

            fs::set_permissions(&closed, fs::Permissions::from_mode(0o755)).unwrap();
            let sizes = sizes.unwrap();
            assert_eq!(sizes.completeness, Completeness::Partial);
            assert_eq!(sizes.logical, ByteSize::new(4));
        }

        #[test]
        fn the_entry_cap_makes_the_sizes_partial() {
            let root = tempfile::tempdir().unwrap();
            let bundle = root.path().join("Tool.app");
            fs::create_dir(&bundle).unwrap();
            for name in ["a", "b", "c"] {
                fs::write(bundle.join(name), b"x").unwrap();
            }

            let capped = measure_at_most(&bundle, &CancelToken::new(), 2).unwrap();
            let whole = measure_at_most(&bundle, &CancelToken::new(), 3).unwrap();

            assert_eq!(capped.completeness, Completeness::Partial);
            assert_eq!(capped.logical, ByteSize::new(2));
            assert_eq!(whole.completeness, Completeness::Complete);
            assert_eq!(whole.logical, ByteSize::new(3));
        }

        #[test]
        fn a_folder_that_became_a_link_or_left_the_volume_is_not_listed() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("folder");
            fs::create_dir(&folder).unwrap();
            let link = root.path().join("link");
            std::os::unix::fs::symlink(&folder, &link).unwrap();
            let file = root.path().join("file");
            fs::write(&file, b"x").unwrap();
            let device = std::os::unix::fs::MetadataExt::dev(&fs::symlink_metadata(&folder).unwrap());

            assert!(!is_folder_on(&link, device));
            assert!(!is_folder_on(&file, device));
            assert!(!is_folder_on(&root.path().join("missing"), device));
            assert!(!is_folder_on(&folder, device.wrapping_add(1)));
            assert!(is_folder_on(&folder, device));
        }

        #[test]
        fn a_cancelled_measure_stops() {
            let root = tempfile::tempdir().unwrap();
            let bundle = root.path().join("Tool.app");
            fs::create_dir(&bundle).unwrap();
            let cancel = CancelToken::new();
            cancel.cancel();

            let error = measure(&bundle, &cancel).unwrap_err();

            assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        }

        #[test]
        fn a_bundle_that_is_a_link_a_file_or_missing_is_not_measured() {
            let root = tempfile::tempdir().unwrap();
            let real = root.path().join("Real.app");
            fs::create_dir(&real).unwrap();
            let link = root.path().join("Link.app");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            let file = root.path().join("File.app");
            fs::write(&file, b"x").unwrap();

            for bundle in [link, file, root.path().join("Missing.app")] {
                assert!(measure(&bundle, &CancelToken::new()).is_err(), "{}", bundle.display());
            }
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::{ByteSize, CancelToken, Completeness, Sizes};

        use super::allocated;
        use crate::bundle_facts::measure::measure;

        #[test]
        fn a_bundle_s_files_add_up() {
            let root = tempfile::tempdir().unwrap();
            let bundle = root.path().join("Tool.app");
            let macos = bundle.join("Contents/MacOS");
            fs::create_dir_all(&macos).unwrap();
            let files =
                [(bundle.join("Contents/Info.plist"), 300), (macos.join("Tool"), 50_000), (bundle.join("x"), 1)];
            for (path, length) in &files {
                fs::write(path, vec![0_u8; *length]).unwrap();
            }

            let sizes = measure(&bundle, &CancelToken::new()).unwrap();

            let allocated: u64 = files.iter().map(|(path, _)| allocated(path)).sum();
            assert_eq!(sizes.logical, ByteSize::new(50_301));
            assert_eq!(sizes.allocated, ByteSize::new(allocated));
            assert_eq!(sizes.reclaimable, ByteSize::new(allocated));
            assert!(allocated >= 50_301);
            assert_eq!(sizes.completeness, Completeness::Complete);
        }

        #[test]
        fn an_empty_bundle_is_zero_and_complete() {
            let root = tempfile::tempdir().unwrap();
            let bundle = root.path().join("Empty.app");
            fs::create_dir_all(bundle.join("Contents/Resources")).unwrap();

            assert_eq!(measure(&bundle, &CancelToken::new()).unwrap(), Sizes::ZERO);
        }
    }
}
