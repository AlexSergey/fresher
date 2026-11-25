//! The Trash view: what waits in each Trash, the share of the app's cleanups, Finder's date, and Open Trash.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    ActionKind, BeginRecord, ByteSize, CleanupId, CleanupJournal, DangerLevel, DiskSpace, EndRecord, Ended, EntryKind,
    ItemIdentity, ItemRecord, JournalItem, Method, NodeKind, PutBackOutcome, PutBackRecord, RecordedOutcome, StateId,
    Timestamp, TrashLine, TrashedItem,
};
use fresher_testing::{FakeVolumes, FixedDiskState, MemoryCleanupJournal};

use crate::Engine;
use crate::testing::{Fixture, START, ScriptedWalk, bytes, scan, text};

/// When the scan's state was taken: an hour before the fixture clock's now.
const TAKEN: u64 = START - 3_600;
/// The home Trash, as the fixture's known folders locate it.
const HOME_TRASH: &str = "/home/.Trash";
/// A second volume's Trash, which no scan root holds.
const BACKUP_TRASH: &str = "/Volumes/Backup/.Trashes/501";
const DAY: u64 = 24 * 60 * 60;

/// The volumes: the home on `Macintosh HD`, mounted at `/`; the second volume `Backup`, at `/Volumes/Backup`.
fn volumes() -> FakeVolumes {
    let disk = |name: &str| FakeVolumes::disk(name, ByteSize::new(1_000), ByteSize::new(400), ByteSize::new(600));
    let backup = DiskSpace { mount_point: PathBuf::from("/Volumes/Backup"), ..disk("Backup") };
    FakeVolumes::default().with_space("/", disk("Macintosh HD")).with_space("/Volumes/Backup", backup)
}

/// A state taken at [`TAKEN`] that measured the home Trash at `size` bytes.
fn measured(size: u64) -> FixedDiskState {
    let nodes = [
        (PathBuf::from("/home"), NodeKind::Folder, bytes(size)),
        (PathBuf::from(HOME_TRASH), NodeKind::Folder, bytes(size)),
    ];
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(TAKEN), nodes)
}

/// A fixture whose scan commits `state`, over [`volumes`].
fn fixture(state: Option<FixedDiskState>) -> Fixture {
    let walk = match state {
        Some(state) => ScriptedWalk::new().committing(vec![Arc::new(state)]),
        None => ScriptedWalk::new(),
    };
    Fixture::new(walk).volumes(volumes())
}

/// The engine of `fixture`, after a scan when `scanned`.
fn engine(fixture: Fixture, scanned: bool) -> Engine {
    let engine = fixture.engine();
    if scanned {
        scan(&engine);
    }
    engine
}

/// An item of a cleanup: its allocated size, its outcome and where its entries landed.
struct Item {
    size: u64,
    outcome: RecordedOutcome,
    landed: Vec<PathBuf>,
}

/// An item moved to the Trash whole, landed as `name` in `folder`.
fn moved(size: u64, folder: &str, name: &str) -> Item {
    Item { size, outcome: RecordedOutcome::Moved, landed: vec![Path::new(folder).join(name)] }
}

/// Records a cleanup by `method` that began at `began` and ended at `ended` (`None`: never ended) with `items`.
fn record(journal: &MemoryCleanupJournal, began: u64, ended: Option<u64>, method: Method, items: &[Item]) -> CleanupId {
    let original = |index: u32| PathBuf::from(format!("/home/item-{index}"));
    let tool = |item: &Item| item.outcome == RecordedOutcome::ToolDone;
    let emptied = |item: &Item| matches!(item.outcome, RecordedOutcome::Emptied { .. });
    let journal_items = (0_u32..)
        .zip(items)
        .map(|(index, item)| JournalItem {
            index,
            path: (!tool(item)).then(|| original(index)),
            identity: Some(ItemIdentity::new(1, u64::from(index) + 1, EntryKind::Folder)),
            more: Vec::new(),
            action: if tool(item) {
                ActionKind::Tool
            } else if emptied(item) {
                ActionKind::Empty
            } else {
                ActionKind::Remove
            },
            category: "alpha.caches".into(),
            key: format!("item-{index}"),
            reason: text("test.finding.reason"),
            danger: DangerLevel::Safe,
            sizes: bytes(item.size),
        })
        .collect();
    let at = Timestamp::from_secs(began);
    let roots = vec![PathBuf::from("/home")];
    let id = journal.begin(&BeginRecord { at, method, roots, space: vec![], items: journal_items }).unwrap();
    for (index, item) in (0_u32..).zip(items) {
        let trashed = (0_u64..)
            .zip(&item.landed)
            .map(|(entry, landed)| TrashedItem {
                original: if emptied(item) { original(index).join(entry.to_string()) } else { original(index) },
                landed: landed.clone(),
                identity: ItemIdentity::new(1, 100 + entry, EntryKind::File),
            })
            .collect();
        journal.item(&ItemRecord { cleanup: id, index, outcome: item.outcome, trashed, went: None }).unwrap();
    }
    if let Some(ended) = ended {
        let end = EndRecord { cleanup: id, at: Timestamp::from_secs(ended), ended: Ended::Completed, space: vec![] };
        journal.end(&end).unwrap();
    }
    id
}

/// Records an attempt to put item `index` of cleanup `id` back that ended in `outcome`.
fn put_back(journal: &MemoryCleanupJournal, id: CleanupId, index: u32, outcome: PutBackOutcome) {
    let record = PutBackRecord { cleanup: id, index, at: Timestamp::from_secs(START), outcome, back: vec![] };
    journal.put_back(&record).unwrap();
}

/// The line of the home Trash, on the home's volume, with no date.
fn home_line(waiting: Option<u64>, from_cleanups: u64) -> TrashLine {
    TrashLine {
        index: 0,
        volume: Some("Macintosh HD".into()),
        waiting: waiting.map(bytes),
        from_cleanups: bytes(from_cleanups),
        gone_by: None,
    }
}

mod negative {
    use std::path::Path;
    use std::sync::Arc;

    use fresher_core::{
        AutoEmpty, CommandError, Method, NotScanned, PutBackOutcome, RecordedOutcome, SkipCode, SystemUiError,
        TrashState, Unreadable,
    };
    use fresher_testing::{FakeCleaner, FakeFinderSettings, FakeSystemUi, MemoryCleanupJournal};

    use super::{
        BACKUP_TRASH, DAY, HOME_TRASH, Item, START, TAKEN, bytes, engine, fixture, home_line, measured, moved,
        put_back, record,
    };

    #[test]
    fn no_scan_and_an_empty_journal_give_no_line() {
        let engine = engine(fixture(None), false);

        assert_eq!(engine.trash().trashes, []);
    }

    #[test]
    fn an_empty_trash_with_nothing_from_cleanups_gives_no_line() {
        let engine = engine(fixture(Some(measured(0))), true);

        assert_eq!(engine.trash().trashes, []);
    }

    #[test]
    fn an_item_gone_from_the_trash_counts_nothing() {
        let cleaner = FakeCleaner::default().locating("/home/.Trash/a", TrashState::Gone);
        let fixture = fixture(None).cleaner(Arc::new(cleaner));
        record(&fixture.journal, TAKEN, Some(TAKEN), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, []);
    }

    #[test]
    fn a_replaced_item_counts_nothing() {
        let cleaner = FakeCleaner::default().locating("/home/.Trash/a", TrashState::Replaced);
        let fixture = fixture(None).cleaner(Arc::new(cleaner));
        record(&fixture.journal, TAKEN, Some(TAKEN), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, []);
    }

    #[test]
    fn an_item_put_back_counts_nothing() {
        let fixture = fixture(None);
        let id = record(&fixture.journal, TAKEN, Some(TAKEN), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
        put_back(&fixture.journal, id, 0, PutBackOutcome::PutBack);
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, []);
    }

    #[test]
    fn an_item_that_landed_outside_any_trash_counts_nothing() {
        let fixture = fixture(None);
        let elsewhere = |folder: &str| Item {
            size: 300,
            // A failed move whose reported landing the journal keeps for put back (`LandedElsewhere`).
            outcome: RecordedOutcome::Failed { status: None },
            landed: vec![Path::new(folder).join("a")],
        };
        // A folder the system reported, a `.Trash` that is not the home's, a volume's `.Trashes` not at its root.
        let items =
            [elsewhere("/home/Documents"), elsewhere("/srv/.Trash"), elsewhere("/Volumes/Backup/x/.Trashes/501")];
        record(&fixture.journal, TAKEN, Some(TAKEN), Method::Trash, &items);
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, []);
    }

    #[test]
    fn a_permanent_cleanup_a_tools_item_and_a_skipped_item_count_nothing() {
        let fixture = fixture(None);
        let deleted = Item { size: 300, outcome: RecordedOutcome::Deleted, landed: vec![] };
        record(&fixture.journal, TAKEN, Some(TAKEN), Method::Permanent, &[deleted]);
        let tool = Item { size: 500, outcome: RecordedOutcome::ToolDone, landed: vec![] };
        let skipped =
            Item { size: 700, outcome: RecordedOutcome::Skipped { reason: SkipCode::CheckFailed }, landed: vec![] };
        record(&fixture.journal, TAKEN, Some(TAKEN), Method::Trash, &[tool, skipped]);
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, []);
    }

    #[test]
    fn a_trash_the_scan_did_not_measure_has_no_waiting() {
        let guarded = NotScanned::Unreadable(Unreadable::Guarded);
        let fixture = fixture(Some(measured(900).with_not_scanned(Path::new(HOME_TRASH), guarded)));
        let items = [moved(300, HOME_TRASH, "a"), moved(200, BACKUP_TRASH, "b")];
        record(&fixture.journal, TAKEN - 10, Some(TAKEN - 5), Method::Trash, &items);
        let engine = engine(fixture, true);

        let trashes = engine.trash().trashes;

        assert_eq!(trashes.len(), 2);
        assert_eq!(trashes[0], home_line(None, 300));
        assert_eq!((trashes[1].waiting, trashes[1].from_cleanups), (None, bytes(200)));
    }

    #[test]
    fn a_journal_that_cannot_be_read_counts_nothing_from_cleanups() {
        let engine = engine(fixture(Some(measured(500))).journal(MemoryCleanupJournal::unreadable()), true);

        assert_eq!(engine.trash().trashes, [home_line(Some(500), 0)]);
    }

    #[test]
    fn open_trash_of_an_index_never_answered_is_refused_and_opens_nothing() {
        let fixture = fixture(Some(measured(500)));
        let system_ui = Arc::clone(&fixture.system_ui);
        let engine = engine(fixture, true);

        let before = engine.open_trash(0);
        engine.trash();
        let past = engine.open_trash(1);

        assert_eq!(before, Err(CommandError::UnknownTrash { index: 0 }));
        assert_eq!(past, Err(CommandError::UnknownTrash { index: 1 }));
        assert_eq!(system_ui.calls(), []);
    }

    #[test]
    fn a_failed_open_is_an_internal_error() {
        let failure = SystemUiError::Failed { reason: "open exited with 1".into() };
        let engine = engine(fixture(Some(measured(500))).system_ui(FakeSystemUi::failing(failure)), true);
        engine.trash();

        assert_eq!(engine.open_trash(0), Err(CommandError::Internal));
    }

    #[test]
    fn finders_setting_off_or_unknown_gives_no_date() {
        for answer in [AutoEmpty::Never, AutoEmpty::Unknown] {
            let fixture = fixture(None).finder(FakeFinderSettings::new(answer));
            record(&fixture.journal, START - DAY, Some(START - DAY), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
            let engine = engine(fixture, false);

            assert_eq!(engine.trash().trashes, [home_line(None, 300)], "{answer:?}");
        }
    }

    #[test]
    fn a_date_already_past_is_not_given() {
        let fixture = fixture(None).finder(FakeFinderSettings::new(AutoEmpty::After30Days));
        let ended = START - 30 * DAY;
        record(&fixture.journal, ended - 60, Some(ended), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, [home_line(None, 300)]);
    }
}

mod positive {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::{AutoEmpty, Method, RecordedOutcome, Timestamp};
    use fresher_testing::{FakeFinderSettings, SystemUiCall};

    use super::{
        BACKUP_TRASH, DAY, HOME_TRASH, Item, START, TAKEN, bytes, engine, fixture, home_line, measured, moved, record,
    };

    #[test]
    fn the_share_counts_an_emptied_folder_once() {
        let fixture = fixture(None);
        let landed = ["x", "y", "z"].iter().map(|name| PathBuf::from(HOME_TRASH).join(name)).collect();
        let emptied = Item { size: 4_096, outcome: RecordedOutcome::Emptied { moved: 3, skipped: 0 }, landed };
        record(&fixture.journal, TAKEN, Some(TAKEN), Method::Trash, &[emptied]);
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, [home_line(None, 4_096)]);
    }

    #[test]
    fn items_moved_after_the_scan_are_added_to_its_measure() {
        let fixture = fixture(Some(measured(1_000)));
        // One cleanup began the moment the state was taken, one after: the scan saw neither.
        record(&fixture.journal, TAKEN, Some(TAKEN + 5), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
        record(&fixture.journal, TAKEN + 10, Some(TAKEN + 20), Method::Trash, &[moved(200, HOME_TRASH, "b")]);
        let engine = engine(fixture, true);

        assert_eq!(engine.trash().trashes, [home_line(Some(1_500), 500)]);
    }

    #[test]
    fn items_moved_before_the_scan_are_not_counted_twice() {
        let fixture = fixture(Some(measured(1_300)));
        record(&fixture.journal, TAKEN - 10, Some(TAKEN - 5), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
        let engine = engine(fixture, true);

        assert_eq!(engine.trash().trashes, [home_line(Some(1_300), 300)]);
    }

    #[test]
    fn waiting_is_never_below_the_share() {
        let fixture = fixture(Some(measured(100)));
        record(&fixture.journal, TAKEN - 10, Some(TAKEN - 5), Method::Trash, &[moved(300, HOME_TRASH, "a")]);
        let engine = engine(fixture, true);

        assert_eq!(engine.trash().trashes, [home_line(Some(300), 300)]);
    }

    #[test]
    fn a_second_volumes_trash_has_its_own_line_named_after_its_volume() {
        let fixture = fixture(Some(measured(1_000)));
        record(&fixture.journal, TAKEN + 10, Some(TAKEN + 20), Method::Trash, &[moved(200, BACKUP_TRASH, "x")]);
        let engine = engine(fixture, true);

        let trashes = engine.trash().trashes;

        assert_eq!(trashes.len(), 2);
        assert_eq!(trashes[0], home_line(Some(1_000), 0));
        assert_eq!((trashes[1].index, trashes[1].volume.as_deref()), (1, Some("Backup")));
        assert_eq!((trashes[1].waiting, trashes[1].from_cleanups), (None, bytes(200)));
    }

    #[test]
    fn the_journal_is_read_past_one_page() {
        let fixture = fixture(None);
        for cleanup in 0..101_u64 {
            let name = format!("item-{cleanup}");
            record(&fixture.journal, TAKEN, Some(TAKEN), Method::Trash, &[moved(10, HOME_TRASH, &name)]);
        }
        let engine = engine(fixture, false);

        assert_eq!(engine.trash().trashes, [home_line(None, 1_010)]);
    }

    #[test]
    fn open_trash_opens_the_folder_of_its_line() {
        let fixture = fixture(Some(measured(500)));
        let system_ui = Arc::clone(&fixture.system_ui);
        let engine = engine(fixture, true);
        engine.trash();

        engine.open_trash(0).unwrap();

        assert_eq!(system_ui.calls(), [SystemUiCall::OpenFolder(PathBuf::from(HOME_TRASH))]);
    }

    #[test]
    fn finders_setting_on_gives_the_latest_end_plus_thirty_days() {
        let fixture = fixture(None).finder(FakeFinderSettings::new(AutoEmpty::After30Days));
        record(&fixture.journal, START - 100, Some(START - 90), Method::Trash, &[moved(100, HOME_TRASH, "a")]);
        record(&fixture.journal, START - 80, Some(START - 60), Method::Trash, &[moved(100, HOME_TRASH, "b")]);
        // Never ended: the moment it began counts.
        record(&fixture.journal, START - 50, None, Method::Trash, &[moved(100, HOME_TRASH, "c")]);
        let engine = engine(fixture, false);

        let trashes = engine.trash().trashes;

        assert_eq!(trashes.len(), 1);
        assert_eq!(trashes[0].gone_by, Some(Timestamp::from_secs(START - 50 + 30 * DAY)));
    }
}
