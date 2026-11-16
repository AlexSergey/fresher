//! The history of cleanups, from the journal, with where each moved item is now; putting items back; showing one in
//! the Trash.

use fresher_core::{
    BlockedReason, ByteSize, CleanupDetail, CleanupId, CleanupItemView, CleanupJournal, CleanupSummary, CleanupsPage,
    CleanupsQuery, CommandError, JournalState, PutBackItem, PutBackOutcome, PutBackRecord, PutBackReport,
    RecordedCleanup, Timestamp, TrashState, TrashedItem,
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
        let outcome = located.put_back(ports, index, &cx);
        let at = Timestamp::from_system_time(shared.ports.clock.now());
        if let Err(error) = ports.journal.put_back(&PutBackRecord { cleanup: id, index, at, outcome }) {
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
struct Located {
    /// Per item index with entries in the Trash: each entry and its state.
    items: Vec<(u32, Vec<(TrashedItem, TrashState)>)>,
    /// The items put back.
    returned: Vec<u32>,
    /// The allocated size of each item, by index.
    sizes: Vec<(u32, u64)>,
}

impl Located {
    fn of(ports: &CleanupPorts, recorded: &RecordedCleanup) -> Self {
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
        let sizes = recorded.begin.items.iter().map(|item| (item.index, item.sizes.allocated.bytes())).collect();
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
        let size = |index: u32| self.sizes.iter().find(|(item, _)| *item == index).map_or(0, |&(_, size)| size);
        self.still_in_trash().map(size).fold(0, u64::saturating_add)
    }

    /// Puts item `index` back, entry by entry: only the entries still in the Trash are tried. Answers `NotTrashed` for
    /// an item that never went to the Trash, the state of an item no entry of which is there, else the first entry's
    /// refusal, else `PutBack`. The caller answers for an item put back already.
    fn put_back(&self, ports: &CleanupPorts, index: u32, cx: &Context) -> PutBackOutcome {
        let Some(state) = self.state(index) else { return PutBackOutcome::NotTrashed };
        match state {
            TrashState::Gone => PutBackOutcome::Gone,
            TrashState::Replaced => PutBackOutcome::Replaced,
            TrashState::InTrash => {
                let outcomes = self.in_trash_entries(index).map(|entry| ports.cleaner.put_back(entry, &cx.borrow()));
                outcomes.fold(
                    PutBackOutcome::PutBack,
                    |first, outcome| {
                        if first == PutBackOutcome::PutBack { outcome } else { first }
                    },
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fresher_core::{CleanupId, Event};
    use fresher_testing::{FakeCleaner, MemoryCleanupJournal};

    use crate::Engine;
    use crate::cleanup::testing::{echo, fixture, scanned, select, wait_for_cleanup};
    use crate::testing::Fixture;

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

    mod negative {
        use std::sync::Arc;

        use fresher_core::{
            BlockedReason, CleanupId, CommandError, Operation, PutBackOutcome, SystemUiError, TrashState,
        };
        use fresher_testing::{FakeCleaner, FakeSystemUi, MemoryCleanupJournal};

        use super::{cleaned, with};
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

        use fresher_core::{ByteSize, CleanupId, Ended, Method, PutBackOutcome, RecordedOutcome, TrashState};
        use fresher_testing::{FakeCleaner, SystemUiCall};

        use super::{clean, cleaned, with};
        use crate::cleanup::testing::fixture;

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
