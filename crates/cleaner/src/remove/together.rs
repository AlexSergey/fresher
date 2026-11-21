//! Several paths that leave as one item: every part was verified before this is called, and each goes in order, the
//! subject first, the way the plan says.

use fresher_core::{Act, CancelToken, Failure, ItemOutcome, PartlyWay, SkipReason, TrashedItem};

use crate::guard::Verified;
use crate::remove::{Removed, Way, delete, to_trash};

/// Acts on the verified parts in order, each with its own `Verified`: to the Trash, or deleted at the single deletion
/// point. The cancel token is read before each part. Every part gone: `Moved` (or `Deleted`) with what each left in
/// the Trash. The first part not gone ends it, and nothing more is tried: before any part went, it is that part's own
/// outcome (`NotReached` for a cancel); after, `Failed` ("only part of it went"), with every part that went in
/// `trashed` so it is journalled and can be put back. It never falls back from the Trash to a delete.
pub(crate) fn together(verified: Vec<Verified>, act: Act<'_>, way: Way<'_>, cancel: &CancelToken) -> Removed {
    if verified.is_empty() {
        return Removed::ended(ItemOutcome::Skipped { reason: SkipReason::CheckFailed });
    }
    let of = u32::try_from(verified.len()).unwrap_or(u32::MAX);
    let mut done = 0_u32;
    let mut trashed = Vec::new();
    for part in verified {
        if cancel.is_cancelled() {
            return stopped(Removed::ended(ItemOutcome::NotReached), done, of, way, trashed);
        }
        let removed = match way {
            Way::Trash(trash) => to_trash(part, act, trash),
            Way::Permanent(consent) => delete(part, act, consent),
        };
        if !matches!(removed.outcome, ItemOutcome::Moved | ItemOutcome::Deleted) {
            return stopped(removed, done, of, way, trashed);
        }
        trashed.extend(removed.trashed);
        done = done.saturating_add(1);
    }
    let outcome = match way {
        Way::Trash(_) => ItemOutcome::Moved,
        Way::Permanent(_) => ItemOutcome::Deleted,
    };
    Removed { outcome, trashed, status: None, went: Some(of) }
}

/// The end of an item whose part `done` (counting from 0) did not go, as `removed` says: its own end when no part went
/// before it, else only part of the item went. A part whose removal left its place all the same (it landed elsewhere,
/// with an entry to put it back from) counts among the parts that went, in `went` and in the text alike.
fn stopped(removed: Removed, done: u32, of: u32, way: Way<'_>, mut trashed: Vec<TrashedItem>) -> Removed {
    let left = !removed.trashed.is_empty();
    let went = if left { done.saturating_add(1) } else { done };
    if done == 0 {
        return Removed { went: left.then_some(went), ..removed };
    }
    trashed.extend(removed.trashed);
    let way = match way {
        Way::Trash(_) => PartlyWay::Trash,
        Way::Permanent(_) => PartlyWay::Deleted,
    };
    Removed {
        outcome: ItemOutcome::Failed { reason: Failure::Partly { done: went, of, way }.text() },
        trashed,
        status: None,
        went: Some(went),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::guard::Verified;
    use crate::test_support::{Setup, verified};

    /// Three parts: a file (the subject), a folder with a file in it, and another file.
    const PARTS: [&str; 3] = ["home/manifest", "home/blobs", "home/other.bin"];

    fn setup() -> Setup {
        Setup::new(
            r#"
            [[file]]
            path = "home/manifest"
            [[file]]
            path = "home/blobs/layer"
            [[file]]
            path = "home/other.bin"
            [[file]]
            path = "home/keep.bin"
            "#,
        )
    }

    fn parts(setup: &Setup) -> Vec<Verified> {
        PARTS.iter().map(|part| verified(setup, part)).collect()
    }

    fn physical(setup: &Setup) -> Vec<PathBuf> {
        PARTS.iter().map(|part| setup.physical(part)).collect()
    }

    mod negative {
        use std::fs::{self, File};
        use std::path::{Path, PathBuf};
        use std::sync::{Arc, Mutex};

        use fresher_core::{
            CancelToken, Failure, ItemOutcome, Landed, Method, PartlyWay, SkipReason, Trash, TrashError,
        };
        use fresher_testing::FakeTrash;

        use super::{PARTS, parts, physical, setup};
        use crate::remove::{Way, together};
        use crate::test_support::{Meddling, confirmed, listing};

        #[test]
        fn no_verified_part_acts_on_nothing() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at(".Trash"));

            let removed = together(Vec::new(), plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Skipped { reason: SkipReason::CheckFailed });
            assert_eq!(trash.moved(), Vec::<PathBuf>::new());
        }

        #[test]
        fn the_trash_refusing_the_second_part_ends_partly_with_the_subject_journalled() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at(".Trash")).refusing(setup.physical(PARTS[1]));

            let removed = together(parts(&setup), plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert_eq!(
                removed.outcome,
                ItemOutcome::Failed { reason: Failure::Partly { done: 1, of: 3, way: PartlyWay::Trash }.text() }
            );
            assert_eq!(removed.went, Some(1));
            assert_eq!(trash.moved(), physical(&setup)[..1]);
            assert_eq!(removed.trashed.len(), 1);
            assert_eq!(removed.trashed[0].original, setup.at(PARTS[0]));
            assert!(removed.trashed[0].landed.exists());
            assert!(!setup.at(PARTS[0]).exists());
            assert!(setup.at("home/blobs/layer").exists() && setup.at(PARTS[2]).exists());
        }

        #[test]
        fn a_cancel_between_parts_ends_partly_and_tries_no_more() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let cancel = CancelToken::new();
            let inner = Arc::new(FakeTrash::new(setup.at(".Trash")));
            let trash = Meddling::new(Arc::clone(&inner), {
                let cancel = cancel.clone();
                move || cancel.cancel()
            });

            let removed = together(parts(&setup), plan.act(), Way::Trash(&trash), &cancel);

            assert_eq!(
                removed.outcome,
                ItemOutcome::Failed { reason: Failure::Partly { done: 1, of: 3, way: PartlyWay::Trash }.text() }
            );
            assert_eq!(removed.went, Some(1));
            assert_eq!(inner.moved(), physical(&setup)[..1]);
            assert_eq!(removed.trashed.len(), 1);
            assert!(setup.at(PARTS[1]).exists() && setup.at(PARTS[2]).exists());
        }

        #[test]
        fn a_cancel_before_the_first_part_moves_no_part() {
            let setup = setup();
            let before = listing(setup.tree.root());
            let plan = confirmed(&setup, &[], Method::Permanent);
            let cancel = CancelToken::new();
            cancel.cancel();

            let removed = together(parts(&setup), plan.act(), Way::Permanent(plan.permanent().unwrap()), &cancel);

            assert_eq!(removed.outcome, ItemOutcome::NotReached);
            assert_eq!(removed.trashed, Vec::new());
            assert_eq!(listing(setup.tree.root()), before);
        }

        #[test]
        fn a_failing_first_part_is_that_parts_own_failure() {
            let setup = setup();
            let before = listing(setup.tree.root());
            let plan = confirmed(&setup, &[], Method::Trash);
            let refusing = FakeTrash::new(setup.at(".Trash")).refusing(setup.physical(PARTS[0]));
            let missing = FakeTrash::new(setup.at(".Trash")).without_trash();

            let refused = together(parts(&setup), plan.act(), Way::Trash(&refusing), &CancelToken::new());
            let no_trash = together(parts(&setup), plan.act(), Way::Trash(&missing), &CancelToken::new());

            assert_eq!(refused.outcome, ItemOutcome::Failed { reason: Failure::TrashRefused.text() });
            assert_eq!(no_trash.outcome, ItemOutcome::Skipped { reason: SkipReason::NoTrash });
            assert_eq!((refused.trashed, no_trash.trashed), (Vec::new(), Vec::new()));
            assert_eq!(refusing.moved(), Vec::<PathBuf>::new());
            assert_eq!(listing(setup.tree.root()), before);
            assert_eq!((refused.went, no_trash.went), (None, None));
        }

        #[test]
        fn a_permanent_delete_failing_on_the_second_part_ends_partly_without_the_trash_text() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Permanent);
            let verified = parts(&setup);
            fs::rename(setup.at(PARTS[1]), setup.at("home/gone")).unwrap();

            let removed =
                together(verified, plan.act(), Way::Permanent(plan.permanent().unwrap()), &CancelToken::new());

            let deleted = Failure::Partly { done: 1, of: 3, way: PartlyWay::Deleted }.text();
            assert_eq!(removed.outcome, ItemOutcome::Failed { reason: deleted });
            assert_ne!(
                removed.outcome,
                ItemOutcome::Failed { reason: Failure::Partly { done: 1, of: 3, way: PartlyWay::Trash }.text() }
            );
            assert_eq!(removed.went, Some(1));
            assert_eq!(removed.trashed, Vec::new());
            assert!(!setup.at(PARTS[0]).exists() && setup.at(PARTS[2]).exists());
        }

        /// A Trash that moves every part as `inner` does but the second, which it moves to `away`, outside any Trash
        /// folder, reporting `decoy` as where it landed.
        struct ElsewhereOnSecond {
            inner: FakeTrash,
            away: PathBuf,
            decoy: PathBuf,
            calls: Mutex<u32>,
        }

        impl Trash for ElsewhereOnSecond {
            fn move_to_trash(&self, path: &Path, held: &File) -> Result<Landed, TrashError> {
                let call = {
                    let mut calls = self.calls.lock().unwrap();
                    *calls += 1;
                    *calls
                };
                if call != 2 {
                    return self.inner.move_to_trash(path, held);
                }
                fs::create_dir_all(&self.away).unwrap();
                let at = self.away.join("part");
                fs::rename(path, &at).unwrap();
                Ok(Landed { at, reported: Some(self.decoy.clone()) })
            }

            fn put_back(&self, landed: &Path, original: &Path) -> Result<(), TrashError> {
                self.inner.put_back(landed, original)
            }
        }

        #[test]
        fn a_part_that_landed_elsewhere_counts_among_the_parts_that_went() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = ElsewhereOnSecond {
                inner: FakeTrash::new(setup.at(".Trash")),
                away: setup.at("away"),
                decoy: setup.at("home/keep.bin"),
                calls: Mutex::new(0),
            };

            let removed = together(parts(&setup), plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert_eq!(
                removed.outcome,
                ItemOutcome::Failed { reason: Failure::Partly { done: 2, of: 3, way: PartlyWay::Trash }.text() }
            );
            assert_eq!(removed.went, Some(2), "the part left its place");
            let originals: Vec<_> = removed.trashed.iter().map(|item| item.original.clone()).collect();
            assert_eq!(originals, [setup.at(PARTS[0]), setup.at(PARTS[1])]);
            assert!(setup.at(PARTS[2]).exists());
        }
    }

    mod positive {
        use fresher_core::{CancelToken, ItemOutcome, Method};
        use fresher_testing::FakeTrash;

        use super::{PARTS, parts, physical, setup};
        use crate::remove::{Way, together};
        use crate::test_support::confirmed;

        #[test]
        fn every_part_goes_to_the_trash_the_subject_first() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at(".Trash"));

            let removed = together(parts(&setup), plan.act(), Way::Trash(&trash), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Moved);
            assert_eq!(removed.went, Some(3));
            assert_eq!(trash.moved(), physical(&setup));
            let originals: Vec<_> = removed.trashed.iter().map(|item| item.original.clone()).collect();
            assert_eq!(originals, PARTS.map(|part| setup.at(part)));
            assert!(removed.trashed.iter().all(|item| item.landed.exists()));
            assert!(PARTS.iter().all(|part| !setup.at(part).exists()));
            assert!(setup.at("home/keep.bin").exists());
        }

        #[test]
        fn the_permanent_way_deletes_each_part_through_the_deletion_point() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Permanent);

            let removed =
                together(parts(&setup), plan.act(), Way::Permanent(plan.permanent().unwrap()), &CancelToken::new());

            assert_eq!(removed.outcome, ItemOutcome::Deleted);
            assert_eq!(removed.trashed, Vec::new());
            assert!(PARTS.iter().all(|part| std::fs::symlink_metadata(setup.at(part)).is_err()));
            assert!(setup.at("home/keep.bin").exists() && !setup.at(".Trash").exists());
        }
    }
}
