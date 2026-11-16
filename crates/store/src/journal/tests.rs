//! The journal through its port, on temporary data folders.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use fresher_core::{FormatVersion, StoreError};
use serde_json::Value;
use tempfile::TempDir;

use super::FileCleanupJournal;
use super::read::Format;
use super::records::VERSION;
use super::rotate::JOURNAL_GENERATION;
use crate::DataFolder;

pub(super) const HEADER: &str = "{\"format\":\"cleanup-journal\",\"version\":1}\n";

pub(super) struct Folder {
    root: TempDir,
}

impl Folder {
    pub(super) fn new() -> Self {
        Self { root: TempDir::new().unwrap() }
    }

    pub(super) fn journal(&self) -> FileCleanupJournal {
        FileCleanupJournal::open(&DataFolder::open(self.root.path()).unwrap())
    }

    /// The journal of a build that writes version 2, with one migration from version 1.
    pub(super) fn journal_at_version_2(&self) -> FileCleanupJournal {
        let format = Format { version: FormatVersion::new(2), migrations: &[mark_categories] };
        FileCleanupJournal::with_format(&DataFolder::open(self.root.path()).unwrap(), format, JOURNAL_GENERATION)
    }

    /// The journal of this version whose generations start at `generation` bytes.
    pub(super) fn journal_with_generation(&self, generation: u64) -> FileCleanupJournal {
        let format = Format { version: VERSION, migrations: &[] };
        FileCleanupJournal::with_format(&DataFolder::open(self.root.path()).unwrap(), format, generation)
    }

    pub(super) fn previous(&self) -> PathBuf {
        self.path("cleanups.previous.jsonl")
    }

    pub(super) fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.root.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }

    pub(super) fn path(&self, name: &str) -> PathBuf {
        self.root.path().join(name)
    }

    pub(super) fn current(&self) -> PathBuf {
        self.path("cleanups.jsonl")
    }

    pub(super) fn text(&self) -> String {
        fs::read_to_string(self.current()).unwrap()
    }

    pub(super) fn lines(&self) -> Vec<String> {
        self.text().lines().map(str::to_owned).collect()
    }
}

/// The version 1 file kept as a golden: three cleanups (completed, cancelled, interrupted) and two put backs.
pub(super) fn fixture() -> Vec<u8> {
    fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/journal/v1.jsonl")).unwrap()
}

/// A migration from version 1 to 2 for the tests: every item's category gets a prefix.
fn mark_categories(mut line: Value) -> Result<Value, StoreError> {
    if let Some(items) = line.get_mut("items").and_then(Value::as_array_mut) {
        for item in items {
            let category = item["category"].as_str().ok_or(StoreError::Broken)?;
            item["category"] = Value::String(format!("migrated.{category}"));
        }
    }
    Ok(line)
}

fn syncs(journal: &FileCleanupJournal) -> u32 {
    journal.syncs.load(Ordering::Relaxed)
}

fn append(path: &Path, bytes: &[u8]) {
    let mut text = fs::read(path).unwrap();
    text.extend_from_slice(bytes);
    fs::write(path, text).unwrap();
}

mod negative {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use fresher_core::{CleanupId, CleanupJournal, Ended, JournalError, JournalProblem, JournalState, RecordedOutcome};

    use super::{Folder, HEADER, append, fixture, syncs};
    use crate::journal::records::sample;

    #[test]
    fn a_missing_file_starts_with_its_header() {
        let folder = Folder::new();
        let journal = folder.journal();
        assert!(!folder.current().exists(), "opening writes nothing");

        journal.begin(&sample::begin(1)).unwrap();

        assert!(folder.text().starts_with(HEADER), "{}", folder.text());
        assert_eq!(folder.lines().len(), 2);
    }

    #[test]
    fn a_torn_last_line_gets_a_newline_before_the_next_record() {
        let folder = Folder::new();
        let first = folder.journal().begin(&sample::begin(2)).unwrap();
        append(&folder.current(), b"{\"type\":\"item\",\"clean");

        let journal = folder.journal();
        let second = journal.begin(&sample::begin(1)).unwrap();

        let lines = folder.lines();
        assert_eq!(lines[2], "{\"type\":\"item\",\"clean");
        assert!(lines[3].starts_with("{\"type\":\"begin\",\"cleanup\":2,"), "{}", lines[3]);
        assert_eq!((first, second), (CleanupId::new(1), CleanupId::new(2)));
        let page = journal.cleanups(0, 10).unwrap();
        assert_eq!((page.total, page.skipped), (2, 1));
    }

    #[test]
    fn a_torn_last_record_never_corrupts_the_records_before_it() {
        let folder = Folder::new();
        let journal = folder.journal();
        let id = journal.begin(&sample::begin(2)).unwrap();
        journal.item(&sample::moved(id, 0)).unwrap();
        journal.sync().unwrap();
        let whole = fs::read(folder.current()).unwrap();
        let before = journal.cleanup(id).unwrap().unwrap();
        journal.end(&sample::end(id)).unwrap();
        let with_end = fs::read(folder.current()).unwrap();

        for cut in whole.len()..with_end.len() - 1 {
            fs::write(folder.current(), &with_end[..cut]).unwrap();
            let journal = folder.journal();

            assert_eq!(journal.cleanup(id).unwrap().as_ref(), Some(&before), "cut at {cut}");
            let next = journal.begin(&sample::begin(1)).unwrap();
            assert_eq!(next, CleanupId::new(2), "cut at {cut}");
            assert_eq!(journal.cleanup(id).unwrap().as_ref(), Some(&before), "cut at {cut}");
            assert_eq!(journal.cleanup(next).unwrap().unwrap().outcome(0), RecordedOutcome::Unknown);
        }
    }

    #[test]
    fn begin_is_synced_before_it_returns() {
        let folder = Folder::new();
        let journal = folder.journal();

        let id = journal.begin(&sample::begin(3)).unwrap();
        assert_eq!(syncs(&journal), 1);
        journal.item(&sample::moved(id, 0)).unwrap();
        assert_eq!(syncs(&journal), 1, "an item is not synced on its own");
        journal.sync().unwrap();
        journal.end(&sample::end(id)).unwrap();

        assert_eq!(syncs(&journal), 3);
    }

    #[test]
    fn the_file_is_private_to_the_user() {
        let folder = Folder::new();

        folder.journal().begin(&sample::begin(1)).unwrap();

        assert_eq!(fs::metadata(folder.current()).unwrap().permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn a_failed_write_is_reported_and_writes_nothing_after_it() {
        let folder = Folder::new();
        fs::create_dir(folder.current()).unwrap();
        let journal = folder.journal();

        assert!(journal.begin(&sample::begin(1)).is_err());
        assert!(journal.item(&sample::moved(CleanupId::new(1), 0)).is_err());
        assert!(folder.current().is_dir());
    }
    #[test]
    fn a_newer_version_is_read_only_and_never_written() {
        let folder = Folder::new();
        let text = String::from_utf8(fixture()).unwrap().replacen("\"version\":1", "\"version\":2", 1);
        fs::write(folder.current(), &text).unwrap();
        let journal = folder.journal();

        assert_eq!(journal.state(), JournalState::ReadOnly { why: JournalProblem::NewerVersion });
        assert_eq!(journal.begin(&sample::begin(1)), Err(JournalError::ReadOnly));
        assert_eq!(journal.item(&sample::moved(CleanupId::new(3), 1)), Err(JournalError::ReadOnly));
        assert_eq!(journal.sync(), Err(JournalError::ReadOnly));
        assert_eq!(folder.text(), text);
        assert_eq!(journal.cleanups(0, 10).unwrap().total, 3, "what can be read is read");
    }

    #[test]
    fn an_unreadable_file_is_read_only() {
        let folder = Folder::new();
        fs::write(folder.current(), fixture()).unwrap();
        fs::set_permissions(folder.current(), fs::Permissions::from_mode(0o000)).unwrap();
        let journal = folder.journal();

        let state = journal.state();
        let begin = journal.begin(&sample::begin(1));
        let page = journal.cleanups(0, 10);

        fs::set_permissions(folder.current(), fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(state, JournalState::ReadOnly { why: JournalProblem::Unreadable });
        assert_eq!(begin, Err(JournalError::ReadOnly));
        assert!(matches!(page, Err(JournalError::Failed { .. })), "{page:?}");
        assert_eq!(fs::read(folder.current()).unwrap(), fixture());
        assert_eq!(folder.names(), ["cleanups.jsonl"]);
    }

    #[test]
    fn a_broken_header_is_moved_aside_on_the_next_begin() {
        let folder = Folder::new();
        fs::write(folder.current(), "not a journal\n").unwrap();
        fs::write(folder.path("cleanups.broken.jsonl"), "an older broken file\n").unwrap();
        let journal = folder.journal();

        assert_eq!(journal.state(), JournalState::Ready);
        assert_eq!(journal.cleanups(0, 10).unwrap().total, 0);
        assert_eq!(folder.text(), "not a journal\n", "opening and reading move nothing");
        let id = journal.begin(&sample::begin(1)).unwrap();

        assert_eq!(id, CleanupId::new(1));
        assert_eq!(fs::read_to_string(folder.path("cleanups.broken.jsonl")).unwrap(), "not a journal\n");
        assert!(folder.text().starts_with(HEADER));
        assert_eq!(folder.names(), ["cleanups.broken.jsonl", "cleanups.jsonl"]);
    }

    #[test]
    fn an_unparsable_line_is_skipped_and_counted() {
        let folder = Folder::new();
        let mut lines: Vec<String> = String::from_utf8(fixture()).unwrap().lines().map(str::to_owned).collect();
        lines.insert(3, "not json".into());
        lines.insert(5, "{\"type\":\"item\",\"cleanup\":1}".into());
        fs::write(folder.current(), lines.join("\n") + "\n{\"type\":\"end\",\"cl").unwrap();

        let page = folder.journal().cleanups(0, 10).unwrap();

        assert_eq!((page.total, page.skipped), (3, 3));
    }

    #[test]
    fn a_begin_without_an_end_reads_as_interrupted() {
        let folder = Folder::new();
        let journal = folder.journal();
        let id = journal.begin(&sample::begin(2)).unwrap();
        journal.item(&sample::moved(id, 0)).unwrap();
        drop(journal);

        let recorded = folder.journal().cleanup(id).unwrap().unwrap();

        assert_eq!(recorded.end, None);
        assert_eq!(recorded.summary().ended, Ended::Interrupted);
        assert_eq!((recorded.outcome(0), recorded.outcome(1)), (RecordedOutcome::Moved, RecordedOutcome::Unknown));
    }

    #[test]
    fn an_unknown_cleanup_reads_as_none() {
        let folder = Folder::new();
        fs::write(folder.current(), fixture()).unwrap();

        assert_eq!(folder.journal().cleanup(CleanupId::new(9)), Ok(None));
        assert_eq!(Folder::new().journal().cleanup(CleanupId::new(1)), Ok(None));
    }
}

mod positive {
    use std::ffi::OsStr;
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use std::path::PathBuf;

    use fresher_core::{
        CleanupId, CleanupJournal, Ended, JournalState, Method, PutBackOutcome, RecordedOutcome, SkipCode,
    };

    use super::{Folder, HEADER, fixture};
    use crate::journal::records::sample;

    #[test]
    fn ids_continue_from_the_highest_in_the_file() {
        let folder = Folder::new();
        fs::write(
            folder.current(),
            format!("{HEADER}{{\"type\":\"put-back\",\"cleanup\":41,\"index\":0,\"at\":1,\"outcome\":{{\"kind\":\"gone\"}}}}\n"),
        )
        .unwrap();
        let journal = folder.journal();

        assert_eq!(journal.state(), JournalState::Ready);
        assert_eq!(journal.begin(&sample::begin(1)).unwrap(), CleanupId::new(42));
        assert_eq!(folder.journal().begin(&sample::begin(1)).unwrap(), CleanupId::new(43));
    }

    #[test]
    fn records_append_in_order() {
        let folder = Folder::new();
        let journal = folder.journal();

        let id = journal.begin(&sample::begin(3)).unwrap();
        journal.item(&sample::moved(id, 0)).unwrap();
        journal.item(&sample::skipped(id, 1)).unwrap();
        journal.end(&sample::end(id)).unwrap();
        journal.put_back(&sample::put_back(id, 0, PutBackOutcome::PutBack)).unwrap();

        let types: Vec<String> =
            folder.lines()[1..].iter().map(|line| line.split('"').nth(3).unwrap().to_owned()).collect();
        assert_eq!(types, ["begin", "item", "item", "end", "put-back"]);
        let recorded = journal.cleanup(id).unwrap().unwrap();
        assert_eq!(recorded.begin, sample::begin(3));
        assert_eq!(recorded.items, [sample::moved(id, 0), sample::skipped(id, 1)]);
        assert_eq!(recorded.end, Some(sample::end(id)));
        assert_eq!(recorded.put_backs, [sample::put_back(id, 0, PutBackOutcome::PutBack)]);
    }

    #[test]
    fn a_non_utf8_path_round_trips_through_the_journal() {
        let folder = Folder::new();
        let journal = folder.journal();
        let mut begin = sample::begin(1);
        begin.items[0].path = Some(PathBuf::from(OsStr::from_bytes(b"/home/\xff\xfe/n\xe9e")));

        let id = journal.begin(&begin).unwrap();

        assert!(folder.text().contains("{\"bytes\":\"2f686f6d652ffffe2f6ee965\"}"), "{}", folder.text());
        assert_eq!(journal.cleanup(id).unwrap().unwrap().begin, begin);
    }

    #[test]
    fn a_cleanup_of_10000_items_writes_one_begin_line() {
        let folder = Folder::new();
        let journal = folder.journal();

        let id = journal.begin(&sample::begin(10_000)).unwrap();

        assert_eq!(folder.lines().len(), 2);
        assert_eq!(journal.cleanup(id).unwrap().unwrap().begin.items.len(), 10_000);
    }
    #[test]
    fn summaries_come_newest_first_and_paged() {
        let folder = Folder::new();
        let journal = folder.journal();
        for _ in 0..5 {
            let id = journal.begin(&sample::begin(1)).unwrap();
            journal.end(&sample::end(id)).unwrap();
        }
        let ids = |offset, limit| -> Vec<u32> {
            journal.cleanups(offset, limit).unwrap().cleanups.iter().map(|summary| summary.cleanup.get()).collect()
        };

        assert_eq!(ids(0, 2), [5, 4]);
        assert_eq!(ids(2, 2), [3, 2]);
        assert_eq!(ids(4, 2), [1]);
        assert_eq!(ids(10, 2), Vec::<u32>::new());
        assert_eq!(journal.cleanups(0, 0).unwrap().total, 5);
    }

    #[test]
    fn a_cleanup_reads_back_with_its_outcomes_and_put_backs() {
        let folder = Folder::new();
        let journal = folder.journal();
        let id = journal.begin(&sample::begin(3)).unwrap();
        journal.item(&sample::moved(id, 0)).unwrap();
        journal.item(&sample::moved(id, 1)).unwrap();
        journal.item(&sample::skipped(id, 2)).unwrap();
        journal.end(&sample::end(id)).unwrap();
        journal.put_back(&sample::put_back(id, 0, PutBackOutcome::Occupied)).unwrap();
        journal.put_back(&sample::put_back(id, 0, PutBackOutcome::PutBack)).unwrap();

        let recorded = journal.cleanup(id).unwrap().unwrap();
        let summary = journal.cleanups(0, 1).unwrap().cleanups[0];

        assert_eq!(recorded.outcome(2), RecordedOutcome::Skipped { reason: SkipCode::InUse });
        let outcomes: Vec<PutBackOutcome> = recorded.put_backs.iter().map(|record| record.outcome).collect();
        assert_eq!(outcomes, [PutBackOutcome::Occupied, PutBackOutcome::PutBack]);
        assert_eq!(summary, recorded.summary());
        assert_eq!((summary.ended, summary.count, summary.in_trash), (Ended::Completed, 3, 1));
    }

    #[test]
    fn a_version_1_file_reads() {
        let folder = Folder::new();
        fs::write(folder.current(), fixture()).unwrap();
        let journal = folder.journal();

        let page = journal.cleanups(0, 10).unwrap();
        let first = journal.cleanup(CleanupId::new(1)).unwrap().unwrap();
        let third = journal.cleanup(CleanupId::new(3)).unwrap().unwrap();

        assert_eq!((page.total, page.skipped), (3, 0));
        let ended: Vec<Ended> = page.cleanups.iter().map(|summary| summary.ended).collect();
        assert_eq!(ended, [Ended::Interrupted, Ended::Cancelled, Ended::Completed]);
        assert_eq!(page.cleanups[1].method, Method::Permanent);
        assert_eq!(page.cleanups[1].deleted.allocated.bytes(), 4096);
        assert_eq!(first.outcome(1), RecordedOutcome::Emptied { moved: 2, skipped: 1 });
        assert_eq!(first.outcome(2), RecordedOutcome::ToolDone);
        assert_eq!(first.items[1].trashed.len(), 2);
        assert_eq!(first.begin.items[2].path, None);
        assert_eq!(first.begin.items[0].reason.key().as_str(), "demo.finding.reason");
        assert_eq!(first.put_backs[1].outcome, PutBackOutcome::Refused { reason: SkipCode::Protected });
        assert_eq!(page.cleanups[2].in_trash, 1, "the emptied folder's items; the first item was put back");
        let path = third.begin.items[0].path.as_deref().unwrap();
        assert_eq!(path.as_os_str().as_bytes(), b"/Users/demo/Library/Caches/n\xe9e");
        assert_eq!(third.outcome(1), RecordedOutcome::Unknown);
        assert_eq!(journal.begin(&sample::begin(1)).unwrap(), CleanupId::new(4));
    }

    #[test]
    fn an_older_version_is_migrated_line_by_line() {
        let folder = Folder::new();
        fs::write(folder.current(), fixture()).unwrap();

        let journal = folder.journal_at_version_2();

        assert_eq!(journal.state(), JournalState::Ready);
        let first = journal.cleanup(CleanupId::new(1)).unwrap().unwrap();
        assert_eq!(first.begin.items[0].category, "migrated.demo.caches");
        assert_eq!(journal.cleanups(0, 10).unwrap().total, 3);
    }
}
