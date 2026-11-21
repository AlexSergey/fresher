//! Whether a version folder is offered, left out or not claimed: the twelve rows of the design's section 6, in their
//! order, on plain values. Nothing here reads the disk.

use fresher_core::{EntryKind, Inside, Timestamp};

use crate::gathered::Candidate;
use crate::pointers::rustup::{self, ToolchainName};
use crate::pointers::{Pointers, nvm};
use crate::toolchains::{Row, Shape};

/// Why a version that would be offered is left out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Special,
    NotYours,
    PartialScan,
    DefaultUnknown,
    PinUnknown,
}

impl Reason {
    /// Every reason, in the order the module declares them.
    pub(crate) const ALL: [Self; 5] =
        [Self::Special, Self::NotYours, Self::PartialScan, Self::DefaultUnknown, Self::PinUnknown];

    /// The key of the reason's sentence.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Special => "unused-toolchains.left-out.special",
            Self::NotYours => "unused-toolchains.left-out.not-yours",
            Self::PartialScan => "unused-toolchains.left-out.partial-scan",
            Self::DefaultUnknown => "unused-toolchains.left-out.default-unknown",
            Self::PinUnknown => "unused-toolchains.left-out.pin-unknown",
        }
    }
}

/// What becomes of one version folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not a version the module may offer, or one something keeps: nothing is said.
    NotClaimed,
    /// It would be offered, but something blocks it: reported as left out with the reason.
    LeftOut(Reason),
    Offered,
}

/// What the scan knows for one row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Scan {
    /// The walk read every folder.
    pub(crate) walk_complete: bool,
    /// The row's switch is on, or the row has none.
    pub(crate) switch_on: bool,
}

/// A version folder's name read by its row's shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Name {
    Toolchain(ToolchainName),
    Node([u32; 3]),
}

/// An installed version of a row whose name fits the row's shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Version {
    /// Its index among the row's installed versions, as [`Pointers`] names them.
    pub(crate) index: usize,
    pub(crate) name: Name,
}

impl Version {
    /// The version whose folder is named `folder`, at `index` among its row's installed versions; `None` when the
    /// name does not fit `shape`.
    pub(crate) fn read(shape: Shape, index: usize, folder: &str) -> Option<Self> {
        let name = match shape {
            Shape::RustupToolchain => Name::Toolchain(rustup::folder(folder)?),
            Shape::NodeVersion => Name::Node(nvm::folder(folder)?),
        };
        Some(Self { index, name })
    }

    /// A rustup channel without a date: what it names moves with every update.
    fn is_moving(&self) -> bool {
        matches!(&self.name, Name::Toolchain(name) if rustup::is_moving(name))
    }
}

/// The first row that applies to `candidate`, a match of `row`'s versions place, named `version` (`None` when off
/// the row's shape), the newest installed Node version when `newest`, with the row's resolved `pointers`:
///
/// 1. not a folder: not claimed;
/// 2. the name off the row's shape: not claimed;
/// 3. the row's switch off: not claimed;
/// 4. a moving rustup channel, or the newest Node version: not claimed;
/// 5. not read in full, or no time at all: not claimed;
/// 6. a known pointer keeps it: not claimed;
/// 7. a socket, a pipe or a device inside: `Special`;
/// 8. not the user's, a system flag, restricted, the user's lock, or such an entry inside: `NotYours`;
/// 9. the walk did not read every folder: `PartialScan`;
/// 10. the default is unknown: `DefaultUnknown`;
/// 11. another pointer or a pin is unknown: `PinUnknown`;
/// 12. offered.
///
/// No time is judged and no order is read: the verdict is the same whenever it is asked.
pub(crate) fn decide(
    candidate: &Candidate,
    row: &Row,
    version: Option<&Version>,
    newest: bool,
    pointers: &Pointers,
    scan: Scan,
) -> Verdict {
    if candidate.kind != EntryKind::Folder {
        return Verdict::NotClaimed;
    }
    let Some(version) = version else { return Verdict::NotClaimed };
    if row.switch.is_some() && !scan.switch_on {
        return Verdict::NotClaimed;
    }
    if newest || version.is_moving() {
        return Verdict::NotClaimed;
    }
    let Some(inside) = candidate.inside.filter(|inside| inside.complete) else { return Verdict::NotClaimed };
    if last_seen(&inside).is_none() {
        return Verdict::NotClaimed;
    }
    if pointers.keeps(version.index) {
        return Verdict::NotClaimed;
    }
    if inside.special {
        return Verdict::LeftOut(Reason::Special);
    }
    let facts = candidate.facts;
    if !facts.owned_by_user || facts.system_immutable || facts.restricted || facts.user_immutable || inside.foreign {
        return Verdict::LeftOut(Reason::NotYours);
    }
    if !scan.walk_complete {
        return Verdict::LeftOut(Reason::PartialScan);
    }
    if pointers.default_unknown() {
        return Verdict::LeftOut(Reason::DefaultUnknown);
    }
    if pointers.other_unknown() {
        return Verdict::LeftOut(Reason::PinUnknown);
    }
    Verdict::Offered
}

/// When the version was last used: the newer of the last change and the last file access inside its folder.
pub(crate) fn last_seen(inside: &Inside) -> Option<Timestamp> {
    inside.newest_change.max(inside.newest_file_access)
}

#[cfg(test)]
mod tests;
