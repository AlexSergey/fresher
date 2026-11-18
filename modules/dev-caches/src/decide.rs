//! Whether a cache folder is offered, kept or not claimed: the rows of the design's section 3, in their order, on
//! plain values. Nothing here reads the disk.

use std::ffi::OsStr;

use fresher_core::{EntryKind, MarkerCheck, RunningSet};

use crate::gathered::{Candidate, Sign};
use crate::name::project;
use crate::tools::{Cache, NameShape};

/// The name of a cache directory tag, whose content the walk checks.
const CACHEDIR_TAG: &str = "CACHEDIR.TAG";

/// The running apps could not be listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoSystem;

/// Why a recognized cache is kept (design, section 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    NotYours,
    Special,
    AppOpen,
    NoSystem,
}

/// What becomes of one cache folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not recognized as the tool's whole cache: nothing is said.
    NotClaimed,
    /// Recognized, kept, and reported as left out with the reason.
    LeftOut(Reason),
    /// Offered in the row's category.
    Offered,
}

/// The first row that applies to `candidate`, a match of `row`'s folder; `sign` is what the folder that must hold the
/// row's sign holds (`None` when that folder was not matched or is not a folder):
///
/// 1. not a folder: not claimed;
/// 2. the row has a name shape and the folder's name does not fit it: not claimed;
/// 3. no sign, a name of the row's sign missing, or a `CACHEDIR.TAG` in it not checked valid: not claimed;
/// 4. the folder not read in full: not claimed;
/// 5. a socket, a pipe or a device inside: `Special`;
/// 6. not the user's, a system flag, restricted, the user's lock, or such an entry inside: `NotYours`;
/// 7. the row names an app and the running apps cannot be listed: `NoSystem`;
/// 8. the row's app runs: `AppOpen`;
/// 9. offered.
///
/// `running` is read only for a row that names an app. The inside's lock names and every time are not read.
pub(crate) fn decide(
    candidate: &Candidate,
    row: &Cache,
    sign: Option<&Sign>,
    running: Result<&RunningSet, NoSystem>,
) -> Verdict {
    if candidate.kind != EntryKind::Folder {
        return Verdict::NotClaimed;
    }
    if row.name.is_some_and(|shape| !fits(shape, candidate.path.file_name())) {
        return Verdict::NotClaimed;
    }
    if !sign.is_some_and(|sign| is_signed(row, sign)) {
        return Verdict::NotClaimed;
    }
    let Some(inside) = candidate.inside.filter(|inside| inside.complete) else { return Verdict::NotClaimed };
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
    Verdict::Offered
}

fn fits(shape: NameShape, name: Option<&OsStr>) -> bool {
    match shape {
        NameShape::ProjectHash => name.and_then(project).is_some(),
    }
}

/// Every name of the row's sign is held, and a `CACHEDIR.TAG` among them is a valid tag.
fn is_signed(row: &Cache, sign: &Sign) -> bool {
    let holds_all = row.sign.iter().all(|name| sign.holds.contains(name));
    let tag_valid = !row.sign.iter().any(|name| name == CACHEDIR_TAG) || sign.marker == Some(MarkerCheck::Valid);
    holds_all && tag_valid
}

#[cfg(test)]
mod tests;
