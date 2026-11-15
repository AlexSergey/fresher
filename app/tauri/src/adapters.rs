//! The app's own implementations of the core's ports, until the crates of later plans provide the real ones.

use std::path::PathBuf;
use std::time::SystemTime;

use fresher_core::{
    Access, AccessProbe, AccessStatus, CancelToken, Clock, DiskWalk, InterestSet, Match, WalkError, WalkProgress,
    WalkSummary,
};

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

/// A walk that finds nothing, until plan 004 builds the real one (`docs/hacks/`).
#[derive(Debug)]
pub(crate) struct NoWalk;

impl DiskWalk for NoWalk {
    fn walk(
        &self,
        _roots: &[PathBuf],
        _interests: &InterestSet,
        _cancel: &CancelToken,
        _on_match: &mut dyn FnMut(Match),
        _on_progress: &mut dyn FnMut(WalkProgress),
    ) -> Result<WalkSummary, WalkError> {
        Ok(WalkSummary::default())
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::time::SystemTime;

        use fresher_core::{Access, AccessProbe, CancelToken, Clock, DiskWalk, InterestSet, WalkSummary};

        use crate::adapters::{NoWalk, SystemClock, UnknownAccess};

        #[test]
        fn the_placeholders_find_nothing_and_know_nothing() {
            let mut matches = 0;
            let summary = NoWalk.walk(
                &["~".into()],
                &InterestSet::new(),
                &CancelToken::new(),
                &mut |_| matches += 1,
                &mut |_| {},
            );

            assert_eq!(summary, Ok(WalkSummary::default()));
            assert_eq!(matches, 0);
            assert_eq!(UnknownAccess.status().full_disk_access, Access::Unknown);
            assert!(SystemClock.now() > SystemTime::UNIX_EPOCH);
        }
    }
}
