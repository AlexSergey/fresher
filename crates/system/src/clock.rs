//! `SystemClock`: the wall clock of the system; `StartedClock`: a clock that starts at a given moment and runs as the
//! system's.

use std::time::{Instant, SystemTime};

use fresher_core::Clock;

/// The wall clock of the system.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// A clock that reads `start` when made and runs from there at the system's pace: a sandbox's ages, counted back from
/// the moment it was generated, stay the same however long ago that was, and the time still moves for whatever
/// measures it (progress, durations).
#[derive(Clone, Copy, Debug)]
pub struct StartedClock {
    start: SystemTime,
    made: Instant,
}

impl StartedClock {
    pub fn at(start: SystemTime) -> Self {
        Self { start, made: Instant::now() }
    }
}

impl Clock for StartedClock {
    fn now(&self) -> SystemTime {
        self.start + self.made.elapsed()
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::time::{Duration, SystemTime};

        use fresher_core::Clock;

        use crate::{StartedClock, SystemClock};

        #[test]
        fn the_clock_runs() {
            assert!(SystemClock.now() > SystemTime::UNIX_EPOCH);
        }

        #[test]
        fn a_started_clock_starts_at_its_moment_and_never_goes_back() {
            let start = SystemTime::UNIX_EPOCH + Duration::from_secs(fresher_testing::FIXTURE_NOW);
            let clock = StartedClock::at(start);

            let first = clock.now();
            let second = clock.now();

            assert!(first >= start && first < start + Duration::from_secs(60), "{first:?}");
            assert!(second >= first);
        }
    }
}
