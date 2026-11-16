//! What the cleaner checks against, as it stands now: the scan roots in the settings, the protection with the user's
//! ignored paths, the access status probed again, and the guarded places under the roots.

use std::path::PathBuf;

use fresher_core::{AccessStatus, BlockedReason, CleanupContext, CommandError, GuardedPlace, Protection};

use crate::engine::Shared;
use crate::error::lock;

/// The owned parts of a [`CleanupContext`].
#[derive(Debug)]
pub(crate) struct Context {
    roots: Vec<PathBuf>,
    protection: Protection,
    access: AccessStatus,
    guarded: Vec<GuardedPlace>,
}

impl Context {
    /// The context now; the access is probed again first (`AccessChanged` when it changed). The protection refuses
    /// the user's ignored paths, what is inside them and what holds them; the cleaner keeps the protected places it
    /// was built with. An ignore list the protection refuses blocks the cleanup: it fails closed.
    pub(crate) fn now(shared: &Shared) -> Result<Self, CommandError> {
        let (roots, ignored) = {
            let settings = lock(&shared.settings)?;
            (settings.scan_roots(), settings.ignored().paths)
        };
        let protection = Protection::default().with_ignored(&ignored).map_err(|error| {
            tracing::warn!(%error, "the ignored paths cannot protect anything");
            CommandError::CleaningBlocked { reason: BlockedReason::Settings }
        })?;
        shared.refresh_access();
        let access = shared.access.status();
        let guarded = shared.ports.access.guarded_places(&roots);
        Ok(Self { roots, protection, access, guarded })
    }

    pub(crate) fn borrow(&self) -> CleanupContext<'_> {
        CleanupContext {
            roots: &self.roots,
            protection: &self.protection,
            access: &self.access,
            guarded: &self.guarded,
        }
    }
}
