//! A flow home's scans and what they found. Pulled in with `#[path]`, beside `home.rs` and `events.rs`, only by the
//! homes that use it.

use fresher_core::{Event, FindingOrder, FindingsQuery, Subject};
use fresher_engine::ScanMode;

use super::events::until;
use super::home::Flow;

impl Flow {
    /// Scans in full and gives the events up to the end.
    pub(crate) fn scan(&self) -> Vec<Event> {
        self.walk(ScanMode::Full)
    }

    /// Shows the last scan again without reading the disk, and gives the events up to the end.
    pub(crate) fn restore(&self) -> Vec<Event> {
        let events = self.engine.subscribe();
        self.engine.restore().unwrap();
        until(&events, |event| matches!(event, Event::ScanFinished { .. }))
    }

    /// Scans in `mode` and gives the events up to the end.
    pub(crate) fn walk(&self, mode: ScanMode) -> Vec<Event> {
        let events = self.engine.subscribe();
        self.engine.start_scan(mode).unwrap();
        until(&events, |event| matches!(event, Event::ScanFinished { .. }))
    }

    /// The findings the engine answers: category and path relative to the sandbox, sorted.
    pub(crate) fn findings(&self) -> Vec<(String, String)> {
        let page = self.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let mut found: Vec<(String, String)> = page
            .findings
            .iter()
            .map(|finding| {
                let finding = finding.finding();
                let subject = match finding.subject() {
                    Subject::Path(path) => self.relative_path(path),
                    Subject::External { id, .. } => id.clone(),
                };
                (finding.category().as_str().to_owned(), subject)
            })
            .collect();
        found.sort();
        found
    }
}
