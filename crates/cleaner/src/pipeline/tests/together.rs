//! Items of several paths: every part checked before the first one moves.

use fresher_core::{CancelToken, ItemOutcome, Method, SkipReason};

use crate::test_support::{Entry, Rig, listing};

const TREE: &str = r#"
    [[file]]
    path = "home/a.bin"
    [[file]]
    path = "home/manifest"
    [[file]]
    path = "home/blobs/one.bin"
    [[file]]
    path = "home/blobs/two.bin"
    [[file]]
    path = "home/c.bin"
    [[file]]
    path = "T/layer.bin"
    "#;

/// The parts: the subject, then two layers; the tests refuse the last ones, so the first passed its checks.
const PARTS: [&str; 3] = ["home/manifest", "home/blobs/one.bin", "home/blobs/two.bin"];

fn rig() -> Rig {
    Rig::new(TREE)
}

/// The rig's together item of `PARTS`, with the identities a rehearsal of its new plan read.
fn item(rig: &Rig) -> Vec<fresher_core::PlanItem> {
    rig.items(vec![rig.together(&PARTS)])
}

/// Executing `items` with either method ends the together item skipped with `reason`, nothing in the Trash,
/// nothing moved, and the tree as `before`.
fn refused_whole(rig: &Rig, items: &[fresher_core::PlanItem], before: &[Entry], reason: &SkipReason) {
    for method in [Method::Permanent, Method::Trash] {
        let (done, _) = rig.execute(items, method, &CancelToken::new());

        assert_eq!(done[0].outcome, ItemOutcome::Skipped { reason: reason.clone() }, "{method:?}");
        assert_eq!(done[0].trashed, Vec::new(), "{method:?}");
    }
    assert_eq!(listing(rig.setup.tree.root()), before);
    assert_eq!(rig.trash.moved(), Vec::<std::path::PathBuf>::new());
}

mod negative {
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::Path;

    use fresher_core::{CancelToken, Cleaner as _, ItemOutcome, Method, Protection, SkipReason};
    use fresher_testing::FakeRunningApps;

    use super::{PARTS, item, refused_whole, rig};
    use crate::guard::Purpose;
    use crate::pipeline::act_on;
    use crate::test_support::{FixedWalk, Rig, confirmed, confirmed_over, listing};

    const MIB: u64 = 1024 * 1024;

    #[test]
    fn a_part_outside_the_roots_moves_no_part() {
        let rig = rig();
        rig.setup.tree.mark_sandbox(Path::new("home")).unwrap();
        rig.setup.tree.mark_sandbox(Path::new("T")).unwrap();
        let items = rig.items(vec![rig.together(&["home/manifest", "T/layer.bin"])]);
        let before = listing(rig.setup.tree.root());
        // Planned over both, but `T` is no longer a scan root when the cleanup starts.
        let plan_roots = vec![rig.at("home"), rig.at("T")];
        let home = vec![rig.at("home")];
        let cx = fresher_core::CleanupContext { roots: &home, ..rig.cx() };

        for method in [Method::Permanent, Method::Trash] {
            let plan = confirmed_over(&plan_roots, &items, method);
            let mut done = Vec::new();
            rig.cleaner.execute(plan, &cx, &CancelToken::new(), &mut |item| done.push(item));

            assert_eq!(done[0].outcome, ItemOutcome::Skipped { reason: SkipReason::NotInSandbox });
            assert_eq!(done[0].trashed, Vec::new());
        }
        assert_eq!(listing(rig.setup.tree.root()), before);
        assert_eq!(rig.trash.moved(), Vec::<std::path::PathBuf>::new());
    }

    #[test]
    fn a_protected_part_moves_no_part() {
        let mut rig = rig();
        let items = item(&rig);
        rig.setup.protection = Protection::new(vec![rig.at(PARTS[2])], vec![], vec![]).unwrap();
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::Protected);
    }

    #[test]
    fn a_part_reached_through_a_link_moves_no_part() {
        let rig = rig();
        let items = item(&rig);
        // The layers' folder swapped for a link after the rehearsal.
        fs::rename(rig.at("home/blobs"), rig.at("home/blobs-real")).unwrap();
        symlink("blobs-real", rig.at("home/blobs")).unwrap();
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::ThroughLink);
    }

    #[test]
    fn a_replaced_part_moves_no_part() {
        let rig = rig();
        let items = item(&rig);
        fs::write(rig.at("home/new.bin"), b"another file").unwrap();
        fs::rename(rig.at("home/new.bin"), rig.at(PARTS[2])).unwrap();
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::Replaced);
    }

    #[test]
    fn a_vanished_part_moves_no_part() {
        let rig = rig();
        let items = item(&rig);
        fs::rename(rig.at(PARTS[2]), rig.at("home/moved.bin")).unwrap();
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::Vanished);
    }

    #[test]
    fn a_grown_part_moves_no_part() {
        let mut rig = rig();
        let items = item(&rig);
        // Each part alone within the margin of the item's size, all of them together past it.
        rig.setup.walk = Box::new(FixedWalk::answering(Ok(33 * MIB)));
        let before = listing(rig.setup.tree.root());

        for method in [Method::Permanent, Method::Trash] {
            let plan = confirmed(&rig.setup, &items, method);
            let removed = act_on(&plan.items()[0], &rig.setup.cx(Purpose::Act), &plan, &rig.cleaner.ports);

            assert_eq!(removed.outcome, ItemOutcome::Skipped { reason: SkipReason::Grown }, "{method:?}");
            assert_eq!(removed.trashed, Vec::new());
        }
        assert_eq!(listing(rig.setup.tree.root()), before);
        assert_eq!(rig.trash.moved(), Vec::<std::path::PathBuf>::new());
    }

    #[test]
    fn a_part_held_open_moves_no_part() {
        let rig = rig();
        let items = item(&rig);
        rig.open_files.hold(rig.at(PARTS[2]), "Preview");
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::InUse { app: "Preview".into() });
    }

    /// A server that mapped a layer into memory: the system's reader lists mapped files with the open ones,
    /// so the fake names it as a holder like any other.
    #[test]
    fn a_mapped_part_moves_no_part() {
        let rig = rig();
        let items = item(&rig);
        rig.open_files.hold(rig.at(PARTS[1]), "ollama");
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::InUse { app: "ollama".into() });
    }

    #[test]
    fn a_part_a_running_app_owns_moves_no_part() {
        let notes = fresher_testing::running_app("Notes", &["two.bin"]).unwrap();
        // Not running when the plan is made, running when the cleanup starts.
        let rig = rig().with_running(FakeRunningApps::new(Vec::new()).then(vec![notes]));
        let items = item(&rig);
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::InUse { app: "Notes".into() });
    }

    #[test]
    fn an_item_without_its_parts_identities_moves_no_part() {
        let rig = rig();
        let items = vec![crate::test_support::plan_item(1, rig.together(&PARTS), 4096)];
        let before = listing(rig.setup.tree.root());

        refused_whole(&rig, &items, &before, &SkipReason::CheckFailed);
    }

    #[test]
    fn a_cancelled_walk_of_a_part_reaches_no_part() {
        let mut rig = rig();
        let items = item(&rig);
        rig.setup.walk = Box::new(FixedWalk::answering(Err(crate::guard::walk::WalkStop::Cancelled)));
        let plan = confirmed(&rig.setup, &items, Method::Trash);

        let removed = act_on(&plan.items()[0], &rig.setup.cx(Purpose::Act), &plan, &rig.cleaner.ports);

        assert_eq!(removed.outcome, ItemOutcome::NotReached);
        assert!(PARTS.iter().all(|part| rig.at(part).exists()));
    }

    #[test]
    #[cfg(debug_assertions)]
    fn a_dry_run_moves_no_part_after_every_check() {
        let mut rig = Rig::dry_run(super::TREE);
        let actions = vec![rig.together(&PARTS), rig.together(&["home/a.bin", "home/c.bin"])];
        let items = rig.items(actions);
        rig.setup.protection = Protection::new(vec![rig.at("home/c.bin")], vec![], vec![]).unwrap();
        let before = listing(rig.setup.tree.root());

        for method in [Method::Permanent, Method::Trash] {
            let (done, _) = rig.execute(&items, method, &CancelToken::new());

            assert_eq!(done[0].outcome, ItemOutcome::Skipped { reason: SkipReason::DryRun }, "{method:?}");
            assert_eq!(done[1].outcome, ItemOutcome::Skipped { reason: SkipReason::Protected }, "{method:?}");
            assert!(done.iter().all(|item| item.trashed.is_empty()));
        }
        assert_eq!(listing(rig.setup.tree.root()), before);
        assert_eq!(rig.trash.moved(), Vec::<std::path::PathBuf>::new());
    }
}

mod positive {
    use std::os::unix::fs::MetadataExt as _;

    use fresher_core::{
        Action, CancelToken, Cleaner as _, Depth, EntryKind, ItemIdentity, ItemOutcome, Method, Rehearsal,
    };

    use super::{PARTS, item, rig};
    use crate::test_support::{listing, physical_of};

    #[test]
    fn a_rehearsal_answers_every_parts_identity() {
        let rig = rig();
        let items = vec![crate::test_support::plan_item(1, rig.together(&PARTS), 4096)];
        let before = listing(rig.setup.tree.root());
        let identities: Vec<_> = PARTS
            .iter()
            .map(|part| {
                let metadata = std::fs::symlink_metadata(rig.at(part)).unwrap();
                ItemIdentity::new(metadata.dev(), metadata.ino(), EntryKind::File)
            })
            .collect();

        for depth in [Depth::Quick, Depth::Full] {
            let rehearsed = rig.cleaner.rehearse(&[rig.setup.tree.root().to_path_buf()], &items, &rig.cx(), depth);

            assert_eq!(rehearsed, [Rehearsal::PassedTogether(identities.clone())], "{depth:?}");
        }
        assert_eq!(item(&rig)[0].part_identities(), identities);
        assert_eq!(listing(rig.setup.tree.root()), before);
    }

    #[test]
    fn a_together_item_beside_single_items_keeps_the_plan_order() {
        let rig = rig();
        let actions =
            vec![Action::Remove(rig.at("home/a.bin")), rig.together(&PARTS), Action::Remove(rig.at("home/c.bin"))];
        let items = rig.items(actions);

        let (done, _) = rig.execute(&items, Method::Trash, &CancelToken::new());

        let seen: Vec<_> = done.iter().map(|item| (item.index, item.outcome.clone(), item.trashed.len())).collect();
        assert_eq!(seen, [(0, ItemOutcome::Moved, 1), (1, ItemOutcome::Moved, 3), (2, ItemOutcome::Moved, 1)]);
        let order = ["home/a.bin", PARTS[0], PARTS[1], PARTS[2], "home/c.bin"];
        let expected: Vec<_> = order.iter().map(|path| physical_of(&rig, &Action::Remove(rig.at(path)))).collect();
        assert_eq!(rig.trash.moved(), expected);
        let originals: Vec<_> = done[1].trashed.iter().map(|item| item.original.clone()).collect();
        assert_eq!(originals, PARTS.map(|part| rig.at(part)));
    }
}
