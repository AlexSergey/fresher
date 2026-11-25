//! Whether the leftovers of an id are offered, kept or not claimed: the rows of the design's section 3, in their
//! order, on plain values. Nothing here reads the disk or asks the system but through `search`.

use std::iter;

use fresher_core::{BundleId, InstalledApp, LeftOutKind, RunningSet, Timestamp};

use crate::exclude::Exclude;
use crate::gathered::Leftover;
use crate::name::related;
use crate::places::{Form, Row};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// A list of the system's could not be read at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NoSystem;

/// Why recognized leftovers are kept (design, section 4), in the order the module declares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    NoSystem,
    AppsUnread,
    Running,
    ScanAgain,
    UsedRecently,
    SpotlightUnread,
    InstalledElsewhere,
    Special,
    NotYours,
}

impl Reason {
    pub(crate) const ALL: [Self; 9] = [
        Self::NoSystem,
        Self::AppsUnread,
        Self::Running,
        Self::ScanAgain,
        Self::UsedRecently,
        Self::SpotlightUnread,
        Self::InstalledElsewhere,
        Self::Special,
        Self::NotYours,
    ];

    /// The reason's name in its text key, `app-leftovers.left-out.<name>`.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::NoSystem => "no-system",
            Self::AppsUnread => "apps-unread",
            Self::Running => "running",
            Self::ScanAgain => "scan-again",
            Self::UsedRecently => "used-recently",
            Self::SpotlightUnread => "spotlight-unread",
            Self::InstalledElsewhere => "installed-elsewhere",
            Self::Special => "special",
            Self::NotYours => "not-yours",
        }
    }

    /// Whether the leftover is kept for what it is (its app is installed elsewhere, a socket, not the user's) or for
    /// what goes on now (an app runs, it changed lately, a list or a place could not be read).
    pub(crate) const fn kind(self) -> LeftOutKind {
        match self {
            Self::InstalledElsewhere | Self::Special | Self::NotYours => LeftOutKind::Kept,
            Self::NoSystem
            | Self::AppsUnread
            | Self::Running
            | Self::ScanAgain
            | Self::UsedRecently
            | Self::SpotlightUnread => LeftOutKind::ForNow,
        }
    }
}

/// What Spotlight answered for an id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Search {
    /// It could not be asked, or it cannot see every app.
    Unread,
    /// A bundle with the id, or with an id it extends, is on the Mac.
    Found,
    NotFound,
}

/// What becomes of every leftover of an id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IdVerdict {
    /// Not recognized as an app's leftovers: nothing is said.
    NotClaimed,
    /// Recognized, every leftover kept and reported as left out with the reason.
    LeftOut(Reason),
    /// Each leftover is decided on its own (`decide_item`).
    Offer,
}

/// What becomes of one leftover of an id that is offered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemVerdict {
    LeftOut(Reason),
    Offered,
}

/// What the scan knows of the system, read once in `finish`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Facts<'a> {
    /// The installed apps, the partial list included.
    pub(crate) installed: Result<&'a [InstalledApp], NoSystem>,
    /// The list of installed apps was read in full.
    pub(crate) complete: bool,
    pub(crate) running: Result<&'a RunningSet, NoSystem>,
    pub(crate) restored: bool,
    /// Every place of the rows that is not optional was seen in full.
    pub(crate) places_complete: bool,
    pub(crate) now: Timestamp,
    pub(crate) days: u32,
}

/// The first row of the design's section 3 that holds for `id` and its `leftovers` (`rows` is the places table):
///
/// 1. the id is excluded: not claimed;
/// 2. no leftover is a folder of a proof row (a place that is absent proves nothing): not claimed;
/// 3. the installed apps cannot be listed: `NoSystem`;
/// 4. an installed app's id or a helper's is the id or related to it: not claimed;
/// 5. the list of installed apps is partial: `AppsUnread`;
/// 6. the running apps cannot be listed: `NoSystem`;
/// 7. a running app's id is related to the id: `Running`;
/// 8. a restored scan, a place not seen in full, or a folder not read in full: `ScanAgain`;
/// 9. a leftover changed within the days, at an unknown time or in the future: `UsedRecently`;
/// 10. Spotlight cannot tell: `SpotlightUnread`;
/// 11. Spotlight finds the app: `InstalledElsewhere`;
/// 12. offered.
///
/// `search` is called only at row 10, at most once.
pub(crate) fn decide_id(
    id: &BundleId,
    leftovers: &[Leftover],
    rows: &[Row],
    exclude: &Exclude,
    facts: &Facts<'_>,
    search: &mut dyn FnMut(&BundleId) -> Search,
) -> IdVerdict {
    if exclude.covers(id) {
        return IdVerdict::NotClaimed;
    }
    if !leftovers.iter().any(|leftover| is_proof(leftover, rows)) {
        return IdVerdict::NotClaimed;
    }
    let Ok(installed) = facts.installed else { return IdVerdict::LeftOut(Reason::NoSystem) };
    if installed.iter().any(|app| is_app_of(app, id)) {
        return IdVerdict::NotClaimed;
    }
    if !facts.complete {
        return IdVerdict::LeftOut(Reason::AppsUnread);
    }
    let Ok(running) = facts.running else { return IdVerdict::LeftOut(Reason::NoSystem) };
    if running.apps().iter().any(|app| app.bundle_ids.iter().any(|open| related(open, id))) {
        return IdVerdict::LeftOut(Reason::Running);
    }
    if facts.restored || !facts.places_complete || leftovers.iter().any(|leftover| !is_read(leftover)) {
        return IdVerdict::LeftOut(Reason::ScanAgain);
    }
    if !leftovers.iter().all(|leftover| is_old(leftover, facts.now, facts.days)) {
        return IdVerdict::LeftOut(Reason::UsedRecently);
    }
    match search(id) {
        Search::Unread => IdVerdict::LeftOut(Reason::SpotlightUnread),
        Search::Found => IdVerdict::LeftOut(Reason::InstalledElsewhere),
        Search::NotFound => IdVerdict::Offer,
    }
}

/// Rows 13 to 15, for one leftover of an id that is offered: a socket, a pipe or a device inside: `Special`; not the
/// user's, a system flag, restricted, the user's lock, or such an entry inside: `NotYours`; otherwise offered. A
/// folder not read in full, which `decide_id` already keeps, is kept here too.
pub(crate) fn decide_item(leftover: &Leftover) -> ItemVerdict {
    if !is_read(leftover) {
        return ItemVerdict::LeftOut(Reason::ScanAgain);
    }
    let (special, foreign) = leftover.inside.map_or((false, false), |inside| (inside.special, inside.foreign));
    if special {
        return ItemVerdict::LeftOut(Reason::Special);
    }
    let facts = leftover.facts;
    if !facts.owned_by_user || facts.system_immutable || facts.restricted || facts.user_immutable || foreign {
        return ItemVerdict::LeftOut(Reason::NotYours);
    }
    ItemVerdict::Offered
}

/// A folder of a proof row: what the system makes only for an app.
fn is_proof(leftover: &Leftover, rows: &[Row]) -> bool {
    leftover.kind == Form::Folder && rows.get(leftover.row).is_some_and(|row| row.proof && row.kind == Form::Folder)
}

/// `app`, or one of its helpers, has `id` or an id related to it.
fn is_app_of(app: &InstalledApp, id: &BundleId) -> bool {
    app.owns(id) || iter::once(&app.bundle_id).chain(&app.helpers).any(|own| related(own, id))
}

/// A file, or a folder read in full.
fn is_read(leftover: &Leftover) -> bool {
    match leftover.kind {
        Form::File => true,
        Form::Folder => leftover.inside.is_some_and(|inside| inside.complete),
    }
}

/// The leftover's newest change is known and at least `days` before `now`: a folder's over everything inside it, a
/// file's newest of its modification and status change, both known.
fn is_old(leftover: &Leftover, now: Timestamp, days: u32) -> bool {
    let newest = match leftover.kind {
        Form::Folder => leftover.inside.and_then(|inside| inside.newest_change),
        Form::File => leftover.modified.zip(leftover.changed).map(|(modified, changed)| modified.max(changed)),
    };
    let threshold = u64::from(days).saturating_mul(SECONDS_PER_DAY);
    newest.is_some_and(|time| time <= now && now.secs() - time.secs() >= threshold)
}

#[cfg(test)]
mod tests;
