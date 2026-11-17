//! The shares of a committed state: every owned byte in one place, by category, by folder and by root.
//!
//! The rule is the claims' (`crate::claims::resolve`) on the tree's own sizes, over every owned node: the deepest
//! owner owns its subtree, an outer owner keeps its sizes minus those of the owners nearest inside it. An owner whose
//! category is ignored, or whose path is at or below an ignored path, is not counted: the bytes it keeps are the
//! user's folders, never an outer owner's.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use fresher_core::{ByteSize, CategoryId, DiskState, NodeId, Sizes};

use crate::claims::minus;
use crate::settings::Ignored;

/// The owned bytes of one state under one pair of ignore lists, built once by [`Shares::of`].
#[derive(Debug)]
pub(crate) struct Shares {
    /// Every owned path, counted or not.
    owners: HashMap<PathBuf, Counted>,
    /// Each owner and its ancestors up to the next owner above: the sizes of the top-most owners at or below it.
    owned: HashMap<NodeId, Sizes>,
    /// Each counted owner and all its ancestors: the bytes the counted owners at or below it keep.
    kept: HashMap<NodeId, Sizes>,
    /// Each counted owner: its category and the bytes it keeps itself, the owners nearest inside it taken out.
    own: HashMap<NodeId, (CategoryId, Sizes)>,
    roots: Vec<RootShare>,
}

/// One scan root split in two: what the counted owners explain and what is left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RootShare {
    pub(crate) root: NodeId,
    pub(crate) explained: Sizes,
    /// `root.sizes − explained`.
    pub(crate) unclaimed: Sizes,
}

/// Whether an owner's bytes are explained or the user's folders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Counted {
    Yes,
    Ignored,
}

/// An owned node.
struct Owner {
    id: NodeId,
    category: CategoryId,
    path: PathBuf,
    sizes: Sizes,
    counted: Counted,
}

impl Shares {
    /// The shares of `state`, the owners `ignored` hides not counted. Pure; a sum past `u64::MAX` bytes stays at the
    /// maximum and is logged once.
    pub(crate) fn of(state: &dyn DiskState, ignored: &Ignored) -> Self {
        let owners: Vec<Owner> = state
            .owned()
            .into_iter()
            .filter_map(|(id, category)| {
                let path = state.path(id)?;
                let sizes = state.node(id)?.sizes;
                let counted = if ignored.hides_place(category, &path) { Counted::Ignored } else { Counted::Yes };
                Some(Owner { id, category, path, sizes, counted })
            })
            .collect();
        let by_path: HashMap<&Path, usize> =
            owners.iter().enumerate().map(|(position, owner)| (owner.path.as_path(), position)).collect();
        let mut sum = Sum::default();
        let mut keeps: Vec<Sizes> = owners.iter().map(|owner| owner.sizes).collect();
        let mut owned: HashMap<NodeId, Sizes> = HashMap::new();
        for owner in &owners {
            sum.add_to(&mut owned, owner.id, owner.sizes);
            for ancestor in owner.path.ancestors().skip(1) {
                if let Some(&outer) = by_path.get(ancestor) {
                    keeps[outer] = minus(keeps[outer], owner.sizes);
                    break;
                }
                let Some(id) = state.find(ancestor) else { break };
                sum.add_to(&mut owned, id, owner.sizes);
            }
        }
        let mut kept: HashMap<NodeId, Sizes> = HashMap::new();
        let mut own: HashMap<NodeId, (CategoryId, Sizes)> = HashMap::new();
        for (owner, keeps) in owners.iter().zip(keeps) {
            if owner.counted == Counted::Ignored {
                continue;
            }
            own.insert(owner.id, (owner.category, keeps));
            sum.add_to(&mut kept, owner.id, keeps);
            for ancestor in owner.path.ancestors().skip(1) {
                let Some(id) = state.find(ancestor) else { break };
                sum.add_to(&mut kept, id, keeps);
            }
        }
        if sum.overflowed {
            tracing::warn!("a share of the storage map passed the largest size and stays at it");
        }
        let owners = owners.into_iter().map(|owner| (owner.path, owner.counted)).collect();
        let mut shares = Self { owners, owned, kept, own, roots: Vec::new() };
        shares.roots = state
            .roots()
            .into_iter()
            .filter_map(|root| {
                let sizes = state.node(root)?.sizes;
                let mut explained = shares.explained(state, root);
                explained.completeness = explained.completeness.max(sizes.completeness);
                let mut unclaimed = minus(sizes, explained);
                unclaimed.completeness = explained.completeness;
                Some(RootShare { root, explained, unclaimed })
            })
            .collect();
        shares
    }

    /// The bytes `category` keeps; nothing for a category that keeps nothing counted.
    #[cfg(test)]
    pub(crate) fn category(&self, category: CategoryId) -> Sizes {
        self.categories().find(|&(id, _)| id == category).map_or(Sizes::ZERO, |(_, sizes)| sizes)
    }

    /// Every counted category, with the bytes it keeps, by id.
    #[cfg(test)]
    pub(crate) fn categories(&self) -> impl Iterator<Item = (CategoryId, Sizes)> {
        let mut categories: std::collections::BTreeMap<CategoryId, Sizes> = std::collections::BTreeMap::new();
        for (_, category, sizes) in self.kept() {
            let total = categories.entry(category).or_insert(Sizes::ZERO);
            *total = Sum::default().add(*total, sizes);
        }
        categories.into_iter()
    }

    /// The bytes of `id` (of `state`, the state these shares are of) that counted owners keep. A node strictly
    /// inside a counted owner is wholly explained but for the owners inside it that are not counted; nothing for an
    /// unknown node.
    pub(crate) fn explained(&self, state: &dyn DiskState, id: NodeId) -> Sizes {
        let get = |map: &HashMap<NodeId, Sizes>| map.get(&id).copied().unwrap_or(Sizes::ZERO);
        let kept = get(&self.kept);
        let Some(path) = state.path(id) else { return kept };
        if self.owners.contains_key(&path) {
            return kept;
        }
        let outer = path.ancestors().skip(1).find_map(|ancestor| self.owners.get(ancestor));
        match (outer, state.node(id)) {
            (Some(Counted::Yes), Some(node)) => {
                minus(node.sizes, get(&self.owned)).checked_add(kept).unwrap_or(node.sizes)
            }
            _ => kept,
        }
    }

    /// The category of the counted owner `id` and the bytes it keeps itself; `None` for a node that is not a counted
    /// owner.
    pub(crate) fn kept_by(&self, id: NodeId) -> Option<(CategoryId, Sizes)> {
        self.own.get(&id).copied()
    }

    /// Every counted owner with its category and the bytes it keeps itself, in no order.
    pub(crate) fn kept(&self) -> impl Iterator<Item = (NodeId, CategoryId, Sizes)> + '_ {
        self.own.iter().map(|(&node, &(category, sizes))| (node, category, sizes))
    }

    /// Every scan root, in the state's order, split into explained and unclaimed.
    pub(crate) fn roots(&self) -> &[RootShare] {
        &self.roots
    }

    /// The bytes these shares hold on the heap, as the standard hash map lays its tables out: per table, its bucket
    /// count (a power of two, from its capacity) times an entry, aligned to 16, plus a control byte per bucket and one
    /// group of them more; and the owners' paths.
    pub(crate) fn heap_bytes(&self) -> usize {
        use std::mem::size_of;

        /// The control bytes past the last bucket: one SIMD group.
        const GROUP: usize = if cfg!(target_arch = "x86_64") { 16 } else { 8 };
        let table = |capacity: usize, entry: usize| {
            let buckets = match capacity {
                0 => return 0,
                1..8 => capacity + 1,
                _ => capacity / 7 * 8,
            };
            (buckets * entry).next_multiple_of(16) + buckets + GROUP
        };
        let paths: usize = self.owners.keys().map(PathBuf::capacity).sum();
        table(self.owners.capacity(), size_of::<(PathBuf, Counted)>())
            + paths
            + table(self.owned.capacity(), size_of::<(NodeId, Sizes)>())
            + table(self.kept.capacity(), size_of::<(NodeId, Sizes)>())
            + table(self.own.capacity(), size_of::<(NodeId, (CategoryId, Sizes))>())
            + self.roots.capacity() * size_of::<RootShare>()
    }
}

/// Adds sizes, remembering whether a sum passed `u64::MAX`.
#[derive(Default)]
struct Sum {
    overflowed: bool,
}

impl Sum {
    fn add(&mut self, total: Sizes, more: Sizes) -> Sizes {
        total.checked_add(more).unwrap_or_else(|| {
            self.overflowed = true;
            let add = |a: ByteSize, b: ByteSize| ByteSize::new(a.bytes().saturating_add(b.bytes()));
            Sizes {
                logical: add(total.logical, more.logical),
                allocated: add(total.allocated, more.allocated),
                reclaimable: add(total.reclaimable, more.reclaimable),
                completeness: total.completeness.max(more.completeness),
            }
        })
    }

    fn add_to(&mut self, map: &mut HashMap<NodeId, Sizes>, id: NodeId, more: Sizes) {
        let total = map.entry(id).or_insert(Sizes::ZERO);
        *total = self.add(*total, more);
    }
}

#[cfg(test)]
#[path = "shares_tests.rs"]
mod tests;
