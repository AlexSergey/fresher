//! Walk progress goes out at most once per 50 ms on the clock, so a fast walk never floods the UI; the last progress
//! held back goes out when the walk ends.

use std::time::SystemTime;

use fresher_core::WalkProgress;

use crate::batch::BATCH_INTERVAL;

/// Lets a progress through when 50 ms have passed since the last one sent; keeps the newest one held back.
#[derive(Debug, Default)]
pub(crate) struct Throttle {
    sent: Option<SystemTime>,
    held: Option<WalkProgress>,
}

impl Throttle {
    /// Offers `progress` at `now`; gives it back when it is due to be sent.
    pub(crate) fn offer(&mut self, progress: WalkProgress, now: SystemTime) -> Option<WalkProgress> {
        // A clock that went back counts as time passed: the progress goes out rather than waits.
        let due = self.sent.is_none_or(|sent| now.duration_since(sent).map_or(true, |waited| waited >= BATCH_INTERVAL));
        if due {
            self.sent = Some(now);
            self.held = None;
            Some(progress)
        } else {
            self.held = Some(progress);
            None
        }
    }

    /// The newest progress held back, if any.
    pub(crate) const fn take(&mut self) -> Option<WalkProgress> {
        self.held.take()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use fresher_core::{ByteSize, WalkProgress};

    fn progress(entries: u64) -> WalkProgress {
        WalkProgress { entries, bytes: ByteSize::new(entries) }
    }

    fn at(millis: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_millis(millis)
    }

    mod negative {
        use super::{at, progress};
        use crate::throttle::Throttle;

        #[test]
        fn holds_progress_back_while_the_clock_stands_still() {
            let mut throttle = Throttle::default();
            throttle.offer(progress(1), at(0));

            for entries in 2..=5 {
                assert_eq!(throttle.offer(progress(entries), at(49)), None);
            }
            assert_eq!(throttle.take(), Some(progress(5)));
        }

        #[test]
        fn nothing_held_back_gives_nothing() {
            let mut throttle = Throttle::default();
            throttle.offer(progress(1), at(0));

            assert_eq!(Throttle::default().take(), None);
            assert_eq!(throttle.take(), None);
        }
    }

    mod positive {
        use super::{at, progress};
        use crate::throttle::Throttle;

        #[test]
        fn sends_the_first_progress_at_once() {
            assert_eq!(Throttle::default().offer(progress(1), at(0)), Some(progress(1)));
        }

        #[test]
        fn sends_a_progress_50_ms_after_the_last_one_sent() {
            let mut throttle = Throttle::default();
            throttle.offer(progress(1), at(100));
            throttle.offer(progress(2), at(120));

            assert_eq!(throttle.offer(progress(3), at(150)), Some(progress(3)));
            assert_eq!(throttle.take(), None);
        }

        #[test]
        fn sends_at_once_when_the_clock_went_back() {
            let mut throttle = Throttle::default();
            throttle.offer(progress(1), at(100));

            assert_eq!(throttle.offer(progress(2), at(50)), Some(progress(2)));
        }
    }
}
