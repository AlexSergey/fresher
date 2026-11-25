//! Reading one small text file a module's matches showed: a version pin, a pointer file. Read-only, never follows a
//! link as the file's own name (a link among its folder's names is a known issue:
//! `docs/known-issues/core-small-file-folder-through-a-link.md`), and the bytes are never logged.

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use crate::{FolderReader, ReadError};

/// The most bytes a module may read of one file.
pub const SMALL_FILE_LIMIT: usize = 4096;

/// What reading one small text file found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SmallFile {
    /// The whole file, as text.
    Text(String),
    /// Nothing of that name is there any more.
    Missing,
    /// Something of that name is there but is not a regular file (a link, a folder, a socket): never followed.
    NotAFile,
    /// Longer than [`SMALL_FILE_LIMIT`].
    TooLong,
    /// Not UTF-8.
    NotText,
    /// It could not be read (permissions, privacy protection, a broken disk).
    Unreadable,
}

/// Reads the first bytes of one file: the port behind `SystemFacts::small_file`.
pub trait SmallFiles: Send + Sync {
    /// The first `limit` bytes of the regular file `name` in `folder`; never follows a link.
    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError>;
}

impl fmt::Debug for dyn SmallFiles {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SmallFiles")
    }
}

/// Small files read through a folder reader: the same no-follow, regular-file-only call the walk uses for
/// `CACHEDIR.TAG`.
pub struct ReaderFiles(pub Arc<dyn FolderReader>);

impl SmallFiles for ReaderFiles {
    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        self.0.head(folder, name, limit)
    }
}

impl fmt::Debug for ReaderFiles {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("ReaderFiles").finish_non_exhaustive()
    }
}

/// What a read of at most `SMALL_FILE_LIMIT + 1` bytes found.
pub(crate) fn judge(read: Result<Vec<u8>, ReadError>) -> SmallFile {
    match read {
        Ok(bytes) if bytes.len() > SMALL_FILE_LIMIT => SmallFile::TooLong,
        Ok(bytes) => String::from_utf8(bytes).map_or(SmallFile::NotText, SmallFile::Text),
        Err(ReadError::Vanished) => SmallFile::Missing,
        Err(ReadError::Replaced) => SmallFile::NotAFile,
        Err(ReadError::NoPermission | ReadError::Guarded | ReadError::Interrupted | ReadError::Failed) => {
            SmallFile::Unreadable
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::{ReadError, SystemFacts};

    use super::SmallFiles;

    /// Answers `answer` to every read, of at most the asked limit; counts the reads.
    struct Answering {
        answer: Result<Vec<u8>, ReadError>,
        reads: AtomicUsize,
    }

    impl SmallFiles for Answering {
        fn head(&self, _folder: &Path, _name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            self.answer.clone().map(|bytes| bytes.into_iter().take(limit).collect())
        }
    }

    fn answering(answer: Result<Vec<u8>, ReadError>) -> Arc<Answering> {
        Arc::new(Answering { answer, reads: AtomicUsize::new(0) })
    }

    fn facts(files: &Arc<Answering>) -> SystemFacts {
        crate::system::facts::nothing_known_but(Arc::clone(files) as Arc<dyn SmallFiles>)
    }

    fn read(answer: Result<Vec<u8>, ReadError>) -> super::SmallFile {
        facts(&answering(answer)).small_file(Path::new("/project"), ".nvmrc")
    }

    mod negative {
        use std::path::Path;

        use crate::ReadError;

        use super::super::{SMALL_FILE_LIMIT, SmallFile};
        use super::read;

        #[test]
        fn a_missing_file_is_missing() {
            assert_eq!(read(Err(ReadError::Vanished)), SmallFile::Missing);
        }

        #[test]
        fn a_link_or_a_folder_is_not_a_file() {
            assert_eq!(read(Err(ReadError::Replaced)), SmallFile::NotAFile);
        }

        #[test]
        fn a_file_over_the_limit_is_too_long() {
            assert_eq!(read(Ok(vec![b'1'; SMALL_FILE_LIMIT + 1])), SmallFile::TooLong);
        }

        #[test]
        fn bytes_that_are_not_utf8_are_not_text() {
            assert_eq!(read(Ok(vec![0xff, 0xfe, b'2'])), SmallFile::NotText);
        }

        #[test]
        fn any_other_read_error_is_unreadable() {
            for error in [ReadError::NoPermission, ReadError::Guarded, ReadError::Interrupted, ReadError::Failed] {
                assert_eq!(read(Err(error)), SmallFile::Unreadable, "{error:?}");
            }
        }

        #[test]
        fn a_system_that_knows_nothing_reads_nothing() {
            let facts = crate::system::nothing_known();

            assert_eq!(facts.small_file(Path::new("/project"), ".nvmrc"), SmallFile::Unreadable);
        }
    }

    mod positive {
        use std::path::Path;
        use std::sync::Arc;
        use std::sync::atomic::Ordering;

        use crate::{EntryAccess, FolderIdentity, FolderListing, FolderReader, ReadError};

        use super::super::{ReaderFiles, SMALL_FILE_LIMIT, SmallFile, SmallFiles};
        use super::{answering, facts, read};

        /// A reader whose only file holds `abc`; it lists and identifies nothing.
        struct OneFile;

        impl FolderReader for OneFile {
            fn identify(&self, _path: &Path) -> Result<FolderIdentity, ReadError> {
                Err(ReadError::Failed)
            }

            fn access(&self, _path: &Path) -> Result<EntryAccess, ReadError> {
                Err(ReadError::Failed)
            }

            fn read(&self, _path: &Path, _expected: FolderIdentity, _out: &mut FolderListing) -> Result<(), ReadError> {
                Err(ReadError::Failed)
            }

            fn head(&self, _folder: &Path, _name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
                Ok(b"abc".iter().copied().take(limit).collect())
            }
        }

        #[test]
        fn a_file_at_the_limit_is_read_whole() {
            assert_eq!(read(Ok(vec![b'1'; SMALL_FILE_LIMIT])), SmallFile::Text("1".repeat(SMALL_FILE_LIMIT)));
        }

        #[test]
        fn an_empty_file_is_empty_text() {
            assert_eq!(read(Ok(Vec::new())), SmallFile::Text(String::new()));
        }

        #[test]
        fn each_call_reads_the_disk_again() {
            let files = answering(Ok(b"20\n".to_vec()));
            let facts = facts(&files);

            for _ in 0..2 {
                assert_eq!(facts.small_file(Path::new("/project"), ".nvmrc"), SmallFile::Text("20\n".into()));
            }
            assert_eq!(files.reads.load(Ordering::SeqCst), 2);
        }

        #[test]
        fn the_reader_files_read_through_the_folder_reader() {
            let files = ReaderFiles(Arc::new(OneFile));

            assert_eq!(files.head(Path::new("/project"), ".nvmrc", 2), Ok(b"ab".to_vec()));
            assert!(format!("{files:?}").contains("ReaderFiles"));
            assert_eq!(format!("{:?}", &files as &dyn SmallFiles), "SmallFiles");
        }
    }
}
