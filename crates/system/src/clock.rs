//! `SystemClock`: the wall clock of the system.

use std::time::SystemTime;

use fresher_core::Clock;

/// The wall clock of the system.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::time::SystemTime;

        use fresher_core::Clock;

        use crate::SystemClock;

        #[test]
        fn the_clock_runs() {
            assert!(SystemClock.now() > SystemTime::UNIX_EPOCH);
        }
    }
}
