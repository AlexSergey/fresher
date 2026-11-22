//! `TreeState`: the core's `DiskState` over a frozen tree and its claims.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use fresher_core::{
    ByteSize, CategoryId, ChildOrder, Completeness, DiskState, ItemRule, NodeId, NodeInfo, NodeKind, NodePage,
    NotScanned, Sizes, StateId, Timestamp, UnreadablePlace,
};

use crate::state::search::Lowered;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, Node, NodeIndex};

mod items;
mod largest;
mod search;

/// The read-only tree of one committed scan. Never changes: a new scan commits a new state.
#[derive(Debug)]
pub(crate) struct TreeState {
    id: StateId,
    taken: Timestamp,
    tree: Tree,
    /// The categories that own nodes; a node's `owner` indexes it.
    owners: Vec<CategoryId>,
    /// The lowercase copy of the names, built on the first search and kept for the next ones.
    lowered: OnceLock<Lowered>,
}

impl TreeState {
    /// The state of `tree` (frozen) with `owners`, the table its nodes' owner indices point into.
    pub(crate) const fn new(id: StateId, taken: Timestamp, tree: Tree, owners: Vec<CategoryId>) -> Self {
        Self { id, taken, tree, owners, lowered: OnceLock::new() }
    }

    pub(crate) const fn tree(&self) -> &Tree {
        &self.tree
    }

    /// The categories the nodes' owner indices point into.
    pub(crate) fn owners(&self) -> &[CategoryId] {
        &self.owners
    }

    /// The bytes this state holds on the heap: its tree's, and the lowercase copy of the names once a search built it.
    pub(crate) fn heap_bytes(&self) -> usize {
        self.tree.heap_bytes() + self.lowered.get().map_or(0, Lowered::heap_bytes)
    }

    /// The tree's node behind `id`; `None` for an id of another state or no node.
    fn index(&self, id: NodeId) -> Option<NodeIndex> {
        if id.state() != self.id {
            return None;
        }
        let index = NodeIndex::new(usize::try_from(id.index()).ok()?)?;
        self.tree.node(index).map(|_| index)
    }

    fn id_of(&self, index: NodeIndex) -> Option<NodeId> {
        u32::try_from(index.get()?).ok().map(|raw| NodeId::new(self.id, raw))
    }

    fn info(&self, index: NodeIndex) -> Option<NodeInfo> {
        let node = self.tree.node(index)?;
        let kind = kind_of(node);
        let folder = kind == NodeKind::Folder || kind == NodeKind::Collapsed;
        let completeness = self.tree.completeness(index).unwrap_or(Completeness::Complete);
        Some(NodeInfo {
            id: self.id_of(index)?,
            name: String::from_utf8_lossy(self.tree.name(index)).into_owned(),
            kind,
            sizes: Sizes {
                logical: ByteSize::new(node.logical),
                allocated: ByteSize::new(node.allocated),
                reclaimable: ByteSize::new(node.reclaimable),
                completeness,
            },
            files: if folder { u64::from(node.files) } else { 0 },
            folders: if folder { u64::from(node.folders) } else { 0 },
            owner: usize::try_from(node.owner).ok().and_then(|owner| self.owners.get(owner)).copied(),
            modified: node.modified(),
            not_scanned: not_scanned(node),
        })
    }
}

fn kind_of(node: &Node) -> NodeKind {
    if node.flags.has(Flag::Collapsed) {
        NodeKind::Collapsed
    } else if node.flags.has(Flag::Folder) {
        NodeKind::Folder
    } else if node.flags.has(Flag::Link) {
        NodeKind::Link
    } else {
        NodeKind::File
    }
}

/// Why the node's folder was not read, if it was not.
fn not_scanned(node: &Node) -> Option<NotScanned> {
    if let Some(reason) = node.flags.unreadable() {
        Some(NotScanned::Unreadable(reason))
    } else if node.flags.has(Flag::OthersFolder) {
        Some(NotScanned::OthersFolder)
    } else if node.flags.has(Flag::SystemItems) {
        Some(NotScanned::SystemItems)
    } else if node.flags.has(Flag::Avoided) {
        Some(NotScanned::Avoided)
    } else if node.flags.has(Flag::OtherVolume) {
        Some(NotScanned::OtherVolume)
    } else {
        None
    }
}

impl DiskState for TreeState {
    fn id(&self) -> StateId {
        self.id
    }

    fn taken(&self) -> Timestamp {
        self.taken
    }

    fn roots(&self) -> Vec<NodeId> {
        self.tree.roots().iter().filter_map(|root| self.id_of(root.node)).collect()
    }

    fn node(&self, id: NodeId) -> Option<NodeInfo> {
        self.info(self.index(id)?)
    }

    /// By size: the largest allocated first, then by name; by name: as the tree keeps them.
    fn children(&self, id: NodeId, order: ChildOrder, offset: u32, limit: u32) -> Option<NodePage> {
        let index = self.index(id)?;
        let mut children: Vec<NodeIndex> = self.tree.children(index).collect();
        if order == ChildOrder::BySize {
            let allocated = |child: NodeIndex| self.tree.node(child).map_or(0, |node| node.allocated);
            children.sort_by(|&left, &right| {
                allocated(right).cmp(&allocated(left)).then_with(|| self.tree.name(left).cmp(self.tree.name(right)))
            });
        }
        let total = u32::try_from(children.len()).unwrap_or(u32::MAX);
        let skip = usize::try_from(offset).unwrap_or(usize::MAX);
        let take = usize::try_from(limit).unwrap_or(usize::MAX);
        let nodes = children.into_iter().skip(skip).take(take).filter_map(|child| self.info(child)).collect();
        Some(NodePage { nodes, offset, total })
    }

    /// The kept file records, the largest allocated first, ties in path order; at most 500. A hard link counted at
    /// another path is left out.
    fn largest_files(&self, limit: u32) -> Vec<NodeInfo> {
        largest::largest_files(&self.tree, limit).into_iter().filter_map(|index| self.info(index)).collect()
    }

    /// Folders and kept files whose name contains `query`, case ignored, the largest first; at most 500. Small files
    /// are folded into their folder, so a search never finds one. The first search builds a lowercase copy of the
    /// names; the next ones reuse it.
    fn search(&self, query: &str, limit: u32) -> Vec<NodeInfo> {
        let Some(query) = search::lowered_query(query) else { return Vec::new() };
        let lowered = self.lowered.get_or_init(|| Lowered::of(&self.tree));
        lowered.search(&self.tree, &query, limit).into_iter().filter_map(|index| self.info(index)).collect()
    }

    /// Depth first from the roots; a folder not scanned is listed and not descended into.
    fn unreadable(&self, limit: u32) -> Vec<UnreadablePlace> {
        let limit = usize::try_from(limit).unwrap_or(usize::MAX);
        let mut places = Vec::new();
        let mut stack: Vec<NodeIndex> = self.tree.roots().iter().rev().map(|root| root.node).collect();
        while let Some(index) = stack.pop() {
            if places.len() >= limit {
                break;
            }
            let Some(node) = self.tree.node(index) else { continue };
            if let Some(reason) = not_scanned(node) {
                if let Some(info) = self.info(index) {
                    places.push(UnreadablePlace { node: info, reason, hidden_folders: u64::from(node.folders) });
                }
                continue;
            }
            let children: Vec<NodeIndex> = self.tree.children(index).collect();
            stack.extend(children.into_iter().rev());
        }
        places
    }

    fn path(&self, id: NodeId) -> Option<PathBuf> {
        self.tree.path(self.index(id)?)
    }

    /// In arena order: a node is pushed after its parent, so an ancestor always comes first.
    fn owned(&self) -> Vec<(NodeId, CategoryId)> {
        (0..self.tree.len())
            .filter_map(|position| {
                let index = NodeIndex::new(position)?;
                let owner = usize::try_from(self.tree.node(index)?.owner).ok()?;
                Some((self.id_of(index)?, *self.owners.get(owner)?))
            })
            .collect()
    }

    fn find(&self, path: &Path) -> Option<NodeId> {
        self.id_of(self.tree.lookup(path)?)
    }

    /// One depth-first pass from `root`; at most 10 000.
    fn items(&self, root: NodeId, rule: &dyn ItemRule, min: ByteSize, limit: u32) -> Option<Vec<NodeId>> {
        let root = self.index(root)?;
        let found = items::items(&self.tree, root, rule, min.bytes(), limit);
        Some(found.into_iter().filter_map(|index| self.id_of(index)).collect())
    }
}

#[cfg(test)]
mod items_tests;
#[cfg(test)]
mod query_tests;
#[cfg(test)]
mod tests;
