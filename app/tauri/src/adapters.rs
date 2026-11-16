//! The app's own implementations of the core's ports, until the crates of later plans provide the real ones.

use std::time::SystemTime;

use fresher_core::{Access, AccessProbe, AccessStatus, Clock};

/// The wall clock of the system.
#[derive(Debug)]
pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// Answers `Unknown` for every permission, until plan 006 asks macOS (`docs/hacks/`).
#[derive(Debug)]
pub(crate) struct UnknownAccess;

impl AccessProbe for UnknownAccess {
    fn status(&self) -> AccessStatus {
        AccessStatus { full_disk_access: Access::Unknown }
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::time::SystemTime;

        use fresher_core::{Access, AccessProbe, Clock};

        use crate::adapters::{SystemClock, UnknownAccess};

        #[test]
        fn the_placeholders_know_nothing_and_the_clock_runs() {
            assert_eq!(UnknownAccess.status().full_disk_access, Access::Unknown);
            assert!(SystemClock.now() > SystemTime::UNIX_EPOCH);
        }
    }
}
