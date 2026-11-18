//! The engine's events a flow waits for. Pulled in with `#[path]` only by the tests and homes that use it.

use std::sync::mpsc::Receiver;
use std::time::Duration;

use fresher_core::Event;

/// Every event until `is_last`; fails after 30 s instead of hanging.
pub(crate) fn until(events: &Receiver<Event>, is_last: impl Fn(&Event) -> bool) -> Vec<Event> {
    let mut seen = Vec::new();
    loop {
        let event = events.recv_timeout(Duration::from_secs(30)).unwrap();
        let last = is_last(&event);
        seen.push(event);
        if last {
            return seen;
        }
    }
}
