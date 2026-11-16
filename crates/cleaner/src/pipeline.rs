//! The cleaner's two entry points: the rehearsal, which checks and does nothing, and the execution of a confirmed
//! plan, item by item in plan order, never in parallel.

use std::path::{Path, PathBuf};
use std::time::Duration;

use fresher_core::{
    Action, CancelToken, CleanupContext, ConfirmedPlan, Depth, Ended, ItemDone, ItemOutcome, OpenSet, PlanItem,
    Protection, Rehearsal, RunningSet, SkipReason,
};

use crate::CleanerPorts;
use crate::guard::walk::HeldWalk;
use crate::guard::{GuardContext, Purpose, Unverified, verify, verify_tool};
use crate::paths::{Roots, expand_home};
use crate::reach::Reach;
use crate::remove::{Removed, Way, delete, empty, run_tool, to_trash};

/// The snapshot of open files and running apps is taken again when this much time has passed on the clock.
pub(crate) const SNAPSHOT_AGE: Duration = Duration::from_secs(2);

/// What is in use at one moment: the open files under the roots and the running apps.
#[derive(Debug)]
struct InUse {
    open: OpenSet,
    running: RunningSet,
}

impl InUse {
    /// Both taken now; `None` when either cannot be: nothing is acted on without them.
    fn take(ports: &CleanerPorts, within: &[PathBuf]) -> Option<Self> {
        let open = ports.open_files.snapshot(within).ok()?;
        let running = RunningSet::new(ports.running.list().ok()?);
        Some(Self { open, running })
    }
}

/// What the cleaner was made with.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cleaning<'a> {
    pub(crate) reach: &'a Reach,
    pub(crate) user: u32,
    pub(crate) home: &'a Path,
    pub(crate) system: &'a Protection,
    pub(crate) ports: &'a CleanerPorts,
}

impl Cleaning<'_> {
    /// The roots an item may lie in: the plan's that the settings still hold, both read with `~` as the home.
    pub(crate) fn roots(&self, plan: &[PathBuf], current: &[PathBuf]) -> Roots {
        Roots::new(&expand_home(plan, self.home), &expand_home(current, self.home))
    }

    fn guard<'a>(
        &'a self,
        cx: &'a CleanupContext<'_>,
        roots: &'a Roots,
        in_use: &'a InUse,
        purpose: Purpose,
        cancel: &'a CancelToken,
    ) -> GuardContext<'a> {
        GuardContext {
            reach: self.reach,
            user: self.user,
            system: self.system,
            protection: cx.protection,
            roots,
            access: cx.access,
            guarded: cx.guarded,
            open: &in_use.open,
            running: &in_use.running,
            purpose,
            walk: &HeldWalk,
            cancel,
        }
    }
}

/// Every check of every item, in order, and nothing done: one snapshot of open files and running apps, then each
/// item's identity or
/// why it would be skipped. It holds an unconfirmed plan's items and no `Act`, so it cannot call a removal. A snapshot
/// that cannot be taken skips every item (`CheckFailed`).
pub(crate) fn rehearse(
    cleaning: &Cleaning<'_>,
    plan_roots: &[PathBuf],
    items: &[PlanItem],
    cx: &CleanupContext<'_>,
    depth: Depth,
) -> Vec<Rehearsal> {
    let roots = cleaning.roots(plan_roots, cx.roots);
    let Some(in_use) = InUse::take(cleaning.ports, &roots.within()) else {
        return items.iter().map(|_| Rehearsal::Skipped(SkipReason::CheckFailed)).collect();
    };
    let never = CancelToken::new();
    let guard = cleaning.guard(cx, &roots, &in_use, Purpose::Rehearse, &never);
    let rehearse_one = |item: &PlanItem| match item.action() {
        Action::Tool(command) => match verify_tool(command, &guard) {
            Ok(()) => Rehearsal::Passed(None),
            Err(reason) => Rehearsal::Skipped(reason),
        },
        Action::Remove(_) | Action::Empty(_) => match verify(item, &guard, depth) {
            Ok(verified) => Rehearsal::Passed(Some(verified.identity())),
            Err(Unverified::Skipped(reason)) => Rehearsal::Skipped(reason),
            Err(Unverified::Cancelled) => Rehearsal::Skipped(SkipReason::CheckFailed),
        },
    };
    items.iter().map(rehearse_one).collect()
}

/// Acts on the confirmed plan's items one at a time, in plan order: the cancel token (once set, every remaining item
/// is `NotReached`), the snapshot of open files taken again when `SNAPSHOT_AGE` has passed, every check with the
/// folder walk, then the item's removal; `out` gets each item's end before the next one starts. A snapshot that cannot
/// be taken skips every item from then on (`CheckFailed`): nothing is acted on without it.
pub(crate) fn execute(
    cleaning: &Cleaning<'_>,
    plan: &ConfirmedPlan,
    cx: &CleanupContext<'_>,
    cancel: &CancelToken,
    out: &mut dyn FnMut(ItemDone),
) -> Ended {
    let ports = cleaning.ports;
    let roots = cleaning.roots(plan.plan().roots(), cx.roots);
    let within = roots.within();
    let mut snapshot = InUse::take(ports, &within);
    let mut taken = ports.clock.now();
    for (index, item) in (0..).zip(plan.items()) {
        let now = ports.clock.now();
        if now.duration_since(taken).map_or(true, |age| age >= SNAPSHOT_AGE) {
            snapshot = snapshot.and_then(|_| InUse::take(ports, &within));
            taken = now;
        }
        let removed = if cancel.is_cancelled() {
            Removed::ended(ItemOutcome::NotReached)
        } else if let Some(in_use) = &snapshot {
            let guard = cleaning.guard(cx, &roots, in_use, Purpose::Act, cancel);
            act_on(item, &guard, plan, ports)
        } else {
            Removed::ended(ItemOutcome::Skipped { reason: SkipReason::CheckFailed })
        };
        out(ItemDone {
            index,
            finding: item.finding(),
            sizes: item.sizes(),
            outcome: removed.outcome,
            trashed: removed.trashed,
            status: removed.status,
        });
    }
    if cancel.is_cancelled() { Ended::Cancelled } else { Ended::Completed }
}

/// One item: every check right before the act, then the removal its action and the plan's method call for.
fn act_on(item: &PlanItem, guard: &GuardContext<'_>, plan: &ConfirmedPlan, ports: &CleanerPorts) -> Removed {
    let act = plan.act();
    if let Action::Tool(command) = item.action() {
        return match verify_tool(command, guard) {
            Ok(()) => run_tool(command, act, &*ports.commands),
            Err(reason) => Removed::ended(ItemOutcome::Skipped { reason }),
        };
    }
    let verified = match verify(item, guard, Depth::Full) {
        Ok(verified) => verified,
        Err(Unverified::Skipped(reason)) => return Removed::ended(ItemOutcome::Skipped { reason }),
        Err(Unverified::Cancelled) => return Removed::ended(ItemOutcome::NotReached),
    };
    let way = match plan.permanent() {
        Some(consent) => Way::Permanent(consent),
        None => Way::Trash(&*ports.trash),
    };
    match (item.action(), way) {
        (Action::Empty(_), way) => empty(verified, act, way, guard.cancel),
        (_, Way::Permanent(consent)) => delete(verified, act, consent),
        (_, Way::Trash(trash)) => to_trash(verified, act, trash),
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Action, ToolCommand, ToolId};

    use crate::test_support::Rig;

    fn rig() -> Rig {
        Rig::new(
            r#"
            [[file]]
            path = "home/a.bin"
            [[file]]
            path = "home/b.bin"
            [[file]]
            path = "home/c.bin"
            [[file]]
            path = "home/cache/inner/d.bin"
            [[file]]
            path = "outside/keep.txt"
            [[symlink]]
            path = "home/cache/to-outside"
            target = "../../outside"
            "#,
        )
    }

    fn tool() -> Action {
        Action::Tool(ToolCommand::new(ToolId::new("docker").unwrap(), vec!["image".into(), "rm".into()]).unwrap())
    }

    fn files(rig: &Rig) -> Vec<Action> {
        ["home/a.bin", "home/b.bin", "home/c.bin"].map(|path| Action::Remove(rig.at(path))).into()
    }

    mod negative {
        use std::time::Duration;

        use fresher_core::{
            Action, CancelToken, Cleaner as _, Depth, Ended, ItemOutcome, Method, Protection, Rehearsal, SkipReason,
        };
        use fresher_testing::{FakeOpenFiles, FakeRunningApps};

        use super::{files, rig, tool};
        use crate::test_support::{listing, plan_item};

        #[test]
        fn a_rehearsal_changes_nothing() {
            let rig = rig();
            let before = listing(rig.setup.tree.root());
            let actions = vec![
                Action::Remove(rig.at("home/a.bin")),
                Action::Remove(rig.at("home/cache")),
                Action::Empty(rig.at("home/cache")),
                Action::Remove(rig.at("home/cache/to-outside")),
                tool(),
            ];
            let items: Vec<_> = (1..).zip(actions).map(|(id, action)| plan_item(id, action, 4096)).collect();
            let roots = vec![rig.setup.tree.root().to_path_buf()];

            for depth in [Depth::Quick, Depth::Full] {
                let rehearsed = rig.cleaner.rehearse(&roots, &items, &rig.cx(), depth);
                assert_eq!(rehearsed[4], Rehearsal::Skipped(SkipReason::NotInSandbox));
                assert!(rehearsed[..4].iter().all(|answer| matches!(answer, Rehearsal::Passed(Some(_)))));
            }

            assert_eq!(listing(rig.setup.tree.root()), before);
            assert_eq!(rig.trash.moved(), Vec::<std::path::PathBuf>::new());
            assert_eq!(rig.runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn a_cancel_leaves_every_later_item_not_reached() {
            let cancel = CancelToken::new();
            let rig = rig().meddling({
                let cancel = cancel.clone();
                move || cancel.cancel()
            });
            let items = rig.items(files(&rig));

            let (done, ended) = rig.execute(&items, Method::Trash, &cancel);

            let outcomes: Vec<_> = done.iter().map(|item| item.outcome.clone()).collect();
            assert_eq!(outcomes, [ItemOutcome::Moved, ItemOutcome::NotReached, ItemOutcome::NotReached]);
            assert_eq!(ended, Ended::Cancelled);
            assert!(rig.at("home/b.bin").exists() && rig.at("home/c.bin").exists());
        }

        #[test]
        fn an_item_that_fails_its_check_is_skipped_and_the_next_goes_on() {
            let mut rig = rig();
            let items = rig.items(files(&rig));
            rig.setup.protection = Protection::new(vec![rig.at("home/a.bin")], vec![], vec![]).unwrap();

            let (done, ended) = rig.execute(&items, Method::Trash, &CancelToken::new());

            assert_eq!(done[0].outcome, ItemOutcome::Skipped { reason: SkipReason::Protected });
            assert_eq!(done[1].outcome, ItemOutcome::Moved);
            assert_eq!(done[2].outcome, ItemOutcome::Moved);
            assert_eq!(ended, Ended::Completed);
            assert!(rig.at("home/a.bin").exists());
        }

        #[test]
        fn a_failed_snapshot_acts_on_nothing() {
            let rig = rig().with_open_files(FakeOpenFiles::failing());
            let items: Vec<_> = (1..).zip(files(&rig)).map(|(id, action)| plan_item(id, action, 4096)).collect();
            let before = listing(rig.setup.tree.root());

            let (done, _) = rig.execute(&items, Method::Permanent, &CancelToken::new());
            let rehearsed =
                rig.cleaner.rehearse(&[rig.setup.tree.root().to_path_buf()], &items, &rig.cx(), Depth::Full);

            assert!(done.iter().all(|item| item.outcome == ItemOutcome::Skipped { reason: SkipReason::CheckFailed }));
            assert!(rehearsed.iter().all(|answer| *answer == Rehearsal::Skipped(SkipReason::CheckFailed)));
            assert_eq!(listing(rig.setup.tree.root()), before);
        }

        #[test]
        fn an_item_a_running_app_owns_is_skipped_as_in_use() {
            let notes = fresher_testing::running_app("Notes", &["b.bin"]).unwrap();
            // Not running when the plan is made, running when the cleanup starts.
            let rig = rig().with_running(FakeRunningApps::new(Vec::new()).then(vec![notes]));
            let items = rig.items(files(&rig));

            let (done, ended) = rig.execute(&items, Method::Trash, &CancelToken::new());

            let outcomes: Vec<_> = done.iter().map(|item| item.outcome.clone()).collect();
            let in_use = ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Notes".into() } };
            assert_eq!(outcomes, [ItemOutcome::Moved, in_use, ItemOutcome::Moved]);
            assert_eq!(ended, Ended::Completed);
            assert!(rig.at("home/b.bin").exists());
        }

        #[test]
        fn a_failed_listing_of_the_running_apps_acts_on_nothing() {
            let rig = rig().with_running(FakeRunningApps::failing("the process table cannot be read"));
            let items: Vec<_> = (1..).zip(files(&rig)).map(|(id, action)| plan_item(id, action, 4096)).collect();
            let before = listing(rig.setup.tree.root());

            let (done, _) = rig.execute(&items, Method::Trash, &CancelToken::new());
            let rehearsed =
                rig.cleaner.rehearse(&[rig.setup.tree.root().to_path_buf()], &items, &rig.cx(), Depth::Full);

            assert!(done.iter().all(|item| item.outcome == ItemOutcome::Skipped { reason: SkipReason::CheckFailed }));
            assert!(rehearsed.iter().all(|answer| *answer == Rehearsal::Skipped(SkipReason::CheckFailed)));
            assert_eq!(listing(rig.setup.tree.root()), before);
        }

        #[test]
        fn an_app_started_after_the_first_snapshot_is_seen_after_two_seconds() {
            let preview = fresher_testing::running_app("Preview", &["c.bin"]).unwrap();
            let rig = rig().with_running(FakeRunningApps::new(Vec::new()).then(vec![preview]));
            let clock = std::sync::Arc::clone(&rig.clock);
            let rig = rig.meddling(move || clock.advance(Duration::from_secs(2)));
            let items = rig.items(files(&rig));
            let listed = rig.running.calls();

            let (done, _) = rig.execute(&items, Method::Trash, &CancelToken::new());

            assert_eq!(done[1].outcome, ItemOutcome::Moved);
            assert_eq!(done[2].outcome, ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Preview".into() } });
            assert_eq!(rig.running.calls() - listed, 2);
        }

        #[test]
        fn a_file_opened_after_the_first_snapshot_is_seen_after_two_seconds() {
            let rig = rig();
            let (clock, open_files) = (std::sync::Arc::clone(&rig.clock), std::sync::Arc::clone(&rig.open_files));
            let (b, c) = (rig.at("home/b.bin"), rig.at("home/c.bin"));
            let rig = rig.meddling(move || {
                open_files.hold(b.clone(), "Preview");
                open_files.hold(c.clone(), "Preview");
                clock.advance(Duration::from_secs(2));
            });
            let mut items = rig.items(files(&rig));
            items.truncate(2);
            let snapshots = rig.open_files.snapshots();

            let (done, _) = rig.execute(&items, Method::Trash, &CancelToken::new());

            assert_eq!(done[0].outcome, ItemOutcome::Moved);
            assert_eq!(done[1].outcome, ItemOutcome::Skipped { reason: SkipReason::InUse { app: "Preview".into() } });
            assert_eq!(rig.open_files.snapshots() - snapshots, 2);
        }

        #[test]
        fn a_file_opened_within_two_seconds_is_not_seen_yet() {
            let rig = rig();
            let (clock, open_files) = (std::sync::Arc::clone(&rig.clock), std::sync::Arc::clone(&rig.open_files));
            let b = rig.at("home/b.bin");
            let rig = rig.meddling(move || {
                open_files.hold(b.clone(), "Preview");
                clock.advance(Duration::from_millis(1_999));
            });
            let items = rig.items(files(&rig));
            let snapshots = rig.open_files.snapshots();

            let (done, _) = rig.execute(&items[..2], Method::Trash, &CancelToken::new());

            assert_eq!(done[1].outcome, ItemOutcome::Moved);
            assert_eq!(rig.open_files.snapshots() - snapshots, 1);
        }

        #[test]
        fn a_tool_item_never_runs_in_the_sandbox() {
            let rig = rig();
            let items = vec![plan_item(1, tool(), 100)];

            let (done, _) = rig.execute(&items, Method::Trash, &CancelToken::new());

            assert_eq!(done[0].outcome, ItemOutcome::Skipped { reason: SkipReason::NotInSandbox });
            assert_eq!(rig.runner.calls(), Vec::<String>::new());
        }

        #[test]
        fn an_item_without_an_identity_is_not_acted_on() {
            let rig = rig();
            let items = vec![plan_item(1, Action::Remove(rig.at("home/a.bin")), 4096)];

            let (done, _) = rig.execute(&items, Method::Permanent, &CancelToken::new());

            assert_eq!(done[0].outcome, ItemOutcome::Skipped { reason: SkipReason::CheckFailed });
            assert!(rig.at("home/a.bin").exists());
        }

        #[test]
        fn a_root_written_with_a_tilde_is_read_under_the_home() {
            let rig = rig();
            let items = rig.items(files(&rig));
            let tilde = vec![std::path::PathBuf::from("~")];
            let cx = fresher_core::CleanupContext { roots: &tilde, ..rig.cx() };
            let plan = crate::test_support::confirmed(&rig.setup, &items[..1], Method::Trash);
            let mut done = Vec::new();

            rig.cleaner.execute(plan, &cx, &CancelToken::new(), &mut |item| done.push(item));

            assert_eq!(done[0].outcome, ItemOutcome::Moved);
        }

        #[test]
        fn a_root_that_cannot_be_made_canonical_acts_on_nothing() {
            let rig = rig();
            let items = rig.items(files(&rig));
            for roots in [vec![std::path::PathBuf::from("home")], vec![std::path::PathBuf::from("~/gone")]] {
                let cx = fresher_core::CleanupContext { roots: &roots, ..rig.cx() };
                let plan = crate::test_support::confirmed_over(&roots, &items, Method::Permanent);
                let mut done = Vec::new();

                rig.cleaner.execute(plan, &cx, &CancelToken::new(), &mut |item| done.push(item));

                assert!(done.iter().all(|item| matches!(item.outcome, ItemOutcome::Skipped { .. })));
            }
            assert!(rig.at("home/a.bin").exists());
        }

        #[test]
        fn the_systems_protection_applies_when_the_context_has_none() {
            let rig = rig().with_system(|rig| Protection::new(vec![rig.at("home/a.bin")], vec![], vec![]).unwrap());
            let items: Vec<_> = (1..).zip(files(&rig)).map(|(id, action)| plan_item(id, action, 4096)).collect();

            let rehearsed =
                rig.cleaner.rehearse(&[rig.setup.tree.root().to_path_buf()], &items, &rig.cx(), Depth::Full);
            let (done, _) = rig.execute(&items, Method::Permanent, &CancelToken::new());

            assert!(matches!(rehearsed[0], Rehearsal::Skipped(SkipReason::NotInSandbox | SkipReason::Protected)));
            assert!(matches!(done[0].outcome, ItemOutcome::Skipped { .. }));
            assert!(rig.at("home/a.bin").exists());
        }

        #[test]
        fn a_root_no_longer_scanned_acts_on_nothing() {
            let rig = rig();
            let items = rig.items(files(&rig));
            let other = vec![rig.at("home")];
            let cx = fresher_core::CleanupContext { roots: &other, ..rig.cx() };
            let plan = crate::test_support::confirmed(&rig.setup, &items, Method::Permanent);
            let mut done = Vec::new();

            rig.cleaner.execute(plan, &cx, &CancelToken::new(), &mut |item| done.push(item));

            assert!(done.iter().all(|item| item.outcome == ItemOutcome::Skipped { reason: SkipReason::NotInSandbox }));
            assert!(rig.at("home/a.bin").exists());
        }
    }

    mod positive {
        use fresher_core::{
            Action, CancelToken, Cleaner as _, Depth, Ended, ItemOutcome, Method, Rehearsal, SkipReason,
        };

        use super::{files, rig};
        use crate::test_support::{DiskImage, plan_item};

        #[test]
        fn items_go_in_plan_order_one_at_a_time() {
            let rig = rig();
            let items = rig.items(files(&rig));
            let plan = crate::test_support::confirmed(&rig.setup, &items, Method::Trash);
            let mut seen = Vec::new();

            let ended = rig.cleaner.execute(plan, &rig.cx(), &CancelToken::new(), &mut |item| {
                seen.push((item.index, item.finding.get(), rig.trash.moved().len(), item.outcome));
            });

            assert_eq!(ended, Ended::Completed);
            assert_eq!(
                seen,
                [(0, 1, 1, ItemOutcome::Moved), (1, 2, 2, ItemOutcome::Moved), (2, 3, 3, ItemOutcome::Moved)]
            );
            assert_eq!(
                rig.trash.moved(),
                files(&rig).iter().map(|action| crate::test_support::physical_of(&rig, action)).collect::<Vec<_>>()
            );
        }

        #[test]
        fn the_trash_method_moves_and_the_permanent_method_deletes() {
            let rig = rig();
            let items = rig.items(vec![Action::Remove(rig.at("home/a.bin")), Action::Empty(rig.at("home/cache"))]);

            let (moved, _) = rig.execute(&items[..1], Method::Trash, &CancelToken::new());
            let (deleted, _) = rig.execute(&items[1..], Method::Permanent, &CancelToken::new());

            assert_eq!(moved[0].outcome, ItemOutcome::Moved);
            assert_eq!(moved[0].trashed[0].original, rig.at("home/a.bin"));
            assert_eq!(deleted[0].outcome, ItemOutcome::Emptied { moved: 2, skipped: 0 });
            assert_eq!(deleted[0].trashed, Vec::new());
            assert_eq!(rig.trash.moved().len(), 1);
            assert!(rig.at("home/cache").is_dir() && rig.at("outside/keep.txt").exists());
        }

        #[test]
        fn a_dry_run_reports_what_would_happen_with_the_folder_walk() {
            let rig = rig();
            let roots = vec![rig.setup.tree.root().to_path_buf()];
            let items = vec![plan_item(1, Action::Remove(rig.at("home/cache")), 4096)];
            let Some(_image) = DiskImage::mount_at(&rig.at("home/cache/inner/mounted")) else { return };

            let quick = rig.cleaner.rehearse(&roots, &items, &rig.cx(), Depth::Quick);
            let full = rig.cleaner.rehearse(&roots, &items, &rig.cx(), Depth::Full);

            assert!(matches!(quick[0], Rehearsal::Passed(Some(_))));
            assert_eq!(full[0], Rehearsal::Skipped(SkipReason::OtherVolume));
        }
    }
}
