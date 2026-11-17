//! The guard: the checks every item passes right before it is acted on, and in every rehearsal, in the design's
//! order; the first that fails skips the item with its reason. It reads the disk only through paths and handles that
//! follow no link.

use std::ffi::OsString;

mod access;
mod facts;
pub(crate) mod identity;
mod in_use;
pub(crate) mod walk;

use fresher_core::{
    AccessStatus, Action, CancelToken, Depth, EntryKind, GuardedPlace, ItemIdentity, OpenSet, PlanItem, Protection,
    RunningSet, SkipReason, ToolCommand,
};

use crate::guard::walk::{SizeWalk, WalkStop};
use crate::paths::{Handle, Placed, Roots, check_protection};
use crate::reach::Reach;

/// Check 11: a folder is `Grown` when it allocates more than this share (in percent) over the confirmed size…
pub(crate) const GROWTH_SHARE: u64 = 25;
/// …and also more than this many bytes over it (64 MiB).
pub(crate) const GROWTH_FLOOR: u64 = 64 * 1024 * 1024;

/// What a check is for: a rehearsal acts on nothing and may read an identity the plan does not hold yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Purpose {
    Rehearse,
    Act,
}

/// Everything the guard checks an item against, as it stands when the cleaner is called.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GuardContext<'a> {
    pub(crate) reach: &'a Reach,
    /// The uid the app runs as.
    pub(crate) user: u32,
    /// The system's protected places, given to the cleaner when it was made.
    pub(crate) system: &'a Protection,
    /// The call's protected places, with the user's ignored paths.
    pub(crate) protection: &'a Protection,
    pub(crate) roots: &'a Roots,
    pub(crate) access: &'a AccessStatus,
    pub(crate) guarded: &'a [GuardedPlace],
    /// The snapshot of open files the pipeline keeps fresh.
    pub(crate) open: &'a OpenSet,
    /// The running apps, taken with the open files.
    pub(crate) running: &'a RunningSet,
    pub(crate) purpose: Purpose,
    /// The walk of check 11.
    pub(crate) walk: &'a dyn SizeWalk,
    /// Read by the walk between folders.
    pub(crate) cancel: &'a CancelToken,
}

/// Why an item did not pass the guard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Unverified {
    /// A check failed: the item is skipped with this reason.
    Skipped(SkipReason),
    /// The cleanup was cancelled during the walk: the item is not reached.
    Cancelled,
}

impl From<SkipReason> for Unverified {
    fn from(reason: SkipReason) -> Self {
        Self::Skipped(reason)
    }
}

/// An item that passed the guard: open through no link, on its root's volume, still what the plan holds, removable,
/// allowed and not in use. Only the guard makes one; every removal consumes one, so a check serves one act.
#[derive(Debug)]
pub struct Verified {
    handle: Handle,
    placed: Placed,
    identity: ItemIdentity,
}

impl Verified {
    pub(crate) const fn placed(&self) -> &Placed {
        &self.placed
    }

    pub(crate) const fn identity(&self) -> ItemIdentity {
        self.identity
    }

    /// The item held open through no link since it was checked.
    pub(crate) const fn handle(&self) -> &Handle {
        &self.handle
    }

    /// What a removal acts on: the handle (kept open until the act), the place and the identity.
    pub(crate) fn into_parts(self) -> (Handle, Placed, ItemIdentity) {
        (self.handle, self.placed, self.identity)
    }
}

/// The checks of a path item, in the design's order: reach, roots, protection, ignored, the open through no link,
/// the volume, the identity, removability, capability, use, and with `Depth::Full` growth.
pub(crate) fn verify(item: &PlanItem, cx: &GuardContext<'_>, depth: Depth) -> Result<Verified, Unverified> {
    let Some(path) = item.action().path() else {
        return Err(SkipReason::CheckFailed.into());
    };
    let placed = cx.roots.place(path);
    let in_reach = match &placed {
        Ok(placed) => cx.reach.admits(&placed.root, &placed.relative, cx.user, cx.system),
        Err(_) => cx.reach.admits_unplaced(),
    };
    if !in_reach {
        return Err(SkipReason::NotInSandbox.into());
    }
    let placed = placed?;
    check_protection(&placed, &[cx.system, cx.protection])?;
    let handle = Handle::open(&placed)?;
    let identity = identity::check(item.identity(), identity::of(handle.metadata()), cx.purpose)?;
    if matches!(item.action(), Action::Empty(_)) && !handle.metadata().is_dir() {
        return Err(SkipReason::KindChanged.into());
    }
    facts::check(&handle, &placed, cx.user)?;
    access::check(&placed, cx.guarded, cx.access)?;
    in_use::check(&placed, cx.open, cx.running)?;
    if depth == Depth::Full && identity.kind() != EntryKind::Symlink {
        let now =
            cx.walk.measure(handle.file(), handle.metadata(), placed.device, cx.cancel).map_err(|stop| match stop {
                WalkStop::Skip(reason) => Unverified::Skipped(reason),
                WalkStop::Cancelled => Unverified::Cancelled,
            })?;
        if grown(item.sizes().allocated.bytes(), now) {
            return Err(SkipReason::Grown.into());
        }
    }
    Ok(Verified { handle, placed, identity })
}

/// Check 11's margins: more than `GROWTH_SHARE` percent and more than `GROWTH_FLOOR` bytes over the confirmed size.
fn grown(confirmed: u64, now: u64) -> bool {
    let over = now.saturating_sub(confirmed);
    over > GROWTH_FLOOR && u128::from(over) * 100 > u128::from(confirmed) * u128::from(GROWTH_SHARE)
}

/// Checks 5 to 7 of an entry of a verified folder that is being emptied: opened beneath the folder through no link,
/// on the root's volume, and still what the folder's listing said (`listed`). The folder itself passed every check,
/// so the entry is inside the roots, not protected, not ignored, allowed and not in use.
pub(crate) fn verify_entry(folder: &Verified, name: &OsString, listed: ItemIdentity) -> Result<Verified, SkipReason> {
    let placed = folder.placed.child(name);
    let handle = Handle::open(&placed)?;
    let identity = identity::check(Some(listed), identity::of(handle.metadata()), Purpose::Act)?;
    Ok(Verified { handle, placed, identity })
}

/// The checks of a tool's item, which has no path: reach (never in the sandbox), its arguments (never forced), and
/// its capability; then the tool's own refusal decides.
pub(crate) fn verify_tool(command: &ToolCommand, cx: &GuardContext<'_>) -> Result<(), SkipReason> {
    if !cx.reach.admits_tools() {
        return Err(SkipReason::NotInSandbox);
    }
    if command.forces() {
        return Err(SkipReason::ForcedTool);
    }
    access::check_tool(command.tool(), cx.access)
}

#[cfg(test)]
mod tests {
    use fresher_core::{Action, Depth, PlanItem, SkipReason};

    use crate::guard::{Purpose, Unverified, Verified, verify};
    use crate::test_support::{Setup, plan_item};

    fn tree() -> Setup {
        Setup::new(
            r#"
            [[file]]
            path = "home/cache/a.bin"
            [[file]]
            path = "home/Desktop/b.txt"
            [[dir]]
            path = "home/folder/inner"
            "#,
        )
    }

    fn check(setup: &Setup, item: &PlanItem, purpose: Purpose) -> Result<Verified, SkipReason> {
        verify(item, &setup.cx(purpose), Depth::Quick).map_err(|error| match error {
            Unverified::Skipped(reason) => reason,
            Unverified::Cancelled => unreachable!("a quick check never walks"),
        })
    }

    /// The folder `home/folder` as a plan item allocating `allocated` bytes, checked at full depth with a walk that
    /// answers `walked`.
    fn walked(
        allocated: u64,
        walked: Result<u64, crate::guard::walk::WalkStop>,
    ) -> (Result<Verified, Unverified>, u32) {
        let mut setup = tree();
        let walk = std::sync::Arc::new(crate::test_support::FixedWalk::answering(walked));
        setup.walk = Box::new(std::sync::Arc::clone(&walk));
        let item = plan_item(1, Action::Remove(setup.at("home/folder")), allocated);
        let answer = verify(&item, &setup.cx(Purpose::Rehearse), Depth::Full);
        (answer, walk.calls())
    }

    const MIB: u64 = 1024 * 1024;

    fn remove(setup: &Setup, relative: &str) -> PlanItem {
        plan_item(1, Action::Remove(setup.at(relative)), 4096)
    }

    /// The item as a rehearsal of a new plan freezes it.
    fn frozen(setup: &Setup, item: PlanItem) -> PlanItem {
        let identity = check(setup, &item, Purpose::Rehearse).unwrap().identity();
        item.with_identity(Some(identity))
    }

    mod negative {
        use std::fs;

        use fresher_core::{
            Access, Action, Capability, GuardedFolder, GuardedPlace, OpenSet, Protection, SkipReason, ToolCommand,
            ToolId,
        };

        use super::{MIB, check, frozen, remove, tree, walked};
        use crate::guard::walk::WalkStop;
        use crate::guard::{GROWTH_FLOOR, Purpose, Unverified, grown, verify, verify_tool};
        use crate::test_support::{Flags, Setup, plan_item};
        use fresher_core::Depth;

        #[test]
        fn an_item_outside_the_sandbox_is_refused_first() {
            let mut setup = Setup::new_without_sandbox("[[file]]\npath = \"home/cache/a.bin\"\n");
            setup.protection = Protection::new(vec![setup.at("home/cache")], vec![], vec![]).unwrap();

            let answer = check(&setup, &remove(&setup, "home/cache/a.bin"), Purpose::Rehearse);

            assert_eq!(answer.unwrap_err(), SkipReason::NotInSandbox);
        }

        #[test]
        fn an_item_outside_the_roots_is_not_in_the_sandbox() {
            let setup = tree();

            let outside = plan_item(1, Action::Remove("/etc/hosts".into()), 4096);

            let answer = check(&setup, &outside, Purpose::Rehearse);

            assert_eq!(answer.unwrap_err(), SkipReason::NotInSandbox);
        }

        #[test]
        fn a_protected_item_in_the_sandbox_is_refused() {
            let mut setup = tree();
            setup.protection = Protection::new(vec![setup.at("home/cache")], vec![], vec![]).unwrap();

            let answer = check(&setup, &remove(&setup, "home/cache/a.bin"), Purpose::Rehearse);

            assert_eq!(answer.unwrap_err(), SkipReason::Protected);
        }

        #[test]
        fn an_item_swapped_for_another_is_replaced() {
            let setup = tree();
            let item = frozen(&setup, remove(&setup, "home/cache/a.bin"));
            fs::write(setup.at("new.bin"), b"other").unwrap();
            fs::rename(setup.at("new.bin"), setup.at("home/cache/a.bin")).unwrap();

            assert_eq!(check(&setup, &item, Purpose::Act).unwrap_err(), SkipReason::Replaced);
        }

        #[test]
        fn a_file_turned_folder_has_changed_kind() {
            let setup = tree();
            let item = frozen(&setup, remove(&setup, "home/cache/a.bin"));
            fs::rename(setup.at("home/cache/a.bin"), setup.at("moved.bin")).unwrap();
            fs::create_dir(setup.at("home/cache/a.bin")).unwrap();

            assert_eq!(check(&setup, &item, Purpose::Act).unwrap_err(), SkipReason::KindChanged);
        }

        #[test]
        fn a_file_to_empty_has_changed_kind() {
            let setup = tree();
            let item = plan_item(1, Action::Empty(setup.at("home/cache/a.bin")), 4096);

            assert_eq!(check(&setup, &item, Purpose::Rehearse).unwrap_err(), SkipReason::KindChanged);
        }

        #[test]
        fn an_act_on_an_item_without_an_identity_is_refused() {
            let setup = tree();

            let answer = check(&setup, &remove(&setup, "home/cache/a.bin"), Purpose::Act);

            assert_eq!(answer.unwrap_err(), SkipReason::CheckFailed);
        }

        #[test]
        fn a_locked_item_is_not_removable() {
            let setup = tree();
            let _flags = Flags::lock(&setup.at("home/cache/a.bin"));

            let answer = check(&setup, &remove(&setup, "home/cache/a.bin"), Purpose::Rehearse);

            assert!(matches!(answer.unwrap_err(), SkipReason::NotRemovable { .. }));
        }

        #[test]
        fn an_item_in_a_guarded_place_without_its_capability_is_refused() {
            let desktop = Capability::Folder(GuardedFolder::Desktop);
            let mut setup = tree();
            setup.guarded = vec![GuardedPlace { path: setup.at("home/Desktop"), capability: desktop }];
            setup.access = Setup::status(&[(desktop, Access::Denied)]);

            let answer = check(&setup, &remove(&setup, "home/Desktop/b.txt"), Purpose::Rehearse);

            assert_eq!(answer.unwrap_err(), SkipReason::MissingCapability { capability: desktop });
        }

        #[test]
        fn a_held_file_inside_a_folder_makes_it_in_use() {
            let mut setup = tree();
            setup.open = OpenSet::new(vec![(setup.physical("home/folder/inner"), "Preview".into())]);

            let answer = check(&setup, &remove(&setup, "home/folder"), Purpose::Rehearse);

            assert_eq!(answer.unwrap_err(), SkipReason::InUse { app: "Preview".into() });
        }

        #[test]
        fn a_held_folder_is_in_use() {
            let mut setup = tree();
            setup.open = OpenSet::new(vec![(setup.at("home/folder"), "Terminal".into())]);

            let answer = check(&setup, &remove(&setup, "home/folder"), Purpose::Rehearse);

            assert_eq!(answer.unwrap_err(), SkipReason::InUse { app: "Terminal".into() });
        }

        #[test]
        fn a_folder_that_grew_past_both_margins_is_grown() {
            let (answer, calls) = walked(MIB, Ok(MIB + 65 * MIB));

            assert_eq!(answer.unwrap_err(), Unverified::Skipped(SkipReason::Grown));
            assert_eq!(calls, 1);
        }

        #[test]
        fn a_folder_with_another_volume_inside_is_another_volume() {
            let (answer, _) = walked(MIB, Err(WalkStop::Skip(SkipReason::OtherVolume)));

            assert_eq!(answer.unwrap_err(), Unverified::Skipped(SkipReason::OtherVolume));
        }

        #[test]
        fn a_cancelled_walk_is_not_reached() {
            let (answer, _) = walked(MIB, Err(WalkStop::Cancelled));

            assert_eq!(answer.unwrap_err(), Unverified::Cancelled);
        }

        #[test]
        fn a_cancelled_walk_of_the_disk_is_not_reached() {
            let setup = tree();
            setup.cancel.cancel();
            let item = plan_item(1, Action::Remove(setup.at("home/folder")), MIB);

            let answer = verify(&item, &setup.cx(Purpose::Rehearse), Depth::Full);

            assert_eq!(answer.unwrap_err(), Unverified::Cancelled);
        }

        #[test]
        fn growth_needs_both_margins() {
            assert!(!grown(4 * GROWTH_FLOOR, 5 * GROWTH_FLOOR));
            assert!(!grown(MIB, MIB + GROWTH_FLOOR));
            assert!(grown(MIB, MIB + GROWTH_FLOOR + 1));
            assert!(grown(4 * GROWTH_FLOOR, 5 * GROWTH_FLOOR + 1));
            assert!(grown(0, u64::MAX));
        }

        #[test]
        fn a_tool_item_in_the_sandbox_is_refused() {
            let setup = tree();
            let command = ToolCommand::new(ToolId::new("docker").unwrap(), vec!["image".into(), "rm".into()]).unwrap();
            let docker = Capability::Tool(command.tool());
            let mut granted = setup;
            granted.access = Setup::status(&[(docker, Access::Granted)]);

            assert_eq!(verify_tool(&command, &granted.cx(Purpose::Act)), Err(SkipReason::NotInSandbox));
        }

        #[test]
        fn a_tool_item_has_no_path_to_verify() {
            let setup = tree();
            let command = ToolCommand::new(ToolId::new("docker").unwrap(), vec!["image".into()]).unwrap();

            let answer = check(&setup, &plan_item(1, Action::Tool(command), 4096), Purpose::Rehearse);

            assert_eq!(answer.unwrap_err(), SkipReason::CheckFailed);
        }
    }

    mod positive {
        use std::os::unix::fs::MetadataExt as _;

        use fresher_core::{Access, Capability, EntryKind, GuardedFolder, GuardedPlace};

        use super::{MIB, check, frozen, remove, tree, walked};
        use crate::guard::Purpose;
        use crate::test_support::Setup;

        #[test]
        fn an_untouched_item_verifies() {
            let setup = tree();
            let item = frozen(&setup, remove(&setup, "home/cache/a.bin"));

            let verified = check(&setup, &item, Purpose::Act).unwrap();

            assert_eq!(Some(verified.identity()), item.identity());
            assert_eq!(verified.placed().lexical, setup.at("home/cache/a.bin"));
            assert_eq!(verified.into_parts().0.metadata().ino(), item.identity().unwrap().file());
        }

        #[test]
        fn a_rehearsal_reads_the_identity() {
            let setup = tree();

            let verified = check(&setup, &remove(&setup, "home/folder"), Purpose::Rehearse).unwrap();

            let on_disk = std::fs::symlink_metadata(setup.at("home/folder")).unwrap();
            assert_eq!(verified.identity().file(), on_disk.ino());
            assert_eq!(verified.identity().device(), on_disk.dev());
            assert_eq!(verified.identity().kind(), EntryKind::Folder);
        }

        #[test]
        fn a_folder_that_grew_within_the_margin_passes() {
            assert!(walked(1024 * MIB, Ok(1124 * MIB)).0.is_ok());
            assert!(walked(MIB, Ok(61 * MIB)).0.is_ok());
        }

        #[test]
        fn a_folder_that_shrank_passes() {
            assert!(walked(1024 * MIB, Ok(0)).0.is_ok());
        }

        #[test]
        fn quick_depth_does_not_walk() {
            let mut setup = tree();
            let walk = std::sync::Arc::new(crate::test_support::FixedWalk::answering(Ok(u64::MAX)));
            setup.walk = Box::new(std::sync::Arc::clone(&walk));

            assert!(check(&setup, &remove(&setup, "home/folder"), Purpose::Rehearse).is_ok());
            assert_eq!(walk.calls(), 0);
        }

        #[test]
        fn a_link_item_is_not_walked() {
            let mut setup = tree();
            std::os::unix::fs::symlink(setup.at("home/folder"), setup.at("home/link")).unwrap();
            let walk = std::sync::Arc::new(crate::test_support::FixedWalk::answering(Ok(u64::MAX)));
            setup.walk = Box::new(std::sync::Arc::clone(&walk));
            let item = crate::test_support::plan_item(1, fresher_core::Action::Remove(setup.at("home/link")), 0);

            assert!(crate::guard::verify(&item, &setup.cx(Purpose::Rehearse), fresher_core::Depth::Full).is_ok());
            assert_eq!(walk.calls(), 0);
        }

        #[test]
        fn a_full_check_of_the_disk_measures_the_folder() {
            let setup = tree();
            let item = crate::test_support::plan_item(1, fresher_core::Action::Remove(setup.at("home/folder")), 0);

            assert!(crate::guard::verify(&item, &setup.cx(Purpose::Rehearse), fresher_core::Depth::Full).is_ok());
        }

        #[test]
        fn a_guarded_place_with_its_capability_passes() {
            let desktop = Capability::Folder(GuardedFolder::Desktop);
            let mut setup = tree();
            setup.guarded = vec![GuardedPlace { path: setup.at("home/Desktop"), capability: desktop }];
            setup.access = Setup::status(&[(desktop, Access::Granted)]);

            assert!(check(&setup, &remove(&setup, "home/Desktop/b.txt"), Purpose::Rehearse).is_ok());
        }
    }
}
