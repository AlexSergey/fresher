//! Whether a file under `~/Library/Logs` is offered, left out or not claimed: the rows of the design's section 4, in
//! their order, on plain values. Row 2 (another module's place) is `names::locate`, checked before. Nothing here reads
//! the disk or the clock, and the access time is never read: reading a log does not keep it.

use fresher_core::{EntryFacts, EntryKind, LeftOutKind, Timestamp};

pub(crate) const DAY: u64 = 24 * 60 * 60;

/// The plain values of one match.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Item {
    pub(crate) kind: EntryKind,
    pub(crate) facts: EntryFacts,
    pub(crate) modified: Option<Timestamp>,
    pub(crate) changed: Option<Timestamp>,
    /// The name marks an app's state or a lock (`names::is_state_or_lock`).
    pub(crate) state_or_lock: bool,
}

/// What the scan knows besides the item.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Scan {
    pub(crate) now: Timestamp,
    /// The setting's days.
    pub(crate) days: u32,
}

/// Why a file is left out (design, section 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Special,
    NotYours,
    Locked,
    Recent,
}

impl Reason {
    /// Every reason, in the order the module declares them.
    pub(crate) const ALL: [Self; 4] = [Self::Special, Self::NotYours, Self::Locked, Self::Recent];

    /// The reason's text key.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Special => "logs.left-out.special",
            Self::NotYours => "logs.left-out.not-yours",
            Self::Locked => "logs.left-out.locked",
            Self::Recent => "logs.left-out.recent",
        }
    }

    /// Whether the file is kept for what it is (a socket, not the user's) or for what goes on now (a recent change, a
    /// lock the user can lift).
    pub(crate) const fn kind(self) -> LeftOutKind {
        match self {
            Self::Special | Self::NotYours => LeftOutKind::Kept,
            Self::Locked | Self::Recent => LeftOutKind::ForNow,
        }
    }
}

/// What becomes of one file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not the module's: reported neither as a finding nor as left out.
    NotClaimed,
    /// Kept, and reported as left out with the reason.
    LeftOut(Reason),
    /// Offered: unchanged for `age_days` whole days, more than the setting's.
    Offered { age_days: u64 },
}

/// The first row that applies to `item`:
///
/// 1. a folder or a link: not claimed;
/// 2. a state or lock name: not claimed;
/// 3. a socket, a pipe or a device: `Special`;
/// 4. not the user's, the system's immutable flag, or restricted: `NotYours`;
/// 5. the user's lock: `Locked`;
/// 6. the newest of the modification and the status change unknown, in the future or not more than the days ago:
///    `Recent`;
/// 7. offered.
pub(crate) fn decide(item: &Item, scan: &Scan) -> Verdict {
    let facts = item.facts;
    if matches!(item.kind, EntryKind::Folder | EntryKind::Symlink) || item.state_or_lock {
        return Verdict::NotClaimed;
    }
    if item.kind == EntryKind::Other {
        return Verdict::LeftOut(Reason::Special);
    }
    if !facts.owned_by_user || facts.system_immutable || facts.restricted {
        return Verdict::LeftOut(Reason::NotYours);
    }
    if facts.user_immutable {
        return Verdict::LeftOut(Reason::Locked);
    }
    let newest = item.modified.zip(item.changed).map(|(modified, changed)| modified.max(changed));
    let Some(age) = newest.and_then(|newest| scan.now.secs().checked_sub(newest.secs())) else {
        return Verdict::LeftOut(Reason::Recent);
    };
    if age <= u64::from(scan.days) * DAY {
        return Verdict::LeftOut(Reason::Recent);
    }
    Verdict::Offered { age_days: age / DAY }
}

#[cfg(test)]
mod tests;
