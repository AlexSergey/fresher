//! App Management, as the engine learns it for the session: macOS guards other developers' app bundles, and no call
//! tells the permission's state without trying a move. It is never probed: `NotAsked` until the first uninstall,
//! then what the move of the bundle did. Nothing here moves, retries or asks anything.

use std::path::Path;
use std::sync::Mutex;

use fresher_core::{Access, Failure, ItemOutcome, KnownFolder};

use crate::engine::Shared;
use crate::error::lock_anyway;

/// Whether App Management guards `bundle`: not when it lies in the user's own apps folder, which is the user's to
/// change without it.
pub(crate) fn guards(shared: &Shared, bundle: &Path) -> bool {
    let own = shared.ports.system.known_folders.locate(KnownFolder::UserApplications);
    !own.is_some_and(|folder| bundle.starts_with(folder))
}

/// The state of App Management for the session.
#[derive(Debug)]
pub(crate) struct Management(Mutex<Access>);

impl Default for Management {
    fn default() -> Self {
        Self(Mutex::new(Access::NotAsked))
    }
}

impl Management {
    pub(crate) fn access(&self) -> Access {
        *lock_anyway(&self.0)
    }

    /// Learns from how an uninstall's item ended, the bundle its first path, `went` of its parts gone: the system
    /// refused to move it to the Trash → `Denied` (the bundle was the user's and its folder writable, so only the
    /// system's guard refuses); it moved, alone or with only some of the other parts → `Granted`; any other outcome
    /// teaches nothing. `true` when the state changed.
    pub(crate) fn learn(&self, outcome: &ItemOutcome, went: Option<u32>) -> bool {
        let learned = match outcome {
            ItemOutcome::Moved => Access::Granted,
            ItemOutcome::Failed { reason } if *reason == Failure::TrashRefused.text() => Access::Denied,
            ItemOutcome::Failed { .. } if went.is_some_and(|went| went >= 1) => Access::Granted,
            _ => return false,
        };
        let mut access = lock_anyway(&self.0);
        let changed = *access != learned;
        *access = learned;
        changed
    }
}

#[cfg(test)]
#[path = "management_tests.rs"]
mod tests;
