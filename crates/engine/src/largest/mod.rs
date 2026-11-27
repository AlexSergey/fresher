//! The Largest screen's queries over the last committed state: the Files list and the Browse view, each row judged
//! by one function. Pure reads that never block a scan: the state and what is kept beside it are taken under the
//! results' lock, the page is built after letting it go.

pub(crate) mod folders;
mod items;
mod judge;
mod page;
mod place;
mod rule;

use std::path::{Path, PathBuf};

use fresher_core::{
    CommandError, DiskState, Finding, FindingId, FoldersPage, FoldersQuery, KnownFolder, LargestPage, LargestQuery,
    NodeId, NodeInfo, NodeKind, Offer, Operation, OwnPlace, Protection, Sizes, Timestamp, Withheld, is_package,
};

pub(crate) use items::ItemsSlot;
pub(crate) use judge::judge;

use crate::Engine;
use crate::engine::Shared;
use crate::error::lock;
use crate::largest::folders::{empty_folders, folders_page, left_alone};
pub(crate) use crate::largest::items::{Item, Items};
use crate::largest::judge::{Above, Ignored, Judged};
use crate::largest::page::{empty_page, files_page};
use crate::largest::place::own_place;
pub(crate) use crate::largest::rule::HomeItems;
use crate::results::{Committed, by_path};
use crate::storage::query::{MAX_LIMIT, Snapshot};
use crate::storage::shares::Shares;

const DAY: u64 = 86_400;

/// `path` lies below the home of `committed`'s state (never the home itself) and no cleanup removed it since the scan.
pub(crate) fn revealable(shared: &Shared, committed: &Committed, path: &Path) -> bool {
    let Some((places, _)) = home_in(shared, &*committed.state) else { return false };
    let protection = protection(shared);
    let below_home = path != places.home && path.starts_with(&places.home);
    below_home && !ground(committed, &places, &protection).is_gone(path)
}

/// The home's places as the system locates them.
struct HomePlaces {
    home: PathBuf,
    library: Option<PathBuf>,
    trash: Option<PathBuf>,
}

/// What a row of the home is judged against: the state, its shares, the home's places, the cleaner's protection, the
/// ignore list, and the picks and what they removed.
pub(crate) struct Ground<'a> {
    pub(crate) state: &'a dyn DiskState,
    pub(crate) shares: &'a Shares,
    pub(crate) home: &'a Path,
    library: Option<&'a Path>,
    trash: Option<&'a Path>,
    protection: &'a Protection,
    ignored: &'a [PathBuf],
    own: &'a [Finding],
    gone: &'a [PathBuf],
}

impl Ground<'_> {
    /// The offer for `node` at `path` and whether it opens in Browse.
    pub(crate) fn judge(&self, node: &NodeInfo, path: &Path) -> (Offer, bool) {
        let above = if self.shares.inside_owned(path) { Above::Claimed } else { Above::of(self.home, path) };
        judge(&Judged {
            node,
            path,
            above,
            library: self.library,
            trash: self.trash,
            protection: self.protection.check(path),
            holds_claimed: self.shares.holds_owned(node.id),
            ignored: self.ignored(path),
        })
    }

    /// What the ignore list holds of `path`: it or a folder above it, a path inside it, or nothing.
    pub(crate) fn ignored(&self, path: &Path) -> Ignored {
        if self.is_ignored(path) {
            Ignored::AtOrAbove
        } else if self.ignored.iter().any(|ignored| ignored.starts_with(path)) {
            Ignored::Inside
        } else {
            Ignored::No
        }
    }

    /// The judgement of `node` at `path` as Files, Browse and a pick must all see it: a node below a folder that does
    /// not open (`Library`, the Trash, a package, an app, a claimed, hidden or protected folder) is never reached, so it
    /// takes that folder's reason and never opens.
    pub(crate) fn judge_reached(&self, node: &NodeInfo, path: &Path) -> (Offer, bool) {
        match self.blocked(path) {
            Some(reason) => (Offer::Withheld(reason), false),
            None => self.judge(node, path),
        }
    }

    /// The reason of the first folder strictly between the home and `path` that does not open; `Hidden` for one that
    /// is offered but does not open (a package), `NotSeen` for one not in the state or a path outside the home.
    fn blocked(&self, path: &Path) -> Option<Withheld> {
        let Ok(inside) = path.strip_prefix(self.home) else { return Some(Withheld::NotSeen) };
        let mut at = self.home.to_path_buf();
        let mut folders: Vec<_> = inside.components().collect();
        folders.pop();
        folders.into_iter().find_map(|part| {
            at.push(part);
            let Some(node) = self.state.find(&at).and_then(|id| self.state.node(id)) else {
                return Some(Withheld::NotSeen);
            };
            match self.judge(&node, &at) {
                (_, true) => None,
                (Offer::Withheld(reason), false) => Some(reason),
                (Offer::Offered, false) => Some(Withheld::Hidden),
            }
        })
    }

    /// The ignore list holds `path` or a folder above it.
    pub(crate) fn is_ignored(&self, path: &Path) -> bool {
        self.ignored.iter().any(|ignored| path.starts_with(ignored))
    }

    /// A cleanup removed a pick at `path` or a folder above it since the scan.
    pub(crate) fn is_gone(&self, path: &Path) -> bool {
        self.gone.iter().any(|gone| path.starts_with(gone))
    }

    /// How many of `folder`'s children in the state a cleanup removed.
    pub(crate) fn gone_children(&self, folder: &Path) -> u32 {
        let gone = self.gone.iter().filter(|gone| gone.parent() == Some(folder) && self.state.find(gone).is_some());
        u32::try_from(gone.count()).unwrap_or(u32::MAX)
    }

    /// The id of the pick of the row at `path`, if the user picked it.
    pub(crate) fn picked(&self, path: &Path) -> Option<FindingId> {
        by_path(self.own, path).map(Finding::id)
    }
}

impl Engine {
    /// One page of the home's largest items of the last state, at most 100 rows. `NoScan` before any commit, `Busy`
    /// while a scan runs, `StaleNode` for a place of another state, `NotOwnItem` for a place outside the home. A state
    /// without the home (another root was scanned) answers an empty page.
    pub fn largest(&self, query: &LargestQuery) -> Result<LargestPage, CommandError> {
        let min = lock(&self.shared.settings)?.largest_min_size();
        let committed = self.committed_for_largest()?;
        let state = &*committed.state;
        if query.place.is_some_and(|place| place.state() != state.id()) {
            return Err(CommandError::StaleNode);
        }
        let Some((places, home)) = home_in(&self.shared, state) else {
            return Ok(empty_page(query.offset, min));
        };
        if let Some(place) = query.place {
            let path = state.path(place).ok_or(CommandError::StaleNode)?;
            if path == places.home || !path.starts_with(&places.home) {
                return Err(CommandError::NotOwnItem);
            }
        }
        let rule = home_rule(&places);
        let items = Items::cached(&committed.largest, min, || Items::of(state, home, &rule, min))?;
        let protection = protection(&self.shared);
        Ok(files_page(&items, query, &ground(&committed, &places, &protection)))
    }

    /// One page of `folder` of the last state (the home when `None`), largest first, with the way from the home; at
    /// most 500 rows. Refused as `largest` is, and `NotOwnItem` for a folder that does not open: `Library`, the Trash,
    /// a package, an app, a claimed or a hidden folder. A state without the home answers an empty page.
    pub fn largest_place(&self, folder: Option<NodeId>, offset: u32, limit: u32) -> Result<OwnPlace, CommandError> {
        let committed = self.committed_for_largest()?;
        let state = &*committed.state;
        if folder.is_some_and(|folder| folder.state() != state.id()) {
            return Err(CommandError::StaleNode);
        }
        let Some((places, home)) = home_in(&self.shared, state) else {
            return Ok(OwnPlace { path: Vec::new(), rows: Vec::new(), offset, total: 0 });
        };
        let protection = protection(&self.shared);
        own_place(&ground(&committed, &places, &protection), folder.unwrap_or(home), offset, limit.min(MAX_LIMIT))
    }

    /// One page of the folders of the home left alone, largest first: refused as `largest` is, a state without the
    /// home answers an empty page. The smallest size and the days nothing changed come from the settings, the time
    /// from the scan, never the clock. The list is computed per page, nothing of it is kept.
    pub fn largest_folders(&self, query: FoldersQuery) -> Result<FoldersPage, CommandError> {
        let (min, days) = {
            let settings = lock(&self.shared.settings)?;
            (settings.largest_folder_min_size(), settings.stale_days())
        };
        let committed = self.committed_for_largest()?;
        let state = &*committed.state;
        let Some((places, home)) = home_in(&self.shared, state) else {
            return Ok(empty_folders(query.offset, min, days));
        };
        let protection = protection(&self.shared);
        let ground = ground(&committed, &places, &protection);
        let before = Timestamp::from_secs(state.taken().secs().saturating_sub(u64::from(days) * DAY));
        let listed = left_alone(&ground, home, &home_rule(&places), min, before);
        let home_sizes = state.node(home).map_or(Sizes::ZERO, |node| node.sizes);
        Ok(folders_page(&listed, query, &ground, home_sizes, min, days))
    }

    /// The last committed state: `NoScan` before any, `Busy` while a scan runs.
    pub(crate) fn committed_for_largest(&self) -> Result<Committed, CommandError> {
        match self.snapshot()? {
            Snapshot::NoScan => Err(CommandError::NoScan),
            Snapshot::Scanning { .. } => Err(CommandError::Busy { running: Operation::Scan }),
            Snapshot::Complete(committed) => Ok(committed),
        }
    }
}

/// The home of `committed`'s state when `node` at `path` may be picked now: below the home (never the home itself),
/// judged offered as Files and Browse judge it (the ignore list taken with the state, the engine's protection), not
/// a plain folder (only a file, a package or a folded subtree is picked) and not removed since the scan. `None`
/// otherwise, and for a state without the home.
pub(crate) fn pickable(shared: &Shared, committed: &Committed, node: &NodeInfo, path: &Path) -> Option<PathBuf> {
    let (places, _) = home_in(shared, &*committed.state)?;
    let protection = protection(shared);
    let ground = ground(committed, &places, &protection);
    let below_home = path != places.home && path.starts_with(&places.home);
    let plain_folder = node.kind == NodeKind::Folder && !is_package(&node.name);
    let offered =
        below_home && !plain_folder && !ground.is_gone(path) && ground.judge_reached(node, path).0 == Offer::Offered;
    offered.then_some(places.home)
}

/// The home's places as the system locates them, and the home's node in `state`; `None` when either is missing.
fn home_in(shared: &Shared, state: &dyn DiskState) -> Option<(HomePlaces, NodeId)> {
    let folders = &*shared.ports.system.known_folders;
    let home = folders.locate(KnownFolder::Home)?;
    let id = state.find(&home)?;
    let library = folders.locate(KnownFolder::Library);
    let trash = folders.locate(KnownFolder::Trash);
    Some((HomePlaces { home, library, trash }, id))
}

/// What the cleaner refuses; nothing for an engine that cannot clean. Never the cleanup context's protection, which
/// adds the ignore list: the ignore list is judged on its own.
fn protection(shared: &Shared) -> Protection {
    shared.cleanup.as_ref().map(|cleanup| cleanup.cleaner.protection()).unwrap_or_default()
}

fn ground<'a>(committed: &'a Committed, places: &'a HomePlaces, protection: &'a Protection) -> Ground<'a> {
    Ground {
        state: &*committed.state,
        shares: committed.shares(),
        home: &places.home,
        library: places.library.as_deref(),
        trash: places.trash.as_deref(),
        protection,
        ignored: committed.ignored_paths(),
        own: &committed.own,
        gone: &committed.gone,
    }
}

/// The rule of the home's items: its `Library` and Trash are skipped at the top.
fn home_rule(places: &HomePlaces) -> HomeItems {
    HomeItems::new(top_name(&places.home, places.library.as_deref()), top_name(&places.home, places.trash.as_deref()))
}

/// `folder` written from `home`: `~` for the home itself, `~/a/b` below it; lossy, for display only.
pub(crate) fn from_home(home: &Path, folder: &Path) -> String {
    let home_itself = Path::new("~");
    match folder.strip_prefix(home) {
        Ok(inside) if inside.as_os_str().is_empty() => home_itself.to_string_lossy().into_owned(),
        Ok(inside) => home_itself.join(inside).to_string_lossy().into_owned(),
        Err(_) => folder.to_string_lossy().into_owned(),
    }
}

/// Runs `with` over a ground of `state` and `home` that holds nothing else: no shares of claimed items, nothing
/// protected, ignored or removed. For the measurements.
pub(crate) fn with_bare_ground<R>(state: &dyn DiskState, home: &Path, with: impl FnOnce(&Ground<'_>) -> R) -> R {
    let shares = Shares::of(state, &crate::settings::Ignored::default(), &crate::withheld::Withheld::offered());
    let protection = Protection::default();
    with(&Ground {
        state,
        shares: &shares,
        home,
        library: None,
        trash: None,
        protection: &protection,
        ignored: &[],
        own: &[],
        gone: &[],
    })
}

/// The name of `place` when it is directly under `home`.
fn top_name(home: &Path, place: Option<&Path>) -> Option<String> {
    let place = place.filter(|place| place.parent() == Some(home))?;
    Some(place.file_name()?.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "largest_tests.rs"]
mod tests;
