//! The cleanup journal: what each cleanup was about to do and what came of it, written before and as it happens,
//! so a mistake can be put back and a crash leaves a record. `fresher-store` writes it in a file of its own format.

use std::path::PathBuf;

use crate::cleanup::{
    Action, ActionKind, CleanupId, CleanupReport, CleanupSummary, ConfirmedPlan, Ended, ItemIdentity, Method, PlanItem,
    PutBackOutcome, RecordedOutcome, TrashedItem,
};
use crate::{ByteSize, DangerLevel, Sizes, Text, Timestamp};

/// Keeps the record of every cleanup. Writes are appends: nothing is rewritten or removed.
pub trait CleanupJournal: Send + Sync {
    /// Whether a new cleanup can be recorded; a cleanup is refused unless `Ready`.
    fn state(&self) -> JournalState;
    /// Records a cleanup before its first item and makes it durable; answers the cleanup's new id.
    fn begin(&self, record: &BeginRecord) -> Result<CleanupId, JournalError>;
    /// Records how one item ended (not yet durable: see `sync`).
    fn item(&self, record: &ItemRecord) -> Result<(), JournalError>;
    /// Makes every record so far durable.
    fn sync(&self) -> Result<(), JournalError>;
    /// Records how the cleanup ended, durably.
    fn end(&self, record: &EndRecord) -> Result<(), JournalError>;
    /// Records an attempt to put an item back, durably.
    fn put_back(&self, record: &PutBackRecord) -> Result<(), JournalError>;
    /// A page of the cleanups, newest first.
    fn cleanups(&self, offset: u32, limit: u32) -> Result<JournalPage, JournalError>;
    /// One cleanup with all its records; `None` when the journal has no such cleanup.
    fn cleanup(&self, id: CleanupId) -> Result<Option<RecordedCleanup>, JournalError>;
}

/// Whether the journal can be written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JournalState {
    Ready,
    /// It is read and shown, never written.
    ReadOnly {
        why: JournalProblem,
    },
}

/// Why the journal is read-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JournalProblem {
    /// A newer version of the app wrote it.
    NewerVersion,
    /// It exists and cannot be read.
    Unreadable,
}

/// Why a record could not be written or the journal read.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum JournalError {
    #[error("the cleanup journal is read-only")]
    ReadOnly,
    #[error("the disk is full")]
    NoSpace,
    #[error("the cleanup journal failed: {reason}")]
    Failed { reason: String },
}

/// The first record of a cleanup: everything it is about to do, written before the first item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeginRecord {
    pub at: Timestamp,
    pub method: Method,
    pub roots: Vec<PathBuf>,
    /// The available space of each affected volume at the start.
    pub space: Vec<VolumeSpace>,
    pub items: Vec<JournalItem>,
}

impl BeginRecord {
    /// The record of `plan`, about to start at `at` with the volumes' `space`. An item of several paths keeps every
    /// part after its subject, with the identity the rehearsal read, so a crash in its middle leaves a record of every
    /// path it was about to move.
    pub fn new(at: Timestamp, plan: &ConfirmedPlan, space: Vec<VolumeSpace>) -> Self {
        let items = plan
            .items()
            .iter()
            .zip(0_u32..)
            .map(|(item, index)| JournalItem {
                index,
                path: item.action().path().map(PathBuf::from),
                identity: item.identity(),
                more: more_parts(item),
                action: item.action().kind(),
                category: item.category().as_str().to_owned(),
                key: item.key().identity().to_owned(),
                reason: item.reason().clone(),
                danger: item.danger(),
                sizes: item.sizes(),
            })
            .collect();
        Self { at, method: plan.method(), roots: plan.plan().roots().to_vec(), space, items }
    }
}

/// The parts of an item of several paths after its subject, each with its identity; empty for any other item.
/// Every part is kept, even one whose identity the item does not carry.
fn more_parts(item: &PlanItem) -> Vec<JournalPart> {
    let Action::RemoveTogether(parts) = item.action() else { return Vec::new() };
    let identities = item.part_identities();
    let identity = |place: usize| identities.get(place).copied().filter(|_| identities.len() == parts.all().len());
    (1..).zip(parts.rest()).map(|(place, path)| JournalPart { path: path.clone(), identity: identity(place) }).collect()
}

/// One item of a cleanup as the journal keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalItem {
    pub index: u32,
    /// `None` for a tool's item; the subject of an item of several paths.
    pub path: Option<PathBuf>,
    pub identity: Option<ItemIdentity>,
    /// The parts of an item of several paths after its subject, in the order they move; empty for any other item.
    pub more: Vec<JournalPart>,
    pub action: ActionKind,
    pub category: String,
    /// The finding's identity within its module.
    pub key: String,
    /// Why the rule claimed it.
    pub reason: Text,
    pub danger: DangerLevel,
    pub sizes: Sizes,
}

/// A part of an item of several paths other than its subject, as the journal keeps it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JournalPart {
    pub path: PathBuf,
    /// The identity the rehearsal read; `None` when the plan item did not carry one for every part.
    pub identity: Option<ItemIdentity>,
}

/// The available space of the volume that holds `root`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VolumeSpace {
    pub root: PathBuf,
    pub available: ByteSize,
}

/// How one item ended.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemRecord {
    pub cleanup: CleanupId,
    pub index: u32,
    pub outcome: RecordedOutcome,
    pub trashed: Vec<TrashedItem>,
    /// How many parts of an item of several paths went, the subject first, whether or not the Trash told where each
    /// landed; `None` for any other item.
    pub went: Option<u32>,
}

/// How a cleanup ended.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndRecord {
    pub cleanup: CleanupId,
    pub at: Timestamp,
    pub ended: Ended,
    pub space: Vec<VolumeSpace>,
}

/// An attempt to put an item back.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PutBackRecord {
    pub cleanup: CleanupId,
    pub index: u32,
    pub at: Timestamp,
    pub outcome: PutBackOutcome,
    /// The parts of an item of several paths this attempt brought back, by their place among the item's parts (the
    /// subject is 0); empty for any other item.
    pub back: Vec<u32>,
}

/// One cleanup as the journal read it back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedCleanup {
    pub cleanup: CleanupId,
    pub begin: BeginRecord,
    pub items: Vec<ItemRecord>,
    /// `None`: the app stopped during the cleanup.
    pub end: Option<EndRecord>,
    pub put_backs: Vec<PutBackRecord>,
}

impl RecordedCleanup {
    /// How an item ended: `Unknown` when the journal has no outcome for it.
    pub fn outcome(&self, index: u32) -> RecordedOutcome {
        self.items
            .iter()
            .rev()
            .find(|record| record.index == index)
            .map_or(RecordedOutcome::Unknown, |record| record.outcome)
    }

    /// The summary the history shows. `in_trash` counts the items moved to the Trash and not put back, as recorded;
    /// the engine checks where they are now.
    pub fn summary(&self) -> CleanupSummary {
        let mut report = CleanupReport::new(self.begin.method);
        for item in &self.begin.items {
            report.count_recorded(&self.outcome(item.index), item.sizes);
        }
        let put_back = |index: u32| {
            self.put_backs.iter().any(|record| record.index == index && record.outcome == PutBackOutcome::PutBack)
        };
        let trashed = self.items.iter().filter(|record| !record.trashed.is_empty() && !put_back(record.index));
        let returned = self.begin.items.iter().filter(|item| put_back(item.index));
        CleanupSummary {
            cleanup: self.cleanup,
            started: self.begin.at,
            method: self.begin.method,
            ended: self.end.as_ref().map_or(Ended::Interrupted, |end| end.ended),
            count: u32::try_from(self.begin.items.len()).unwrap_or(u32::MAX),
            counts: report.counts,
            moved: report.moved,
            deleted: report.deleted,
            in_trash: u32::try_from(trashed.count()).unwrap_or(u32::MAX),
            put_back: u32::try_from(returned.count()).unwrap_or(u32::MAX),
        }
    }
}

/// A page of the journal's cleanups.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalPage {
    /// Newest first.
    pub cleanups: Vec<CleanupSummary>,
    /// How many cleanups the journal holds.
    pub total: u32,
    /// Lines that could not be read (a line torn by a crash) and were skipped.
    pub skipped: u32,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cleanup::plan::sample;
    use crate::cleanup::{
        BeginRecord, CleanupId, CleanupPlan, ConfirmedPlan, EndRecord, Ended, ItemIdentity, ItemRecord, PlanId,
        PlanItem, PutBackRecord, RecordedCleanup, RecordedOutcome, TrashedItem,
    };
    use crate::{EntryKind, ScanId, Timestamp};

    fn cleanup(items: Vec<ItemRecord>, end: Option<Ended>, put_backs: Vec<PutBackRecord>) -> RecordedCleanup {
        let echo = serde_json::json!({
            "plan": 7, "count": 3, "allocated": 12_388, "highest": "medium", "method": "trash",
            "highRiskUnderstood": false,
        });
        let confirmed =
            sample::plan().confirm(&serde_json::from_value(echo).unwrap(), Timestamp::from_secs(sample::MADE)).unwrap();
        let id = CleanupId::new(2);
        RecordedCleanup {
            cleanup: id,
            begin: BeginRecord::new(Timestamp::from_secs(5), &confirmed, vec![]),
            items,
            end: end.map(|ended| EndRecord { cleanup: id, at: Timestamp::from_secs(6), ended, space: vec![] }),
            put_backs,
        }
    }

    /// The confirmed plan of `item` alone, a medium item of 4 KiB.
    fn confirmed_one(item: PlanItem) -> ConfirmedPlan {
        let plan = CleanupPlan::new(
            PlanId::new(7),
            Timestamp::from_secs(sample::MADE),
            ScanId::new(1),
            vec!["/home".into()],
            vec![item],
            vec![],
        )
        .unwrap();
        let echo = serde_json::json!({
            "plan": 7, "count": 1, "allocated": 4096, "highest": "medium", "method": "trash",
            "highRiskUnderstood": false,
        });
        plan.confirm(&serde_json::from_value(echo).unwrap(), Timestamp::from_secs(sample::MADE)).unwrap()
    }

    fn moved(index: u32) -> ItemRecord {
        let trashed = TrashedItem {
            original: PathBuf::from(format!("/home/cache-{index}")),
            landed: PathBuf::from(format!("/home/.Trash/cache-{index}")),
            identity: ItemIdentity::new(1, u64::from(index), EntryKind::File),
        };
        ItemRecord {
            cleanup: CleanupId::new(2),
            index,
            outcome: RecordedOutcome::Moved,
            trashed: vec![trashed],
            went: None,
        }
    }

    mod negative {
        use std::path::PathBuf;

        use super::{cleanup, confirmed_one, moved};
        use crate::cleanup::plan::sample;
        use crate::cleanup::{
            BeginRecord, CleanupId, Ended, ItemIdentity, JournalPart, PlanItem, PutBackOutcome, PutBackRecord,
            RecordedOutcome,
        };
        use crate::{DangerLevel, EntryKind, Timestamp};

        #[test]
        fn an_item_without_an_identity_for_every_part_still_keeps_every_part() {
            let finding = sample::together(4, DangerLevel::Medium, 4096);
            let short = PlanItem::new(&finding).with_part_identities(vec![ItemIdentity::new(1, 10, EntryKind::File)]);

            let none = BeginRecord::new(Timestamp::from_secs(5), &confirmed_one(PlanItem::new(&finding)), vec![]);
            let partial = BeginRecord::new(Timestamp::from_secs(5), &confirmed_one(short), vec![]);

            let layer = [JournalPart { path: PathBuf::from("/home/layer-4"), identity: None }];
            assert_eq!(none.items[0].more, layer);
            assert_eq!(partial.items[0].more, layer);
        }

        #[test]
        fn a_cleanup_without_an_end_is_interrupted_and_its_items_unknown() {
            let recorded = cleanup(vec![moved(0)], None, vec![]);

            let summary = recorded.summary();

            assert_eq!(summary.ended, Ended::Interrupted);
            assert_eq!(recorded.outcome(1), RecordedOutcome::Unknown);
            assert_eq!((summary.counts.moved, summary.counts.unknown), (1, 2));
        }

        #[test]
        fn an_item_put_back_is_no_longer_in_the_trash() {
            let put_back = |outcome| PutBackRecord {
                cleanup: CleanupId::new(2),
                index: 0,
                at: Timestamp::from_secs(9),
                outcome,
                back: vec![],
            };
            let occupied = cleanup(vec![moved(0)], Some(Ended::Completed), vec![put_back(PutBackOutcome::Occupied)]);
            let done = cleanup(vec![moved(0)], Some(Ended::Completed), vec![put_back(PutBackOutcome::PutBack)]);

            assert_eq!((occupied.summary().in_trash, occupied.summary().put_back), (1, 0));
            assert_eq!((done.summary().in_trash, done.summary().put_back), (0, 1));
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};

        use super::{cleanup, confirmed_one, moved};
        use crate::cleanup::plan::sample;
        use crate::cleanup::{ActionKind, BeginRecord, CleanupId, Ended, ItemIdentity, JournalPart, Method, PlanItem};
        use crate::{ByteSize, DangerLevel, EntryKind, Timestamp};

        #[test]
        fn the_begin_record_holds_every_item_of_the_plan() {
            let recorded = cleanup(vec![], None, vec![]);
            let begin = &recorded.begin;

            assert_eq!((begin.at, begin.method), (Timestamp::from_secs(5), Method::Trash));
            assert_eq!(begin.roots, [Path::new("/home")]);
            assert_eq!(begin.items.len(), 3);
            let first = &begin.items[0];
            assert_eq!((first.index, first.action, first.danger), (0, ActionKind::Remove, DangerLevel::Low));
            assert_eq!(first.path.as_deref(), Some(Path::new("/home/cache-1")));
            assert_eq!((first.category.as_str(), first.key.as_str()), ("demo.caches", "item-1"));
            assert_eq!(first.reason.key().as_str(), "demo.finding.reason");
            assert_eq!(begin.items[2].path, None);
            assert_eq!(begin.items[2].action, ActionKind::Tool);
        }

        #[test]
        fn an_item_of_several_paths_keeps_every_part_after_its_subject() {
            let identities = vec![ItemIdentity::new(1, 10, EntryKind::File), ItemIdentity::new(1, 11, EntryKind::File)];
            let item =
                PlanItem::new(&sample::together(4, DangerLevel::Medium, 4096)).with_part_identities(identities.clone());

            let begin = BeginRecord::new(Timestamp::from_secs(5), &confirmed_one(item), vec![]);

            let item = &begin.items[0];
            assert_eq!((item.path.as_deref(), item.identity), (Some(Path::new("/home/model-4")), Some(identities[0])));
            assert_eq!(
                item.more,
                [JournalPart { path: PathBuf::from("/home/layer-4"), identity: Some(identities[1]) }]
            );
            assert_eq!(cleanup(vec![], None, vec![]).begin.items[0].more, []);
        }

        #[test]
        fn a_summary_sums_what_was_moved() {
            let summary = cleanup(vec![moved(0), moved(1)], Some(Ended::Cancelled), vec![]).summary();

            assert_eq!(summary.cleanup, CleanupId::new(2));
            assert_eq!((summary.started, summary.ended, summary.count), (Timestamp::from_secs(5), Ended::Cancelled, 3));
            assert_eq!(summary.moved.allocated, ByteSize::new(4096 + 8192));
            assert_eq!((summary.in_trash, summary.counts.unknown), (2, 1));
            assert_eq!(summary.put_back, 0);
        }
    }
}
