//! Whether a folder of `Caches` is offered as an installed app's cache, kept or not claimed: the rows of the design's
//! section 4, in their order, on plain values. Nothing here reads the disk or asks the system but through `running`.

use fresher_core::{BundleId, EntryKind, InstalledApp, RunningSet};

use crate::exclude::Exclude;
use crate::gathered::Candidate;
use crate::owner::{Owner, id_of, installed, owner};

/// One of the system's lists (the installed or the running apps) could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoSystem;

/// Why a cache the module recognized is kept (design, section 5), in the order the module declares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Special,
    NotYours,
    NoSystem,
    AppOpen,
}

impl Reason {
    pub(crate) const ALL: [Self; 4] = [Self::Special, Self::NotYours, Self::NoSystem, Self::AppOpen];

    /// The key of the reason's sentence.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Special => "app-caches.left-out.special",
            Self::NotYours => "app-caches.left-out.not-yours",
            Self::NoSystem => "app-caches.left-out.no-system",
            Self::AppOpen => "app-caches.left-out.app-open",
        }
    }
}

/// What becomes of one folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict<'a> {
    /// Not recognized as an installed app's cache: nothing is said.
    NotClaimed,
    /// Recognized, kept, and reported as left out with the reason.
    LeftOut(Reason),
    /// Offered as a cache of the owner's app.
    Offered(Owner<'a>),
}

/// How a candidate names its app.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Naming<'t> {
    /// By the folder's own name, a bundle id (a row of `places.toml`).
    FolderName,
    /// By the bundle id of its row of `apps.toml`; the folder's name says nothing of its app.
    Table(&'t BundleId),
}

/// The first row that applies to `candidate`, its app named by `naming`:
///
/// 1. not a folder (a file, a link, a socket): not claimed;
/// 2. `FolderName`: the name gives no id, or `exclude` keeps the id; `Table(id)`: `exclude` keeps `id`: not claimed;
/// 3. the installed apps cannot be listed: not claimed, and nothing said;
/// 4. `FolderName`: no installed app names the folder (`owner`); `Table(id)`: no installed app has `id`
///    (`installed`): not claimed;
/// 5. not read in full: not claimed;
/// 6. nothing in it (logical size 0): not claimed;
/// 7. a socket, a pipe or a device inside: `Special`;
/// 8. not the user's, a system flag, restricted, the user's lock, or such an entry inside: `NotYours`;
/// 9. the running apps cannot be listed: `NoSystem`;
/// 10. a running app has the owner's bundle id or a helper's, or one of the path's names is a running app's: `AppOpen`;
/// 11. offered.
///
/// `running` is called only when a candidate reaches row 9. No time is judged: the verdict is the same whenever it is
/// asked.
pub(crate) fn decide<'a, 'r>(
    candidate: &Candidate,
    naming: Naming<'_>,
    exclude: &Exclude,
    apps: Result<&'a [InstalledApp], NoSystem>,
    running: impl FnOnce() -> Result<&'r RunningSet, NoSystem>,
) -> Verdict<'a> {
    if candidate.kind != EntryKind::Folder {
        return Verdict::NotClaimed;
    }
    let named = match naming {
        Naming::FolderName => candidate.path.file_name().and_then(id_of),
        Naming::Table(id) => Some(id.clone()),
    };
    let Some(id) = named else { return Verdict::NotClaimed };
    if exclude.keeps(&id) {
        return Verdict::NotClaimed;
    }
    let Ok(apps) = apps else { return Verdict::NotClaimed };
    let found = match naming {
        Naming::FolderName => owner(id.as_str(), apps),
        Naming::Table(_) => installed(&id, apps),
    };
    let Some(owner) = found else { return Verdict::NotClaimed };
    let Some(inside) = candidate.inside.filter(|inside| inside.complete) else { return Verdict::NotClaimed };
    if candidate.sizes.logical.bytes() == 0 {
        return Verdict::NotClaimed;
    }
    if inside.special {
        return Verdict::LeftOut(Reason::Special);
    }
    let facts = candidate.facts;
    if !facts.owned_by_user || facts.system_immutable || facts.restricted || facts.user_immutable || inside.foreign {
        return Verdict::LeftOut(Reason::NotYours);
    }
    let Ok(running) = running() else { return Verdict::LeftOut(Reason::NoSystem) };
    let runs = running.apps().iter().any(|open| open.bundle_ids.iter().any(|id| owner.app.owns(id)));
    if runs || running.owner_of(&candidate.path).is_some() {
        return Verdict::LeftOut(Reason::AppOpen);
    }
    Verdict::Offered(owner)
}

#[cfg(test)]
mod tests;
