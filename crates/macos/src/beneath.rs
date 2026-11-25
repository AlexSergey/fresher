//! Acting beneath a held folder: every call names one entry of a folder that is held open, never a path, so a folder
//! swapped for a link after it was opened cannot send a call anywhere else. Folders are opened through no link in
//! any component (`O_NOFOLLOW_ANY`); an entry is a link as itself. The cleaner removes with these (plan 008; the
//! known issue `docs/known-issues/cleaner-act-by-path.md`).

use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsFd as _;
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};

use fresher_core::{EntryKind, FolderListing, ItemIdentity, ReadError};

use crate::bulk::{device_id, read_held};
use crate::ffi::beneath::stat_at;
use crate::ffi::bulk::open_at;

pub use crate::ffi::beneath::remove_at;

/// Opens the folder at `path` for reading; a link in any component fails with `ELOOP`, anything but a folder with
/// `ENOTDIR`.
pub fn open_folder(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW_ANY).open(path)
}

/// Opens the folder `name` of the held folder `folder` for reading (`O_NOFOLLOW_ANY`, which `O_NOFOLLOW` beside it
/// would make `EINVAL`); a link fails with `ELOOP`, anything but a folder
/// with `ENOTDIR`, a name that is not one plain name with `InvalidInput`.
pub fn open_folder_at(folder: &File, name: &OsStr) -> io::Result<File> {
    let name = crate::ffi::c_name(name)?;
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW_ANY;
    open_at(folder.as_fd(), &name, flags).map(File::from)
}

/// The identity of the entry `name` of the held folder `folder` (its volume, file id and kind), a link as itself.
pub fn identity_at(folder: &File, name: &OsStr) -> io::Result<ItemIdentity> {
    let status = stat_at(folder.as_fd(), name)?;
    let kind = match status.st_mode & libc::S_IFMT {
        libc::S_IFDIR => EntryKind::Folder,
        libc::S_IFLNK => EntryKind::Symlink,
        libc::S_IFREG => EntryKind::File,
        _ => EntryKind::Other,
    };
    Ok(ItemIdentity::new(device_id(status.st_dev), status.st_ino, kind))
}

/// The path of the held item `file` as its volume spells it now (`F_GETPATH`): each name as the disk holds it, in
/// its case and normal form, whatever spelling opened it.
pub fn path_of(file: &File) -> io::Result<PathBuf> {
    crate::ffi::trash::path_of(file.as_fd())
}

/// Every entry of the held folder `folder`, with its attributes, into `out` (cleared first), with the bulk read the
/// scan uses; the folder is read from its first entry even if it was read before.
pub fn read_folder(folder: &File, out: &mut FolderListing) -> Result<(), ReadError> {
    read_held(folder, out)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;

    use tempfile::TempDir;

    /// `folder/a.bin`, `folder/inner/b.bin`, `outside/c.bin`, and `folder/link` to `outside`.
    fn tree() -> TempDir {
        let root = tempfile::tempdir().unwrap();
        for folder in ["folder/inner", "outside"] {
            fs::create_dir_all(root.path().join(folder)).unwrap();
        }
        for file in ["folder/a.bin", "folder/inner/b.bin", "outside/c.bin"] {
            fs::write(root.path().join(file), b"x").unwrap();
        }
        symlink(root.path().join("outside"), root.path().join("folder/link")).unwrap();
        root
    }

    /// The temporary folder's canonical path: `/var` is a link of the system.
    fn canonical(root: &TempDir) -> std::path::PathBuf {
        fs::canonicalize(root.path()).unwrap()
    }

    mod negative {
        use std::ffi::OsStr;
        use std::io;

        use super::{canonical, tree};
        use crate::beneath::{identity_at, open_folder, open_folder_at};

        #[test]
        fn a_link_in_any_component_is_refused() {
            let root = tree();
            let root = canonical(&root);

            for path in [root.join("folder/link"), root.join("folder/link/inner")] {
                assert_eq!(open_folder(&path).unwrap_err().raw_os_error(), Some(libc::ELOOP), "{}", path.display());
            }
        }

        #[test]
        fn a_link_beneath_a_held_folder_is_refused() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();

            assert_eq!(open_folder_at(&folder, OsStr::new("link")).unwrap_err().raw_os_error(), Some(libc::ELOOP));
        }

        #[test]
        fn a_file_is_not_opened_as_a_folder() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();

            assert_eq!(open_folder_at(&folder, OsStr::new("a.bin")).unwrap_err().raw_os_error(), Some(libc::ENOTDIR));
        }

        #[test]
        fn a_name_that_is_not_one_plain_name_is_refused() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();

            for name in ["", ".", "..", "inner/b.bin", "link/c.bin", "a\0b"] {
                let name = OsStr::new(name);
                assert_eq!(open_folder_at(&folder, name).unwrap_err().kind(), io::ErrorKind::InvalidInput, "{name:?}");
                assert_eq!(identity_at(&folder, name).unwrap_err().kind(), io::ErrorKind::InvalidInput, "{name:?}");
            }
        }

        #[test]
        fn a_missing_entry_is_not_found() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();

            assert_eq!(identity_at(&folder, OsStr::new("gone")).unwrap_err().kind(), io::ErrorKind::NotFound);
        }

        #[test]
        fn a_folder_moved_away_is_still_the_held_one() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();
            std::fs::rename(root.path().join("folder"), root.path().join("moved")).unwrap();
            std::os::unix::fs::symlink(root.path().join("outside"), root.path().join("folder")).unwrap();

            assert!(identity_at(&folder, OsStr::new("a.bin")).is_ok());
            assert_eq!(identity_at(&folder, OsStr::new("c.bin")).unwrap_err().kind(), io::ErrorKind::NotFound);
        }

        #[test]
        fn a_volume_mounted_inside_has_another_device() {
            use std::os::unix::fs::MetadataExt as _;

            use crate::disk_image::{DiskImage, Format};

            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let point = image.mount_point();
            let parent = open_folder(point.parent().unwrap()).unwrap();

            let mounted = open_folder_at(&parent, point.file_name().unwrap()).unwrap();

            assert_ne!(mounted.metadata().unwrap().dev(), parent.metadata().unwrap().dev());
        }
    }

    mod positive {
        use std::ffi::OsStr;
        use std::fs;
        use std::os::unix::fs::MetadataExt as _;

        use fresher_core::{EntryKind, FolderListing};

        use super::{canonical, tree};
        use crate::beneath::{identity_at, open_folder, open_folder_at, read_folder};

        #[test]
        fn an_entry_has_the_identity_std_reads() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();

            for (name, kind) in [("a.bin", EntryKind::File), ("inner", EntryKind::Folder), ("link", EntryKind::Symlink)]
            {
                let identity = identity_at(&folder, OsStr::new(name)).unwrap();
                let metadata = fs::symlink_metadata(root.path().join("folder").join(name)).unwrap();
                assert_eq!(
                    (identity.device(), identity.file(), identity.kind()),
                    (metadata.dev(), metadata.ino(), kind)
                );
            }
        }

        #[test]
        fn a_folder_beneath_a_held_folder_opens() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();

            let inner = open_folder_at(&folder, OsStr::new("inner")).unwrap();

            assert_eq!(inner.metadata().unwrap().ino(), fs::metadata(root.path().join("folder/inner")).unwrap().ino());
        }

        #[test]
        fn a_held_folder_lists_its_entries_each_time_it_is_read() {
            let root = tree();
            let folder = open_folder(&canonical(&root).join("folder")).unwrap();
            let mut listing = FolderListing::new();

            for _ in 0..2 {
                read_folder(&folder, &mut listing).unwrap();
                let mut names: Vec<_> = listing.iter().map(|(name, entry)| (name.to_vec(), entry.kind)).collect();
                names.sort_by(|left, right| left.0.cmp(&right.0));
                assert_eq!(
                    names,
                    [
                        (b"a.bin".to_vec(), EntryKind::File),
                        (b"inner".to_vec(), EntryKind::Folder),
                        (b"link".to_vec(), EntryKind::Symlink)
                    ]
                );
            }
        }
    }
}
