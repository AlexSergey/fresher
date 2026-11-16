//! Reading the journal back: line by line, never the whole file at once, into the cleanups it records.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use fresher_core::{CleanupId, FormatVersion, JournalError, RecordedCleanup};

use super::file::{Found, HEADER_LIMIT, header_of, journal_error};
use super::records::Line;
use crate::migrate::{Migration, migrate};

/// The version this build writes and the migrations that bring an older file's records to it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Format {
    pub(super) version: FormatVersion,
    pub(super) migrations: &'static [Migration],
}

/// Reads every record of the journal file at `path` into `into`. A missing file has none; a file that is not a
/// journal is not read. A line that does not parse — a line torn by a crash — is skipped and counted.
pub(super) fn read_file(path: &Path, format: Format, into: &mut Collected) -> Result<(), JournalError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(journal_error(&error)),
    };
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    (&mut reader).take(HEADER_LIMIT).read_until(b'\n', &mut line).map_err(|error| journal_error(&error))?;
    let Found::Journal(version) = header_of(&line) else {
        return Ok(());
    };
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line).map_err(|error| journal_error(&error))? == 0 {
            return Ok(());
        }
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        match parse(&line, version, format) {
            Some(record) => into.add(record),
            None => into.skipped = into.skipped.saturating_add(1),
        }
    }
}

/// One line as a record of the current version: an older one goes through the migrations first; one of a newer
/// version is read as far as this build understands it.
fn parse(line: &[u8], version: FormatVersion, format: Format) -> Option<Line> {
    if version >= format.version {
        return serde_json::from_slice(line).ok();
    }
    let value = serde_json::from_slice(line).ok()?;
    let migrated = migrate(value, version, format.version, format.migrations).ok()?;
    serde_json::from_value(migrated).ok()
}

/// The cleanups the records add up to, in the order they began.
#[derive(Debug, Default)]
pub(super) struct Collected {
    /// Only this cleanup's records are kept; `None` keeps every cleanup.
    only: Option<CleanupId>,
    pub(super) cleanups: Vec<RecordedCleanup>,
    places: HashMap<u32, usize>,
    /// The highest cleanup id in any record read.
    pub(super) highest: u32,
    /// Lines that did not parse or held a value the core refuses.
    pub(super) skipped: u32,
}

impl Collected {
    pub(super) fn only(id: CleanupId) -> Self {
        Self { only: Some(id), ..Self::default() }
    }

    fn add(&mut self, line: Line) {
        let id = line.cleanup();
        self.highest = self.highest.max(id);
        if self.only.is_some_and(|only| only.get() != id) {
            return;
        }
        if let Line::Begin(begin) = line {
            match begin.into_record() {
                // A second begin of the same id is not expected; the first one stands.
                Ok(record) if !self.places.contains_key(&id) => {
                    self.places.insert(id, self.cleanups.len());
                    self.cleanups.push(RecordedCleanup {
                        cleanup: CleanupId::new(id),
                        begin: record,
                        items: Vec::new(),
                        end: None,
                        put_backs: Vec::new(),
                    });
                }
                Ok(_) => {}
                Err(_) => self.skipped = self.skipped.saturating_add(1),
            }
            return;
        }
        // A record of a cleanup whose begin is not here (its generation was dropped) has nothing to belong to.
        let Some(cleanup) = self.places.get(&id).and_then(|&place| self.cleanups.get_mut(place)) else {
            return;
        };
        match line {
            Line::Item(item) => cleanup.items.push(item.into_record()),
            Line::End(end) => cleanup.end = Some(end.into_record()),
            Line::PutBack(put_back) => cleanup.put_backs.push(put_back.into_record()),
            Line::Begin(_) => {}
        }
    }
}
