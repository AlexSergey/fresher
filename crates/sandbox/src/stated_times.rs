use std::path::Path;

use fresher_core::{EntryAccess, FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError};
use fresher_testing::StatedFacts;

/// The folder reader of sandbox mode and the autotest: the real one, reporting for each entry the sandbox's world aged
/// (`World::aged`) the status change its age means, instead of the disk's. The disk cannot be told one: macOS moves a
/// file's ctime to the moment of any write and offers no way to set it back, so every generated file would be "changed
/// today" whatever its `age`. Every other entry, and every error, is the real reader's.
#[derive(Debug)]
pub struct StatedTimes<R> {
    inner: R,
    aged: StatedFacts,
}

impl<R: FolderReader> StatedTimes<R> {
    /// `inner`, with the status changes of `aged` (only their `changed` is read) at exactly their paths.
    pub const fn new(inner: R, aged: StatedFacts) -> Self {
        Self { inner, aged }
    }
}

impl<R: FolderReader> FolderReader for StatedTimes<R> {
    fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError> {
        self.inner.identify(path)
    }

    fn access(&self, path: &Path) -> Result<EntryAccess, ReadError> {
        self.inner.access(path)
    }

    fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
        self.inner.read(path, expected, out)?;
        if !self.aged.holds_in(path) {
            return Ok(());
        }
        let entries: Vec<(Vec<u8>, RawEntry)> = out.iter().map(|(name, entry)| (name.to_vec(), *entry)).collect();
        out.clear();
        for (name, entry) in entries {
            let aged = std::str::from_utf8(&name).ok().and_then(|text| self.aged.get(&path.join(text)));
            let entry = aged.map_or(entry, |aged| RawEntry { changed: aged.changed.or(entry.changed), ..entry });
            out.push(&name, entry)?;
        }
        Ok(())
    }

    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.inner.head(folder, name, limit)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_core::{
        EntryAccess, EntryKind, FolderIdentity, FolderListing, FolderReader, RawEntry, ReadError, Sizes, Timestamp,
    };
    use fresher_testing::{Stated, StatedFacts};

    use super::StatedTimes;

    const IDENTITY: FolderIdentity = FolderIdentity { device: 1, file_id: 2 };
    const TODAY: Timestamp = Timestamp::from_secs(1_000_000);
    const AGED: Timestamp = Timestamp::from_secs(500);

    /// A reader of one folder, `/box`, holding `old` and `new`, both changed today; any other folder is gone.
    struct Disk;

    impl FolderReader for Disk {
        fn identify(&self, _path: &Path) -> Result<FolderIdentity, ReadError> {
            Ok(IDENTITY)
        }

        fn access(&self, _path: &Path) -> Result<EntryAccess, ReadError> {
            Ok(EntryAccess::default())
        }

        fn read(&self, path: &Path, _expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError> {
            if path != Path::new("/box") {
                return Err(ReadError::Vanished);
            }
            let entry = RawEntry {
                kind: EntryKind::File,
                device: 1,
                file_id: 3,
                links: 1,
                sizes: Sizes::ZERO,
                clone: None,
                modified: Some(TODAY),
                changed: Some(TODAY),
                accessed: None,
                cloud_only: false,
                access: EntryAccess::default(),
                error: None,
            };
            out.push(b"old", entry)?;
            out.push(b"new", entry)
        }

        fn head(&self, _folder: &Path, _name: &str, _limit: usize) -> Result<Vec<u8>, ReadError> {
            Ok(b"head".to_vec())
        }
    }

    fn aged() -> StatedFacts {
        let mut facts = StatedFacts::new();
        facts.insert("/box/old".into(), Stated { changed: Some(AGED), ..Stated::default() });
        facts
    }

    fn read(reader: &StatedTimes<Disk>, folder: &str) -> Result<Vec<(String, Option<Timestamp>)>, ReadError> {
        let mut listing = FolderListing::new();
        reader.read(Path::new(folder), IDENTITY, &mut listing)?;
        Ok(listing.iter().map(|(name, entry)| (String::from_utf8_lossy(name).into_owned(), entry.changed)).collect())
    }

    mod negative {
        use fresher_core::ReadError;
        use fresher_testing::StatedFacts;

        use super::{Disk, StatedTimes, TODAY, aged, read};

        #[test]
        fn an_entry_the_world_did_not_age_passes_through() {
            let reader = StatedTimes::new(Disk, aged());

            let entries = read(&reader, "/box").unwrap();

            assert_eq!(entries[1], ("new".to_owned(), Some(TODAY)));
        }

        #[test]
        fn a_world_that_aged_nothing_changes_nothing() {
            let reader = StatedTimes::new(Disk, StatedFacts::new());

            let entries = read(&reader, "/box").unwrap();

            assert_eq!(entries.iter().map(|(_, changed)| *changed).collect::<Vec<_>>(), [Some(TODAY), Some(TODAY)]);
        }

        #[test]
        fn an_error_passes_through() {
            let reader = StatedTimes::new(Disk, aged());

            assert_eq!(read(&reader, "/elsewhere"), Err(ReadError::Vanished));
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::FolderReader;

        use super::{AGED, Disk, IDENTITY, StatedTimes, aged, read};

        #[test]
        fn an_aged_entry_is_changed_at_its_age() {
            let reader = StatedTimes::new(Disk, aged());

            let entries = read(&reader, "/box").unwrap();

            assert_eq!(entries[0], ("old".to_owned(), Some(AGED)));
        }

        #[test]
        fn identify_access_and_head_are_the_inner_reader() {
            let reader = StatedTimes::new(Disk, aged());

            assert_eq!(reader.identify(Path::new("/box")), Ok(IDENTITY));
            assert!(reader.access(Path::new("/box/old")).is_ok());
            assert_eq!(reader.head(Path::new("/box"), "old", 4), Ok(b"head".to_vec()));
        }
    }
}
