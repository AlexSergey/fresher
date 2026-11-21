//! An emptied folder: each of its entries goes on its own, checked again beneath the verified folder; the folder
//! stays.

use fresher_core::{Act, CancelToken, EntryKind, ItemOutcome, PermanentConsent, SkipReason, Trash};

use crate::beneath::{self, entries};
use crate::guard::{self, Verified};
use crate::paths::open_failure;
use crate::remove::{Removed, delete_entry, to_trash};

/// Where the entries of a removal of several go: an emptied folder's, or the parts of an item of several paths.
#[derive(Clone, Copy)]
pub(crate) enum Way<'a> {
    Trash(&'a dyn Trash),
    Permanent(&'a PermanentConsent),
}

/// Empties the verified folder: its place must still hold it, then its entries are listed from its held handle and
/// each is checked against the identity the listing gave right before it goes — moved to the Trash (opened beneath
/// the folder through no link, checks 5 to 7) or deleted beneath the held handle. The cancel token is read between
/// entries. `Emptied { moved, skipped }` counts the entries that went and those that did not; what went to the Trash
/// is returned for the journal.
pub(crate) fn empty(folder: Verified, act: Act<'_>, way: Way<'_>, cancel: &CancelToken) -> Removed {
    if let Err(reason) = in_place(&folder) {
        return Removed::ended(ItemOutcome::Skipped { reason });
    }
    let Ok(entries) = entries(folder.handle().file()) else {
        return Removed::ended(ItemOutcome::Skipped { reason: SkipReason::CheckFailed });
    };
    let (mut moved, mut skipped) = (0_u32, 0_u32);
    let mut trashed = Vec::new();
    for entry in entries {
        if cancel.is_cancelled() {
            break;
        }
        let Some(listed) = entry.identity else {
            skipped = skipped.saturating_add(1);
            continue;
        };
        let removed = match way {
            Way::Trash(trash) => match guard::verify_entry(&folder, &entry.name, listed) {
                Ok(entry) => to_trash(entry, act, trash),
                Err(_) => Removed::ended(ItemOutcome::NotReached),
            },
            Way::Permanent(consent) => delete_entry(&folder, &entry.name, listed, act, consent),
        };
        if matches!(removed.outcome, ItemOutcome::Moved | ItemOutcome::Deleted) {
            moved = moved.saturating_add(1);
        } else {
            skipped = skipped.saturating_add(1);
        }
        trashed.extend(removed.trashed);
    }
    drop(folder);
    Removed { outcome: ItemOutcome::Emptied { moved, skipped }, trashed, status: None, went: None }
}

/// The folder's place, its parent opened through no link, still holds the verified folder; a link there is
/// `ThroughLink`, anything else `Replaced`.
fn in_place(folder: &Verified) -> Result<(), SkipReason> {
    let physical = &folder.placed().physical;
    let (Some(parent), Some(name)) = (physical.parent(), physical.file_name()) else {
        return Err(SkipReason::CheckFailed);
    };
    let parent = beneath::open_folder(parent).map_err(|error| open_failure(&error))?;
    let found = beneath::identity_at(&parent, name).map_err(|error| open_failure(&error))?;
    if found == folder.identity() {
        Ok(())
    } else if found.kind() == EntryKind::Symlink {
        Err(SkipReason::ThroughLink)
    } else {
        Err(SkipReason::Replaced)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::CancelToken;
    use fresher_testing::FakeTrash;

    use crate::test_support::Meddling;

    /// A Trash that swaps `entry` for a link to `target` on its first move.
    fn swapping(trash: PathBuf, entry: PathBuf, target: PathBuf) -> Meddling {
        Meddling::new(Arc::new(FakeTrash::new(trash)), move || {
            fs::rename(&entry, entry.with_extension("away")).unwrap();
            symlink(&target, &entry).unwrap();
        })
    }

    /// A Trash that cancels the cleanup on its first move.
    fn cancelling(trash: PathBuf, cancel: CancelToken) -> Meddling {
        Meddling::new(Arc::new(FakeTrash::new(trash)), move || cancel.cancel())
    }

    mod negative {
        use std::fs;

        use fresher_core::{CancelToken, ItemOutcome, Method, SkipReason};

        use super::{cancelling, swapping};
        use crate::remove::{Way, empty};
        use crate::test_support::{Flags, Setup, confirmed, verified};

        fn setup() -> Setup {
            Setup::new(
                r#"
                [[file]]
                path = "home/cache/a.bin"
                [[file]]
                path = "home/cache/b.bin"
                [[file]]
                path = "home/cache/c.bin"
                [[file]]
                path = "outside/keep.txt"
                "#,
            )
        }

        #[test]
        fn an_emptied_folder_skips_an_entry_swapped_for_a_link() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = swapping(setup.at(".Trash"), setup.physical("home/cache/b.bin"), setup.at("outside/keep.txt"));

            let removed = empty(verified(&setup, "home/cache"), plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Emptied { moved: 2, skipped: 1 });
            assert!(fs::symlink_metadata(setup.at("home/cache/b.bin")).unwrap().file_type().is_symlink());
            assert!(setup.at("outside/keep.txt").exists());
            assert_eq!(removed.trashed.len(), 2);
        }

        #[test]
        fn a_folder_swapped_for_a_link_after_the_check_is_not_emptied_for_good() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Permanent);
            let folder = verified(&setup, "home/cache");
            fs::rename(setup.at("home/cache"), setup.at("home/away")).unwrap();
            std::os::unix::fs::symlink(setup.at("outside"), setup.at("home/cache")).unwrap();

            let removed = empty(folder, plan.act(), Way::Permanent(plan.permanent().unwrap()), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Skipped { reason: SkipReason::ThroughLink });
            assert!(setup.at("outside/keep.txt").exists());
            assert_eq!(fs::read_dir(setup.at("home/away")).unwrap().count(), 3);
        }

        #[test]
        fn an_entry_the_system_refuses_to_delete_is_skipped_and_the_others_go() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Permanent);
            let _flags = Flags::lock(&setup.at("home/cache/b.bin"));

            let removed = empty(
                verified(&setup, "home/cache"),
                plan.act(),
                Way::Permanent(plan.permanent().unwrap()),
                &CancelToken::new(),
            );

            assert_eq!(removed.outcome, ItemOutcome::Emptied { moved: 2, skipped: 1 });
            assert!(setup.at("home/cache/b.bin").exists());
            assert!(!setup.at("home/cache/a.bin").exists() && !setup.at("home/cache/c.bin").exists());
        }

        #[test]
        fn an_emptied_folder_stops_between_entries_when_cancelled() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let cancel = CancelToken::new();
            let trash = cancelling(setup.at(".Trash"), cancel.clone());

            let removed = empty(verified(&setup, "home/cache"), plan.act(), Way::Trash(&trash), &cancel);

            assert_eq!(removed.outcome, ItemOutcome::Emptied { moved: 1, skipped: 0 });
            assert!(!setup.at("home/cache/a.bin").exists());
            assert!(setup.at("home/cache/b.bin").exists() && setup.at("home/cache/c.bin").exists());
        }

        #[test]
        fn an_entry_without_a_trash_is_skipped() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = fresher_testing::FakeTrash::new(setup.at(".Trash")).without_trash();

            let removed = empty(verified(&setup, "home/cache"), plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Emptied { moved: 0, skipped: 3 });
            assert_eq!(removed.trashed, Vec::new());
        }

        #[test]
        fn a_folder_that_is_gone_is_not_emptied() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let folder = verified(&setup, "home/cache");
            fs::rename(setup.at("home/cache"), setup.at("moved")).unwrap();
            let trash = fresher_testing::FakeTrash::new(setup.at(".Trash"));

            let removed = empty(folder, plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert!(matches!(removed.outcome, ItemOutcome::Skipped { .. }));
            assert_eq!(trash.moved(), Vec::<std::path::PathBuf>::new());
        }
    }

    mod positive {
        use fresher_core::{CancelToken, ItemOutcome, Method};
        use fresher_testing::FakeTrash;

        use crate::remove::{Way, empty};
        use crate::test_support::{Setup, confirmed, verified};

        fn setup() -> Setup {
            Setup::new(
                r#"
                [[file]]
                path = "home/cache/a.bin"
                [[file]]
                path = "home/cache/inner/b.bin"
                [[symlink]]
                path = "home/cache/link"
                target = "../../outside"
                [[file]]
                path = "outside/keep.txt"
                "#,
            )
        }

        #[test]
        fn an_emptied_folder_stays_and_its_entries_go() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at(".Trash"));

            let removed = empty(verified(&setup, "home/cache"), plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Emptied { moved: 3, skipped: 0 });
            assert!(setup.at("home/cache").is_dir());
            assert_eq!(std::fs::read_dir(setup.at("home/cache")).unwrap().count(), 0);
            let originals: Vec<_> = removed.trashed.iter().map(|item| item.original.clone()).collect();
            assert_eq!(
                originals,
                [setup.at("home/cache/a.bin"), setup.at("home/cache/inner"), setup.at("home/cache/link")]
            );
            assert!(setup.at("outside/keep.txt").exists());
        }

        #[test]
        fn an_emptied_folder_can_be_emptied_for_good() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Permanent);
            let consent = plan.permanent().unwrap();

            let removed =
                empty(verified(&setup, "home/cache"), plan.act(), Way::Permanent(consent), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Emptied { moved: 3, skipped: 0 });
            assert_eq!(removed.trashed, Vec::new());
            assert!(setup.at("home/cache").is_dir());
            assert!(setup.at("outside/keep.txt").exists());
        }
    }
}
