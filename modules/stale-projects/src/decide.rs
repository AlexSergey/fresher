//! What becomes of each artifact and cache of a project group: the tables of the design's section 3.4, row by row,
//! on plain values. Nothing here reads the disk or asks the system.

use fresher_core::{Artifact, ByteSize, Completeness, LastChange, Sizes, Timestamp};

use crate::CACHE_UNUSED_DAYS;

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// What the scan knows, read once in `finish`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Scan {
    pub(crate) now: Timestamp,
    /// The setting `stale-projects.unused-for-days`.
    pub(crate) days: u32,
}

/// What a project group is, by its last activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Staleness {
    /// Something changed within the days, or in the future.
    Active,
    /// A folder could not be read, or nothing dates it: it cannot be proven stale.
    Unread,
    /// Every folder read, nothing changed since `since`, before the days.
    Stale { since: Timestamp },
}

/// Why an artifact or a cache is kept (design, section 3.5), in the order the module declares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    Unread,
    Recent,
    CacheRecent,
}

impl Reason {
    pub(crate) const ALL: [Self; 3] = [Self::Unread, Self::Recent, Self::CacheRecent];

    /// The reason's text key.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Unread => "stale-projects.left-out.unread",
            Self::Recent => "stale-projects.left-out.recent",
            Self::CacheRecent => "stale-projects.left-out.cache-recent",
        }
    }
}

/// What becomes of an artifact or a cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// Not the module's: nothing is said.
    NotClaimed,
    /// Recognized and kept, reported as left out with the reason.
    LeftOut(Reason),
    Offered,
}

/// The last activity of a group from its members' own: the newest time any member saw, `Known` only when every
/// member's is `Known`, `Unknown` when none has a time. A commit in a nested repository keeps the whole group active,
/// and a member that cannot be dated keeps the group from being proven stale.
pub(crate) fn group_activity(members: impl IntoIterator<Item = LastChange>) -> LastChange {
    let mut newest: Option<Timestamp> = None;
    let mut known = true;
    for activity in members {
        let time = match activity {
            LastChange::Known(time) => time,
            LastChange::AtLeast(time) => {
                known = false;
                time
            }
            LastChange::Unknown => {
                known = false;
                continue;
            }
        };
        newest = Some(newest.map_or(time, |newest| newest.max(time)));
    }
    LastChange::of(newest, known)
}

/// The group table, on the group's last activity:
///
/// 1. `Known(t)` or `AtLeast(t)`, `t` within the days or in the future: `Active`;
/// 2. `AtLeast(t)` with `t` before the days, or `Unknown`: `Unread`;
/// 3. `Known(t)`, `t` before the days: `Stale` since `t`.
pub(crate) fn staleness(activity: LastChange, scan: &Scan) -> Staleness {
    match activity {
        LastChange::Known(time) | LastChange::AtLeast(time) if is_within(time, scan.days, scan.now) => {
            Staleness::Active
        }
        LastChange::Known(since) => Staleness::Stale { since },
        LastChange::AtLeast(_) | LastChange::Unknown => Staleness::Unread,
    }
}

/// The artifact table, for an artifact of a group:
///
/// 1. not matched by the module: not claimed;
/// 2. the group active: `Recent`;
/// 3. the group unread: `Unread`;
/// 4. the group stale, the artifact's own last change within the days or in the future (`Known` or `AtLeast`):
///    `Recent` (a package installed last week into an old project's environment);
/// 5. the group stale, the artifact's own last change not `Known`, or its sizes not complete: `Unread`;
/// 6. the group stale: offered.
pub(crate) fn artifact(artifact: &Artifact, matched: bool, staleness: Staleness, scan: &Scan) -> Verdict {
    if !matched {
        return Verdict::NotClaimed;
    }
    match staleness {
        Staleness::Active => Verdict::LeftOut(Reason::Recent),
        Staleness::Unread => Verdict::LeftOut(Reason::Unread),
        Staleness::Stale { .. } => match artifact.last_change {
            LastChange::Known(time) | LastChange::AtLeast(time) if is_within(time, scan.days, scan.now) => {
                Verdict::LeftOut(Reason::Recent)
            }
            LastChange::Known(_) if is_complete(artifact.sizes) => Verdict::Offered,
            LastChange::Known(_) | LastChange::AtLeast(_) | LastChange::Unknown => Verdict::LeftOut(Reason::Unread),
        },
    }
}

/// The cache table, for a cache of a Rust project whose `target` is not offered, whatever the group's age:
///
/// 1. not matched by the module: not claimed;
/// 2. its last change within `CACHE_UNUSED_DAYS` or in the future (`Known` or `AtLeast`): `CacheRecent`;
/// 3. `Unknown`, `AtLeast(t)` before those days, or its sizes not complete: `Unread`;
/// 4. offered.
pub(crate) fn cache(cache: &Artifact, matched: bool, scan: &Scan) -> Verdict {
    if !matched {
        return Verdict::NotClaimed;
    }
    match cache.last_change {
        LastChange::Known(time) | LastChange::AtLeast(time) if is_within(time, CACHE_UNUSED_DAYS, scan.now) => {
            Verdict::LeftOut(Reason::CacheRecent)
        }
        LastChange::Known(_) if is_complete(cache.sizes) => Verdict::Offered,
        LastChange::Known(_) | LastChange::AtLeast(_) | LastChange::Unknown => Verdict::LeftOut(Reason::Unread),
    }
}

/// `sizes` less every one of `offered`, field by field, never below zero; the completeness of `sizes` kept.
pub(crate) fn less(sizes: Sizes, offered: &[Sizes]) -> Sizes {
    let minus = |from: ByteSize, part: ByteSize| ByteSize::new(from.bytes().saturating_sub(part.bytes()));
    offered.iter().fold(sizes, |left, part| Sizes {
        logical: minus(left.logical, part.logical),
        allocated: minus(left.allocated, part.allocated),
        reclaimable: minus(left.reclaimable, part.reclaimable),
        completeness: left.completeness,
    })
}

/// `time` is after `now − days`, or after `now`; when `now` is less than `days` after the epoch, every time is.
fn is_within(time: Timestamp, days: u32, now: Timestamp) -> bool {
    let span = u64::from(days).saturating_mul(SECONDS_PER_DAY);
    now.secs().checked_sub(span).is_none_or(|start| time.secs() > start)
}

fn is_complete(sizes: Sizes) -> bool {
    sizes.completeness == Completeness::Complete
}

#[cfg(test)]
mod tests;
