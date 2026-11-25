//! Where items moved to the Trash are now, and putting one back. Put back removes nothing and never replaces
//! anything; its original place must pass the guard's path checks today.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt as _;

use fresher_core::{CleanupContext, PutBackOutcome, SkipReason, TrashError, TrashState, TrashedItem};

use crate::beneath::path_of;
use crate::guard::identity;
use crate::paths::{LastLink, Placed, check_protection, open_failure, open_no_link};
use crate::pipeline::Cleaning;

/// Where each item is: `InTrash` when its landed path (looked at without following a link at its end) holds its
/// identity, `Replaced` when something else is there, `Gone` when nothing is or it cannot be read.
pub(crate) fn locate(items: &[TrashedItem]) -> Vec<TrashState> {
    items.iter().map(state).collect()
}

fn state(item: &TrashedItem) -> TrashState {
    match fs::symlink_metadata(&item.landed) {
        Ok(metadata) if identity::of(&metadata) == item.identity => TrashState::InTrash,
        Ok(_) => TrashState::Replaced,
        Err(_) => TrashState::Gone,
    }
}

/// Moves one item back: it must still be in the Trash; its original path must be inside a current root, in reach,
/// not protected nor ignored, and its folder must open through no link on the root's volume and pass the protection
/// again as the disk spells it; then the `Trash` port
/// moves it, never replacing anything. Afterwards the original path must hold the item. A reach that does not act
/// (the dry run) refuses every item (`DryRun`) before any other check or port is called.
pub(crate) fn put_back(cleaning: &Cleaning<'_>, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
    match state(item) {
        TrashState::InTrash => {}
        TrashState::Gone => return PutBackOutcome::Gone,
        TrashState::Replaced => return PutBackOutcome::Replaced,
    }
    let refused = |reason: SkipReason| PutBackOutcome::Refused { reason: reason.code() };
    #[cfg(debug_assertions)]
    if !cleaning.reach.acts() {
        return refused(SkipReason::DryRun);
    }
    let roots = cleaning.roots(cx.roots, cx.roots);
    let placed = match roots.place(&item.original) {
        Ok(placed) => placed,
        Err(reason) => return refused(reason),
    };
    if !cleaning.reach.admits(&placed.root, &placed.relative, cleaning.user, cleaning.system) {
        return refused(SkipReason::NotInSandbox);
    }
    if let Err(reason) = check_protection(&placed, &[cleaning.system, cx.protection]) {
        return refused(reason);
    }
    let (Some(parent), Some(name)) = (placed.physical.parent(), placed.physical.file_name()) else {
        return refused(SkipReason::CheckFailed);
    };
    let folder = match open_no_link(parent, LastLink::Refuse) {
        Ok(folder) => folder,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return PutBackOutcome::NoParent,
        Err(error) => return refused(open_failure(&error)),
    };
    match folder.metadata() {
        Ok(metadata) if !metadata.is_dir() => return PutBackOutcome::NoParent,
        Ok(metadata) if metadata.dev() != placed.device => return refused(SkipReason::OtherVolume),
        Ok(_) => {}
        Err(_) => return refused(SkipReason::CheckFailed),
    }
    // The journal's path may spell a folder in another case or normal form than the disk does, which the protection,
    // comparing names as written, would not see: it is checked on the disk's own spelling of the folder too.
    let on_disk = match path_of(&folder) {
        Ok(path) => Placed { physical: path.join(name), ..placed.clone() },
        Err(_) => return refused(SkipReason::CheckFailed),
    };
    if let Err(reason) = check_protection(&on_disk, &[cleaning.system, cx.protection]) {
        return refused(reason);
    }
    match cleaning.ports.trash.put_back(&item.landed, &placed.physical) {
        Ok(()) => match fs::symlink_metadata(&placed.physical) {
            Ok(metadata) if identity::of(&metadata) == item.identity => PutBackOutcome::PutBack,
            _ => PutBackOutcome::Failed,
        },
        Err(TrashError::Occupied) => PutBackOutcome::Occupied,
        Err(TrashError::NoParent) => PutBackOutcome::NoParent,
        Err(TrashError::NotFound) => PutBackOutcome::Gone,
        Err(TrashError::ThroughLink) => refused(SkipReason::ThroughLink),
        Err(TrashError::NoTrash | TrashError::NotAllowed | TrashError::Refused { .. }) => PutBackOutcome::Failed,
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Action, CancelToken, Method, TrashedItem};

    use crate::test_support::Rig;

    const TREE: &str = r#"
            [[file]]
            path = "home/cache/a.bin"
            [[file]]
            path = "home/cache/b.bin"
            [[dir]]
            path = "elsewhere"
            [[dir]]
            path = "home/keep"
            "#;

    fn rig() -> Rig {
        Rig::new(TREE)
    }

    /// Moves `relative` of the rig to the Trash with a confirmed plan, and gives what was journalled.
    fn trashed(rig: &Rig, action: Action) -> Vec<TrashedItem> {
        let items = rig.items(vec![action]);
        let (done, _) = rig.execute(&items, Method::Trash, &CancelToken::new());
        done.into_iter().flat_map(|item| item.trashed).collect()
    }

    fn file(rig: &Rig) -> TrashedItem {
        trashed(rig, Action::Remove(rig.at("home/cache/a.bin"))).remove(0)
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;

        use fresher_core::{
            Cleaner as _, CleanupContext, Protection, PutBackOutcome, SkipCode, TrashState, TrashedItem,
        };

        use super::{file, rig};

        #[test]
        fn an_item_no_longer_in_the_trash_is_gone() {
            let rig = rig();
            let item = file(&rig);
            fs::rename(&item.landed, rig.at("elsewhere/a.bin")).unwrap();

            assert_eq!(rig.cleaner.locate(std::slice::from_ref(&item)), [TrashState::Gone]);
            assert_eq!(rig.cleaner.put_back(&item, &rig.cx()), PutBackOutcome::Gone);
            assert!(!rig.at("home/cache/a.bin").exists());
        }

        #[test]
        fn an_item_replaced_in_the_trash_is_replaced() {
            let rig = rig();
            let item = file(&rig);
            fs::rename(&item.landed, rig.at("elsewhere/a.bin")).unwrap();
            fs::write(&item.landed, b"another").unwrap();

            assert_eq!(rig.cleaner.locate(std::slice::from_ref(&item)), [TrashState::Replaced]);
            assert_eq!(rig.cleaner.put_back(&item, &rig.cx()), PutBackOutcome::Replaced);
            assert!(!rig.at("home/cache/a.bin").exists());
        }

        #[test]
        fn put_back_never_replaces_an_item_at_the_original_place() {
            let rig = rig();
            let item = file(&rig);
            fs::write(rig.at("home/cache/a.bin"), b"made again").unwrap();

            assert_eq!(rig.cleaner.put_back(&item, &rig.cx()), PutBackOutcome::Occupied);
            assert_eq!(fs::read(rig.at("home/cache/a.bin")).unwrap(), b"made again");
            assert_eq!(rig.cleaner.locate(std::slice::from_ref(&item)), [TrashState::InTrash]);
        }

        #[test]
        fn put_back_into_a_removed_folder_is_refused() {
            let rig = rig();
            let item = file(&rig);
            fs::rename(rig.at("home/cache"), rig.at("elsewhere/cache")).unwrap();

            assert_eq!(rig.cleaner.put_back(&item, &rig.cx()), PutBackOutcome::NoParent);
            assert_eq!(rig.cleaner.locate(std::slice::from_ref(&item)), [TrashState::InTrash]);
        }

        #[test]
        fn put_back_into_a_file_where_the_folder_was_is_refused() {
            let rig = rig();
            let item = file(&rig);
            fs::rename(rig.at("home/cache"), rig.at("elsewhere/cache")).unwrap();
            fs::write(rig.at("home/cache"), b"a file").unwrap();

            assert_eq!(rig.cleaner.put_back(&item, &rig.cx()), PutBackOutcome::NoParent);
        }

        #[test]
        fn put_back_outside_the_current_roots_is_refused() {
            let rig = rig();
            let item = file(&rig);
            let other = vec![rig.at("elsewhere")];
            let cx = CleanupContext { roots: &other, ..rig.cx() };

            assert_eq!(rig.cleaner.put_back(&item, &cx), PutBackOutcome::Refused { reason: SkipCode::OutsideRoots });
            assert_eq!(rig.cleaner.locate(std::slice::from_ref(&item)), [TrashState::InTrash]);
        }

        #[test]
        fn put_back_through_a_link_is_refused() {
            let rig = rig();
            let item = file(&rig);
            fs::rename(rig.at("home/cache"), rig.at("elsewhere/cache")).unwrap();
            symlink(rig.at("elsewhere"), rig.at("home/cache")).unwrap();

            assert_eq!(
                rig.cleaner.put_back(&item, &rig.cx()),
                PutBackOutcome::Refused { reason: SkipCode::ThroughLink }
            );
            assert!(!rig.at("elsewhere/a.bin").exists());
            assert_eq!(rig.cleaner.locate(std::slice::from_ref(&item)), [TrashState::InTrash]);
        }

        #[test]
        fn put_back_into_a_protected_place_is_refused() {
            let mut rig = rig();
            let item = file(&rig);
            rig.setup.protection = Protection::default().with_ignored(&[rig.at("home/cache")]).unwrap();

            assert_eq!(rig.cleaner.put_back(&item, &rig.cx()), PutBackOutcome::Refused { reason: SkipCode::Ignored });
        }

        #[test]
        #[cfg(debug_assertions)]
        fn a_dry_run_puts_nothing_back() {
            use fresher_core::{Trash as _, TrashedItem};

            let rig = crate::test_support::Rig::dry_run(super::TREE);
            let original = rig.at("home/cache/a.bin");
            let held = fs::File::open(&original).unwrap();
            let landed = rig.trash.move_to_trash(&original, &held).unwrap().at;
            let identity = crate::guard::identity::of(&fs::symlink_metadata(&landed).unwrap());
            let trashed = TrashedItem { original: original.clone(), landed: landed.clone(), identity };

            let outcome = rig.cleaner.put_back(&trashed, &rig.cx());

            assert_eq!(outcome, PutBackOutcome::Refused { reason: SkipCode::DryRun });
            assert_eq!(rig.cleaner.locate(std::slice::from_ref(&trashed)), [TrashState::InTrash]);
            assert!(landed.exists());
            assert!(!original.exists());
        }

        #[test]
        fn put_back_into_a_protected_folder_spelt_in_another_case_is_refused() {
            let mut rig = rig();
            let item = file(&rig);
            if !rig.at("HOME/KEEP").exists() {
                return;
            }
            let keep = fs::canonicalize(rig.at("home/keep")).unwrap();
            rig.setup.protection = Protection::new(vec![keep], vec![], vec![]).unwrap();
            let written = TrashedItem { original: rig.at("home/KEEP/a.bin"), ..item.clone() };

            assert_eq!(
                rig.cleaner.put_back(&written, &rig.cx()),
                PutBackOutcome::Refused { reason: SkipCode::Protected }
            );
            assert!(!rig.at("home/keep/a.bin").exists());
            assert_eq!(rig.cleaner.locate(&[item]), [TrashState::InTrash]);
        }

        #[test]
        fn put_back_outside_the_sandbox_is_refused() {
            let rig = rig();
            let item = file(&rig);
            let marker = rig.setup.tree.root().join(fresher_core::SANDBOX_MARKER);
            fs::rename(&marker, rig.at("elsewhere/marker")).unwrap();

            assert_eq!(
                rig.cleaner.put_back(&item, &rig.cx()),
                PutBackOutcome::Refused { reason: SkipCode::NotInSandbox }
            );
        }
    }

    mod positive {
        use std::os::unix::fs::MetadataExt as _;

        use fresher_core::{Action, Cleaner as _, PutBackOutcome, TrashState};

        use super::{file, rig, trashed};

        #[test]
        fn an_item_in_the_trash_is_located() {
            let rig = rig();
            let item = file(&rig);

            assert_eq!(rig.cleaner.locate(&[item.clone(), item]), [TrashState::InTrash, TrashState::InTrash]);
        }

        #[test]
        fn put_back_restores_the_item_with_its_identity() {
            let rig = rig();
            let item = file(&rig);

            assert_eq!(rig.cleaner.put_back(&item, &rig.cx()), PutBackOutcome::PutBack);
            let back = std::fs::symlink_metadata(rig.at("home/cache/a.bin")).unwrap();
            assert_eq!((back.dev(), back.ino()), (item.identity.device(), item.identity.file()));
            assert_eq!(rig.cleaner.locate(&[item]), [TrashState::Gone]);
        }

        #[test]
        fn put_back_of_an_emptied_folders_entry_restores_it_into_the_kept_folder() {
            let rig = rig();
            let entries = trashed(&rig, Action::Empty(rig.at("home/cache")));
            assert_eq!(entries.len(), 2);

            assert_eq!(rig.cleaner.put_back(&entries[1], &rig.cx()), PutBackOutcome::PutBack);
            assert!(rig.at("home/cache/b.bin").exists());
            assert!(!rig.at("home/cache/a.bin").exists());
        }
    }
}
