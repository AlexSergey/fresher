//! The journal file on the disk: what its first line says it is, and appending to it.

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::Path;

use fresher_core::{FormatVersion, JournalError};

use super::records::{FORMAT, Header, header_line};

/// The mode of a journal file the store creates: only the user can read or write it.
const PRIVATE_MODE: u32 = 0o600;
/// The most of a file read to find its header: a first line longer than this is not a journal's.
pub(super) const HEADER_LIMIT: u64 = 4096;

/// What a journal file is, by its first line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Found {
    /// No file, or an empty one: the header was never written.
    Missing,
    /// A journal of this version.
    Journal(FormatVersion),
    /// Something else: moved aside before the journal is written again.
    Broken,
    /// The file exists and cannot be opened or read.
    Unreadable,
}

/// Reads the first line of the file at `path`.
pub(super) fn inspect(path: &Path) -> Found {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Found::Missing,
        Err(_) => return Found::Unreadable,
    };
    let mut first = Vec::new();
    match BufReader::new(file).take(HEADER_LIMIT).read_until(b'\n', &mut first) {
        Ok(0) => Found::Missing,
        Ok(_) => header_of(&first),
        Err(_) => Found::Unreadable,
    }
}

/// What a first line says the file is.
pub(super) fn header_of(line: &[u8]) -> Found {
    match serde_json::from_slice::<Header>(line) {
        Ok(header) if header.format == FORMAT && header.version.get() > 0 => Found::Journal(header.version),
        _ => Found::Broken,
    }
}

/// The journal file open for appending: every write goes to its end.
#[derive(Debug)]
pub(super) struct Appender {
    file: File,
}

impl Appender {
    /// Opens the file at `path` for appending, creating it private to the user with its header line at `version` when
    /// it is missing or empty. A last line torn by a crash is closed with a newline, so the next record starts on a
    /// line of its own.
    pub(super) fn open(path: &Path, version: FormatVersion) -> Result<Self, JournalError> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .mode(PRIVATE_MODE)
            .open(path)
            .map_err(|error| journal_error(&error))?;
        let mut appender = Self { file };
        let length = appender.len()?;
        if length == 0 {
            appender.append(&header_line(version)?)?;
            appender.sync()?;
            // A new file's name is durable only once its folder is synced; the records that follow rely on it.
            let folder = path.parent().ok_or(JournalError::Failed { reason: "no folder".into() })?;
            File::open(folder).and_then(|opened| opened.sync_all()).map_err(|error| journal_error(&error))?;
        } else {
            let mut last = [0_u8];
            appender.file.read_exact_at(&mut last, length - 1).map_err(|error| journal_error(&error))?;
            if last != *b"\n" {
                appender.append(b"\n")?;
            }
        }
        Ok(appender)
    }

    /// Appends `line` (a whole line with its newline); durable only after `sync`.
    pub(super) fn append(&mut self, line: &[u8]) -> Result<(), JournalError> {
        self.file.write_all(line).map_err(|error| journal_error(&error))
    }

    /// Makes every line appended so far durable.
    pub(super) fn sync(&self) -> Result<(), JournalError> {
        self.file.sync_data().map_err(|error| journal_error(&error))
    }

    /// The length of the file now.
    pub(super) fn len(&self) -> Result<u64, JournalError> {
        Ok(self.file.metadata().map_err(|error| journal_error(&error))?.len())
    }
}

/// The journal's error for an I/O error: a full disk is told apart; the rest keeps only its kind, never a path.
pub(super) fn journal_error(error: &io::Error) -> JournalError {
    match error.kind() {
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded => JournalError::NoSpace,
        kind => JournalError::Failed { reason: kind.to_string() },
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;
        use std::io;
        use std::os::unix::fs::PermissionsExt;

        use fresher_core::JournalError;
        use tempfile::TempDir;

        use crate::journal::file::{Appender, Found, inspect, journal_error};
        use crate::journal::records::VERSION;

        #[test]
        fn a_missing_or_empty_file_has_no_header() {
            let root = TempDir::new().unwrap();
            let path = root.path().join("cleanups.jsonl");

            assert_eq!(inspect(&path), Found::Missing);
            fs::write(&path, b"").unwrap();
            assert_eq!(inspect(&path), Found::Missing);
        }

        #[test]
        fn a_first_line_that_is_not_a_journal_header_is_broken() {
            let root = TempDir::new().unwrap();
            let path = root.path().join("cleanups.jsonl");

            for text in [
                &b"not json\n"[..],
                b"{\"format\":\"settings\",\"version\":1}\n",
                b"{\"format\":\"cleanup-journal\",\"version\":0}\n",
                b"{\"format\":\"cleanup-journal\"}\n",
                &[b'x'; 5000],
            ] {
                fs::write(&path, text).unwrap();
                assert_eq!(inspect(&path), Found::Broken, "{}", String::from_utf8_lossy(text));
            }
        }

        #[test]
        fn a_file_that_cannot_be_opened_is_unreadable() {
            let root = TempDir::new().unwrap();
            let path = root.path().join("cleanups.jsonl");
            fs::write(&path, b"{\"format\":\"cleanup-journal\",\"version\":1}\n").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

            assert_eq!(inspect(&path), Found::Unreadable);
            assert!(Appender::open(&path, VERSION).is_err());
        }

        #[test]
        fn a_full_disk_is_told_apart_and_no_error_holds_a_path() {
            let full = io::Error::new(io::ErrorKind::StorageFull, "/home/secret");
            let denied = io::Error::new(io::ErrorKind::PermissionDenied, "/home/secret");

            assert_eq!(journal_error(&full), JournalError::NoSpace);
            assert_eq!(journal_error(&denied), JournalError::Failed { reason: "permission denied".into() });
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::FormatVersion;
        use tempfile::TempDir;

        use crate::journal::file::{Appender, Found, inspect};
        use crate::journal::records::VERSION;

        #[test]
        fn a_journal_header_names_its_version() {
            let root = TempDir::new().unwrap();
            let path = root.path().join("cleanups.jsonl");
            fs::write(&path, b"{\"format\":\"cleanup-journal\",\"version\":3}\n{\"type\":").unwrap();

            assert_eq!(inspect(&path), Found::Journal(FormatVersion::new(3)));
        }

        #[test]
        fn an_appender_on_a_whole_file_adds_nothing_before_the_next_line() {
            let root = TempDir::new().unwrap();
            let path = root.path().join("cleanups.jsonl");
            Appender::open(&path, VERSION).unwrap().append(b"{}\n").unwrap();

            let mut appender = Appender::open(&path, VERSION).unwrap();
            appender.append(b"[]\n").unwrap();

            assert_eq!(fs::read(&path).unwrap(), b"{\"format\":\"cleanup-journal\",\"version\":1}\n{}\n[]\n");
            assert_eq!(appender.len().unwrap(), 47);
        }
    }
}
