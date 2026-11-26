//! The single deletion point of the workspace: the only code allowed to call a deleting function (`std::fs`'s and
//! `fresher-macos`'s `remove_at`; checked by `tests/deletion_point.rs`). It deletes only a verified item, for a
//! confirmed plan whose method was the permanent one, and only beneath held folders: every removal names one entry of
//! a folder held open through no link, never a path.

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::os::fd::AsFd as _;

use fresher_core::{Act, EntryKind, Failure, ItemIdentity, ItemOutcome, PermanentConsent, SkipReason};

use crate::beneath::{self, Listed, descend, entries};
use crate::guard::Verified;
use crate::paths::open_failure;
use crate::remove::Removed;

/// Deletes the verified item beneath its parent folder, opened through no link in any component: the parent's entry
/// must still be the verified item, which is then removed as `remove_beneath` says. Afterwards the entry must no
/// longer be the item. Anything refused on the way fails the item; what was removed before the refusal stays removed.
pub(crate) fn delete(verified: Verified, _act: Act<'_>, _consent: &PermanentConsent) -> Removed {
    let (handle, placed, identity) = verified.into_parts();
    let (Some(parent), Some(name)) = (placed.physical.parent(), placed.physical.file_name()) else {
        return failed();
    };
    let deleted = beneath::open_folder(parent)
        .map_err(|error| open_failure(&error))
        .and_then(|parent| remove_and_check(&parent, name, identity, placed.device));
    drop(handle);
    ended(deleted.is_ok())
}

/// Deletes the entry `name` of the verified folder being emptied, beneath the folder's held handle; the entry must
/// still be `listed`, as the folder's listing gave it.
pub(crate) fn delete_entry(
    folder: &Verified,
    name: &OsStr,
    listed: ItemIdentity,
    _act: Act<'_>,
    _consent: &PermanentConsent,
) -> Removed {
    ended(remove_and_check(folder.handle().file(), name, listed, folder.placed().device).is_ok())
}

fn ended(deleted: bool) -> Removed {
    if deleted { Removed::ended(ItemOutcome::Deleted) } else { failed() }
}

fn failed() -> Removed {
    Removed::ended(ItemOutcome::Failed { reason: Failure::DeleteFailed.text() })
}

/// Removes the entry, then checks the folder's entry `name` is no longer `expected`.
fn remove_and_check(parent: &File, name: &OsStr, expected: ItemIdentity, device: u64) -> Result<(), SkipReason> {
    remove_beneath(parent, name, expected, device)?;
    match beneath::identity_at(parent, name) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(found) if found != expected => Ok(()),
        _ => Err(SkipReason::CheckFailed),
    }
}

/// A folder being emptied before it is removed: held open, with the entry it is in the folder above and what is
/// left to remove in it.
struct Level {
    folder: File,
    name: OsString,
    identity: ItemIdentity,
    pending: std::vec::IntoIter<Listed>,
}

/// Removes the entry `name` of the held folder `parent`, which must be `expected` on `device`: a file or a link at
/// once (a link as the link), a folder after everything in it, depth first, one open folder per level. Each folder is
/// opened beneath the one above through no link and must be what its listing said, on `device` (a volume mounted
/// inside is never crossed); each entry is checked against its listing right before it is removed. The first refusal
/// stops it.
fn remove_beneath(parent: &File, name: &OsStr, expected: ItemIdentity, device: u64) -> Result<(), SkipReason> {
    if expected.device() != device {
        return Err(SkipReason::OtherVolume);
    }
    if expected.kind() != EntryKind::Folder {
        return unlink(parent, name, expected);
    }
    let folder = descend(parent, name, expected, device)?;
    let pending = entries(&folder)?.into_iter();
    let mut levels = vec![Level { folder, name: name.to_os_string(), identity: expected, pending }];
    while let Some(level) = levels.last_mut() {
        let Some(entry) = level.pending.next() else {
            let Some(done) = levels.pop() else { break };
            drop(done.folder);
            unlink(levels.last().map_or(parent, |above| &above.folder), &done.name, done.identity)?;
            continue;
        };
        let identity = entry.identity.ok_or(SkipReason::CheckFailed)?;
        if identity.device() != device {
            return Err(SkipReason::OtherVolume);
        }
        if identity.kind() == EntryKind::Folder {
            let folder = descend(&level.folder, &entry.name, identity, device)?;
            let pending = entries(&folder)?.into_iter();
            levels.push(Level { folder, name: entry.name, identity, pending });
        } else {
            unlink(&level.folder, &entry.name, identity)?;
        }
    }
    Ok(())
}

/// Removes the entry `name` of the held folder `folder` if it is still `expected`: the one call of a deleting
/// function in the workspace (the walk above and `own_data` reach it).
#[expect(
    clippy::disallowed_methods,
    reason = "the single deletion point of the workspace (docs/architecture/removal.md)"
)]
pub(crate) fn unlink(folder: &File, name: &OsStr, expected: ItemIdentity) -> Result<(), SkipReason> {
    let found = beneath::identity_at(folder, name).map_err(|error| open_failure(&error))?;
    if found != expected {
        return Err(SkipReason::Replaced);
    }
    beneath::remove_at(folder.as_fd(), name, expected.kind()).map_err(|_| SkipReason::CheckFailed)
}

#[cfg(test)]
mod tests {
    use fresher_core::{ConfirmedPlan, ItemOutcome, Method};

    use crate::remove::permanent::delete;
    use fresher_testing::Swapper;

    use crate::test_support::{Setup, confirmed, verified};

    fn setup() -> Setup {
        Setup::new(
            r#"
            [[file]]
            path = "home/cache/a.bin"
            [[file]]
            path = "home/cache/inner/b.bin"
            [[file]]
            path = "outside/keep.txt"
            [[symlink]]
            path = "home/cache/inner/to-outside"
            target = "../../../outside"
            [[symlink]]
            path = "home/link"
            target = "outside/keep.txt"
            "#,
        )
    }

    /// The item `home/cache/inner` and its twin outside the root, `outside/inner`, with the same names inside.
    fn twins() -> Setup {
        Setup::new(
            r#"
            [[file]]
            path = "home/cache/inner/a.bin"
            [[file]]
            path = "home/cache/inner/deep/b.bin"
            [[file]]
            path = "home/cache/inner/deep/c.bin"
            [[file]]
            path = "outside/inner/a.bin"
            [[file]]
            path = "outside/inner/deep/b.bin"
            [[file]]
            path = "outside/inner/deep/c.bin"
            "#,
        )
    }

    const TWIN: [&str; 3] = ["outside/inner/a.bin", "outside/inner/deep/b.bin", "outside/inner/deep/c.bin"];

    fn permanent(setup: &Setup) -> ConfirmedPlan {
        confirmed(setup, &[], Method::Permanent)
    }

    /// Deletes `home/cache/inner` of fresh twins again and again while `folder` is swapped for a link to `target`
    /// between the check and the end of the delete: whatever the timing, nothing behind the link goes.
    fn race(folder: &str, target: &str) {
        for _ in 0..100 {
            let setup = twins();
            let plan = permanent(&setup);
            let item = verified(&setup, "home/cache/inner");

            let swapper = Swapper::start(&setup.at(folder), &setup.at(target)).unwrap();
            let removed = delete(item, plan.act(), plan.permanent().unwrap());
            drop(swapper);

            assert!(matches!(removed.outcome, ItemOutcome::Deleted | ItemOutcome::Failed { .. }), "{removed:?}");
            for twin in TWIN {
                assert!(setup.at(twin).exists(), "{twin} went");
            }
        }
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::PathBuf;

        use fresher_core::{Failure, ItemOutcome};

        use super::{TWIN, permanent, race, setup, twins};
        use crate::remove::permanent::delete;
        use crate::test_support::{DiskImage, Flags, listing, verified};

        fn failed() -> ItemOutcome {
            ItemOutcome::Failed { reason: Failure::DeleteFailed.text() }
        }

        #[test]
        fn a_parent_swapped_for_a_link_during_the_delete_never_deletes_behind_it() {
            race("home/cache", "outside");
        }

        #[test]
        fn a_folder_inside_swapped_for_a_link_during_the_delete_never_deletes_behind_it() {
            race("home/cache/inner/deep", "outside/inner/deep");
        }

        #[test]
        fn a_parent_swapped_for_a_link_after_the_check_deletes_nothing() {
            let setup = twins();
            let plan = permanent(&setup);
            let item = verified(&setup, "home/cache/inner");
            fs::rename(setup.at("home/cache"), setup.at("home/away")).unwrap();
            symlink(setup.at("outside"), setup.at("home/cache")).unwrap();

            let removed = delete(item, plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, failed());
            assert!(TWIN.iter().all(|twin| setup.at(twin).exists()));
            assert!(setup.at("home/away/inner/deep/c.bin").exists());
        }

        #[test]
        fn a_folder_replaced_after_the_check_is_not_deleted() {
            let setup = twins();
            let plan = permanent(&setup);
            let item = verified(&setup, "home/cache/inner");
            fs::rename(setup.at("home/cache/inner"), setup.at("home/cache/away")).unwrap();
            fs::create_dir(setup.at("home/cache/inner")).unwrap();
            fs::write(setup.at("home/cache/inner/new.bin"), b"new").unwrap();

            let removed = delete(item, plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, failed());
            assert!(setup.at("home/cache/inner/new.bin").exists());
            assert!(setup.at("home/cache/away/deep/c.bin").exists());
        }

        #[test]
        fn a_locked_entry_inside_stops_the_delete_and_the_rest_stays() {
            let setup = twins();
            fs::write(setup.at("home/cache/inner/z.bin"), b"z").unwrap();
            let plan = permanent(&setup);
            let item = verified(&setup, "home/cache/inner");
            let _flags = Flags::lock(&setup.at("home/cache/inner/deep/b.bin"));

            let removed = delete(item, plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, failed());
            let left: Vec<PathBuf> = listing(&setup.at("home/cache")).into_iter().map(|entry| entry.0).collect();
            let expected = ["inner", "inner/deep", "inner/deep/b.bin", "inner/deep/c.bin", "inner/z.bin"];
            assert_eq!(left, expected.map(PathBuf::from));
        }

        #[test]
        fn a_volume_mounted_inside_is_never_crossed() {
            let setup = twins();
            let Some(image) = DiskImage::mount_at(&setup.at("home/cache/inner/deep/mounted")) else { return };
            fs::write(image.mount_point().join("on-the-image"), b"kept").unwrap();
            let plan = permanent(&setup);
            let item = verified(&setup, "home/cache/inner");

            let removed = delete(item, plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, failed());
            assert!(image.mount_point().join("on-the-image").exists());
        }

        #[test]
        fn a_delete_the_system_refuses_fails_and_leaves_the_item() {
            let setup = setup();
            let plan = permanent(&setup);
            let item = verified(&setup, "home/cache/a.bin");
            let _flags = Flags::lock(&setup.at("home/cache/a.bin"));

            let removed = delete(item, plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, failed());
            assert!(setup.at("home/cache/a.bin").exists());
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::ItemOutcome;

        use super::{TWIN, permanent, setup, twins};
        use crate::remove::permanent::delete;
        use crate::test_support::verified;

        #[test]
        fn a_folder_is_deleted_with_everything_in_it_and_its_twin_outside_stays() {
            let setup = twins();
            fs::hard_link(setup.at("home/cache/inner/a.bin"), setup.at("home/cache/inner/deep/same.bin")).unwrap();
            fs::create_dir_all(setup.at("home/cache/inner/empty/emptier")).unwrap();
            let plan = permanent(&setup);

            let removed = delete(verified(&setup, "home/cache/inner"), plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, ItemOutcome::Deleted);
            assert!(fs::symlink_metadata(setup.at("home/cache/inner")).is_err());
            assert!(setup.at("home/cache").is_dir());
            assert!(TWIN.iter().all(|twin| setup.at(twin).exists()));
        }

        #[test]
        fn a_permanent_delete_removes_a_folder_without_following_a_link_inside() {
            let setup = setup();
            let plan = permanent(&setup);

            let removed = delete(verified(&setup, "home/cache"), plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, ItemOutcome::Deleted);
            assert!(fs::symlink_metadata(setup.at("home/cache")).is_err());
            assert!(setup.at("outside/keep.txt").exists());
        }

        #[test]
        fn a_link_item_is_deleted_as_the_link() {
            let setup = setup();
            let plan = permanent(&setup);

            let removed = delete(verified(&setup, "home/link"), plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, ItemOutcome::Deleted);
            assert!(fs::symlink_metadata(setup.at("home/link")).is_err());
            assert!(setup.at("outside/keep.txt").exists());
        }

        #[test]
        fn a_file_is_deleted() {
            let setup = setup();
            let plan = permanent(&setup);

            let removed = delete(verified(&setup, "home/cache/a.bin"), plan.act(), plan.permanent().unwrap());

            assert_eq!(removed.outcome, ItemOutcome::Deleted);
            assert!(!setup.at("home/cache/a.bin").exists());
        }
    }
}
