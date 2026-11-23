//! Fakes of the cleanup's ports: the open files, the cleanup journal, and a cleaner for the engine's tests.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use fresher_core::{
    Action, BeginRecord, CancelToken, Cleaner, CleanupContext, CleanupId, CleanupJournal, ConfirmedPlan, Depth,
    EndRecord, Ended, EntryKind, Failure, FindingId, ItemDone, ItemIdentity, ItemOutcome, ItemRecord, JournalError,
    JournalPage, JournalProblem, JournalState, Method, OpenFiles, OpenFilesError, OpenSet, PartlyWay, PlanItem,
    PutBackOutcome, PutBackRecord, RecordedCleanup, Rehearsal, SkipReason, TrashState, TrashedItem,
};

use crate::fakes::lock;

/// The files the user's processes hold open, as a test sets them; or a snapshot that fails.
#[derive(Debug, Default)]
pub struct FakeOpenFiles {
    held: Mutex<Vec<(PathBuf, String)>>,
    failing: bool,
    snapshots: Mutex<u32>,
}

impl FakeOpenFiles {
    /// Every snapshot fails, as when the process table cannot be read.
    pub fn failing() -> Self {
        Self { failing: true, ..Self::default() }
    }

    /// `app` holds `path` open from now on.
    #[must_use]
    pub fn holding(self, path: impl Into<PathBuf>, app: &str) -> Self {
        self.hold(path, app);
        self
    }

    /// `app` opens `path` now: the next snapshot sees it.
    pub fn hold(&self, path: impl Into<PathBuf>, app: &str) {
        lock(&self.held).push((path.into(), app.to_owned()));
    }

    /// How many snapshots were taken.
    pub fn snapshots(&self) -> u32 {
        *lock(&self.snapshots)
    }
}

impl OpenFiles for FakeOpenFiles {
    fn snapshot(&self, within: &[PathBuf]) -> Result<OpenSet, OpenFilesError> {
        *lock(&self.snapshots) += 1;
        if self.failing {
            return Err(OpenFilesError { reason: "failing by the test".into() });
        }
        let inside = |path: &Path| within.iter().any(|root| path.starts_with(root));
        Ok(OpenSet::new(lock(&self.held).iter().filter(|(path, _)| inside(path)).cloned().collect()))
    }
}

/// A cleanup journal in memory: the records as the port was given them, read back as the store would.
#[derive(Debug, Default)]
pub struct MemoryCleanupJournal {
    cleanups: Mutex<Vec<RecordedCleanup>>,
    read_only: bool,
    unreadable: bool,
    writes_left: Option<Mutex<u32>>,
    syncs: Mutex<u32>,
}

impl MemoryCleanupJournal {
    /// A journal written by a newer version of the app: read, never written.
    pub fn read_only() -> Self {
        Self { read_only: true, ..Self::default() }
    }

    /// A journal that cannot be read: every page and record answers an error.
    pub fn unreadable() -> Self {
        Self { unreadable: true, ..Self::default() }
    }

    /// A journal whose first `writes` records are written, and every one after fails.
    pub fn failing_after(writes: u32) -> Self {
        Self { writes_left: Some(Mutex::new(writes)), ..Self::default() }
    }

    /// Every cleanup recorded, oldest first.
    pub fn recorded(&self) -> Vec<RecordedCleanup> {
        lock(&self.cleanups).clone()
    }

    /// How many times the records were made durable.
    pub fn syncs(&self) -> u32 {
        *lock(&self.syncs)
    }

    fn read(&self) -> Result<(), JournalError> {
        if self.unreadable {
            return Err(JournalError::Failed { reason: "unreadable by the test".into() });
        }
        Ok(())
    }

    fn write(&self) -> Result<(), JournalError> {
        if self.read_only {
            return Err(JournalError::ReadOnly);
        }
        if let Some(left) = &self.writes_left {
            let mut left = lock(left);
            if *left == 0 {
                return Err(JournalError::Failed { reason: "failing by the test".into() });
            }
            *left -= 1;
        }
        Ok(())
    }

    fn with_cleanup(&self, id: CleanupId, change: impl FnOnce(&mut RecordedCleanup)) -> Result<(), JournalError> {
        self.write()?;
        let mut cleanups = lock(&self.cleanups);
        let cleanup = cleanups
            .iter_mut()
            .find(|cleanup| cleanup.cleanup == id)
            .ok_or_else(|| JournalError::Failed { reason: format!("no cleanup {id}") })?;
        change(cleanup);
        Ok(())
    }
}

impl CleanupJournal for MemoryCleanupJournal {
    fn state(&self) -> JournalState {
        if self.read_only { JournalState::ReadOnly { why: JournalProblem::NewerVersion } } else { JournalState::Ready }
    }

    fn begin(&self, record: &BeginRecord) -> Result<CleanupId, JournalError> {
        self.write()?;
        let mut cleanups = lock(&self.cleanups);
        let id = CleanupId::new(u32::try_from(cleanups.len()).unwrap_or(u32::MAX).saturating_add(1));
        cleanups.push(RecordedCleanup {
            cleanup: id,
            begin: record.clone(),
            items: Vec::new(),
            end: None,
            put_backs: Vec::new(),
        });
        *lock(&self.syncs) += 1;
        Ok(id)
    }

    fn item(&self, record: &ItemRecord) -> Result<(), JournalError> {
        self.with_cleanup(record.cleanup, |cleanup| cleanup.items.push(record.clone()))
    }

    fn sync(&self) -> Result<(), JournalError> {
        *lock(&self.syncs) += 1;
        Ok(())
    }

    fn end(&self, record: &EndRecord) -> Result<(), JournalError> {
        self.with_cleanup(record.cleanup, |cleanup| cleanup.end = Some(record.clone()))?;
        self.sync()
    }

    fn put_back(&self, record: &PutBackRecord) -> Result<(), JournalError> {
        self.with_cleanup(record.cleanup, |cleanup| cleanup.put_backs.push(record.clone()))?;
        self.sync()
    }

    fn cleanups(&self, offset: u32, limit: u32) -> Result<JournalPage, JournalError> {
        self.read()?;
        let cleanups = lock(&self.cleanups);
        let page =
            cleanups.iter().rev().skip(offset as usize).take(limit as usize).map(RecordedCleanup::summary).collect();
        Ok(JournalPage { cleanups: page, total: u32::try_from(cleanups.len()).unwrap_or(u32::MAX), skipped: 0 })
    }

    fn cleanup(&self, id: CleanupId) -> Result<Option<RecordedCleanup>, JournalError> {
        self.read()?;
        Ok(lock(&self.cleanups).iter().find(|cleanup| cleanup.cleanup == id).cloned())
    }
}

/// A plan the fake cleaner was given to execute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutedPlan {
    pub method: Method,
    pub items: Vec<PlanItem>,
}

/// A cleaner for the engine's tests: it touches no disk, answers scripted rehearsals and outcomes, and records every
/// plan it is given. By default every item passes, a path item moves to `/fake-trash/<index>` (or is deleted, by the
/// permanent method), an item of several paths passes with an identity per part and moves part by part to
/// `/fake-trash/<index>-<part>`, a tool's item is done, every trashed item is in the Trash and puts back. A scripted
/// skip or outcome names a finding, so it applies to every part of an item of several paths.
#[derive(Debug, Default)]
pub struct FakeCleaner {
    skips: HashMap<FindingId, SkipReason>,
    outcomes: HashMap<FindingId, ItemOutcome>,
    partly: HashMap<FindingId, u32>,
    unknown_landings: HashMap<FindingId, u32>,
    elsewhere: HashMap<FindingId, u32>,
    states: Mutex<HashMap<PathBuf, TrashState>>,
    put_backs: Mutex<HashMap<PathBuf, PutBackOutcome>>,
    rehearsed: Mutex<Vec<Vec<FindingId>>>,
    executed: Mutex<Vec<ExecutedPlan>>,
    put_backs_tried: Mutex<Vec<PathBuf>>,
    protection: fresher_core::Protection,
}

impl FakeCleaner {
    /// The cleaner's own protection is `protection` (none by default); it checks nothing against it.
    #[must_use]
    pub fn protecting(mut self, protection: fresher_core::Protection) -> Self {
        self.protection = protection;
        self
    }

    /// `finding` fails a check, in a rehearsal and in an execution.
    #[must_use]
    pub fn skipping(mut self, finding: FindingId, reason: SkipReason) -> Self {
        self.skips.insert(finding, reason);
        self
    }

    /// `finding` ends as `outcome` when executed.
    #[must_use]
    pub fn ending(mut self, finding: FindingId, outcome: ItemOutcome) -> Self {
        self.outcomes.insert(finding, outcome);
        self
    }

    /// Only the first `done` parts of `finding`, an item of several paths, go to the Trash: it ends `Failed` with
    /// `Partly`, those parts trashed.
    #[must_use]
    pub fn partly(mut self, finding: FindingId, done: u32) -> Self {
        self.partly.insert(finding, done);
        self
    }

    /// Part `part` of `finding`, an item of several paths, goes to the Trash at a place the Trash does not tell: it
    /// counts as moved and has no trashed entry.
    #[must_use]
    pub fn landing_unknown(mut self, finding: FindingId, part: u32) -> Self {
        self.unknown_landings.insert(finding, part);
        self
    }

    /// Part `part` of `finding`, an item of several paths that goes only `partly`, left its place all the same and
    /// has a trashed entry, which the record's `went` does not count (as a record may not): put back must go by the
    /// entries too.
    #[must_use]
    pub fn landing_elsewhere(mut self, finding: FindingId, part: u32) -> Self {
        self.elsewhere.insert(finding, part);
        self
    }

    /// The item that landed at `landed` is in `state`.
    #[must_use]
    pub fn locating(self, landed: impl Into<PathBuf>, state: TrashState) -> Self {
        self.now_locating(landed, state);
        self
    }

    /// Putting back the item that landed at `landed` ends as `outcome`.
    #[must_use]
    pub fn putting_back(self, landed: impl Into<PathBuf>, outcome: PutBackOutcome) -> Self {
        self.now_putting_back(landed, outcome);
        self
    }

    /// From now on, the item that landed at `landed` is in `state`.
    pub fn now_locating(&self, landed: impl Into<PathBuf>, state: TrashState) {
        lock(&self.states).insert(landed.into(), state);
    }

    /// From now on, putting back the item that landed at `landed` ends as `outcome`.
    pub fn now_putting_back(&self, landed: impl Into<PathBuf>, outcome: PutBackOutcome) {
        lock(&self.put_backs).insert(landed.into(), outcome);
    }

    /// The findings of each rehearsal, in order.
    pub fn rehearsed(&self) -> Vec<Vec<FindingId>> {
        lock(&self.rehearsed).clone()
    }

    /// Every plan executed, in order.
    pub fn executed(&self) -> Vec<ExecutedPlan> {
        lock(&self.executed).clone()
    }

    /// The landed path of every item asked to be put back, in the order asked.
    pub fn put_backs_tried(&self) -> Vec<PathBuf> {
        lock(&self.put_backs_tried).clone()
    }

    /// The identity the fake reads for an item: the one it has, or one made from its finding.
    fn identity(item: &PlanItem) -> Option<ItemIdentity> {
        item.action().path()?;
        Some(item.identity().unwrap_or_else(|| ItemIdentity::new(1, item.finding().get().into(), EntryKind::Folder)))
    }

    /// The identity the fake reads for each part of an item of several paths, in part order: the ones it has, or
    /// ones made from its finding and the part's place; empty for any other item.
    fn part_identities(item: &PlanItem) -> Vec<ItemIdentity> {
        if !matches!(item.action(), Action::RemoveTogether(_)) {
            return Vec::new();
        }
        if !item.part_identities().is_empty() {
            return item.part_identities().to_vec();
        }
        let made = |part: u64| ItemIdentity::new(part + 1, item.finding().get().into(), EntryKind::File);
        (0_u64..).zip(item.action().parts()).map(|(part, _)| made(part)).collect()
    }

    fn done(&self, index: u32, item: &PlanItem, method: Method) -> ItemDone {
        let path = item.action().path();
        let parts = Self::part_identities(item);
        let partly = self.partly.get(&item.finding()).copied().filter(|_| !parts.is_empty());
        let outcome = match (self.skips.get(&item.finding()), self.outcomes.get(&item.finding()), partly) {
            (Some(reason), _, _) => ItemOutcome::Skipped { reason: reason.clone() },
            (None, Some(outcome), _) => outcome.clone(),
            (None, None, Some(done)) => {
                let of = u32::try_from(parts.len()).unwrap_or(u32::MAX);
                ItemOutcome::Failed { reason: Failure::Partly { done, of, way: PartlyWay::Trash }.text() }
            }
            (None, None, None) => match (path, method) {
                (None, _) => ItemOutcome::ToolDone,
                (Some(_), Method::Trash) => ItemOutcome::Moved,
                (Some(_), Method::Permanent) => ItemOutcome::Deleted,
            },
        };
        let went = match (&outcome, partly) {
            (ItemOutcome::Moved, _) => parts.len(),
            (ItemOutcome::Failed { .. }, Some(done)) => usize::try_from(done).unwrap_or(usize::MAX),
            _ => 0,
        };
        let unknown = self.unknown_landings.get(&item.finding()).copied();
        let elsewhere = self.elsewhere.get(&item.finding()).copied().filter(|_| partly.is_some());
        let trashed = match (&outcome, path, Self::identity(item)) {
            (ItemOutcome::Moved | ItemOutcome::Failed { .. }, Some(_), Some(_)) if !parts.is_empty() => item
                .action()
                .parts()
                .iter()
                .zip(parts)
                .zip(0_u32..)
                .filter(|&(_, part)| usize::try_from(part).is_ok_and(|part| part < went) || Some(part) == elsewhere)
                .filter(|&(_, part)| Some(part) != unknown)
                .map(|((original, identity), part)| TrashedItem {
                    original: original.clone(),
                    landed: Path::new("/fake-trash").join(format!("{index}-{part}")),
                    identity,
                })
                .collect(),
            (ItemOutcome::Moved, Some(original), Some(identity)) => vec![TrashedItem {
                original: original.to_path_buf(),
                landed: Path::new("/fake-trash").join(index.to_string()),
                identity,
            }],
            _ => Vec::new(),
        };
        let status = path.is_none().then_some(i32::from(!matches!(outcome, ItemOutcome::ToolDone)));
        let went = (went > 0).then(|| u32::try_from(went).unwrap_or(u32::MAX));
        ItemDone { index, finding: item.finding(), sizes: item.sizes(), outcome, trashed, status, went }
    }
}

impl Cleaner for FakeCleaner {
    fn rehearse(
        &self,
        _roots: &[PathBuf],
        items: &[PlanItem],
        _cx: &CleanupContext<'_>,
        _depth: Depth,
    ) -> Vec<Rehearsal> {
        lock(&self.rehearsed).push(items.iter().map(PlanItem::finding).collect());
        items
            .iter()
            .map(|item| match self.skips.get(&item.finding()) {
                Some(reason) => Rehearsal::Skipped(reason.clone()),
                None if matches!(item.action(), Action::RemoveTogether(_)) => {
                    Rehearsal::PassedTogether(Self::part_identities(item))
                }
                None => Rehearsal::Passed(Self::identity(item)),
            })
            .collect()
    }

    fn execute(
        &self,
        plan: ConfirmedPlan,
        _cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended {
        let method = plan.method();
        lock(&self.executed).push(ExecutedPlan { method, items: plan.items().to_vec() });
        let mut ended = Ended::Completed;
        for (item, index) in plan.items().iter().zip(0_u32..) {
            if cancel.is_cancelled() {
                ended = Ended::Cancelled;
                let rest = ItemDone {
                    outcome: ItemOutcome::NotReached,
                    trashed: Vec::new(),
                    status: None,
                    went: None,
                    ..self.done(index, item, method)
                };
                out(rest);
            } else {
                out(self.done(index, item, method));
            }
        }
        ended
    }

    fn protection(&self) -> fresher_core::Protection {
        self.protection.clone()
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        let states = lock(&self.states);
        items.iter().map(|item| states.get(&item.landed).copied().unwrap_or(TrashState::InTrash)).collect()
    }

    fn put_back(&self, item: &TrashedItem, _cx: &CleanupContext<'_>) -> PutBackOutcome {
        lock(&self.put_backs_tried).push(item.landed.clone());
        lock(&self.put_backs).get(&item.landed).copied().unwrap_or(PutBackOutcome::PutBack)
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{
        AccessStatus, ByteSize, CategoryDescriptor, CategoryId, CleanupContext, CleanupPlan, Confirmation,
        ConfirmedPlan, DangerLevel, Finding, FindingId, ModuleId, NewFinding, Parts, PlanId, PlanItem, Protection,
        Removability, Removal, ScanId, Sizes, Subject, Text, TextKey, Timestamp,
    };

    fn finding(id: u32) -> Finding {
        let demo = ModuleId::new("demo").unwrap();
        let category = CategoryDescriptor::new(CategoryId::new(demo, "demo.caches").unwrap(), DangerLevel::Low);
        let path = std::path::PathBuf::from(format!("/home/cache-{id}"));
        let text = Text::new(TextKey::new("demo.finding.title").unwrap());
        let sizes = Sizes::complete(ByteSize::new(1), ByteSize::new(4096));
        let new = NewFinding::new(
            &category,
            format!("{id}"),
            Subject::Path(path.clone()),
            sizes,
            text.clone(),
            text,
            Removal::MoveToTrash(path),
        );
        Finding::new(FindingId::new(id), new, Removability::Removable)
    }

    fn items() -> Vec<PlanItem> {
        [1, 2, 3].map(|id| PlanItem::new(&finding(id))).to_vec()
    }

    /// The item of a finding whose `/home/model-<id>` and `/home/layer-<id>` leave together.
    fn together(id: u32) -> PlanItem {
        let base = finding(id);
        let inner = base.finding();
        let category = CategoryDescriptor::new(inner.category(), inner.danger());
        let paths = vec![format!("/home/model-{id}").into(), format!("/home/layer-{id}").into()];
        let parts = Parts::new(paths).unwrap();
        let new = NewFinding::new(
            &category,
            format!("{id}"),
            Subject::Path(parts.subject().to_path_buf()),
            inner.sizes(),
            inner.title().clone(),
            inner.reason().clone(),
            Removal::MoveTogether(parts),
        );
        PlanItem::new(&Finding::new(FindingId::new(id), new, Removability::Removable))
    }

    fn confirmed(method: &str) -> ConfirmedPlan {
        let plan =
            CleanupPlan::new(PlanId::new(1), Timestamp::from_secs(0), ScanId::new(1), vec![], items(), vec![]).unwrap();
        let echo: Confirmation = serde_json::from_value(serde_json::json!({
            "plan": 1, "count": 3, "allocated": 3 * 4096, "highest": "low", "method": method,
            "highRiskUnderstood": false,
        }))
        .unwrap();
        plan.confirm(&echo, Timestamp::from_secs(1)).unwrap()
    }

    fn with_context<T>(run: impl FnOnce(&CleanupContext<'_>) -> T) -> T {
        let (protection, access) = (Protection::default(), AccessStatus::default());
        run(&CleanupContext { roots: &[], protection: &protection, access: &access, guarded: &[] })
    }

    mod negative {
        use std::path::PathBuf;

        use fresher_core::{
            BeginRecord, CancelToken, Cleaner, CleanupJournal, Ended, FindingId, ItemOutcome, JournalError,
            JournalState, OpenFiles, Rehearsal, SkipReason, Timestamp,
        };

        use super::{confirmed, items, together, with_context};
        use crate::{FakeCleaner, FakeOpenFiles, MemoryCleanupJournal};

        #[test]
        fn a_skipped_together_item_is_skipped_whole() {
            let cleaner = FakeCleaner::default().skipping(FindingId::new(4), SkipReason::Grown);

            let rehearsal = with_context(|cx| cleaner.rehearse(&[], &[together(4)], cx, fresher_core::Depth::Quick));

            assert_eq!(rehearsal, [Rehearsal::Skipped(SkipReason::Grown)]);
        }

        #[test]
        fn the_memory_journal_refuses_when_read_only() {
            let journal = MemoryCleanupJournal::read_only();
            let begin = BeginRecord::new(Timestamp::from_secs(1), &confirmed("trash"), vec![]);

            assert!(matches!(journal.state(), JournalState::ReadOnly { .. }));
            assert_eq!(journal.begin(&begin), Err(JournalError::ReadOnly));
            assert_eq!(journal.recorded(), []);
        }

        #[test]
        fn the_memory_journal_fails_after_its_writes() {
            let journal = MemoryCleanupJournal::failing_after(1);
            let begin = BeginRecord::new(Timestamp::from_secs(1), &confirmed("trash"), vec![]);
            journal.begin(&begin).unwrap();

            assert!(matches!(journal.begin(&begin), Err(JournalError::Failed { .. })));
        }

        #[test]
        fn an_unreadable_memory_journal_answers_errors_on_every_read() {
            let journal = MemoryCleanupJournal::unreadable();
            let begin = BeginRecord::new(Timestamp::from_secs(1), &confirmed("trash"), vec![]);
            let id = journal.begin(&begin).unwrap();

            assert!(matches!(journal.cleanups(0, 10), Err(JournalError::Failed { .. })));
            assert!(matches!(journal.cleanup(id), Err(JournalError::Failed { .. })));
        }

        #[test]
        fn a_failing_snapshot_answers_an_error() {
            let open = FakeOpenFiles::failing();

            assert!(open.snapshot(&[PathBuf::from("/home")]).is_err());
            assert_eq!(open.snapshots(), 1);
        }

        #[test]
        fn a_skip_is_scripted_and_a_cancel_reaches_nothing() {
            let cleaner = FakeCleaner::default().skipping(FindingId::new(2), SkipReason::Grown);
            let cancel = CancelToken::new();
            cancel.cancel();
            let mut outcomes = Vec::new();

            let rehearsal = with_context(|cx| cleaner.rehearse(&[], &items(), cx, fresher_core::Depth::Quick));
            let ended = with_context(|cx| {
                cleaner.execute(confirmed("trash"), cx, &cancel, &mut |done| outcomes.push(done.outcome))
            });

            assert_eq!(rehearsal[1], Rehearsal::Skipped(SkipReason::Grown));
            assert_eq!(ended, Ended::Cancelled);
            assert_eq!(outcomes, [ItemOutcome::NotReached, ItemOutcome::NotReached, ItemOutcome::NotReached]);
        }
    }

    mod positive {
        use std::path::{Path, PathBuf};

        use fresher_core::{
            BeginRecord, CancelToken, Cleaner, CleanupJournal, Depth, EndRecord, Ended, Failure, FindingId,
            ItemOutcome, Method, OpenFiles, PartlyWay, PutBackOutcome, PutBackRecord, Rehearsal, Timestamp, TrashState,
        };

        use super::{confirmed, items, together, with_context};
        use crate::{FakeCleaner, FakeOpenFiles, MemoryCleanupJournal};

        #[test]
        fn the_fake_cleaner_rehearses_and_moves_a_together_item_part_by_part() {
            let cleaner = FakeCleaner::default();
            let item = together(4);
            let plan = fresher_core::CleanupPlan::new(
                fresher_core::PlanId::new(1),
                Timestamp::from_secs(0),
                fresher_core::ScanId::new(1),
                vec![],
                vec![item.clone()],
                vec![],
            )
            .unwrap();
            let echo: fresher_core::Confirmation = serde_json::from_value(serde_json::json!({
                "plan": 1, "count": 1, "allocated": 4096, "highest": "low", "method": "trash",
                "highRiskUnderstood": false,
            }))
            .unwrap();
            let mut done = Vec::new();

            let rehearsal = with_context(|cx| cleaner.rehearse(&[], std::slice::from_ref(&item), cx, Depth::Full));
            let Rehearsal::PassedTogether(identities) = rehearsal[0].clone() else { unreachable!("{rehearsal:?}") };
            let confirmed = plan.confirm(&echo, Timestamp::from_secs(1)).unwrap();
            with_context(|cx| cleaner.execute(confirmed, cx, &CancelToken::new(), &mut |item| done.push(item)));

            assert_eq!(identities.len(), 2);
            assert_ne!(identities[0], identities[1]);
            assert_eq!(done[0].outcome, ItemOutcome::Moved);
            let trashed: Vec<_> = done[0]
                .trashed
                .iter()
                .map(|item| (item.original.clone(), item.landed.clone(), item.identity))
                .collect();
            assert_eq!(
                trashed,
                [
                    (PathBuf::from("/home/model-4"), PathBuf::from("/fake-trash/0-0"), identities[0]),
                    (PathBuf::from("/home/layer-4"), PathBuf::from("/fake-trash/0-1"), identities[1]),
                ]
            );
        }

        #[test]
        fn the_fake_cleaner_records_the_confirmed_plan() {
            let cleaner = FakeCleaner::default().ending(FindingId::new(3), ItemOutcome::NotReached);
            let mut done = Vec::new();

            let ended = with_context(|cx| {
                cleaner.execute(confirmed("permanent"), cx, &CancelToken::new(), &mut |item| done.push(item))
            });

            assert_eq!(ended, Ended::Completed);
            let executed = cleaner.executed();
            assert_eq!(executed.len(), 1);
            assert_eq!(executed[0].method, Method::Permanent);
            assert_eq!(executed[0].items, items());
            let outcomes: Vec<_> = done.iter().map(|item| item.outcome.clone()).collect();
            assert_eq!(outcomes, [ItemOutcome::Deleted, ItemOutcome::Deleted, ItemOutcome::NotReached]);
        }

        #[test]
        fn the_fake_cleaner_moves_locates_and_puts_back() {
            let cleaner = FakeCleaner::default()
                .locating("/fake-trash/1", TrashState::Gone)
                .putting_back("/fake-trash/1", PutBackOutcome::Gone);
            let mut done = Vec::new();

            let rehearsal = with_context(|cx| cleaner.rehearse(&[], &items(), cx, Depth::Full));
            with_context(|cx| {
                cleaner.execute(confirmed("trash"), cx, &CancelToken::new(), &mut |item| done.push(item))
            });

            assert!(matches!(rehearsal[0], Rehearsal::Passed(Some(_))));
            assert_eq!(cleaner.rehearsed(), [[1, 2, 3].map(FindingId::new).to_vec()]);
            let trashed: Vec<_> = done.iter().flat_map(|item| item.trashed.clone()).collect();
            assert_eq!(trashed[1].landed, Path::new("/fake-trash/1"));
            assert_eq!(cleaner.locate(&trashed), [TrashState::InTrash, TrashState::Gone, TrashState::InTrash]);
            let answers = with_context(|cx| (cleaner.put_back(&trashed[1], cx), cleaner.put_back(&trashed[0], cx)));
            assert_eq!(answers, (PutBackOutcome::Gone, PutBackOutcome::PutBack));
            assert_eq!(cleaner.put_backs_tried(), [Path::new("/fake-trash/1"), Path::new("/fake-trash/0")]);
        }

        #[test]
        fn the_fake_cleaner_moves_part_of_a_together_item_as_scripted() {
            let execute = |cleaner: &FakeCleaner| {
                let plan = fresher_core::CleanupPlan::new(
                    fresher_core::PlanId::new(1),
                    Timestamp::from_secs(0),
                    fresher_core::ScanId::new(1),
                    vec![],
                    vec![together(4)],
                    vec![],
                )
                .unwrap();
                let echo: fresher_core::Confirmation = serde_json::from_value(serde_json::json!({
                    "plan": 1, "count": 1, "allocated": 4096, "highest": "low", "method": "trash",
                    "highRiskUnderstood": false,
                }))
                .unwrap();
                let mut done = Vec::new();
                let confirmed = plan.confirm(&echo, Timestamp::from_secs(1)).unwrap();
                with_context(|cx| cleaner.execute(confirmed, cx, &CancelToken::new(), &mut |item| done.push(item)));
                done.remove(0)
            };
            let landed = |done: &fresher_core::ItemDone| -> Vec<PathBuf> {
                done.trashed.iter().map(|item| item.landed.clone()).collect()
            };

            let partly = execute(&FakeCleaner::default().partly(FindingId::new(4), 1));
            let unknown = execute(&FakeCleaner::default().landing_unknown(FindingId::new(4), 1));

            assert_eq!(
                partly.outcome,
                ItemOutcome::Failed { reason: Failure::Partly { done: 1, of: 2, way: PartlyWay::Trash }.text() }
            );
            assert_eq!((landed(&partly), partly.went), (vec![PathBuf::from("/fake-trash/0-0")], Some(1)));
            assert_eq!(unknown.outcome, ItemOutcome::Moved);
            assert_eq!((landed(&unknown), unknown.went), (vec![PathBuf::from("/fake-trash/0-0")], Some(2)));
        }

        #[test]
        fn the_memory_journal_reads_back_what_it_was_given() {
            let journal = MemoryCleanupJournal::default();
            let plan = confirmed("trash");
            let begin = BeginRecord::new(Timestamp::from_secs(1), &plan, vec![]);
            let cleaner = FakeCleaner::default();

            let id = journal.begin(&begin).unwrap();
            with_context(|cx| {
                cleaner.execute(plan, cx, &CancelToken::new(), &mut |done| journal.item(&done.record(id)).unwrap());
            });
            journal.sync().unwrap();
            let end = EndRecord { cleanup: id, at: Timestamp::from_secs(2), ended: Ended::Completed, space: vec![] };
            journal.end(&end).unwrap();
            let put_back = PutBackRecord {
                cleanup: id,
                index: 0,
                at: Timestamp::from_secs(3),
                outcome: PutBackOutcome::PutBack,
                back: vec![],
            };
            journal.put_back(&put_back).unwrap();

            let page = journal.cleanups(0, 10).unwrap();
            assert_eq!((page.total, page.cleanups[0].in_trash, page.cleanups[0].counts.moved), (1, 2, 3));
            let recorded = journal.cleanup(id).unwrap().unwrap();
            assert_eq!((recorded.items.len(), recorded.put_backs.len()), (3, 1));
            assert_eq!(journal.syncs(), 4);
            assert_eq!(journal.recorded().len(), 1);
            assert!(journal.cleanup(fresher_core::CleanupId::new(9)).unwrap().is_none());
        }

        #[test]
        fn the_fake_open_files_answer_what_is_held_within() {
            let open = FakeOpenFiles::default().holding("/home/a/db", "Notes").holding("/elsewhere/x", "Mail");

            let first = open.snapshot(&[PathBuf::from("/home")]).unwrap();
            open.hold("/home/b", "Music");
            let second = open.snapshot(&[PathBuf::from("/home")]).unwrap();

            assert_eq!(first.holder_under(Path::new("/home/a")), Some("Notes"));
            assert_eq!(first.len(), 1);
            assert_eq!(second.holder_under(Path::new("/home/b")), Some("Music"));
            assert_eq!(open.snapshots(), 2);
        }
    }
}
