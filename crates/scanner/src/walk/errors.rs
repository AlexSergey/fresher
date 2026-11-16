//! What a folder that fails to read becomes: never an error of the walk, an unreadable folder of a kind.

use std::path::Path;

use fresher_core::{FolderIdentity, FolderListing, FolderReader, ReadError, Unreadable};

/// How many times a read interrupted by a signal is tried again before it counts as failed.
const RETRIES: usize = 3;

/// Reads the folder, trying an interrupted read again up to three times.
pub(crate) fn read_folder(
    reader: &dyn FolderReader,
    path: &Path,
    expected: FolderIdentity,
    listing: &mut FolderListing,
) -> Result<(), ReadError> {
    let mut tries = 0;
    loop {
        match reader.read(path, expected, listing) {
            Err(ReadError::Interrupted) if tries < RETRIES => tries += 1,
            result => return result,
        }
    }
}

/// The kind of unreadable folder a read error makes: `EACCES` and `EPERM` by name; a folder removed or replaced
/// during the walk, or still interrupted after the retries, is `Failed`.
pub(crate) const fn unreadable(error: ReadError) -> Unreadable {
    match error {
        ReadError::NoPermission => Unreadable::NoPermission,
        ReadError::Guarded => Unreadable::Guarded,
        ReadError::Vanished | ReadError::Replaced | ReadError::Interrupted | ReadError::Failed => Unreadable::Failed,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use fresher_core::{FolderIdentity, FolderListing, FolderReader, ReadError};

    /// Answers `Interrupted` for its first `failures` reads, then an empty folder.
    struct Interrupting {
        failures: usize,
        reads: AtomicUsize,
    }

    impl FolderReader for Interrupting {
        fn identify(&self, _path: &Path) -> Result<FolderIdentity, ReadError> {
            Ok(FolderIdentity { device: 1, file_id: 1 })
        }

        fn access(&self, _path: &Path) -> Result<fresher_core::EntryAccess, ReadError> {
            Ok(fresher_core::EntryAccess::default())
        }

        fn read(&self, _path: &Path, _expected: FolderIdentity, _out: &mut FolderListing) -> Result<(), ReadError> {
            let read = self.reads.fetch_add(1, Ordering::Relaxed);
            if read < self.failures { Err(ReadError::Interrupted) } else { Ok(()) }
        }

        fn head(&self, _folder: &Path, _name: &str, _limit: usize) -> Result<Vec<u8>, ReadError> {
            Err(ReadError::Failed)
        }
    }

    fn read(failures: usize) -> (Result<(), ReadError>, usize) {
        let reader = Interrupting { failures, reads: AtomicUsize::new(0) };
        let result = super::read_folder(
            &reader,
            Path::new("/a"),
            FolderIdentity { device: 1, file_id: 1 },
            &mut FolderListing::new(),
        );
        (result, reader.reads.load(Ordering::Relaxed))
    }

    mod negative {
        use fresher_core::{ReadError, Unreadable};

        use super::read;
        use crate::walk::errors::unreadable;

        #[test]
        fn a_read_interrupted_four_times_fails() {
            assert_eq!(read(4), (Err(ReadError::Interrupted), 4));
            assert_eq!(unreadable(ReadError::Interrupted), Unreadable::Failed);
        }

        #[test]
        fn a_vanished_or_replaced_folder_is_failed() {
            assert_eq!(unreadable(ReadError::Vanished), Unreadable::Failed);
            assert_eq!(unreadable(ReadError::Replaced), Unreadable::Failed);
            assert_eq!(unreadable(ReadError::Failed), Unreadable::Failed);
        }
    }

    mod positive {
        use fresher_core::{ReadError, Unreadable};

        use super::read;
        use crate::walk::errors::unreadable;

        #[test]
        fn an_interrupted_read_is_tried_again_three_times() {
            assert_eq!(read(3), (Ok(()), 4));
            assert_eq!(read(0), (Ok(()), 1));
        }

        #[test]
        fn permissions_keep_their_kind() {
            assert_eq!(unreadable(ReadError::NoPermission), Unreadable::NoPermission);
            assert_eq!(unreadable(ReadError::Guarded), Unreadable::Guarded);
        }
    }
}
