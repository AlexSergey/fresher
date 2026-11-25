//! Whether a top-level temporary item is offered or kept: the rows of the design's section 7, in their order, on
//! plain values. Nothing here reads the disk or the clock.

use std::ffi::OsStr;

use fresher_core::{EntryFacts, EntryKind, Inside, LeftOutKind, RunningSet, Timestamp};

use crate::names::{is_lock, is_system, named_after};

const DAY: u64 = 24 * 60 * 60;

/// The plain values of one match.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Item<'a> {
    pub(crate) name: &'a OsStr,
    pub(crate) kind: EntryKind,
    pub(crate) facts: EntryFacts,
    pub(crate) modified: Option<Timestamp>,
    pub(crate) changed: Option<Timestamp>,
    /// The entry's own last access, a file's or a link's; `None` for a folder, whose access moves when the walk lists
    /// it (its files' newest access is `inside.newest_file_access`).
    pub(crate) accessed: Option<Timestamp>,
    /// What a folder holds; `None` for anything else.
    pub(crate) inside: Option<Inside>,
}

/// The running apps could not be listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoSystem;

/// What the scan knows besides the item.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Scan<'a> {
    pub(crate) now: Timestamp,
    /// The setting's days.
    pub(crate) days: u32,
    pub(crate) running: Result<&'a RunningSet, NoSystem>,
}

/// Why an item is kept (design, section 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Lock,
    Special,
    System,
    Locked,
    NoSystem,
    AppOpen,
    Recent,
}

impl Reason {
    /// Every reason, in the order the module declares their keys.
    pub(crate) const ALL: [Self; 7] =
        [Self::System, Self::Locked, Self::AppOpen, Self::Special, Self::Lock, Self::Recent, Self::NoSystem];

    /// Whether the item is kept for what it is (the system's, a socket, a lock) or for what goes on now (an open app,
    /// a recent change, a lock the user can lift, the running apps not listed).
    pub(crate) const fn kind(self) -> LeftOutKind {
        match self {
            Self::System | Self::Special | Self::Lock => LeftOutKind::Kept,
            Self::Locked | Self::AppOpen | Self::Recent | Self::NoSystem => LeftOutKind::ForNow,
        }
    }
}

/// What becomes of one item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Kept, and reported as left out with the reason.
    LeftOut(Reason),
    /// Not the module's to answer: the walk could not read the whole folder, and reports what it could not read.
    NotClaimed,
    /// Offered: nothing in it changed for `age_days` whole days, more than the setting's.
    Offered { age_days: u64 },
}

/// The first row that applies to `item`:
///
/// 1. a lock or marker name, or a folder holding a lock file's name: `Lock`;
/// 2. a socket, a pipe or a device, or a folder holding one: `Special`;
/// 3. not the user's, a system flag, restricted, a folder holding such an entry, or the system's name: `System`;
/// 4. the user's lock: `Locked`;
/// 5. the running apps cannot be listed: `NoSystem`;
/// 6. named after a running app, with or without one leading dot, compared as bytes: `AppOpen`;
/// 7. a folder whose inside was not read in full: not claimed;
/// 8. the newest change unknown, in the future or not more than the days ago, or a file read within the days:
///    `Recent`;
/// 9. offered.
pub(crate) fn decide(item: &Item<'_>, scan: &Scan<'_>) -> Verdict {
    let inside = item.inside.as_ref();
    let folder = item.kind == EntryKind::Folder;
    let facts = item.facts;
    if is_lock(item.name) || (folder && inside.is_some_and(|inside| inside.lock_named)) {
        return Verdict::LeftOut(Reason::Lock);
    }
    if item.kind == EntryKind::Other || (folder && inside.is_some_and(|inside| inside.special)) {
        return Verdict::LeftOut(Reason::Special);
    }
    if !facts.owned_by_user
        || facts.system_immutable
        || facts.restricted
        || (folder && inside.is_some_and(|inside| inside.foreign))
        || is_system(item.name)
    {
        return Verdict::LeftOut(Reason::System);
    }
    if facts.user_immutable {
        return Verdict::LeftOut(Reason::Locked);
    }
    let Ok(running) = scan.running else { return Verdict::LeftOut(Reason::NoSystem) };
    if running.apps().iter().any(|app| app.bundle_ids.iter().any(|id| named_after(item.name, id))) {
        return Verdict::LeftOut(Reason::AppOpen);
    }
    let (newest, accessed) = if folder {
        let Some(inside) = inside.filter(|inside| inside.complete) else { return Verdict::NotClaimed };
        (newest(&[item.modified, item.changed, inside.newest_change]), inside.newest_file_access)
    } else {
        (newest(&[item.modified, item.changed]), item.accessed)
    };
    let days = u64::from(scan.days) * DAY;
    let Some(age) = newest.and_then(|newest| scan.now.secs().checked_sub(newest.secs())) else {
        return Verdict::LeftOut(Reason::Recent);
    };
    let read_lately = accessed.is_some_and(|accessed| scan.now.secs().saturating_sub(accessed.secs()) <= days);
    if age <= days || read_lately {
        return Verdict::LeftOut(Reason::Recent);
    }
    Verdict::Offered { age_days: age / DAY }
}

/// The newest of `times`; `None` when one of them is unknown.
fn newest(times: &[Option<Timestamp>]) -> Option<Timestamp> {
    times.iter().try_fold(None, |newest: Option<Timestamp>, time| time.map(|time| newest.max(Some(time))))?
}

#[cfg(test)]
mod tests;
