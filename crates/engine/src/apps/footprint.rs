//! An app's footprint in a committed state: the children of each place of the table named after the app's bundle id
//! or a helper's, each kept (listed, never removed) when something else may need it or it is not known in full.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fresher_core::{
    Access, BundleId, ChildOrder, Completeness, DiskState, FootprintKind, InstalledApp, KeptReason, KnownFolders,
    NodeId, NodeInfo, NodeKind, Sizes, TeamId, Timestamp,
};

use super::places::{NameShape, PLACES, Place};
use crate::storage::attribution::{ids, named_after};

/// The most children asked of a folder at once.
const PAGE: u32 = 500;

/// One file or folder of an app's footprint, outside its bundle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Part {
    pub(crate) kind: FootprintKind,
    pub(crate) node: NodeId,
    pub(crate) path: PathBuf,
    pub(crate) sizes: Sizes,
    pub(crate) modified: Option<Timestamp>,
    /// Why it is never removed; `None` when it goes with the choices its kind goes with.
    pub(crate) kept: Option<KeptReason>,
}

/// What the footprint is read with besides the state.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FootprintCx<'a> {
    /// The access to other apps' data: a container is kept unless it allows.
    pub(crate) other_apps_data: Access,
    /// The paths the user ignores.
    pub(crate) ignored: &'a [PathBuf],
    /// Whether the installed list holds every installed app; when it does not, another app may use any part, and
    /// every part is kept.
    pub(crate) complete: bool,
}

/// How a child names the app.
#[derive(Clone, Copy, Debug)]
enum Naming<'a> {
    /// By `id`, the longest id of the installed apps that names it.
    Id(&'a BundleId),
    /// A group container that starts with the app's own team id.
    OwnTeam,
}

/// The parts of `installed[app]` in `state`, in the table's order, then by path. Nothing for an index past the list.
pub(crate) fn footprint(
    state: &dyn DiskState,
    system: &dyn KnownFolders,
    installed: &[InstalledApp],
    app: usize,
    cx: &FootprintCx<'_>,
) -> Vec<Part> {
    if app >= installed.len() {
        return Vec::new();
    }
    let ids = ids(installed);
    let mut parts = Vec::new();
    for place in PLACES {
        let Some(folder) = system.locate(place.folder) else { continue };
        let folder = place.under.iter().fold(folder, |path, name| path.join(name));
        let Some(node) = state.find(&folder) else { continue };
        let mut found: Vec<Part> = children(state, node)
            .into_iter()
            .filter_map(|child| {
                let naming = naming(place, &child.name, installed, &ids, app)?;
                let path = state.path(child.id)?;
                let kept = kept(place.kind, naming, &child, &path, installed, app, cx);
                Some(Part {
                    kind: place.kind,
                    node: child.id,
                    path,
                    sizes: child.sizes,
                    modified: child.modified,
                    kept,
                })
            })
            .collect();
        found.sort_by(|a, b| a.path.cmp(&b.path));
        parts.extend(found);
    }
    parts
}

/// Every child of `node`, page by page.
fn children(state: &dyn DiskState, node: NodeId) -> Vec<NodeInfo> {
    let mut all = Vec::new();
    loop {
        let offset = u32::try_from(all.len()).unwrap_or(u32::MAX);
        let Some(page) = state.children(node, ChildOrder::ByName, offset, PAGE) else { break };
        let done = page.nodes.is_empty() || offset.saturating_add(PAGE) >= page.total;
        all.extend(page.nodes);
        if done {
            break;
        }
    }
    all
}

/// How `name`, a child of `place`, names `installed[app]`; `None` when it does not have the place's shape or names
/// another app or none.
fn naming<'a>(
    place: &Place,
    name: &str,
    installed: &'a [InstalledApp],
    ids: &HashMap<String, usize>,
    app: usize,
) -> Option<Naming<'a>> {
    let owner = installed.get(app)?;
    let exact = |stem: &str| by_id(installed, ids, app, stem).filter(|id| id.folded().eq_ignore_ascii_case(stem));
    match place.name {
        NameShape::Id => exact(name).map(Naming::Id),
        NameShape::IdWithSuffix(suffix) => exact(strip_suffix(name, suffix)?).map(Naming::Id),
        NameShape::IdDotted(suffix) => by_id(installed, ids, app, strip_suffix(name, suffix)?).map(Naming::Id),
        NameShape::TeamPrefixed => {
            let (prefix, rest) = name.split_once('.')?;
            if rest.is_empty() {
                return None;
            }
            if owner.team_id.as_ref().is_some_and(|team| team.as_str() == prefix) {
                return Some(Naming::OwnTeam);
            }
            // Another prefix (`group.`, another team's) after which the app's id comes: shown, always kept.
            by_id(installed, ids, app, rest).map(Naming::Id)
        }
    }
}

/// The longest id of the installed apps that names `stem`, when it is one of `installed[app]`'s own or its helpers'.
fn by_id<'a>(
    installed: &'a [InstalledApp],
    ids: &HashMap<String, usize>,
    app: usize,
    stem: &str,
) -> Option<&'a BundleId> {
    let first = installed.get(named_after(installed, ids, stem)?)?;
    let id = std::iter::once(&first.bundle_id)
        .chain(&first.helpers)
        .filter(|id| id.names(stem))
        .max_by_key(|id| id.folded().len())?;
    installed.get(app)?.owns(id).then_some(id)
}

/// `name` without `suffix` (case ignored), when something is left.
fn strip_suffix<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    let at = name.len().checked_sub(suffix.len()).filter(|&at| at > 0)?;
    let (stem, tail) = (name.get(..at)?, name.get(at..)?);
    tail.eq_ignore_ascii_case(suffix).then_some(stem)
}

/// Why the part `node` at `path` stays, in the order the reasons are told.
fn kept(
    kind: FootprintKind,
    naming: Naming<'_>,
    node: &NodeInfo,
    path: &Path,
    installed: &[InstalledApp],
    app: usize,
    cx: &FootprintCx<'_>,
) -> Option<KeptReason> {
    let owner = installed.get(app)?;
    let others = || installed.iter().enumerate().filter(move |&(at, other)| at != app && other.path != owner.path);
    // A helper's id outside the app's own (a framework's updater) may name other apps' files too.
    let foreign = |id: &BundleId| *id != owner.bundle_id && !owner.bundle_id.names(id.as_str());
    let shared = !cx.complete
        || match naming {
            Naming::Id(id) => foreign(id) || others().any(|(_, other)| other.owns(id)),
            Naming::OwnTeam => false,
        };
    let team_shared = kind == FootprintKind::GroupContainer
        && match (naming, &owner.team_id) {
            (Naming::OwnTeam, Some(team)) => others().any(|(_, other)| same_team(other.team_id.as_ref(), team)),
            _ => true,
        };
    let guarded = matches!(kind, FootprintKind::Container | FootprintKind::GroupContainer);
    let reasons = [
        (shared, KeptReason::Shared),
        (team_shared, KeptReason::TeamShared),
        (node.kind == NodeKind::Link, KeptReason::Link),
        (guarded && !cx.other_apps_data.allows(), KeptReason::NeedsPermission),
        (node.not_scanned.is_some() || node.sizes.completeness != Completeness::Complete, KeptReason::NotScanned),
        (cx.ignored.iter().any(|ignored| ignored.starts_with(path) || path.starts_with(ignored)), KeptReason::Ignored),
    ];
    reasons.into_iter().find_map(|(holds, reason)| holds.then_some(reason))
}

fn same_team(other: Option<&TeamId>, team: &TeamId) -> bool {
    other == Some(team)
}

#[cfg(test)]
#[path = "footprint_tests.rs"]
mod tests;
