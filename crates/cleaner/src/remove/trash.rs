//! A move to the Trash, through the `Trash` port, and the check of where the item landed (the design's call 37).

use std::path::Path;

use fresher_core::{Act, Failure, ItemOutcome, SkipReason, Trash, TrashError, TrashedItem};

use crate::guard::Verified;
use crate::remove::{Removed, identity_at};

/// The names of the Trash folders: `~/.Trash` and a volume's `.Trashes`.
const TRASH_FOLDERS: [&str; 2] = [".Trash", ".Trashes"];

/// Moves the verified item to the Trash with its held handle, then checks where it went (the design's call 37):
///
/// - `Moved` to the reported landed path when that path holds the verified identity;
/// - else `Moved` to where the held handle's item is now (`at`), when that is not its original place, lies in a Trash
///   folder and holds it;
/// - else, when `at` names a place, the item is not in the Trash and the call moved something else: `Failed`;
/// - else (`at` is empty: the system could not say where the handle's item is) the original place decides: holding
///   the verified item still, it is `Failed` as above; empty or holding something else, the item left it and is
///   `Moved`, but where to is unknown, so nothing is journalled for put back.
///
/// What the call moved instead of the verified item is never journalled: where it came from is unknown, and put back
/// would move it to the verified item's place. Finder's own Put Back knows its place.
///
/// A volume without a Trash skips the item: never a permanent delete instead.
pub(crate) fn to_trash(verified: Verified, _act: Act<'_>, trash: &dyn Trash) -> Removed {
    let (handle, placed, identity) = verified.into_parts();
    let landed = match trash.move_to_trash(&placed.physical, handle.file()) {
        Ok(landed) => landed,
        Err(error) => return Removed::ended(refused(&error)),
    };
    let trashed = |landed: &Path, identity| TrashedItem {
        original: placed.lexical.clone(),
        landed: landed.to_path_buf(),
        identity,
    };
    let moved = |landed: &Path| Removed {
        outcome: ItemOutcome::Moved,
        trashed: vec![trashed(landed, identity)],
        status: None,
        went: None,
    };
    if let Some(reported) = landed.reported.as_deref().filter(|reported| identity_at(reported) == Some(identity)) {
        return moved(reported);
    }
    let located = !landed.at.as_os_str().is_empty();
    let left = landed.at != placed.physical && landed.at != placed.lexical;
    if located && left && in_trash_folder(&landed.at) && identity_at(&landed.at) == Some(identity) {
        return moved(&landed.at);
    }
    if !located && identity_at(&placed.physical) != Some(identity) {
        return Removed::ended(ItemOutcome::Moved);
    }
    Removed::ended(ItemOutcome::Failed { reason: Failure::LandedElsewhere.text() })
}

fn refused(error: &TrashError) -> ItemOutcome {
    match error {
        TrashError::NoTrash => ItemOutcome::Skipped { reason: SkipReason::NoTrash },
        TrashError::NotFound => ItemOutcome::Skipped { reason: SkipReason::Vanished },
        TrashError::ThroughLink => ItemOutcome::Skipped { reason: SkipReason::ThroughLink },
        TrashError::NotAllowed | TrashError::Occupied | TrashError::NoParent | TrashError::Refused { .. } => {
            ItemOutcome::Failed { reason: Failure::TrashRefused.text() }
        }
    }
}

fn in_trash_folder(path: &Path) -> bool {
    path.iter().any(|name| TRASH_FOLDERS.iter().any(|trash| name == *trash))
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::path::{Path, PathBuf};

    use fresher_core::{Landed, Trash, TrashError};

    /// A Trash that moves a decoy instead of the item, as the system's call does when a parent was swapped for a
    /// link in that instant.
    #[derive(Debug)]
    struct DecoyTrash {
        decoy: PathBuf,
        dir: PathBuf,
    }

    impl Trash for DecoyTrash {
        fn move_to_trash(&self, path: &Path, _held: &File) -> Result<Landed, TrashError> {
            fs::create_dir_all(&self.dir).unwrap();
            let landed = self.dir.join("decoy");
            fs::rename(&self.decoy, &landed).unwrap();
            Ok(Landed { at: path.to_path_buf(), reported: Some(landed) })
        }

        fn put_back(&self, _landed: &Path, _original: &Path) -> Result<(), TrashError> {
            Err(TrashError::NotAllowed)
        }
    }

    /// A Trash whose call moves the item (or not) and cannot say where the handle's item is: `at` is empty and no path
    /// is reported.
    #[derive(Debug)]
    struct Unlocated {
        moves: bool,
        dir: PathBuf,
    }

    impl Trash for Unlocated {
        fn move_to_trash(&self, path: &Path, _held: &File) -> Result<Landed, TrashError> {
            if self.moves {
                fs::create_dir_all(&self.dir).unwrap();
                fs::rename(path, self.dir.join("somewhere")).unwrap();
            }
            Ok(Landed { at: PathBuf::new(), reported: None })
        }

        fn put_back(&self, _landed: &Path, _original: &Path) -> Result<(), TrashError> {
            Err(TrashError::NotAllowed)
        }
    }

    mod negative {
        use std::fs;

        use fresher_core::{Failure, ItemOutcome, Method, SkipReason};
        use fresher_testing::FakeTrash;

        use super::{DecoyTrash, Unlocated};
        use crate::remove::trash::to_trash;
        use crate::test_support::{Setup, confirmed, verified};

        fn setup() -> Setup {
            Setup::new("[[file]]\npath = \"home/a.bin\"\n\n[[file]]\npath = \"home/decoy.bin\"\n")
        }

        #[test]
        fn a_volume_without_a_trash_skips_and_deletes_nothing() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at(".Trash")).without_trash();

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Skipped { reason: SkipReason::NoTrash });
            assert_eq!(removed.trashed, Vec::new());
            assert!(setup.at("home/a.bin").exists());
        }

        #[test]
        fn a_refused_move_fails_and_leaves_the_item() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at(".Trash")).refusing(setup.physical("home/a.bin"));

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Failed { reason: Failure::TrashRefused.text() });
            assert!(setup.at("home/a.bin").exists());
        }

        #[test]
        fn an_item_gone_before_the_move_has_vanished() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let item = verified(&setup, "home/a.bin");
            fs::rename(setup.at("home/a.bin"), setup.at("moved.bin")).unwrap();

            let removed = to_trash(item, plan.act(), &FakeTrash::new(setup.at(".Trash")));

            assert_eq!(removed.outcome, ItemOutcome::Skipped { reason: SkipReason::Vanished });
        }

        #[test]
        fn a_landed_item_that_is_not_the_verified_one_fails_and_is_not_journalled() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = DecoyTrash { decoy: setup.at("home/decoy.bin"), dir: setup.at(".Trash") };

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Failed { reason: Failure::LandedElsewhere.text() });
            assert_eq!(removed.trashed, Vec::new(), "never journalled to come back at the verified item's place");
            assert!(setup.at(".Trash/decoy").exists());
            assert!(setup.at("home/a.bin").exists());
        }

        #[test]
        fn an_item_that_never_left_a_folder_named_like_a_trash_is_not_moved() {
            let setup =
                Setup::new("[[file]]\npath = \"home/app/.Trash/a.bin\"\n\n[[file]]\npath = \"home/decoy.bin\"\n");
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = DecoyTrash { decoy: setup.at("home/decoy.bin"), dir: setup.at(".Trash") };

            let removed = to_trash(verified(&setup, "home/app/.Trash/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Failed { reason: Failure::LandedElsewhere.text() });
            assert!(setup.at("home/app/.Trash/a.bin").exists());
        }

        #[test]
        fn an_unlocated_move_that_left_the_item_in_place_fails() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = Unlocated { moves: false, dir: setup.at(".Trash") };

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Failed { reason: Failure::LandedElsewhere.text() });
            assert_eq!(removed.trashed, Vec::new());
        }

        #[test]
        fn a_handle_found_outside_the_trash_is_not_moved() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at("not-a-trash")).reporting_nothing();

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Failed { reason: Failure::LandedElsewhere.text() });
        }
    }

    mod positive {
        use fresher_core::{ItemOutcome, Method};
        use fresher_testing::FakeTrash;

        use crate::remove::identity_at;
        use crate::remove::trash::to_trash;
        use crate::test_support::{Setup, confirmed, verified};

        fn setup() -> Setup {
            Setup::new("[[file]]\npath = \"home/a.bin\"\n")
        }

        #[test]
        fn an_item_goes_to_the_trash_and_its_landed_path_is_returned() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let identity = identity_at(&setup.at("home/a.bin")).unwrap();

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &FakeTrash::new(setup.at(".Trash")));

            assert_eq!(removed.outcome, ItemOutcome::Moved);
            assert_eq!(removed.trashed[0].landed, setup.at(".Trash/0-a.bin"));
            assert_eq!(removed.trashed[0].identity, identity);
            assert_eq!(removed.trashed[0].original, setup.at("home/a.bin"));
            assert!(!setup.at("home/a.bin").exists());
        }

        #[test]
        fn an_unlocated_move_is_moved_with_nothing_to_put_back() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = super::Unlocated { moves: true, dir: setup.at(".Trash") };

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Moved);
            assert_eq!(removed.trashed, Vec::new());
        }

        #[test]
        fn a_move_without_a_reported_path_lands_where_the_held_handle_is() {
            let setup = setup();
            let plan = confirmed(&setup, &[], Method::Trash);
            let trash = FakeTrash::new(setup.at(".Trash")).reporting_nothing();

            let removed = to_trash(verified(&setup, "home/a.bin"), plan.act(), &trash);

            assert_eq!(removed.outcome, ItemOutcome::Moved);
            assert_eq!(removed.trashed[0].landed, setup.at(".Trash/0-a.bin"));
        }
    }
}
