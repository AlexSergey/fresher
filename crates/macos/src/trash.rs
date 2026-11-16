//! `MacTrash`: the system's Trash, and put back.

use std::fs::{self, File};
use std::io;
use std::os::fd::AsFd as _;
use std::path::{Path, PathBuf};

use fresher_core::{Landed, Trash, TrashError};

use crate::ffi::rename::rename_exclusive;
use crate::ffi::trash::{TrashFailure, path_of, trash_item};

/// The domain of the Trash call's errors.
const COCOA: &str = "NSCocoaErrorDomain";
/// `NSFileNoSuchFileError`: nothing is at the path.
const NO_SUCH_FILE: isize = 4;
/// `NSFileWriteNoPermissionError`: a locked item, a read-only folder, a volume that refuses a `.Trashes` folder.
const NO_PERMISSION: isize = 513;
/// `NSFeatureUnsupportedError`: the volume has no Trash (a read-only volume, 01/01).
const UNSUPPORTED: isize = 3328;

/// The system's Trash: `NSFileManager`'s Trash call, which puts the item in its own volume's Trash and writes what
/// Finder's Put Back reads; put back is a rename that never replaces anything and follows no link. Never deletes.
#[derive(Clone, Copy, Debug, Default)]
pub struct MacTrash;

impl MacTrash {
    pub const fn new() -> Self {
        Self
    }
}

impl Trash for MacTrash {
    /// `at` is read from `held` after the call: where the verified item is now, even when the call moved another
    /// item or named no landed path (design, call 37). When the system cannot say, `at` is empty, which lies in no
    /// Trash folder, so the cleaner trusts only a `reported` path that holds the verified identity.
    fn move_to_trash(&self, path: &Path, held: &File) -> Result<Landed, TrashError> {
        if !path.is_absolute() {
            return Err(TrashError::Refused { reason: "the path is not absolute".to_owned() });
        }
        let is_directory = held.metadata().map_err(|error| refused(&error))?.is_dir();
        let reported = trash_item(path, is_directory).map_err(|failure| trash_error(&failure))?;
        let at = path_of(held.as_fd()).unwrap_or_else(|error| {
            tracing::warn!(%error, "the moved item's place is unknown");
            PathBuf::new()
        });
        Ok(Landed { at, reported })
    }

    fn put_back(&self, landed: &Path, original: &Path) -> Result<(), TrashError> {
        rename_exclusive(landed, original).map_err(|error| {
            let landed_is_gone = fs::symlink_metadata(landed).is_err();
            put_back_error(&error, landed_is_gone)
        })
    }
}

/// The Trash call's error in the core's words; no variant names the item.
fn trash_error(failure: &TrashFailure) -> TrashError {
    match failure {
        TrashFailure::BadPath => TrashError::Refused { reason: "the path cannot name a file".to_owned() },
        TrashFailure::Refused { domain, code } if domain == COCOA => match *code {
            NO_SUCH_FILE => TrashError::NotFound,
            NO_PERMISSION => TrashError::NotAllowed,
            UNSUPPORTED => TrashError::NoTrash,
            code => TrashError::Refused { reason: format!("{COCOA} {code}") },
        },
        TrashFailure::Refused { domain, code } => TrashError::Refused { reason: format!("{domain} {code}") },
    }
}

/// A failed put back in the core's words. `ENOENT` is the landed item gone or the original folder gone.
fn put_back_error(error: &io::Error, landed_is_gone: bool) -> TrashError {
    match error.raw_os_error() {
        Some(libc::EEXIST) => TrashError::Occupied,
        Some(libc::ENOENT) if landed_is_gone => TrashError::NotFound,
        Some(libc::ENOENT) => TrashError::NoParent,
        Some(libc::ELOOP) => TrashError::ThroughLink,
        _ => refused(error),
    }
}

fn refused(error: &io::Error) -> TrashError {
    TrashError::Refused { reason: error.raw_os_error().map_or_else(|| error.kind().to_string(), errno_text) }
}

fn errno_text(errno: i32) -> String {
    format!("errno {errno}")
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File, OpenOptions};
    use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _};
    use std::path::{Path, PathBuf};

    use crate::disk_image::DiskImage;

    /// The item's device and file id, its last component not followed.
    fn identity(path: &Path) -> Option<(u64, u64)> {
        fs::symlink_metadata(path).ok().map(|metadata| (metadata.dev(), metadata.ino()))
    }

    /// The handle the cleaner holds: no link followed anywhere, a link that is the item opened as the link.
    fn hold(path: &Path) -> File {
        OpenOptions::new().read(true).custom_flags(libc::O_SYMLINK | libc::O_NOFOLLOW_ANY).open(path).unwrap()
    }

    /// The image's Trash folder of this user (the owner of a file the test makes), where its items land.
    fn trash_of(image: &DiskImage) -> PathBuf {
        let user = tempfile::tempfile().unwrap().metadata().unwrap().uid();
        image.mount_point().join(".Trashes").join(user.to_string())
    }

    fn file(path: &Path) -> PathBuf {
        fs::write(path, b"an item of the test").unwrap();
        path.to_path_buf()
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::Path;

        use fresher_core::{Trash as _, TrashError};

        use super::{file, hold, identity, trash_of};
        use crate::MacTrash;
        use crate::disk_image::{DiskImage, Format};
        use crate::ffi::trash::TrashFailure;
        use crate::trash::{put_back_error, trash_error};

        #[test]
        fn an_error_of_another_domain_or_an_unknown_code_is_a_refusal_without_a_path() {
            let foreign = TrashFailure::Refused { domain: "NSOSStatusErrorDomain".to_owned(), code: -120 };
            let unknown = TrashFailure::Refused { domain: "NSCocoaErrorDomain".to_owned(), code: 640 };

            assert_eq!(trash_error(&foreign), TrashError::Refused { reason: "NSOSStatusErrorDomain -120".to_owned() });
            assert_eq!(trash_error(&unknown), TrashError::Refused { reason: "NSCocoaErrorDomain 640".to_owned() });
            assert!(matches!(trash_error(&TrashFailure::BadPath), TrashError::Refused { .. }));
        }

        #[test]
        fn an_unknown_put_back_error_is_a_refusal() {
            let across = std::io::Error::from_raw_os_error(libc::EXDEV);
            let unnumbered = std::io::Error::from(std::io::ErrorKind::InvalidInput);

            assert_eq!(put_back_error(&across, false), TrashError::Refused { reason: "errno 18".to_owned() });
            assert!(matches!(put_back_error(&unnumbered, false), TrashError::Refused { .. }));
        }

        #[test]
        fn a_relative_path_is_refused() {
            let folder = tempfile::tempdir().unwrap();
            let held = hold(&fs::canonicalize(folder.path()).unwrap());

            let moved = MacTrash.move_to_trash(Path::new("relative/item"), &held);

            assert!(matches!(moved, Err(TrashError::Refused { .. })));
        }

        #[test]
        fn a_missing_item_is_not_found() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let held = hold(image.mount_point());

            let moved = MacTrash.move_to_trash(&image.mount_point().join("missing"), &held);

            assert_eq!(moved, Err(TrashError::NotFound));
        }

        #[test]
        fn a_volume_without_a_trash_moves_nothing() {
            let Some(image) = DiskImage::mount_read_only(Format::Apfs, |root| {
                file(&root.join("item"));
            }) else {
                return;
            };
            let item = image.mount_point().join("item");
            let before = identity(&item);

            let moved = MacTrash.move_to_trash(&item, &hold(&item));

            assert_eq!(moved, Err(TrashError::NoTrash));
            assert_eq!(identity(&item), before);
            assert!(before.is_some());
        }

        #[test]
        fn put_back_never_replaces_an_existing_item() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let item = file(&image.mount_point().join("item"));
            let landed = MacTrash.move_to_trash(&item, &hold(&item)).unwrap().at;
            let moved = identity(&landed);
            let newcomer = identity(&file(&item));

            let put_back = MacTrash.put_back(&landed, &item);

            assert_eq!(put_back, Err(TrashError::Occupied));
            assert_eq!(identity(&item), newcomer);
            assert_eq!(identity(&landed), moved);
            fs::rename(&item, image.mount_point().join("aside")).unwrap();
            MacTrash.put_back(&landed, &item).unwrap();
            assert_eq!(identity(&item), moved);
        }

        #[test]
        fn put_back_into_a_missing_folder_is_refused() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let folder = image.mount_point().join("folder");
            fs::create_dir(&folder).unwrap();
            let item = file(&folder.join("item"));
            let landed = MacTrash.move_to_trash(&item, &hold(&item)).unwrap().at;
            fs::rename(&folder, image.mount_point().join("renamed")).unwrap();

            let put_back = MacTrash.put_back(&landed, &item);

            assert_eq!(put_back, Err(TrashError::NoParent));
            assert!(identity(&landed).is_some());
            fs::rename(image.mount_point().join("renamed"), &folder).unwrap();
            MacTrash.put_back(&landed, &item).unwrap();
        }

        #[test]
        fn put_back_of_a_missing_landed_item_is_not_found() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };

            let put_back = MacTrash.put_back(&trash_of(&image).join("missing"), &image.mount_point().join("item"));

            assert_eq!(put_back, Err(TrashError::NotFound));
        }

        #[test]
        fn put_back_through_a_link_is_refused() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let root = image.mount_point();
            let item = file(&root.join("item"));
            let landed = MacTrash.move_to_trash(&item, &hold(&item)).unwrap().at;
            fs::create_dir(root.join("real")).unwrap();
            symlink(root.join("real"), root.join("via")).unwrap();

            let put_back = MacTrash.put_back(&landed, &root.join("via").join("item"));

            assert_eq!(put_back, Err(TrashError::ThroughLink));
            assert!(identity(&root.join("real").join("item")).is_none());
            assert!(identity(&landed).is_some());
            MacTrash.put_back(&landed, &item).unwrap();
        }

        #[test]
        fn a_link_is_moved_as_the_link() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let root = image.mount_point();
            let target = file(&root.join("target"));
            let target_before = identity(&target);
            let link = root.join("link");
            symlink(&target, &link).unwrap();
            let link_before = identity(&link);

            let landed = MacTrash.move_to_trash(&link, &hold(&link)).unwrap();

            assert!(fs::symlink_metadata(&landed.at).unwrap().file_type().is_symlink());
            assert_eq!(identity(&landed.at), link_before);
            assert_eq!(fs::read_link(&landed.at).unwrap(), target);
            assert_eq!(identity(&target), target_before);
            assert!(identity(&link).is_none());
            MacTrash.put_back(&landed.at, &link).unwrap();
            assert_eq!(identity(&link), link_before);
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::{Landed, Trash as _, TrashError};

        use super::{file, hold, identity, trash_of};
        use crate::MacTrash;
        use crate::disk_image::{DiskImage, Format};
        use crate::trash::put_back_error;

        #[test]
        fn each_known_put_back_error_has_its_meaning() {
            let error = |errno| std::io::Error::from_raw_os_error(errno);

            assert_eq!(put_back_error(&error(libc::EEXIST), false), TrashError::Occupied);
            assert_eq!(put_back_error(&error(libc::ENOENT), false), TrashError::NoParent);
            assert_eq!(put_back_error(&error(libc::ENOENT), true), TrashError::NotFound);
            assert_eq!(put_back_error(&error(libc::ELOOP), false), TrashError::ThroughLink);
        }

        #[test]
        fn an_item_lands_in_the_trash_of_its_volume_with_its_identity() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let item = file(&image.mount_point().join("item.txt"));
            let before = identity(&item);

            let Landed { reported, .. } = MacTrash.move_to_trash(&item, &hold(&item)).unwrap();

            let reported = reported.unwrap();
            assert_eq!(reported.parent(), Some(trash_of(&image).as_path()));
            assert_eq!(identity(&reported), before);
            assert!(identity(&item).is_none());
        }

        #[test]
        fn the_held_handle_names_the_landed_path() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let item = file(&image.mount_point().join("item.txt"));
            let held = hold(&item);

            let landed = MacTrash.move_to_trash(&item, &held).unwrap();

            assert_eq!(Some(landed.at.clone()), landed.reported);
            assert!(landed.at.starts_with(trash_of(&image)));
        }

        #[test]
        fn put_back_restores_the_item_and_its_identity() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let item = file(&image.mount_point().join("item.txt"));
            let before = identity(&item);
            let landed = MacTrash.move_to_trash(&item, &hold(&item)).unwrap().at;

            MacTrash.put_back(&landed, &item).unwrap();

            assert_eq!(identity(&item), before);
            assert!(identity(&landed).is_none());
            assert_eq!(fs::read(&item).unwrap(), b"an item of the test");
        }

        #[test]
        fn a_folder_moves_as_one() {
            let Some(image) = DiskImage::mount(Format::Apfs) else { return };
            let folder = image.mount_point().join("folder");
            fs::create_dir_all(folder.join("inner")).unwrap();
            let inner = identity(&file(&folder.join("inner").join("deep.txt")));
            let before = identity(&folder);

            let landed = MacTrash.move_to_trash(&folder, &hold(&folder)).unwrap();

            assert_eq!(landed.reported.as_ref(), Some(&landed.at));
            assert_eq!(identity(&landed.at), before);
            assert_eq!(identity(&landed.at.join("inner").join("deep.txt")), inner);
            assert!(identity(&folder).is_none());
            MacTrash.put_back(&landed.at, &folder).unwrap();
            assert_eq!(identity(&folder.join("inner").join("deep.txt")), inner);
        }
    }
}
