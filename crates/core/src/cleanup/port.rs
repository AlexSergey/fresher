//! The `Cleaner` port: the one way anything is removed, implemented by `fresher-cleaner`.

use std::path::PathBuf;

use crate::cleanup::{
    CleanupId, ConfirmedPlan, Ended, ItemIdentity, ItemOutcome, ItemRecord, ItemReport, PlanItem, Protection,
    PutBackOutcome, SkipReason, TrashState,
};
use crate::{AccessStatus, CancelToken, FindingId, GuardedPlace, Sizes};

/// Checks items right before they are acted on, removes the items of a confirmed plan, and puts items back.
pub trait Cleaner: Send + Sync {
    /// Runs every check of the guard on each item of an unconfirmed plan, in order, and does nothing: each answer is
    /// the item's identity or why it would be skipped. An item of several paths runs every check on each part, in
    /// order: its answer is the first refused part's reason, or `PassedTogether` with every part's identity. `roots`
    /// are the roots of the scan the items come from, its places included. It has no `Act`, so it cannot remove
    /// anything.
    fn rehearse(&self, roots: &[PathBuf], items: &[PlanItem], cx: &CleanupContext<'_>, depth: Depth) -> Vec<Rehearsal>;

    /// Acts on the items of `plan`, one at a time in plan order: the checks again right before each item, then its
    /// removal; `out` gets each item's end before the next one starts. Once `cancel` is set, every item not started
    /// ends `NotReached`. Answers `Completed` or `Cancelled`. The plan is taken by value: it runs once.
    fn execute(
        &self,
        plan: ConfirmedPlan,
        cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended;

    /// The protected places this cleaner keeps away from, the system's list: the one source of what may never be
    /// removed (a scan walks no place outside its roots that it refuses for ever).
    fn protection(&self) -> Protection;

    /// Where each item moved to the Trash is now; follows no link.
    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState>;

    /// Moves one item back from the Trash to its original place, which must pass the guard's path checks today;
    /// never replaces anything.
    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome;
}

/// What the guard checks against, as it stands when the cleaner is called.
#[derive(Clone, Copy, Debug)]
pub struct CleanupContext<'a> {
    /// The scan roots in the settings now, and the places the last scan walked outside them (the temporary folders):
    /// an item under a root no longer scanned is outside the roots.
    pub roots: &'a [PathBuf],
    /// The protected places, with the user's ignored paths.
    pub protection: &'a Protection,
    pub access: &'a AccessStatus,
    /// The places the system guards, each with its capability.
    pub guarded: &'a [GuardedPlace],
}

/// How far a rehearsal goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Depth {
    /// Every check but the walk of a folder's contents: what a plan is made with.
    Quick,
    /// Every check: the dry run, and what runs right before each removal.
    Full,
}

/// One item's answer of a rehearsal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rehearsal {
    /// Every check passed; the identity read from the disk (`None` for a tool's item).
    Passed(Option<ItemIdentity>),
    /// Every check passed on every part of an item of several paths; the identity of each, in part order.
    PassedTogether(Vec<ItemIdentity>),
    Skipped(SkipReason),
}

/// One item of a cleanup as it ended: what the events, the report and the journal need of it, since the plan itself
/// went to the cleaner.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemDone {
    /// Its place in the plan.
    pub index: u32,
    pub finding: FindingId,
    /// Its sizes as the plan held them.
    pub sizes: Sizes,
    pub outcome: ItemOutcome,
    /// What went to the Trash: the item, or each entry of an emptied folder.
    pub trashed: Vec<TrashedItem>,
    /// A tool's exit status, when a tool ran.
    pub status: Option<i32>,
    /// How many parts of an item of several paths went, the subject first; `None` for any other item.
    pub went: Option<u32>,
}

impl ItemDone {
    /// What the events carry.
    pub fn report(&self) -> ItemReport {
        ItemReport { index: self.index, finding: self.finding, outcome: self.outcome.clone() }
    }

    /// What the journal keeps.
    pub fn record(&self, cleanup: CleanupId) -> ItemRecord {
        ItemRecord {
            cleanup,
            index: self.index,
            outcome: self.outcome.recorded(self.status),
            trashed: self.trashed.clone(),
            went: self.went,
        }
    }
}

/// Something moved to the Trash: where it was, where it landed, and what it was.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TrashedItem {
    pub original: PathBuf,
    pub landed: PathBuf,
    pub identity: ItemIdentity,
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::path::PathBuf;

        use crate::cleanup::{CleanupId, ItemDone, ItemIdentity, ItemOutcome, RecordedOutcome, TrashedItem};
        use crate::{EntryKind, FindingId, Sizes};

        #[test]
        fn an_item_done_gives_its_report_and_its_record() {
            let trashed = TrashedItem {
                original: PathBuf::from("/home/a"),
                landed: PathBuf::from("/home/.Trash/a"),
                identity: ItemIdentity::new(1, 2, EntryKind::File),
            };
            let done = ItemDone {
                index: 3,
                finding: FindingId::new(9),
                sizes: Sizes::ZERO,
                outcome: ItemOutcome::Moved,
                trashed: vec![trashed.clone()],
                status: None,
                went: None,
            };

            let report = done.report();
            let record = done.record(CleanupId::new(4));

            assert_eq!((report.index, report.finding, report.outcome), (3, FindingId::new(9), ItemOutcome::Moved));
            assert_eq!((record.cleanup, record.index, record.outcome), (CleanupId::new(4), 3, RecordedOutcome::Moved));
            assert_eq!(record.trashed, [trashed]);
        }
    }
}
