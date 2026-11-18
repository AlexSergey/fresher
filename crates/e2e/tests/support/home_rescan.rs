//! A flow home's rescans: the change journal a home scripts, and a scan that must be a rescan. Pulled in with
//! `#[path]`, beside `home.rs`, `home_engine.rs` and `home_scan.rs`, only by the homes that rescan.

use fresher_core::{Changes, Event, JournalPosition, ScanKind};
use fresher_engine::ScanMode;
use fresher_sandbox::Sandbox;
use fresher_testing::FakeChangeJournal;

use super::home::Flow;
use super::home_engine::VOLUME;

/// Where the journal of the one volume stands; every answer ends there too.
const AT: JournalPosition = JournalPosition { journal: 1, cursor: 10 };

/// The journal that answers, at each rescan in order, that the folders `changed` (relative to `sandbox`) changed. A
/// rescan past the last answer meets a gap, and the scan reads every folder.
pub(crate) fn journal(sandbox: &Sandbox, changed: &[Vec<&str>]) -> FakeChangeJournal {
    changed.iter().fold(FakeChangeJournal::default().at(VOLUME, AT), |journal, changed| {
        let folders = changed.iter().map(|relative| sandbox.path().join(relative)).collect();
        journal.answering(Ok(Changes { folders, subtrees: Vec::new(), end: AT }))
    })
}

impl Flow {
    /// Scans again, reading only what the journal names, and gives the events up to the end; fails when the scan
    /// was not a rescan.
    pub(crate) fn rescan(&self) -> Vec<Event> {
        let events = self.walk(ScanMode::Auto);
        let kind = events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        });
        assert!(matches!(kind, Some(ScanKind::Rescan { .. })), "{kind:?}");
        events
    }
}
