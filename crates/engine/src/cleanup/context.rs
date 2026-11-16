//! What the cleaner checks against, as it stands now: the scan roots in the settings, the protection, the access
//! status probed again, and the guarded places under the roots.

use std::path::PathBuf;

use fresher_core::{AccessStatus, CleanupContext, GuardedPlace, Protection};

use crate::engine::Shared;
use crate::error::{EngineError, lock};

/// The owned parts of a [`CleanupContext`].
#[derive(Debug)]
pub(crate) struct Context {
    roots: Vec<PathBuf>,
    protection: Protection,
    access: AccessStatus,
    guarded: Vec<GuardedPlace>,
}

impl Context {
    /// The context now; the access is probed again first (`AccessChanged` when it changed). The protection holds no
    /// ignored path: the ignore list does not exist yet (plan 005), and the cleaner keeps the protected places it was
    /// built with.
    pub(crate) fn now(shared: &Shared) -> Result<Self, EngineError> {
        let roots = lock(&shared.settings)?.scan_roots();
        shared.refresh_access();
        let access = shared.access.status();
        let guarded = shared.ports.access.guarded_places(&roots);
        Ok(Self { roots, protection: Protection::default(), access, guarded })
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
