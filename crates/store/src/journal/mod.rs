//! `cleanups.jsonl`: the core's `CleanupJournal` on an append-only file in the data folder.
//!
//! A header line, then one record per line, appended as the cleanup goes: the `begin` record with every item, synced
//! before the first item is touched; each item's outcome; the `end`; and every put back later. Nothing is rewritten
//! and the store deletes nothing: past the threshold the file becomes `cleanups.previous.jsonl` by a rename. The file holds the paths of the user's own items, for put back: it stays private to
//! the user and no path is ever logged.

mod file;
mod path_text;
mod read;
mod records;
mod rotate;

use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use fresher_core::{
    BeginRecord, CleanupId, CleanupJournal, EndRecord, ItemRecord, JournalError, JournalPage, JournalProblem,
    JournalState, PutBackRecord, RecordedCleanup, StoreError,
};
use serde_json::Value;

use self::file::{Appender, Found, inspect, journal_error};
use self::read::{Collected, Format, read_file};
use self::records::{BeginLine, EndLine, ItemLine, Line, PutBackLine, VERSION, to_line};
use self::rotate::{JOURNAL_GENERATION, PREVIOUS_FILE_NAME};
use crate::DataFolder;
use crate::migrate::Migration;

/// The current generation of the journal.
const FILE_NAME: &str = "cleanups.jsonl";
/// Where a file that is not a journal is kept, renamed, before the journal is written again.
const BROKEN_FILE_NAME: &str = "cleanups.broken.jsonl";
/// The migrations of the journal's records, one per version step.
const MIGRATIONS: &[Migration] = &[items_of_several_paths];

/// Version 1 → 2, for items of several paths, which version 1 did not have: every item of a `begin` gets no parts
/// after its subject (`more: []`), an `item` no count of parts that went (`went: null`), a `put-back` no parts brought
/// back (`back: []`). Any other line is unchanged.
fn items_of_several_paths(mut line: Value) -> Result<Value, StoreError> {
    let record = line.as_object_mut().ok_or(StoreError::Broken)?;
    match record.get("type").and_then(Value::as_str) {
        Some("begin") => {
            let items = record.get_mut("items").and_then(Value::as_array_mut).ok_or(StoreError::Broken)?;
            for item in items {
                item.as_object_mut().ok_or(StoreError::Broken)?.insert("more".into(), Value::Array(Vec::new()));
            }
        }
        Some("item") => {
            record.insert("went".into(), Value::Null);
        }
        Some("put-back") => {
            record.insert("back".into(), Value::Array(Vec::new()));
        }
        _ => {}
    }
    Ok(line)
}

/// The cleanup journal in the data folder.
///
/// It never fails to open: a file it must not write — from a newer version of the app, or one it cannot read — makes
/// it read-only, and the engine refuses a cleanup until it is `Ready`.
#[derive(Debug)]
pub struct FileCleanupJournal {
    folder: DataFolder,
    format: Format,
    /// The size at which a `begin` starts a new generation.
    generation: u64,
    inner: Mutex<Inner>,
    /// How many times a record was synced: lets the tests see that `begin` is durable before it returns.
    #[cfg(test)]
    syncs: std::sync::atomic::AtomicU32,
}

#[derive(Debug)]
struct Inner {
    /// What the current file was when it was last looked at.
    found: Found,
    /// The id the next `begin` takes.
    next: u32,
    /// Open while records are being appended; dropped after a failed write, so the next one starts on a new line.
    appender: Option<Appender>,
}

impl FileCleanupJournal {
    /// The journal in `folder`, at the version this build writes.
    pub fn open(folder: &DataFolder) -> Self {
        Self::with_format(folder, Format { version: VERSION, migrations: MIGRATIONS }, JOURNAL_GENERATION)
    }

    fn with_format(folder: &DataFolder, format: Format, generation: u64) -> Self {
        let journal = Self {
            folder: folder.clone(),
            format,
            generation,
            inner: Mutex::new(Inner { found: Found::Missing, next: 1, appender: None }),
            #[cfg(test)]
            syncs: std::sync::atomic::AtomicU32::new(0),
        };
        let found = journal.path(FILE_NAME).map_or(Found::Unreadable, |path| inspect(&path));
        let mut history = Collected::default();
        // An id is never taken twice from what can be read: each generation is read on its own, so one that cannot
        // be read does not hide the other; a current file that cannot be read is never written.
        for name in [PREVIOUS_FILE_NAME, FILE_NAME] {
            let read = journal.path(name).and_then(|path| read_file(&path, format, &mut history));
            if read.is_err() {
                tracing::warn!("a cleanup journal file could not be read to find the next id");
            }
        }
        let state = state_of(found, format);
        if let JournalState::ReadOnly { why } = state {
            tracing::warn!(?why, "the cleanup journal is read-only");
        }
        let mut inner = journal.lock();
        inner.found = found;
        inner.next = history.highest.saturating_add(1);
        drop(inner);
        journal
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn path(&self, name: &str) -> Result<PathBuf, JournalError> {
        self.folder.file(name).map_err(|error| JournalError::Failed { reason: error.to_string() })
    }

    /// Reads every generation, oldest first, into `into`.
    fn read_all(&self, into: &mut Collected) -> Result<(), JournalError> {
        read_file(&self.path(PREVIOUS_FILE_NAME)?, self.format, into)?;
        read_file(&self.path(FILE_NAME)?, self.format, into)
    }

    /// Reads every generation while no record is being written, so no line is read half-written.
    fn read_while_locked(&self, into: &mut Collected) -> Result<(), JournalError> {
        let _writing = self.lock();
        self.read_all(into)
    }

    /// The open appender, after the checks that come before any write.
    fn appender<'inner>(&self, inner: &'inner mut Inner) -> Result<&'inner mut Appender, JournalError> {
        if state_of(inner.found, self.format) != JournalState::Ready {
            return Err(JournalError::ReadOnly);
        }
        if inner.appender.is_none() {
            match inner.found {
                Found::Broken => {
                    self.rename(FILE_NAME, BROKEN_FILE_NAME)?;
                    inner.found = Found::Missing;
                }
                // A file never mixes two versions: an older one becomes the previous generation.
                Found::Journal(version) if version < self.format.version => self.start_generation(inner)?,
                Found::Missing | Found::Journal(_) | Found::Unreadable => {}
            }
        }
        match &mut inner.appender {
            Some(appender) => Ok(appender),
            slot @ None => {
                let appender = Appender::open(&self.path(FILE_NAME)?, self.format.version)?;
                inner.found = Found::Journal(self.format.version);
                Ok(slot.insert(appender))
            }
        }
    }

    /// Renames the store's file `from` over `to`: a rename, never a delete.
    fn rename(&self, from: &str, to: &str) -> Result<(), JournalError> {
        fs::rename(self.path(from)?, self.path(to)?).map_err(|error| journal_error(&error))
    }

    /// Appends `line`, synced when `sync` says so. A failed write closes the appender: the next write reopens the
    /// file and starts on a new line.
    fn append(&self, inner: &mut Inner, line: &[u8], sync: bool) -> Result<(), JournalError> {
        let result = self.appender(inner).and_then(|appender| {
            appender.append(line)?;
            if sync { self.sync_appender(appender) } else { Ok(()) }
        });
        if result.is_err() {
            inner.appender = None;
        }
        result
    }

    #[cfg_attr(not(test), expect(clippy::unused_self, reason = "the tests count the syncs on the journal"))]
    fn sync_appender(&self, appender: &Appender) -> Result<(), JournalError> {
        #[cfg(test)]
        self.syncs.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        appender.sync()
    }
}

/// Whether a journal whose current file is `found` may be written.
fn state_of(found: Found, format: Format) -> JournalState {
    match found {
        Found::Unreadable => JournalState::ReadOnly { why: JournalProblem::Unreadable },
        Found::Journal(version) if version > format.version => {
            JournalState::ReadOnly { why: JournalProblem::NewerVersion }
        }
        Found::Missing | Found::Journal(_) | Found::Broken => JournalState::Ready,
    }
}

impl CleanupJournal for FileCleanupJournal {
    fn state(&self) -> JournalState {
        state_of(self.lock().found, self.format)
    }

    fn begin(&self, record: &BeginRecord) -> Result<CleanupId, JournalError> {
        let mut inner = self.lock();
        let id = CleanupId::new(inner.next);
        let line = to_line(&Line::Begin(BeginLine::new(id, record)))?;
        self.appender(&mut inner)?;
        self.rotate_if_due(&mut inner)?;
        // The id is spent even when the write fails: a begin that reached the disk is never followed by another of
        // the same id.
        inner.next = inner.next.checked_add(1).ok_or(JournalError::Failed { reason: "no ids left".into() })?;
        self.append(&mut inner, &line, true)?;
        Ok(id)
    }

    fn item(&self, record: &ItemRecord) -> Result<(), JournalError> {
        let line = to_line(&Line::Item(ItemLine::new(record)))?;
        self.append(&mut self.lock(), &line, false)
    }

    fn sync(&self) -> Result<(), JournalError> {
        let mut inner = self.lock();
        let result = self.appender(&mut inner).and_then(|appender| self.sync_appender(appender));
        if result.is_err() {
            inner.appender = None;
        }
        result
    }

    fn end(&self, record: &EndRecord) -> Result<(), JournalError> {
        let line = to_line(&Line::End(EndLine::new(record)))?;
        self.append(&mut self.lock(), &line, true)
    }

    fn put_back(&self, record: &PutBackRecord) -> Result<(), JournalError> {
        let line = to_line(&Line::PutBack(PutBackLine::new(record)))?;
        self.append(&mut self.lock(), &line, true)
    }

    fn cleanups(&self, offset: u32, limit: u32) -> Result<JournalPage, JournalError> {
        let mut history = Collected::default();
        self.read_while_locked(&mut history)?;
        let total = u32::try_from(history.cleanups.len()).unwrap_or(u32::MAX);
        let page = history
            .cleanups
            .iter()
            .rev()
            .skip(usize::try_from(offset).unwrap_or(usize::MAX))
            .take(usize::try_from(limit).unwrap_or(usize::MAX))
            .map(RecordedCleanup::summary)
            .collect();
        Ok(JournalPage { cleanups: page, total, skipped: history.skipped })
    }

    fn cleanup(&self, id: CleanupId) -> Result<Option<RecordedCleanup>, JournalError> {
        let mut history = Collected::only(id);
        self.read_while_locked(&mut history)?;
        Ok(history.cleanups.into_iter().next())
    }
}

#[cfg(test)]
mod tests;
