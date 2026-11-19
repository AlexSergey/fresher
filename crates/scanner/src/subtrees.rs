//! What the last walk saw below a folder, read from its tree before the commit: no disk is read.

use std::collections::HashSet;

use fresher_core::{ByteSize, Completeness, Sizes, Subtree, SubtreeQuery};

use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};

/// The answer to each of `queries`, in their order, from the frozen `tree`.
pub(crate) fn subtrees(tree: &Tree, queries: &[SubtreeQuery]) -> Vec<Option<Subtree>> {
    queries.iter().map(|query| subtree(tree, query)).collect()
}

/// The folder's frozen sums and the newest time and completeness of every folder below it that no skip holds. A skip
/// that is no node, or not below the folder, is never met on the way down: it is ignored.
fn subtree(tree: &Tree, query: &SubtreeQuery) -> Option<Subtree> {
    let folder = tree.lookup(&query.folder)?;
    let node = tree.node(folder)?;
    if !node.flags.has(Flag::Folder) {
        return None;
    }
    let skipped: HashSet<NodeIndex> = query.skip.iter().filter_map(|skip| tree.lookup(skip)).collect();
    let mut newest = None;
    let mut complete = true;
    let mut stack = vec![folder];
    while let Some(index) = stack.pop() {
        let Some(visited) = tree.node(index) else { continue };
        newest = newest.max(visited.modified());
        let flags = visited.flags;
        if flags.is_incomplete() || flags.has(Flag::OtherVolume) || flags.has(Flag::Collapsed) {
            complete = false;
        }
        // A folder claimed whole below the queried one keeps no children: what changed inside it is not known.
        if index != folder && flags.has(Flag::Opaque) {
            complete = false;
        }
        // A folder claimed whole or collapsed has no children: it counts with its own time alone.
        stack.extend(tree.children(index).filter(|child| {
            !skipped.contains(child) && tree.node(*child).is_some_and(|child| child.flags.has(Flag::Folder))
        }));
    }
    let sizes = Sizes {
        logical: ByteSize::new(node.logical),
        allocated: ByteSize::new(node.allocated),
        reclaimable: ByteSize::new(node.reclaimable),
        completeness: tree.completeness(folder).unwrap_or(Completeness::Complete),
    };
    Some(Subtree { sizes, newest, complete })
}

#[cfg(test)]
#[path = "subtrees_tests.rs"]
mod tests;
