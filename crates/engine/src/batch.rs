//! Findings and a cleanup's outcomes go out in batches, so neither floods the UI: at 256, when 50 ms have passed on
//! the clock since the batch opened, and when a module or the cleanup finishes.

use std::mem;
use std::time::{Duration, SystemTime};

/// The most findings or outcomes in one batch.
pub(crate) const BATCH_LIMIT: usize = 256;
/// The longest a finding waits in an open batch, on the `Clock` port.
pub(crate) const BATCH_INTERVAL: Duration = Duration::from_millis(50);

/// Collects findings or outcomes until a batch is due.
#[derive(Debug)]
pub(crate) struct Batcher<T> {
    pending: Vec<T>,
    opened: Option<SystemTime>,
}

impl<T> Default for Batcher<T> {
    fn default() -> Self {
        Self { pending: Vec::new(), opened: None }
    }
}

impl<T> Batcher<T> {
    /// Adds `item` at `now`; gives the batch back when it is full or old enough to send.
    pub(crate) fn push(&mut self, item: T, now: SystemTime) -> Option<Vec<T>> {
        let opened = *self.opened.get_or_insert(now);
        self.pending.push(item);
        // A clock that went back counts as time passed: the batch goes out rather than waits.
        let due = now.duration_since(opened).map_or(true, |waited| waited >= BATCH_INTERVAL);
        if due || self.pending.len() >= BATCH_LIMIT { self.take() } else { None }
    }

    /// The open batch, if it holds anything.
    pub(crate) fn take(&mut self) -> Option<Vec<T>> {
        self.opened = None;
        if self.pending.is_empty() { None } else { Some(mem::take(&mut self.pending)) }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use fresher_core::{DangerLevel, Finding, FindingId, Removability, Sizes};

    use crate::testing::{category, finding};

    fn numbered(id: u32) -> Finding {
        let caches = category("alpha.caches", DangerLevel::Safe);
        Finding::new(FindingId::new(id), finding(&caches, format!("/demo/{id}"), Sizes::ZERO), Removability::Removable)
    }

    fn at(millis: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_millis(millis)
    }

    mod negative {
        use super::{at, numbered};
        use crate::batch::{BATCH_LIMIT, Batcher};

        #[test]
        fn holds_findings_while_the_clock_stands_still() {
            let mut batcher = Batcher::default();

            for id in 1..u32::try_from(BATCH_LIMIT).unwrap() {
                assert_eq!(batcher.push(numbered(id), at(0)), None);
            }
        }

        #[test]
        fn an_empty_batch_is_not_sent() {
            assert_eq!(Batcher::<u32>::default().take(), None);
        }
    }

    mod positive {
        use super::{at, numbered};
        use crate::batch::{BATCH_LIMIT, Batcher};

        #[test]
        fn sends_a_full_batch() {
            let mut batcher = Batcher::default();
            let limit = u32::try_from(BATCH_LIMIT).unwrap();
            for id in 1..limit {
                batcher.push(numbered(id), at(0));
            }

            let batch = batcher.push(numbered(limit), at(0)).unwrap();

            assert_eq!(batch.len(), BATCH_LIMIT);
            assert_eq!(batcher.take(), None);
        }

        #[test]
        fn sends_a_batch_50_ms_after_it_opened() {
            let mut batcher = Batcher::default();
            batcher.push(numbered(1), at(100));

            assert_eq!(batcher.push(numbered(2), at(149)), None);
            assert_eq!(batcher.push(numbered(3), at(150)), Some(vec![numbered(1), numbered(2), numbered(3)]));
        }

        #[test]
        fn sends_at_once_when_the_clock_went_back() {
            let mut batcher = Batcher::default();
            batcher.push(numbered(1), at(100));

            assert_eq!(batcher.push(numbered(2), at(50)), Some(vec![numbered(1), numbered(2)]));
        }

        #[test]
        fn take_empties_the_open_batch() {
            let mut batcher = Batcher::default();
            batcher.push(numbered(1), at(0));

            assert_eq!(batcher.take(), Some(vec![numbered(1)]));
            assert_eq!(batcher.take(), None);
        }
    }
}
