//! The history of cleanups, from the journal, with where each moved item is now; putting items back; showing one in
//! the Trash.

use std::path::{Path, PathBuf};

use fresher_core::{
    BlockedReason, ByteSize, CleanupDetail, CleanupId, CleanupItemView, CleanupJournal, CleanupSummary, CleanupsPage,
    CleanupsQuery, CommandError, JournalItem, JournalState, PutBackItem, PutBackOutcome, PutBackRecord, PutBackReport,
    RecordedCleanup, RecordedOutcome, Sizes, SkipCode, Timestamp, TrashState, TrashedItem,
};

use crate::cleanup::context::Context;
use crate::engine::{CleanupPorts, Shared};
use crate::error::{EngineError, lock};

/// A page of the history, newest first, at most [`CleanupsQuery::MAX_LIMIT`] cleanups. Each cleanup's "in the Trash"
/// counts the items the cleaner finds still there; `waiting` sums their allocated sizes over the page. A journal
/// that cannot be written says so in `blocked`; one that cannot be read answers an empty page.
pub(crate) fn page(shared: &Shared, offset: u32, limit: u32) -> CleanupsPage {
    let limit = CleanupsQuery::new(offset, limit).limit();
    let mut page = CleanupsPage {
        cleanups: Vec::new(),
        offset,
        total: 0,
        waiting: ByteSize::ZERO,
        blocked: Some(BlockedReason::Journal),
    };
    let Some(ports) = &shared.cleanup else { return page };
    if ports.journal.state() == JournalState::Ready {
        page.blocked = None;
    }
    let read = match ports.journal.cleanups(offset, limit) {
        Ok(read) => read,
        Err(error) => {
            tracing::warn!(%error, "the history of cleanups could not be read");
            return page;
        }
    };
    page.total = read.total;
    let mut waiting = 0_u64;
    for summary in read.cleanups {
        match ports.journal.cleanup(summary.cleanup) {
            Ok(Some(recorded)) => {
                let located = Located::of(ports, &recorded);
                waiting = waiting.saturating_add(located.waiting());
                page.cleanups.push(CleanupSummary { in_trash: located.in_trash(), ..summary });
            }
            Ok(None) | Err(_) => page.cleanups.push(summary),
        }
    }
    page.waiting = ByteSize::new(waiting);
    page
}

/// One cleanup with every item: its outcome, where it is now, how putting it back last ended.
pub(crate) fn detail(shared: &Shared, id: CleanupId) -> Result<CleanupDetail, CommandError> {
    let (ports, recorded) = recorded(shared, id)?;
    let located = Located::of(ports, &recorded);
    let items = recorded
        .begin
        .items
        .iter()
        .map(|item| CleanupItemView {
            index: item.index,
            path: item.path.clone(),
            category: item.category.clone(),
            danger: item.danger,
            sizes: item.sizes,
            action: item.action,
            outcome: recorded.outcome(item.index),
            trash: located.state(item.index),
            put_back: last_put_back(&recorded, item.index),
        })
        .collect();
    Ok(CleanupDetail { summary: CleanupSummary { in_trash: located.in_trash(), ..recorded.summary() }, items })
}

/// Puts back `items` of cleanup `id` — every item still in the Trash when `None` — each through the cleaner with the
/// context as it stands now, each attempt journalled. An item put back already answers `PutBack` again, with no
/// attempt and no record: the journal's last answer for it stays the one that put it back. A short operation: refused while a scan or a cleanup runs, and
/// none starts until it ends. An index the cleanup does not have refuses the whole request before any item.
pub(crate) fn put_back(shared: &Shared, id: CleanupId, items: Option<&[u32]>) -> Result<PutBackReport, CommandError> {
    let machine = lock(&shared.machine)?;
    machine.idle()?;
    let (ports, recorded) = recorded(shared, id)?;
    if ports.journal.state() != JournalState::Ready {
        return Err(CommandError::CleaningBlocked { reason: BlockedReason::Journal });
    }
    let located = Located::of(ports, &recorded);
    let indexes: Vec<u32> = match items {
        Some(items) => {
            if !items.iter().all(|&index| has_item(&recorded, index)) {
                return Err(CommandError::UnknownCleanupItem);
            }
            let mut unique: Vec<u32> = Vec::with_capacity(items.len());
            for &index in items {
                if !unique.contains(&index) {
                    unique.push(index);
                }
            }
            unique
        }
        None => located.still_in_trash().collect(),
    };
    let cx = Context::now(shared)?;
    let mut report = PutBackReport { items: Vec::with_capacity(indexes.len()) };
    for index in indexes {
        if located.returned.contains(&index) {
            report.items.push(PutBackItem::new(index, PutBackOutcome::PutBack));
            continue;
        }
        let (outcome, back) = located.put_back(ports, &recorded, index, &cx);
        let at = Timestamp::from_system_time(shared.ports.clock.now());
        if let Err(error) = ports.journal.put_back(&PutBackRecord { cleanup: id, index, at, outcome, back }) {
            tracing::warn!(%error, cleanup = id.get(), index, "an attempt to put an item back was not recorded");
        }
        report.items.push(PutBackItem::new(index, outcome));
    }
    Ok(report)
}

/// Shows item `index` of cleanup `id` in the Trash, selected: the landed path stays in the core. Refused for an item
/// that is not in the Trash now (`NotInTrash`).
pub(crate) fn reveal(shared: &Shared, id: CleanupId, index: u32) -> Result<(), CommandError> {
    let (ports, recorded) = recorded(shared, id)?;
    if !has_item(&recorded, index) {
        return Err(CommandError::UnknownCleanupItem);
    }
    let located = Located::of(ports, &recorded);
    let landed = located.in_trash_entries(index).next().ok_or(CommandError::NotInTrash)?;
    shared.ports.system_ui.reveal(&landed.landed).map_err(|error| EngineError::SystemUi(error).into())
}

/// The cleanup ports and cleanup `id` as the journal recorded it; `UnknownCleanup` for an id it does not have.
fn recorded(shared: &Shared, id: CleanupId) -> Result<(&CleanupPorts, RecordedCleanup), CommandError> {
    let unknown = CommandError::UnknownCleanup { id };
    let ports = shared.cleanup.as_ref().ok_or_else(|| unknown.clone())?;
    let recorded = read(&*ports.journal, id)?.ok_or(unknown)?;
    Ok((ports, recorded))
}

fn read(journal: &dyn CleanupJournal, id: CleanupId) -> Result<Option<RecordedCleanup>, EngineError> {
    journal.cleanup(id).map_err(EngineError::Journal)
}

fn has_item(recorded: &RecordedCleanup, index: u32) -> bool {
    recorded.begin.items.iter().any(|item| item.index == index)
}

/// How the last attempt to put item `index` back ended.
fn last_put_back(recorded: &RecordedCleanup, index: u32) -> Option<PutBackOutcome> {
    recorded.put_backs.iter().rev().find(|record| record.index == index).map(|record| record.outcome)
}

/// Where each item of one cleanup moved to the Trash and not put back is now, entry by entry (an emptied folder has
/// one entry per item that went).
pub(crate) struct Located {
    /// Per item index with entries in the Trash: each entry and its state.
    items: Vec<(u32, Vec<(TrashedItem, TrashState)>)>,
    /// The items put back.
    returned: Vec<u32>,
    /// The sizes of each item, by index.
    sizes: Vec<(u32, Sizes)>,
}

impl Located {
    pub(crate) fn of(ports: &CleanupPorts, recorded: &RecordedCleanup) -> Self {
        let put_back = |index: u32| {
            recorded.put_backs.iter().any(|record| record.index == index && record.outcome == PutBackOutcome::PutBack)
        };
        let (mut items, mut returned) = (Vec::new(), Vec::new());
        for item in &recorded.begin.items {
            if put_back(item.index) {
                returned.push(item.index);
                continue;
            }
            let latest = recorded.items.iter().rev().find(|record| record.index == item.index);
            let Some(record) = latest.filter(|record| !record.trashed.is_empty()) else { continue };
            let states = ports.cleaner.locate(&record.trashed);
            let entries =
                record.trashed.iter().cloned().zip(states.into_iter().chain(std::iter::repeat(TrashState::Gone)));
            items.push((item.index, entries.collect()));
        }
        let sizes = recorded.begin.items.iter().map(|item| (item.index, item.sizes)).collect();
        Self { items, returned, sizes }
    }

    fn entries(&self, index: u32) -> Option<&[(TrashedItem, TrashState)]> {
        self.items.iter().find(|(item, _)| *item == index).map(|(_, entries)| entries.as_slice())
    }

    /// Where item `index` is now: in the Trash when an entry of it is, else replaced when one was, else gone; `None`
    /// for an item that never went to the Trash or was put back.
    fn state(&self, index: u32) -> Option<TrashState> {
        let entries = self.entries(index)?;
        let any = |wanted: TrashState| entries.iter().any(|(_, state)| *state == wanted);
        Some(if any(TrashState::InTrash) {
            TrashState::InTrash
        } else if any(TrashState::Replaced) {
            TrashState::Replaced
        } else {
            TrashState::Gone
        })
    }

    fn still_in_trash(&self) -> impl Iterator<Item = u32> + '_ {
        self.items.iter().map(|&(index, _)| index).filter(|&index| self.state(index) == Some(TrashState::InTrash))
    }

    fn in_trash_entries(&self, index: u32) -> impl Iterator<Item = &TrashedItem> {
        let entries = self.entries(index).unwrap_or_default();
        entries.iter().filter(|(_, state)| *state == TrashState::InTrash).map(|(entry, _)| entry)
    }

    fn in_trash(&self) -> u32 {
        u32::try_from(self.still_in_trash().count()).unwrap_or(u32::MAX)
    }

    /// The allocated size of the items still in the Trash.
    fn waiting(&self) -> u64 {
        self.still_in_trash().map(|index| self.sizes_of(index).allocated.bytes()).fold(0, u64::saturating_add)
    }

    /// Each item still in the Trash, once: its index, its sizes and its first entry there.
    pub(crate) fn waiting_items(&self) -> impl Iterator<Item = (u32, Sizes, &TrashedItem)> {
        self.still_in_trash()
            .filter_map(|index| self.in_trash_entries(index).next().map(|entry| (index, self.sizes_of(index), entry)))
    }

    fn sizes_of(&self, index: u32) -> Sizes {
        self.sizes.iter().find(|(item, _)| *item == index).map_or(Sizes::ZERO, |&(_, sizes)| sizes)
    }

    /// Puts item `index` back, entry by entry: only the entries still in the Trash are tried. Answers `NotTrashed` for
    /// an item that never went to the Trash, the state of an item no entry of which is there, else the first entry's
    /// refusal, else `PutBack`. An item of several paths puts back as [`Self::put_back_together`] says; with the
    /// answer come the parts of it brought back. The caller answers for an item put back already.
    fn put_back(
        &self,
        ports: &CleanupPorts,
        recorded: &RecordedCleanup,
        index: u32,
        cx: &Context,
    ) -> (PutBackOutcome, Vec<u32>) {
        let Some(state) = self.state(index) else { return (PutBackOutcome::NotTrashed, Vec::new()) };
        let begun = recorded.begin.items.iter().find(|item| item.index == index);
        let went = recorded.items.iter().rev().find(|record| record.index == index).and_then(|record| record.went);
        if let Some(item) = begun.filter(|item| !item.more.is_empty() || went.is_some()) {
            return self.put_back_together(ports, recorded, item, went, cx);
        }
        let outcome =
            match state {
                TrashState::Gone => PutBackOutcome::Gone,
                TrashState::Replaced => PutBackOutcome::Replaced,
                TrashState::InTrash => {
                    let outcomes =
                        self.in_trash_entries(index).map(|entry| ports.cleaner.put_back(entry, &cx.borrow()));
                    outcomes.fold(PutBackOutcome::PutBack, |first, outcome| {
                        if first == PutBackOutcome::PutBack { outcome } else { first }
                    })
                }
            };
        (outcome, Vec::new())
    }

    /// Puts back an item of several paths that went to the Trash, so a model never comes back as a manifest that
    /// names a layer still in the Trash: the parts other than the subject that went and are not back yet first, in
    /// the reverse of the order they moved; then the subject, only when every other part that went is back (brought
    /// back now, or by an earlier attempt as the journal records). Otherwise the subject stays in the Trash and the
    /// answer is the first refusal. A part that went with no entry (the Trash did not tell where it landed), or whose
    /// entry is no longer in the Trash and was never brought back, cannot come back and keeps the subject there.
    /// Answers the parts brought back by their place among the item's parts (the subject is 0). A record that does not
    /// hold together — more parts went than the item has, or an entry of a path that is none of its parts — puts
    /// nothing back.
    fn put_back_together(
        &self,
        ports: &CleanupPorts,
        recorded: &RecordedCleanup,
        item: &JournalItem,
        went: Option<u32>,
        cx: &Context,
    ) -> (PutBackOutcome, Vec<u32>) {
        let entries = self.entries(item.index).unwrap_or_default();
        let paths: Vec<&Path> =
            item.path.iter().map(PathBuf::as_path).chain(item.more.iter().map(|part| part.path.as_path())).collect();
        let fits = went.is_none_or(|went| usize::try_from(went).is_ok_and(|went| went <= paths.len()))
            && entries.iter().all(|(entry, _)| paths.contains(&entry.original.as_path()));
        if item.path.is_none() || !fits {
            return (PutBackOutcome::Refused { reason: SkipCode::CheckFailed }, Vec::new());
        }
        let entry = |path: &Path| entries.iter().find(|(entry, _)| entry.original == path);
        let try_one = |found: Option<&(TrashedItem, TrashState)>| match found {
            Some((trashed, TrashState::InTrash)) => ports.cleaner.put_back(trashed, &cx.borrow()),
            Some((_, TrashState::Replaced)) => PutBackOutcome::Replaced,
            Some((_, TrashState::Gone)) | None => PutBackOutcome::Gone,
        };
        let earlier: Vec<u32> = recorded
            .put_backs
            .iter()
            .filter(|record| record.index == item.index)
            .flat_map(|record| record.back.iter().copied())
            .collect();
        // The cleaner counts the parts that went, the subject first, whether or not the Trash told where each landed;
        // a part with an entry left its place all the same, so every part up to the last one with an entry went too.
        let counted = match went {
            Some(went) => usize::try_from(went).unwrap_or(usize::MAX).saturating_sub(1),
            None if recorded.outcome(item.index) == RecordedOutcome::Moved => item.more.len(),
            None => 0,
        };
        let with_entry = item.more.iter().rposition(|part| entry(&part.path).is_some()).map_or(0, |last| last + 1);
        let went = counted.max(with_entry);
        let others: Vec<(u32, &Path)> =
            (1_u32..).zip(&item.more).take(went).map(|(place, part)| (place, part.path.as_path())).collect();
        let (mut back, mut refusal) = (Vec::new(), None);
        for &(place, path) in others.iter().rev().filter(|(place, _)| !earlier.contains(place)) {
            match try_one(entry(path)) {
                PutBackOutcome::PutBack => back.push(place),
                answer => {
                    refusal.get_or_insert(answer);
                }
            }
        }
        if let Some(refusal) = refusal {
            return (refusal, back);
        }
        let outcome = try_one(item.path.as_deref().and_then(entry));
        if outcome == PutBackOutcome::PutBack {
            back.push(0);
        }
        (outcome, back)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::{CleanupId, DangerLevel, Event};
    use fresher_testing::{FakeCleaner, MemoryCleanupJournal};

    use crate::Engine;
    use crate::cleanup::testing::{echo, fixture, scanned, select, wait_for_cleanup};
    use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, category, together};

    /// An engine after a scan of `fixture` and a cleanup of findings `findings` by `method`; the cleanup's id.
    fn cleaned(fixture: Fixture, findings: &[u32], method: &str) -> (Engine, CleanupId) {
        let engine = scanned(fixture);
        let id = clean(&engine, findings, method);
        (engine, id)
    }

    fn clean(engine: &Engine, findings: &[u32], method: &str) -> CleanupId {
        let events = engine.subscribe();
        let view = engine.prepare_cleanup(&select(findings, &[])).unwrap();
        let id = engine.confirm_cleanup(&echo(&view, method)).unwrap();
        let seen = wait_for_cleanup(&events);
        assert!(matches!(seen.last(), Some(Event::CleanupFinished { cleanup, .. }) if *cleanup == id));
        id
    }

    /// A fixture whose cleaner is `cleaner`, and the journal to look at.
    fn with(cleaner: FakeCleaner) -> (Fixture, Arc<MemoryCleanupJournal>) {
        let fixture = fixture().cleaner(Arc::new(cleaner));
        let journal = Arc::clone(&fixture.journal);
        (fixture, journal)
    }

    /// The finding of the model whose parts leave together: `/home/models/m` (its subject), `/home/blobs/a` and
    /// `/home/blobs/b`. Findings 1 to 3 are the parts on their own.
    const MODEL: u32 = 4;

    /// An engine after a scan of module `alpha`, which finds the model, and a cleanup of the model by `cleaner`, which
    /// lands its parts at `/fake-trash/0-<part>`; the cleanup's id, the cleaner and the journal.
    fn cleaned_model(cleaner: FakeCleaner) -> (Engine, CleanupId, Arc<FakeCleaner>, Arc<MemoryCleanupJournal>) {
        let walk = ScriptedWalk::new()
            .folder("model", "/home/models/m", bytes(1))
            .folder("layer", "/home/blobs/a", bytes(1))
            .folder("layer", "/home/blobs/b", bytes(1));
        let models = category("alpha.models", DangerLevel::Medium);
        let module = TestModule::builder("alpha")
            .category("alpha.files", DangerLevel::Safe)
            .category("alpha.models", DangerLevel::Medium)
            .folder("model")
            .folder("layer")
            .on_finish(move |_, out| {
                out.push(together(&models, &["/home/models/m", "/home/blobs/a", "/home/blobs/b"], bytes(3)));
                Ok(())
            });
        let cleaner = Arc::new(cleaner);
        let fixture = Fixture::new(walk).module(module).cleaner(Arc::clone(&cleaner));
        let journal = Arc::clone(&fixture.journal);
        let engine = scanned(fixture);
        let id = clean(&engine, &[MODEL], "trash");
        (engine, id, cleaner, journal)
    }

    /// `/fake-trash/0-<part>`, where part `part` of the model landed.
    fn landed(part: u32) -> PathBuf {
        PathBuf::from(format!("/fake-trash/0-{part}"))
    }

    mod negative {
        use std::path::PathBuf;
        use std::sync::Arc;

        use fresher_core::{
            ActionKind, BeginRecord, BlockedReason, CleanupId, CleanupJournal, CommandError, DangerLevel, EndRecord,
            Ended, EntryKind, FindingId, ItemIdentity, ItemRecord, JournalItem, Method, Operation, PutBackOutcome,
            RecordedOutcome, SkipCode, SystemUiError, Timestamp, TrashState, TrashedItem,
        };
        use fresher_testing::{FakeCleaner, FakeSystemUi, MemoryCleanupJournal};

        use super::{MODEL, cleaned, cleaned_model, landed, with};
        use crate::cleanup::testing::{echo, fixture, scanned, select};
        use crate::testing::{HeldCleaner, ScriptedWalk, engine, wait_for_finish};

        #[test]
        fn put_back_while_cleaning_is_busy() {
            let (cleaner, gate) = HeldCleaner::new(FakeCleaner::default());
            let engine = scanned(fixture().cleaner(Arc::new(cleaner)));
            let events = engine.subscribe();
            let view = engine.prepare_cleanup(&select(&[1], &[])).unwrap();
            let id = engine.confirm_cleanup(&echo(&view, "trash")).unwrap();

            assert_eq!(engine.put_back(id, None), Err(CommandError::Busy { running: Operation::Clean }));

            gate.release();
            crate::cleanup::testing::wait_for_cleanup(&events);
        }

        #[test]
        fn put_back_while_scanning_is_busy() {
            let (walk, gate) = ScriptedWalk::held();
            let engine = engine(walk);
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();

            let error = engine.put_back(CleanupId::new(1), None);

            assert_eq!(error, Err(CommandError::Busy { running: Operation::Scan }));
            gate.release();
            wait_for_finish(&events);
        }

        #[test]
        fn an_unknown_cleanup_is_refused() {
            let (engine, _) = cleaned(fixture(), &[1], "trash");
            let unknown = CleanupId::new(9);
            let refused = Err(CommandError::UnknownCleanup { id: unknown });

            assert_eq!(engine.cleanup(unknown).map(|_| ()), refused);
            assert_eq!(engine.put_back(unknown, None).map(|_| ()), refused);
            assert_eq!(engine.reveal_in_trash(unknown, 0), refused);
        }

        #[test]
        fn an_unknown_item_is_refused() {
            let (fixture, journal) = with(FakeCleaner::default());
            let (engine, id) = cleaned(fixture, &[1, 2], "trash");

            assert_eq!(engine.put_back(id, Some(&[0, 7])), Err(CommandError::UnknownCleanupItem));
            assert_eq!(engine.reveal_in_trash(id, 7), Err(CommandError::UnknownCleanupItem));
            assert_eq!(journal.recorded()[0].put_backs, []);
        }

        #[test]
        fn an_item_gone_from_the_trash_is_not_put_back() {
            let cleaner = FakeCleaner::default()
                .locating("/fake-trash/0", TrashState::Gone)
                .locating("/fake-trash/1", TrashState::Replaced);
            let (engine, id) = cleaned(with(cleaner).0, &[1, 2, 3], "trash");

            let some = engine.put_back(id, Some(&[0, 1])).unwrap();
            let all = engine.put_back(id, None).unwrap();

            let outcomes: Vec<_> = some.items.iter().map(|item| item.outcome).collect();
            assert_eq!(outcomes, [PutBackOutcome::Gone, PutBackOutcome::Replaced]);
            assert_eq!(all.items.iter().map(|item| item.index).collect::<Vec<_>>(), [2]);
            assert_eq!(some.items[0].text, PutBackOutcome::Gone.text());
        }

        #[test]
        fn an_occupied_place_is_reported_and_the_item_stays() {
            let (fixture, journal) =
                with(FakeCleaner::default().putting_back("/fake-trash/1", PutBackOutcome::Occupied));
            let (engine, id) = cleaned(fixture, &[1, 2, 3], "trash");

            let report = engine.put_back(id, Some(&[1, 1])).unwrap();

            assert_eq!(report.items.len(), 1);
            assert_eq!(report.items[0].outcome, PutBackOutcome::Occupied);
            let detail = engine.cleanup(id).unwrap();
            assert_eq!(detail.items[1].trash, Some(TrashState::InTrash));
            assert_eq!(detail.items[1].put_back, Some(PutBackOutcome::Occupied));
            assert_eq!(detail.summary.in_trash, 3);
            assert_eq!(journal.recorded()[0].put_backs[0].outcome, PutBackOutcome::Occupied);
        }

        #[test]
        fn an_item_put_back_already_is_not_put_back_again_nor_journalled() {
            let (fixture, journal) = with(FakeCleaner::default());
            let (engine, id) = cleaned(fixture, &[1], "trash");
            engine.put_back(id, Some(&[0])).unwrap();

            let again = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(again.items[0].outcome, PutBackOutcome::PutBack);
            assert_eq!(journal.recorded()[0].put_backs.len(), 1);
            assert_eq!(engine.reveal_in_trash(id, 0), Err(CommandError::NotInTrash));
            let detail = engine.cleanup(id).unwrap();
            assert_eq!((detail.items[0].trash, detail.items[0].put_back), (None, Some(PutBackOutcome::PutBack)));
        }

        #[test]
        fn reveal_of_an_item_no_longer_in_the_trash_is_refused() {
            let fixture = with(FakeCleaner::default().locating("/fake-trash/0", TrashState::Gone)).0;
            let system_ui = Arc::clone(&fixture.system_ui);
            let (engine, id) = cleaned(fixture, &[1], "trash");

            assert_eq!(engine.reveal_in_trash(id, 0), Err(CommandError::NotInTrash));
            assert_eq!(system_ui.calls(), []);
        }

        #[test]
        fn a_system_that_cannot_reveal_is_an_internal_error() {
            let fixture = fixture().system_ui(FakeSystemUi::failing(SystemUiError::Unsupported));
            let (engine, id) = cleaned(fixture, &[1], "trash");

            assert_eq!(engine.reveal_in_trash(id, 0), Err(CommandError::Internal));
        }

        #[test]
        fn a_permanent_cleanups_items_cannot_be_put_back() {
            let (engine, id) = cleaned(fixture(), &[1, 2], "permanent");

            let all = engine.put_back(id, None).unwrap();
            let one = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(all.items, []);
            assert_eq!(one.items[0].outcome, PutBackOutcome::NotTrashed);
            assert_eq!(engine.cleanup(id).unwrap().items[0].trash, None);
            assert_eq!(engine.reveal_in_trash(id, 0), Err(CommandError::NotInTrash));
        }

        #[test]
        fn a_together_item_whose_layer_cannot_come_back_keeps_its_subject_in_the_trash() {
            let (engine, id, cleaner, _) =
                cleaned_model(FakeCleaner::default().putting_back(landed(1), PutBackOutcome::Occupied));

            let report = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(report.items[0].outcome, PutBackOutcome::Occupied);
            assert_eq!(cleaner.put_backs_tried(), [landed(2), landed(1)], "never the subject");
            let detail = engine.cleanup(id).unwrap();
            assert_eq!(
                (detail.items[0].trash, detail.items[0].put_back),
                (Some(TrashState::InTrash), Some(PutBackOutcome::Occupied))
            );
        }

        #[test]
        fn a_together_item_whose_subject_is_gone_puts_back_the_rest_and_says_gone() {
            let (engine, id, cleaner, _) = cleaned_model(FakeCleaner::default().locating(landed(0), TrashState::Gone));

            let report = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(report.items[0].outcome, PutBackOutcome::Gone);
            assert_eq!(cleaner.put_backs_tried(), [landed(2), landed(1)]);
        }

        #[test]
        fn a_together_part_whose_landing_is_unknown_keeps_its_subject_in_the_trash() {
            let (engine, id, cleaner, _) =
                cleaned_model(FakeCleaner::default().landing_unknown(FindingId::new(MODEL), 1));

            let report = engine.put_back(id, None).unwrap();

            assert_eq!(report.items[0].outcome, PutBackOutcome::Gone);
            assert_eq!(cleaner.put_backs_tried(), [landed(2)], "the layer with no entry cannot come back");
        }

        #[test]
        fn a_together_part_gone_from_the_trash_keeps_its_subject_there() {
            let (engine, id, cleaner, _) = cleaned_model(FakeCleaner::default().locating(landed(2), TrashState::Gone));

            let report = engine.put_back(id, None).unwrap();

            assert_eq!(report.items[0].outcome, PutBackOutcome::Gone);
            assert_eq!(cleaner.put_backs_tried(), [landed(1)]);
            assert_eq!(engine.cleanup(id).unwrap().items[0].trash, Some(TrashState::InTrash));
        }

        #[test]
        fn a_partly_item_whose_last_moved_part_landed_unknown_never_brings_the_subject_back() {
            let model = FindingId::new(MODEL);
            let (engine, id, cleaner, _) =
                cleaned_model(FakeCleaner::default().partly(model, 2).landing_unknown(model, 1));

            let report = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(report.items[0].outcome, PutBackOutcome::Gone);
            assert_eq!(cleaner.put_backs_tried(), Vec::<PathBuf>::new(), "the subject stays in the Trash");
            assert_eq!(engine.cleanup(id).unwrap().items[0].trash, Some(TrashState::InTrash));
        }

        #[test]
        fn a_retry_where_a_part_vanished_from_both_places_keeps_the_subject() {
            let (engine, id, cleaner, _) =
                cleaned_model(FakeCleaner::default().putting_back(landed(1), PutBackOutcome::Occupied));
            engine.put_back(id, Some(&[0])).unwrap();
            cleaner.now_locating(landed(2), TrashState::Gone);
            cleaner.now_locating(landed(1), TrashState::Gone);

            let retry = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(retry.items[0].outcome, PutBackOutcome::Gone);
            assert_eq!(cleaner.put_backs_tried(), [landed(2), landed(1)], "nothing more is tried, never the subject");
            assert_eq!(engine.cleanup(id).unwrap().items[0].trash, Some(TrashState::InTrash));
        }

        #[test]
        fn a_partly_item_whose_part_landed_elsewhere_keeps_its_subject_until_that_part_is_back() {
            let model = FindingId::new(MODEL);
            let cleaner = FakeCleaner::default()
                .partly(model, 1)
                .landing_elsewhere(model, 1)
                .putting_back(landed(1), PutBackOutcome::Occupied);
            let (engine, id, cleaner, _) = cleaned_model(cleaner);

            let first = engine.put_back(id, Some(&[0])).unwrap();
            cleaner.now_putting_back(landed(1), PutBackOutcome::PutBack);
            let retry = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(first.items[0].outcome, PutBackOutcome::Occupied);
            assert_eq!(retry.items[0].outcome, PutBackOutcome::PutBack);
            assert_eq!(
                cleaner.put_backs_tried(),
                [landed(1), landed(1), landed(0)],
                "the subject only once it is back"
            );
        }

        #[test]
        fn a_together_record_that_does_not_hold_together_puts_nothing_back() {
            let cleaner = Arc::new(FakeCleaner::default());
            let fixture = fixture().cleaner(Arc::clone(&cleaner));
            let journal = Arc::clone(&fixture.journal);
            let engine = scanned(fixture);
            let subject = PathBuf::from("/home/models/m");
            let item = JournalItem {
                index: 0,
                path: Some(subject.clone()),
                identity: Some(ItemIdentity::new(1, 1, EntryKind::File)),
                more: Vec::new(),
                action: ActionKind::Remove,
                category: "alpha.caches".into(),
                key: "m".into(),
                reason: crate::testing::text("test.finding.reason"),
                danger: DangerLevel::Safe,
                sizes: crate::testing::bytes(2),
            };
            let begin = BeginRecord {
                at: Timestamp::from_secs(1),
                method: Method::Trash,
                roots: vec![PathBuf::from("/home")],
                space: vec![],
                items: vec![item],
            };
            let id = journal.begin(&begin).unwrap();
            let trashed = |original: &str, part: u32| TrashedItem {
                original: PathBuf::from(original),
                landed: landed(part),
                identity: ItemIdentity::new(1, u64::from(part) + 1, EntryKind::File),
            };
            let trashed = vec![trashed("/home/models/m", 0), trashed("/home/blobs/a", 1)];
            let record = ItemRecord { cleanup: id, index: 0, outcome: RecordedOutcome::Moved, trashed, went: Some(2) };
            journal.item(&record).unwrap();
            journal
                .end(&EndRecord { cleanup: id, at: Timestamp::from_secs(2), ended: Ended::Completed, space: vec![] })
                .unwrap();

            let report = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(report.items[0].outcome, PutBackOutcome::Refused { reason: SkipCode::CheckFailed });
            assert_eq!(cleaner.put_backs_tried(), Vec::<PathBuf>::new());
        }

        #[test]
        fn a_read_only_journal_shows_its_history_and_blocks() {
            let engine = scanned(fixture().journal(MemoryCleanupJournal::read_only()));

            let page = engine.cleanups(0, 10);

            assert_eq!(page.blocked, Some(BlockedReason::Journal));
            assert_eq!((page.cleanups.len(), page.total), (0, 0));
        }
    }

    mod positive {
        use std::path::PathBuf;
        use std::sync::Arc;

        use fresher_core::{
            ByteSize, CleanupId, Ended, FindingId, Method, PutBackOutcome, RecordedOutcome, TrashState,
        };
        use fresher_testing::{FakeCleaner, SystemUiCall};

        use super::{MODEL, clean, cleaned, cleaned_model, landed, with};
        use crate::cleanup::testing::fixture;

        #[test]
        fn a_together_item_comes_back_parts_first_and_the_subject_last() {
            let (engine, id, cleaner, _) = cleaned_model(FakeCleaner::default());
            let detail = engine.cleanup(id).unwrap();
            assert_eq!(detail.items.len(), 1);
            assert_eq!(detail.items[0].path, Some(PathBuf::from("/home/models/m")));

            let report = engine.put_back(id, None).unwrap();

            assert_eq!(
                report.items.iter().map(|item| (item.index, item.outcome)).collect::<Vec<_>>(),
                [(0, PutBackOutcome::PutBack)]
            );
            assert_eq!(cleaner.put_backs_tried(), [landed(2), landed(1), landed(0)]);
            assert_eq!(engine.cleanup(id).unwrap().summary.in_trash, 0);
        }

        #[test]
        fn a_retry_after_one_refused_part_brings_the_subject_back_once_that_part_comes_back() {
            let (engine, id, cleaner, journal) =
                cleaned_model(FakeCleaner::default().putting_back(landed(1), PutBackOutcome::Occupied));
            let first = engine.put_back(id, Some(&[0])).unwrap();
            cleaner.now_locating(landed(2), TrashState::Gone);
            cleaner.now_putting_back(landed(1), PutBackOutcome::PutBack);

            let retry = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(
                (first.items[0].outcome, retry.items[0].outcome),
                (PutBackOutcome::Occupied, PutBackOutcome::PutBack)
            );
            assert_eq!(cleaner.put_backs_tried(), [landed(2), landed(1), landed(1), landed(0)]);
            let backs: Vec<_> = journal.recorded()[0].put_backs.iter().map(|record| record.back.clone()).collect();
            assert_eq!(backs, [vec![2], vec![1, 0]]);
            assert_eq!(engine.cleanup(id).unwrap().summary.in_trash, 0);
        }

        #[test]
        fn a_partly_moved_item_puts_back_what_went() {
            let (engine, id, cleaner, _) = cleaned_model(FakeCleaner::default().partly(FindingId::new(MODEL), 2));
            assert!(matches!(engine.cleanup(id).unwrap().items[0].outcome, RecordedOutcome::Failed { .. }));

            let report = engine.put_back(id, Some(&[0])).unwrap();

            assert_eq!(report.items[0].outcome, PutBackOutcome::PutBack);
            assert_eq!(cleaner.put_backs_tried(), [landed(1), landed(0)], "the layer that never moved is not tried");
        }

        #[test]
        fn history_is_newest_first_with_trash_counts() {
            let (engine, first) = cleaned(
                with(FakeCleaner::default().locating("/fake-trash/1", TrashState::Gone)).0,
                &[1, 2, 4],
                "trash",
            );
            let second = clean(&engine, &[3], "trash");

            let page = engine.cleanups(0, 500);

            let summaries: Vec<_> = page.cleanups.iter().map(|summary| (summary.cleanup, summary.in_trash)).collect();
            assert_eq!(summaries, [(second, 1), (first, 2)]);
            assert_eq!((page.total, page.offset, page.blocked), (2, 0, None));
            assert_eq!(page.cleanups[1].method, Method::Trash);
            assert_eq!(page.cleanups[1].ended, Ended::Completed);
            assert_eq!(
                engine.cleanups(1, 1).cleanups.iter().map(|summary| summary.cleanup).collect::<Vec<_>>(),
                [first]
            );
        }

        #[test]
        fn waiting_in_the_trash_sums_only_items_still_there() {
            let (engine, _) = cleaned(
                with(FakeCleaner::default().locating("/fake-trash/1", TrashState::Gone)).0,
                &[1, 2, 4],
                "trash",
            );
            clean(&engine, &[3], "trash");

            // Cleanup 1 acted on 1 (10 bytes), 4 (5, gone from the Trash) and 2 (20); cleanup 2 on 3 (30).
            assert_eq!(engine.cleanups(0, 10).waiting, ByteSize::new(10 + 20 + 30));
        }

        #[test]
        fn the_detail_names_every_item_with_its_outcome_and_place() {
            let (engine, id) = cleaned(fixture(), &[1, 2, 3], "trash");

            let detail = engine.cleanup(id).unwrap();

            assert_eq!(detail.summary.cleanup, id);
            assert_eq!(detail.summary.in_trash, 3);
            let items: Vec<_> =
                detail.items.iter().map(|item| (item.index, item.path.clone(), item.outcome, item.trash)).collect();
            assert_eq!(
                items,
                [
                    (0, Some(PathBuf::from("/home/b/Caches")), RecordedOutcome::Moved, Some(TrashState::InTrash)),
                    (1, Some(PathBuf::from("/home/a/Caches")), RecordedOutcome::Moved, Some(TrashState::InTrash)),
                    (2, Some(PathBuf::from("/home/Logs")), RecordedOutcome::Moved, Some(TrashState::InTrash)),
                ]
            );
            assert_eq!(detail.items[2].category, "beta.logs");
            assert_eq!(detail.items[0].put_back, None);
        }

        #[test]
        fn put_back_all_restores_every_item_still_in_the_trash_and_journals_each() {
            let (fixture, journal) = with(FakeCleaner::default());
            let (engine, id) = cleaned(fixture, &[1, 2, 3], "trash");

            let report = engine.put_back(id, None).unwrap();

            let items: Vec<_> = report.items.iter().map(|item| (item.index, item.outcome)).collect();
            assert_eq!(
                items,
                [(0, PutBackOutcome::PutBack), (1, PutBackOutcome::PutBack), (2, PutBackOutcome::PutBack)]
            );
            let records: Vec<_> =
                journal.recorded()[0].put_backs.iter().map(|record| (record.cleanup, record.index)).collect();
            assert_eq!(records, [(id, 0), (id, 1), (id, 2)]);
            assert_eq!(engine.cleanup(id).unwrap().summary.in_trash, 0);
            assert_eq!(engine.cleanups(0, 10).waiting, ByteSize::ZERO);
            assert_eq!(engine.put_back(id, None).unwrap().items, []);
            assert_eq!(id, CleanupId::new(1));
        }

        #[test]
        fn reveal_asks_the_system_ui_for_the_landed_path() {
            let fixture = fixture();
            let system_ui = Arc::clone(&fixture.system_ui);
            let (engine, id) = cleaned(fixture, &[1, 2, 3], "trash");

            engine.reveal_in_trash(id, 1).unwrap();

            assert_eq!(system_ui.calls(), [SystemUiCall::Reveal(PathBuf::from("/fake-trash/1"))]);
        }

        #[test]
        fn an_engine_without_a_journal_has_no_history() {
            let fixture = fixture();
            let ports = crate::Ports {
                walk: Arc::new(crate::testing::ScriptedWalk::new()),
                settings: fixture.settings,
                clock: fixture.clock,
                commands: fixture.commands,
                access: fixture.access,
                system_ui: fixture.system_ui,
                system: fixture.system,
            };
            let engine = crate::Engine::new(crate::Registry::new().unwrap(), ports).unwrap();

            let page = engine.cleanups(0, 10);

            assert_eq!((page.cleanups.len(), page.blocked), (0, Some(fresher_core::BlockedReason::Journal)));
            let unknown = CleanupId::new(1);
            assert_eq!(
                engine.cleanup(unknown).map(|_| ()),
                Err(fresher_core::CommandError::UnknownCleanup { id: unknown })
            );
        }
    }
}
