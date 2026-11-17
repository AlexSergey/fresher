//! The map by meaning: the apps, the explained categories no app names, the named places and the largest of the
//! user's folders, then the rest of the user's folders. Every byte of the roots is in exactly one leaf.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use fresher_core::{
    AppRef, BundleId, CategoryId, CategoryShare, ChildOrder, DiskState, Finding, MapPart, MeaningGroup, NodeId,
    NodeKind, PartKind, PlaceName, PlaceShare, Sizes, Subject, VolumeMap,
};

use crate::claims::minus;
use crate::results::add;
use crate::storage::attribution::AppPlaces;
use crate::storage::shares::{RootShare, Shares};

/// The most top-level folders of one root listed by name; the rest are in "Your folders".
const LARGEST_FOLDERS: usize = 8;

/// The by-meaning groups of the scan roots `roots` of `state` (one volume's): the apps largest first, then the
/// categories no app names, then the Trash and the largest unclaimed top-level folders of each root, then the rest of
/// the unclaimed bytes as one `YourFolders` part. `findings` are the scan's, the ignored ones too: an ignored finding
/// of an app is the app's data. Nothing outside `roots` is in a group.
pub(crate) fn meaning(
    state: &dyn DiskState,
    shares: &Shares,
    places: &AppPlaces,
    findings: &[Finding],
    roots: &[NodeId],
) -> Vec<MeaningGroup> {
    let scope = Scope::new(state, roots);
    let mut apps: BTreeMap<BundleId, Gathered> = BTreeMap::new();
    let mut taken: HashMap<CategoryId, Sizes> = HashMap::new();
    let mut counted: HashSet<NodeId> = HashSet::new();
    let mut claimed: Vec<(NodeId, Keeper)> = places
        .data()
        .filter(|&(node, _)| scope.holds_node(state, node))
        .map(|(node, app)| (node, Keeper::App(app.clone())))
        .collect();
    for finding in findings {
        let new = finding.finding();
        let (Some(id), Subject::Path(path)) = (new.app(), new.subject()) else { continue };
        if !scope.holds(path) {
            continue;
        }
        let (Some(app), Some(node)) = (places.app(id), state.find(path)) else { continue };
        if let Some((category, sizes)) = shares.kept_by(node).filter(|&(category, _)| category == new.category())
            && counted.insert(node)
        {
            let group = apps.entry(app.id.clone()).or_insert_with(|| Gathered::new(app.clone()));
            let total = group.categories.entry(category).or_insert(Sizes::ZERO);
            *total = add(*total, sizes);
            let total = taken.entry(category).or_insert(Sizes::ZERO);
            *total = add(*total, sizes);
        }
        claimed.push((node, Keeper::App(app)));
    }
    if let Some(trash) = places.trash().filter(|&trash| scope.holds_node(state, trash)) {
        claimed.push((trash, Keeper::Trash));
    }

    let mut trash = None;
    let outermost = outermost(state, shares, claimed);
    for place in &outermost {
        let (node, sizes) = (place.node, place.unclaimed);
        match &place.keeper {
            Keeper::App(app) => {
                let group = apps.entry(app.id.clone()).or_insert_with(|| Gathered::new(app.clone()));
                group.data.push(PlaceShare { node, name: PlaceName::Folder(place.name.clone()), sizes });
            }
            Keeper::Trash => trash = Some(PlaceShare { node, name: PlaceName::Trash, sizes }),
        }
    }

    let mut groups: Vec<MeaningGroup> = Vec::new();
    let mut apps: Vec<(Sizes, MeaningGroup)> = apps.into_values().filter_map(Gathered::into_group).collect();
    apps.sort_by_key(|(sizes, _)| Reverse(sizes.allocated));
    groups.extend(apps.into_iter().map(|(_, group)| group));

    let mut categories: Vec<CategoryShare> = scope
        .categories(state, shares)
        .into_iter()
        .map(|(category, sizes)| CategoryShare {
            category,
            sizes: minus(sizes, taken.get(&category).copied().unwrap_or(Sizes::ZERO)),
        })
        .filter(|share| !is_empty(share.sizes))
        .collect();
    categories.sort_by_key(|share| Reverse(share.sizes.allocated));
    groups.extend(categories.into_iter().map(MeaningGroup::Category));

    let mut rest = scope.roots(shares).fold(Sizes::ZERO, |total, root| add(total, root.unclaimed));
    for place in &outermost {
        rest = minus(rest, place.unclaimed);
    }
    groups.extend(trash.filter(|trash| !is_empty(trash.sizes)).map(MeaningGroup::Place));
    for folder in largest_folders(state, shares, &scope, &outermost) {
        rest = minus(rest, folder.sizes);
        groups.push(MeaningGroup::Place(folder));
    }
    groups.push(MeaningGroup::Part(MapPart { kind: PartKind::YourFolders, sizes: rest }));
    groups
}

/// The lines both views end with: the volume's parts that are not files of the scan (the remainder, the system's
/// volumes, the other volumes, the disk's own space and Free), appended to `groups` in the volume's order.
pub(crate) fn append_volume_parts(groups: &mut Vec<MeaningGroup>, volume: &VolumeMap) {
    let scanned =
        |kind: &PartKind| matches!(kind, PartKind::Explained | PartKind::YourFolders | PartKind::NotScannedYet);
    groups.extend(volume.parts().iter().filter(|part| !scanned(&part.kind)).cloned().map(MeaningGroup::Part));
}

/// The scan roots one view is of.
struct Scope {
    roots: Vec<NodeId>,
    paths: Vec<PathBuf>,
}

impl Scope {
    fn new(state: &dyn DiskState, roots: &[NodeId]) -> Self {
        Self { roots: roots.to_vec(), paths: roots.iter().filter_map(|&root| state.path(root)).collect() }
    }

    fn holds(&self, path: &Path) -> bool {
        self.paths.iter().any(|root| path.starts_with(root))
    }

    fn holds_node(&self, state: &dyn DiskState, node: NodeId) -> bool {
        state.path(node).is_some_and(|path| self.holds(&path))
    }

    /// The shares of the roots in scope, in the state's order.
    fn roots<'a>(&'a self, shares: &'a Shares) -> impl Iterator<Item = &'a RootShare> + 'a {
        shares.roots().iter().filter(|root| self.roots.contains(&root.root))
    }

    /// The bytes each counted category keeps under the roots in scope.
    fn categories(&self, state: &dyn DiskState, shares: &Shares) -> BTreeMap<CategoryId, Sizes> {
        let mut categories = BTreeMap::new();
        for (node, category, sizes) in shares.kept() {
            if self.holds_node(state, node) {
                let total = categories.entry(category).or_insert(Sizes::ZERO);
                *total = add(*total, sizes);
            }
        }
        categories
    }
}

/// Whom a claimed place belongs to.
#[derive(Clone, Debug)]
enum Keeper {
    App(AppRef),
    Trash,
}

/// A claimed place no other claimed place holds.
struct Claimed {
    node: NodeId,
    path: PathBuf,
    name: String,
    /// The place's bytes no counted owner keeps.
    unclaimed: Sizes,
    keeper: Keeper,
}

/// The places of `claimed` that no other one holds, each node once (its first keeper), so no byte is in two.
fn outermost(state: &dyn DiskState, shares: &Shares, claimed: Vec<(NodeId, Keeper)>) -> Vec<Claimed> {
    let mut seen: HashSet<NodeId> = HashSet::new();
    let places: Vec<(NodeId, PathBuf, Keeper)> = claimed
        .into_iter()
        .filter(|(node, _)| seen.insert(*node))
        .filter_map(|(node, keeper)| Some((node, state.path(node)?, keeper)))
        .collect();
    let paths: HashSet<&Path> = places.iter().map(|(_, path, _)| path.as_path()).collect();
    let held: Vec<bool> =
        places.iter().map(|(_, path, _)| path.ancestors().skip(1).any(|above| paths.contains(above))).collect();
    places
        .into_iter()
        .zip(held)
        .filter(|&(_, held)| !held)
        .filter_map(|((node, path, keeper), _)| {
            let info = state.node(node)?;
            let unclaimed = minus(info.sizes, shares.explained(state, node));
            Some(Claimed { node, path, name: info.name, unclaimed, keeper })
        })
        .collect()
}

/// The largest top-level folders of each root, at most [`LARGEST_FOLDERS`] per root, each with its unclaimed bytes
/// less those of the claimed places inside it; a folder that is a claimed place or inside one is left out. Only
/// folders are listed, since a row opens the place view: a top-level file (or link, or folded subtree) stays in the
/// rest of the user's folders.
fn largest_folders(state: &dyn DiskState, shares: &Shares, scope: &Scope, claimed: &[Claimed]) -> Vec<PlaceShare> {
    let claimed_paths: HashSet<&Path> = claimed.iter().map(|place| place.path.as_path()).collect();
    let mut folders = Vec::new();
    for root in scope.roots(shares) {
        let Some(page) = state.children(root.root, ChildOrder::ByName, 0, u32::MAX) else { continue };
        let mut tops: Vec<(PathBuf, PlaceShare)> = page
            .nodes
            .into_iter()
            .filter(|child| child.kind == NodeKind::Folder)
            .filter_map(|child| {
                let path = state.path(child.id)?;
                if path.ancestors().any(|at| claimed_paths.contains(at)) {
                    return None;
                }
                let unclaimed = minus(child.sizes, shares.explained(state, child.id));
                Some((path, PlaceShare { node: child.id, name: PlaceName::Folder(child.name), sizes: unclaimed }))
            })
            .collect();
        for place in claimed {
            let top = tops.iter_mut().find(|(path, _)| place.path.starts_with(path));
            if let Some((_, top)) = top {
                top.sizes = minus(top.sizes, place.unclaimed);
            }
        }
        let mut tops: Vec<PlaceShare> =
            tops.into_iter().map(|(_, top)| top).filter(|top| !is_empty(top.sizes)).collect();
        tops.sort_by_key(|top| Reverse(top.sizes.allocated));
        folders.extend(tops.into_iter().take(LARGEST_FOLDERS));
    }
    folders
}

/// One app's group as it is gathered.
struct Gathered {
    app: AppRef,
    categories: BTreeMap<CategoryId, Sizes>,
    data: Vec<PlaceShare>,
}

impl Gathered {
    fn new(app: AppRef) -> Self {
        Self { app, categories: BTreeMap::new(), data: Vec::new() }
    }

    /// The group with its total, the largest first inside it; `None` for an app with no bytes.
    fn into_group(self) -> Option<(Sizes, MeaningGroup)> {
        let mut categories: Vec<CategoryShare> = self
            .categories
            .into_iter()
            .map(|(category, sizes)| CategoryShare { category, sizes })
            .filter(|share| !is_empty(share.sizes))
            .collect();
        categories.sort_by_key(|share| Reverse(share.sizes.allocated));
        let mut data: Vec<PlaceShare> = self.data.into_iter().filter(|place| !is_empty(place.sizes)).collect();
        data.sort_by_key(|place| Reverse(place.sizes.allocated));
        let sizes = categories
            .iter()
            .map(|share| share.sizes)
            .chain(data.iter().map(|place| place.sizes))
            .fold(Sizes::ZERO, add);
        (!is_empty(sizes)).then_some((sizes, MeaningGroup::App { app: self.app, sizes, categories, data }))
    }
}

/// Whether `sizes` holds no byte in any of its sizes.
fn is_empty(sizes: Sizes) -> bool {
    sizes.logical.bytes() == 0 && sizes.allocated.bytes() == 0 && sizes.reclaimable.bytes() == 0
}

#[cfg(test)]
#[path = "meaning_tests.rs"]
mod tests;
