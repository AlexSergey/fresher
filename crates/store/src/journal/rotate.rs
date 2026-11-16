//! Two generations: `cleanups.jsonl` grows until a `begin` finds it at the threshold, then it is renamed over
//! `cleanups.previous.jsonl` and a new file starts. The oldest generation drops off by that rename; nothing is deleted.

use fresher_core::JournalError;

use super::file::Found;
use super::{FILE_NAME, FileCleanupJournal, Inner};

/// The size at which the current file becomes the previous generation: two generations keep the journal near 8 MiB.
pub(super) const JOURNAL_GENERATION: u64 = 4 << 20;
/// The generation before the current one.
pub(super) const PREVIOUS_FILE_NAME: &str = "cleanups.previous.jsonl";

impl FileCleanupJournal {
    /// At a `begin`, with the current file open: starts a new generation when the file has reached the threshold. A
    /// cleanup never spans two generations, since only a `begin` rotates; a `begin` larger than a generation is still
    /// written whole, into the new file.
    pub(super) fn rotate_if_due(&self, inner: &mut Inner) -> Result<(), JournalError> {
        let length = match &inner.appender {
            Some(appender) => appender.len()?,
            None => return Ok(()),
        };
        if length < self.generation {
            return Ok(());
        }
        self.start_generation(inner)
    }

    /// Renames the current file over the previous generation; the next write starts a new file with its header.
    pub(super) fn start_generation(&self, inner: &mut Inner) -> Result<(), JournalError> {
        inner.appender = None;
        self.rename(FILE_NAME, PREVIOUS_FILE_NAME)?;
        inner.found = Found::Missing;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use fresher_core::{CleanupJournal, PutBackOutcome};

        use crate::journal::records::sample;
        use crate::journal::tests::Folder;

        #[test]
        fn a_file_under_the_threshold_is_not_rotated() {
            let folder = Folder::new();
            let journal = folder.journal_with_generation(1 << 20);

            for _ in 0..10 {
                let id = journal.begin(&sample::begin(3)).unwrap();
                journal.end(&sample::end(id)).unwrap();
            }

            assert_eq!(folder.names(), ["cleanups.jsonl"]);
            assert_eq!(journal.cleanups(0, 0).unwrap().total, 10);
        }

        #[test]
        fn rotation_never_happens_inside_a_cleanup() {
            let folder = Folder::new();
            let journal = folder.journal_with_generation(100);

            let id = journal.begin(&sample::begin(3)).unwrap();
            for index in 0..3 {
                journal.item(&sample::moved(id, index)).unwrap();
            }
            journal.end(&sample::end(id)).unwrap();
            journal.put_back(&sample::put_back(id, 0, PutBackOutcome::PutBack)).unwrap();

            assert_eq!(folder.names(), ["cleanups.jsonl"]);
            assert_eq!(folder.lines().len(), 7);
        }

        #[test]
        fn rotation_replaces_only_the_previous_generation() {
            let folder = Folder::new();
            fs::write(folder.previous(), "the oldest generation\n").unwrap();
            fs::write(folder.path("cleanups.broken.jsonl"), "kept\n").unwrap();
            fs::write(folder.path("settings.json"), "{}\n").unwrap();
            let journal = folder.journal_with_generation(100);
            journal.begin(&sample::begin(1)).unwrap();
            let current = fs::read(folder.current()).unwrap();

            journal.begin(&sample::begin(1)).unwrap();

            assert_eq!(fs::read(folder.previous()).unwrap(), current);
            assert_eq!(fs::read_to_string(folder.path("cleanups.broken.jsonl")).unwrap(), "kept\n");
            assert_eq!(fs::read_to_string(folder.path("settings.json")).unwrap(), "{}\n");
            assert_eq!(
                folder.names(),
                ["cleanups.broken.jsonl", "cleanups.jsonl", "cleanups.previous.jsonl", "settings.json"]
            );
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::{CleanupId, CleanupJournal, PutBackOutcome};

        use crate::journal::records::sample;
        use crate::journal::tests::{Folder, HEADER, fixture};

        #[test]
        fn a_file_past_the_threshold_rotates_at_the_next_begin() {
            let folder = Folder::new();
            let journal = folder.journal_with_generation(100);
            let first = journal.begin(&sample::begin(1)).unwrap();
            journal.end(&sample::end(first)).unwrap();
            let before = fs::read(folder.current()).unwrap();

            let second = journal.begin(&sample::begin(1)).unwrap();

            assert_eq!(fs::read(folder.previous()).unwrap(), before);
            assert!(folder.text().starts_with(HEADER));
            assert!(folder.lines()[1].starts_with("{\"type\":\"begin\",\"cleanup\":2,"), "{}", folder.text());
            assert_eq!(second, CleanupId::new(2));
        }

        #[test]
        fn a_begin_larger_than_a_generation_is_written_whole() {
            let folder = Folder::new();
            let journal = folder.journal_with_generation(1000);

            let id = journal.begin(&sample::begin(50)).unwrap();

            assert!(fs::metadata(folder.current()).unwrap().len() > 1000);
            assert_eq!(folder.lines().len(), 2);
            assert_eq!(journal.cleanup(id).unwrap().unwrap().begin.items.len(), 50);
        }

        #[test]
        fn history_reads_across_both_generations() {
            let folder = Folder::new();
            let journal = folder.journal_with_generation(100);
            let old = journal.begin(&sample::begin(2)).unwrap();
            journal.item(&sample::moved(old, 0)).unwrap();
            journal.end(&sample::end(old)).unwrap();
            let new = journal.begin(&sample::begin(1)).unwrap();

            journal.put_back(&sample::put_back(old, 0, PutBackOutcome::PutBack)).unwrap();

            let ids: Vec<u32> =
                journal.cleanups(0, 10).unwrap().cleanups.iter().map(|summary| summary.cleanup.get()).collect();
            assert_eq!(ids, [new.get(), old.get()]);
            let recorded = journal.cleanup(old).unwrap().unwrap();
            assert_eq!((recorded.items.len(), recorded.put_backs.len()), (1, 1));
            assert!(fs::read_to_string(folder.previous()).unwrap().contains("\"type\":\"end\""));
        }

        #[test]
        fn ids_continue_across_a_rotation() {
            let folder = Folder::new();
            let journal = folder.journal_with_generation(100);
            journal.begin(&sample::begin(1)).unwrap();
            journal.begin(&sample::begin(1)).unwrap();
            drop(journal);
            fs::rename(folder.current(), folder.previous()).unwrap();

            let reopened = folder.journal_with_generation(100);

            assert_eq!(reopened.begin(&sample::begin(1)).unwrap(), CleanupId::new(3));
        }

        #[test]
        fn ids_continue_from_the_current_file_when_the_previous_cannot_be_read() {
            use std::os::unix::fs::PermissionsExt;

            let folder = Folder::new();
            let journal = folder.journal_with_generation(100);
            journal.begin(&sample::begin(1)).unwrap();
            journal.begin(&sample::begin(1)).unwrap();
            journal.begin(&sample::begin(1)).unwrap();
            drop(journal);
            fs::set_permissions(folder.previous(), fs::Permissions::from_mode(0o000)).unwrap();

            let next = folder.journal_with_generation(100).begin(&sample::begin(1));

            fs::set_permissions(folder.previous(), fs::Permissions::from_mode(0o600)).unwrap();
            assert_eq!(next.unwrap(), CleanupId::new(4));
        }

        #[test]
        fn an_older_version_becomes_the_previous_generation_at_the_next_write() {
            let folder = Folder::new();
            fs::write(folder.current(), fixture()).unwrap();
            let journal = folder.journal_at_version_2();

            let id = journal.begin(&sample::begin(1)).unwrap();

            assert_eq!(id, CleanupId::new(4));
            assert_eq!(fs::read(folder.previous()).unwrap(), fixture());
            assert!(folder.text().starts_with("{\"format\":\"cleanup-journal\",\"version\":2}\n"));
            let page = journal.cleanups(0, 10).unwrap();
            assert_eq!((page.total, page.skipped), (4, 0));
            let first = journal.cleanup(CleanupId::new(1)).unwrap().unwrap();
            assert_eq!(first.begin.items[0].category, "migrated.demo.caches");
        }
    }
}
