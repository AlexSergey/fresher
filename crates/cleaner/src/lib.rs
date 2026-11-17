//! The only code of Fresher that removes anything from the disk.
//!
//! [`DiskCleaner`] implements the core's `Cleaner` port: it checks each item of a plan right before it is acted on
//! (the guard's eleven checks, failing closed), moves it to the Trash, empties it, deletes it permanently or runs its
//! tool — only for a plan the user confirmed — and puts items back from the Trash. In a development build or a test it
//! acts only below a sandbox folder ([`Reach`]). How it works and why: `docs/architecture/removal.md`.

#![forbid(unsafe_code)]

mod beneath;
mod guard;
mod paths;
mod pipeline;
mod put_back;
mod reach;
mod remove;
#[cfg(test)]
mod test_support;

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use fresher_core::{
    CancelToken, Cleaner, CleanupContext, Clock, CommandRunner, ConfirmedPlan, Depth, Ended, ItemDone, OpenFiles,
    PlanItem, Protection, PutBackOutcome, Rehearsal, RunningApps, Trash, TrashState, TrashedItem,
};

use crate::pipeline::Cleaning;

pub use guard::Verified;
pub use reach::{Reach, RealDiskToken, SandboxError, SandboxFolder};

/// The ports the cleaner acts through. The app gives the system's Trash to the cleaner and to nothing else.
pub struct CleanerPorts {
    pub trash: Arc<dyn Trash>,
    pub open_files: Arc<dyn OpenFiles>,
    /// The running apps: an item a running app owns is skipped as in use.
    pub running: Arc<dyn RunningApps>,
    pub commands: Arc<dyn CommandRunner>,
    pub clock: Arc<dyn Clock>,
}

impl fmt::Debug for CleanerPorts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("CleanerPorts").finish_non_exhaustive()
    }
}

/// The cleaner of the app: the guard, the removals and put back, on the disk through `std`, the held-folder calls of
/// `fresher-macos` and its ports.
#[derive(Debug)]
pub struct DiskCleaner {
    reach: Reach,
    user: u32,
    home: PathBuf,
    protection: Protection,
    ports: CleanerPorts,
}

impl DiskCleaner {
    /// A cleaner acting as `user` (the uid the app runs as) within `reach`. `home` is the user's home folder, against
    /// which a root written `~` or `~/…` is read. `protection` is the system's list of protected places: it is always
    /// checked besides the context's own, and it decides where a sandbox marker counts. In sandbox mode
    /// ([`Reach::SandboxFolder`]) it is the real Mac's list joined with the sandbox home's
    /// (`Protection::joined`), so both apply.
    pub const fn new(reach: Reach, user: u32, home: PathBuf, protection: Protection, ports: CleanerPorts) -> Self {
        Self { reach, user, home, protection, ports }
    }

    fn cleaning(&self) -> Cleaning<'_> {
        Cleaning { reach: &self.reach, user: self.user, home: &self.home, system: &self.protection, ports: &self.ports }
    }
}

impl Cleaner for DiskCleaner {
    fn rehearse(&self, roots: &[PathBuf], items: &[PlanItem], cx: &CleanupContext<'_>, depth: Depth) -> Vec<Rehearsal> {
        pipeline::rehearse(&self.cleaning(), roots, items, cx, depth)
    }

    fn execute(
        &self,
        plan: ConfirmedPlan,
        cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended {
        pipeline::execute(&self.cleaning(), &plan, cx, cancel, out)
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        put_back::locate(items)
    }

    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
        put_back::put_back(&self.cleaning(), item, cx)
    }
}
