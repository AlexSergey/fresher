//! Whether a store's folder is offered as a model, kept or not claimed: the rows of the design's section 4, in their
//! order, on plain values. Nothing here reads the disk.

use std::path::Path;

use fresher_core::{EntryKind, Inside, RunningSet, Timestamp};

use crate::gathered::Candidate;
use crate::name::{self, ModelName};
use crate::stores::{Categories, Store};

/// The running apps could not be listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoSystem;

/// Why a model the module recognized is kept (design, section 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    NotYours,
    Special,
    AppOpen,
    NoSystem,
}

impl Reason {
    /// Every reason, in the order the module declares them.
    pub(crate) const ALL: [Self; 4] = [Self::NotYours, Self::Special, Self::AppOpen, Self::NoSystem];

    /// The key of the reason's sentence.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::NotYours => "local-ai-models.left-out.not-yours",
            Self::Special => "local-ai-models.left-out.special",
            Self::AppOpen => "local-ai-models.left-out.app-open",
            Self::NoSystem => "local-ai-models.left-out.no-system",
        }
    }
}

/// What becomes of one folder of a store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not recognized as one model of the store: nothing is said.
    NotClaimed,
    /// Recognized, kept, and reported as left out with the reason.
    LeftOut(Reason),
    /// Offered in `category`, named `name`, last opened or changed at `seen`.
    Offered { category: &'static str, name: ModelName, seen: Timestamp },
}

/// The first row that applies to `candidate`, a match of `row`'s place:
///
/// 1. not a folder: not claimed;
/// 2. the name does not fit the row's shape, or its kind has no category: not claimed;
/// 3. a name of the row's sign not held: not claimed;
/// 4. the folder not read in full, or no time at all: not claimed;
/// 5. a socket, a pipe or a device inside: `Special`;
/// 6. not the user's, a system flag, restricted, the user's lock, or such an entry inside: `NotYours`;
/// 7. the row names an app and the running apps cannot be listed: `NoSystem`;
/// 8. the row's app runs: `AppOpen`;
/// 9. offered.
///
/// `running` is read only for a row that names an app. The inside's lock names are not read, and no time is judged:
/// the verdict is the same whenever it is asked.
pub(crate) fn decide(candidate: &Candidate, row: &Store, running: Result<&RunningSet, NoSystem>) -> Verdict {
    if candidate.kind != EntryKind::Folder {
        return Verdict::NotClaimed;
    }
    let Some((category, name)) = model(row, &candidate.path) else { return Verdict::NotClaimed };
    if !row.sign.iter().all(|name| candidate.holds.contains(name)) {
        return Verdict::NotClaimed;
    }
    let Some(inside) = candidate.inside.filter(|inside| inside.complete) else { return Verdict::NotClaimed };
    let Some(seen) = last_seen(&inside) else { return Verdict::NotClaimed };
    if inside.special {
        return Verdict::LeftOut(Reason::Special);
    }
    let facts = candidate.facts;
    if !facts.owned_by_user || facts.system_immutable || facts.restricted || facts.user_immutable || inside.foreign {
        return Verdict::LeftOut(Reason::NotYours);
    }
    if let Some(app) = &row.app {
        let Ok(running) = running else { return Verdict::LeftOut(Reason::NoSystem) };
        if running.apps().iter().any(|open| open.bundle_ids.contains(app)) {
            return Verdict::LeftOut(Reason::AppOpen);
        }
    }
    Verdict::Offered { category, name, seen }
}

/// The category and the name of the model whose folder is `path` in `row`'s store: `None` when the name does not fit
/// the row's shape or its kind is not one the row lists.
pub(crate) fn model(row: &Store, path: &Path) -> Option<(&'static str, ModelName)> {
    let name = name::read(row.shape, path)?;
    let category = match &row.categories {
        Categories::One(category) => *category,
        Categories::ByKind(kinds) => {
            let kind = name.kind.as_deref()?;
            kinds.iter().find(|(listed, _)| listed == kind).map(|(_, category)| *category)?
        }
    };
    Some((category, name))
}

/// When the model was last used: the newer of the last change and the last file access inside its folder.
pub(crate) fn last_seen(inside: &Inside) -> Option<Timestamp> {
    inside.newest_change.max(inside.newest_file_access)
}

#[cfg(test)]
mod tests;
