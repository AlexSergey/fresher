//! `HistoryStream`: one replay of the change journal, from a start to `HistoryDone`, then released.

use std::path::PathBuf;
use std::sync::PoisonError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use fresher_core::{CancelToken, JournalGap};

use crate::ffi::c_path;
use crate::ffi::fsevents::{Event, Stream};

/// How often a wait for the history checks the cancel token.
const POLL: Duration = Duration::from_millis(50);

/// The streams alive now; every one is released by the end of the call that made it.
static LIVE: AtomicUsize = AtomicUsize::new(0);

/// A started stream replaying the changes under some paths since an event id.
#[derive(Debug)]
pub(super) struct HistoryStream {
    stream: Stream,
}

impl HistoryStream {
    pub(super) fn start(paths: &[PathBuf], since: u64) -> Result<Self, JournalGap> {
        let paths = paths.iter().map(|path| c_path(path)).collect::<Result<Vec<_>, _>>().map_err(|error| {
            tracing::debug!(%error, "a watched path is not a C string");
            JournalGap::Failed
        })?;
        let stream = Stream::start(&paths, since).map_err(|error| {
            tracing::warn!(%error, "the change journal could not be read");
            JournalGap::Failed
        })?;
        LIVE.fetch_add(1, Ordering::Relaxed);
        Ok(Self { stream })
    }

    /// Waits for the end of the history (checking the cancel token every 50 ms, and giving up after `limit`), then
    /// flushes the stream, so a change numbered before the end that arrives late is not missed, then releases the
    /// stream and answers every event received. The flush comes last: before the history is over it blocks for as
    /// long as the replay takes, out of reach of the cancel token.
    pub(super) fn collect(self, cancel: &CancelToken, limit: Duration) -> Result<Vec<Event>, JournalGap> {
        let started = Instant::now();
        let inbox = self.stream.inbox();
        let mut received = inbox.received.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if cancel.is_cancelled() {
                return Err(JournalGap::Cancelled);
            }
            if received.history_done {
                break;
            }
            if started.elapsed() >= limit {
                tracing::warn!(?limit, "the change journal did not finish its history in time");
                return Err(JournalGap::Failed);
            }
            received = inbox.arrived.wait_timeout(received, POLL).unwrap_or_else(PoisonError::into_inner).0;
        }
        drop(received);
        self.stream.flush();
        let mut received = inbox.received.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(std::mem::take(&mut received.events))
    }
}

impl Drop for HistoryStream {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::Relaxed);
    }
}

/// How many streams are alive now.
#[cfg(test)]
pub(super) fn live_streams() -> usize {
    LIVE.load(Ordering::Relaxed)
}
