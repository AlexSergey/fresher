//! Whether a folder of one of the module's places is offered, kept or not claimed: the rows of the design's section 4,
//! in their order, and the newest device support versions of each platform, on plain values. Nothing here reads the
//! disk or asks the system but through `running`.

use std::collections::BTreeSet;

use fresher_core::{EntryKind, LeftOutKind, RunningSet};

use crate::gathered::Candidate;
use crate::name::{ArchiveName, DeviceSupportName, Version, archive, device_support};
use crate::places::{Row, Shape};

/// The bundle ids of the apps that keep every candidate while one of them runs: Xcode and the Simulator.
pub(crate) const XCODE_APPS: [&str; 2] = ["com.apple.dt.Xcode", "com.apple.iphonesimulator"];

/// The running apps could not be listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoSystem;

/// Why a folder the module recognized is kept (design, section 5), in the order the module declares them.
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
            Self::Special => "xcode.left-out.special",
            Self::NotYours => "xcode.left-out.not-yours",
            Self::NoSystem => "xcode.left-out.no-system",
            Self::AppOpen => "xcode.left-out.app-open",
        }
    }

    /// Whether the folder is kept for what it is (a socket inside, not the user's) or for what goes on now (Xcode is
    /// open, the open apps could not be listed).
    pub(crate) const fn kind(self) -> LeftOutKind {
        match self {
            Self::Special | Self::NotYours => LeftOutKind::Kept,
            Self::NoSystem | Self::AppOpen => LeftOutKind::ForNow,
        }
    }
}

/// What a folder's name read as, by its row's shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Named {
    DeviceSupport(DeviceSupportName),
    /// One folder of a fixed place; its name is not judged.
    Fixed,
    Archive(ArchiveName),
}

/// What becomes of one folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not recognized: nothing is said.
    NotClaimed,
    /// Recognized, kept, and reported as left out with the reason; its name still counts among the newest versions.
    LeftOut(Reason, Named),
    /// Offered.
    Offered(Named),
}

/// The first row that applies to `candidate`, a match of `row`'s place:
///
/// 1. not a folder (a file, a link, a socket): not claimed;
/// 2. the name does not read as the row's shape: not claimed;
/// 3. a name of the row's sign is not inside: not claimed;
/// 4. not read in full: not claimed;
/// 5. nothing in it (logical size 0): not claimed;
/// 6. a socket, a pipe or a device inside: `Special`;
/// 7. not the user's, a system flag, restricted, the user's lock, or such an entry inside: `NotYours`;
/// 8. the running apps cannot be listed: `NoSystem`;
/// 9. a running app has one of [`XCODE_APPS`] among its bundle ids, ASCII case ignored: `AppOpen`;
/// 10. offered.
///
/// `running` is called only when a candidate reaches row 8. No time is judged: the verdict is the same whenever it is
/// asked.
pub(crate) fn decide<'r>(
    candidate: &Candidate,
    row: &Row,
    running: impl FnOnce() -> Result<&'r RunningSet, NoSystem>,
) -> Verdict {
    if candidate.kind != EntryKind::Folder {
        return Verdict::NotClaimed;
    }
    let named = match row.shape {
        Shape::DeviceSupport => candidate.path.file_name().and_then(device_support).map(Named::DeviceSupport),
        Shape::Fixed => Some(Named::Fixed),
        Shape::Archive => archive(&candidate.path).map(Named::Archive),
    };
    let Some(named) = named else { return Verdict::NotClaimed };
    if !row.sign.iter().all(|name| candidate.holds.contains(name)) {
        return Verdict::NotClaimed;
    }
    let Some(inside) = candidate.inside.filter(|inside| inside.complete) else { return Verdict::NotClaimed };
    if candidate.sizes.logical.bytes() == 0 {
        return Verdict::NotClaimed;
    }
    if inside.special {
        return Verdict::LeftOut(Reason::Special, named);
    }
    let facts = candidate.facts;
    if !facts.owned_by_user || facts.system_immutable || facts.restricted || facts.user_immutable || inside.foreign {
        return Verdict::LeftOut(Reason::NotYours, named);
    }
    let Ok(running) = running() else { return Verdict::LeftOut(Reason::NoSystem, named) };
    let open = running.apps().iter().any(|app| {
        app.bundle_ids.iter().any(|id| XCODE_APPS.iter().any(|xcode| id.as_str().eq_ignore_ascii_case(xcode)))
    });
    if open {
        return Verdict::LeftOut(Reason::AppOpen, named);
    }
    Verdict::Offered(named)
}

/// The newest device support versions: per `platform` of its row, the two highest distinct versions among the device
/// support folders `decided` offered or left out. A folder whose `(platform, version)` is in the set is recent.
pub(crate) fn recent<'a>(decided: impl IntoIterator<Item = (&'a Row, &'a Verdict)>) -> BTreeSet<(String, Version)> {
    let mut all: BTreeSet<(&str, &Version)> = BTreeSet::new();
    for (row, verdict) in decided {
        let (Verdict::Offered(Named::DeviceSupport(name)) | Verdict::LeftOut(_, Named::DeviceSupport(name))) = verdict
        else {
            continue;
        };
        if let Some(platform) = row.platform.as_deref() {
            all.insert((platform, &name.version));
        }
    }
    let mut newest: BTreeSet<(String, Version)> = BTreeSet::new();
    let mut platforms: Vec<&str> = all.iter().map(|(platform, _)| *platform).collect();
    platforms.dedup();
    for platform in platforms {
        let versions = all.iter().rev().filter(|(of, _)| *of == platform).take(2);
        newest.extend(versions.map(|(_, version)| (platform.to_owned(), (*version).clone())));
    }
    newest
}

#[cfg(test)]
mod tests;
